/**
 * BidirectionalSyncFinal - CRDT+LevelDB整合版 (Sprint6)
 * 约束: ≤280行 | Yjs CRDT | LevelDB持久化 | ISyncEngine兼容
 */
import { EventEmitter } from 'events';
import * as crypto from 'crypto';

// Type imports (resolved at runtime via require)
type PeerId = string; type ChunkId = string;

interface Chunk {
  id: ChunkId; data: Buffer; mtime: number;
  hash: string; vectorClock: { [nodeId: string]: number };
  crdtState?: Uint8Array;
}

interface SyncResult {
  success: boolean; pushed: number; pulled: number;
  conflicts: number; conflictsResolved: number; duration: number; error?: string;
}

interface PushResult { pushed: number; persisted: boolean; }
interface PullResult { pulled: number; merged: number; }

interface SyncOperation {
  id: string; type: 'PUSH' | 'PULL' | 'SYNC';
  peerId: PeerId; timestamp: number; retryCount: number;
}

// CRDT Engine Interface (from B-40/01)
interface ICrdtEngine {
  merge(local: Chunk, remote: Chunk): Chunk;
  encodeState(chunk: Chunk): Uint8Array;
  decodeState(state: Uint8Array): Partial<Chunk>;
}

// LevelDB Queue Interface (from B-40/02)
interface IQueueDb {
  getQueue(): Promise<SyncOperation[]>;
  saveQueue(queue: SyncOperation[]): Promise<void>;
  appendOperation(op: SyncOperation): Promise<void>;
  clearQueue(): Promise<void>;
}

export class BidirectionalSyncFinal extends EventEmitter {
  private dcManager: any;
  private crdtEngine: ICrdtEngine;      // B-40/01: CRDT引擎
  private queueDb: IQueueDb;            // B-40/02: LevelDB持久化
  private chunkStorage: Map<ChunkId, Chunk> = new Map();
  private offlineQueue: SyncOperation[] = [];
  private maxQueueSize: number = 1000;
  private connectedPeers: Set<PeerId> = new Set();
  private syncInProgress: Set<PeerId> = new Set();

  constructor(dcManager: any, crdtEngine: ICrdtEngine, queueDb: IQueueDb) {
    super();
    this.dcManager = dcManager;
    this.crdtEngine = crdtEngine;
    this.queueDb = queueDb;
    this.setupEventHandlers();
    this.restoreQueue(); // 启动时恢复队列
  }

  private setupEventHandlers(): void {
    this.dcManager.on('text-message', ({ peerId, text }: any) => {
      try {
        const msg = JSON.parse(text);
        if (msg.type === 'sync-manifest') this.handleManifest(peerId, msg.manifest);
        if (msg.type === 'sync-request') this.handleSyncRequest(peerId, msg.chunkIds);
        if (msg.type === 'sync-chunk') this.handleChunk(peerId, msg.chunk);
      } catch (e) { this.emit('error', peerId, e); }
    });
    this.dcManager.on('open', (peerId: PeerId) => {
      this.connectedPeers.add(peerId);
      this.emit('peer-connected', peerId);
      this.flushQueue();
    });
    this.dcManager.on('close', (peerId: PeerId) => {
      this.connectedPeers.delete(peerId);
      this.syncInProgress.delete(peerId);
      this.emit('peer-disconnected', peerId);
    });
  }

  // ISyncEngine: sync - 向后兼容
  async sync(peerId: PeerId, sharedSecret?: string): Promise<SyncResult> {
    const startTime = Date.now();
    const result: SyncResult = { success: false, pushed: 0, pulled: 0, conflicts: 0, conflictsResolved: 0, duration: 0 };
    try {
      if (!this.isConnected(peerId)) {
        this.queueOperation({ id: crypto.randomUUID(), type: 'SYNC', peerId, timestamp: Date.now(), retryCount: 0 });
        throw new Error('E_SYNC_001: Peer offline, queued');
      }
      this.syncInProgress.add(peerId);
      if (sharedSecret) {
        const key = this.dcManager.deriveKey(sharedSecret);
        this.emit('key-derived', peerId, key.length);
      }
      await this.sendManifest(peerId);
      const pushRes = await this.push(peerId);
      const pullRes = await this.pull(peerId);
      result.pushed = pushRes.pushed;
      result.pulled = pullRes.pulled;
      result.success = true;
      this.emit('sync-complete', peerId, result);
      // CRDT合并后自动持久化
      await this.persistState();
    } catch (error: any) {
      result.error = error.message;
      this.emit('sync-error', peerId, error);
    } finally {
      result.duration = Date.now() - startTime;
      this.syncInProgress.delete(peerId);
    }
    return result;
  }

  // ISyncEngine: push - 向后兼容
  async push(peerId: PeerId, chunkIds?: string[]): Promise<PushResult> {
    if (!this.isConnected(peerId)) {
      this.queueOperation({ id: crypto.randomUUID(), type: 'PUSH', peerId, timestamp: Date.now(), retryCount: 0 });
      throw new Error('E_SYNC_002: Peer offline, push queued');
    }
    let pushed = 0;
    const chunksToPush = chunkIds 
      ? chunkIds.map(id => this.chunkStorage.get(id)).filter(Boolean)
      : Array.from(this.chunkStorage.values());
    for (const chunk of chunksToPush) {
      if (!chunk) continue;
      this.dcManager.sendText(JSON.stringify({ 
        type: 'sync-chunk', 
        chunk: { ...chunk, data: chunk.data.toString('base64'), crdtState: chunk.crdtState ? Array.from(chunk.crdtState) : undefined }
      }), peerId);
      pushed++;
      this.emit('chunk-pushed', peerId, chunk.id);
    }
    // 推送后持久化队列状态
    await this.queueDb.saveQueue(this.offlineQueue);
    return { pushed, persisted: true };
  }

  // ISyncEngine: pull - 向后兼容
  async pull(peerId: PeerId, chunkIds?: string[]): Promise<PullResult> {
    if (!this.isConnected(peerId)) {
      this.queueOperation({ id: crypto.randomUUID(), type: 'PULL', peerId, timestamp: Date.now(), retryCount: 0 });
      throw new Error('E_SYNC_003: Peer offline, pull queued');
    }
    this.dcManager.sendText(JSON.stringify({ type: 'sync-request', chunkIds }), peerId);
    return new Promise((resolve) => {
      let pulled = 0, merged = 0;
      const handler = ({ peerId: from, text }: any) => {
        if (from !== peerId) return;
        try {
          const msg = JSON.parse(text);
          if (msg.type === 'sync-chunk') { 
            const conflict = this.handleChunkWithCRDT(peerId, msg.chunk);
            if (conflict) merged++;
            pulled++;
          }
          if (msg.type === 'sync-complete') { 
            this.dcManager.off('text-message', handler); 
            resolve({ pulled, merged }); 
          }
        } catch (e) {}
      };
      this.dcManager.on('text-message', handler);
      setTimeout(() => { this.dcManager.off('text-message', handler); resolve({ pulled, merged }); }, 15000);
    });
  }

  // CRDT冲突解决 - 自动合并
  private handleChunkWithCRDT(peerId: PeerId, chunkData: any): boolean {
    const newChunk: Chunk = {
      id: chunkData.id,
      data: Buffer.from(chunkData.data, 'base64'),
      mtime: chunkData.mtime,
      hash: chunkData.hash,
      vectorClock: chunkData.vectorClock || {},
      crdtState: chunkData.crdtState ? new Uint8Array(chunkData.crdtState) : undefined
    };
    const existing = this.chunkStorage.get(newChunk.id);
    if (existing && existing.hash !== newChunk.hash) {
      // 使用CRDT引擎合并
      const merged = this.crdtEngine.merge(existing, newChunk);
      this.chunkStorage.set(merged.id, merged);
      this.emit('chunk-merged', peerId, merged.id);
      return true;
    }
    this.chunkStorage.set(newChunk.id, newChunk);
    this.emit('chunk-received', peerId, newChunk);
    return false;
  }

  // 队列操作 - 持久化到LevelDB
  private async queueOperation(op: SyncOperation): Promise<void> {
    if (this.offlineQueue.length >= this.maxQueueSize) this.offlineQueue.shift();
    this.offlineQueue.push(op);
    await this.queueDb.appendOperation(op);
    this.emit('operation-queued', op);
  }

  // 刷新队列 - 恢复后同步
  async flushQueue(): Promise<void> {
    if (this.offlineQueue.length === 0) return;
    const toFlush = [...this.offlineQueue];
    this.offlineQueue = [];
    for (const op of toFlush) {
      if (!this.isConnected(op.peerId)) {
        if (op.retryCount < 3) { op.retryCount++; this.offlineQueue.push(op); }
        else this.emit('queue-failed', op, new Error('Max retries'));
        continue;
      }
      try {
        if (op.type === 'SYNC') await this.sync(op.peerId);
        else if (op.type === 'PUSH') await this.push(op.peerId);
        else if (op.type === 'PULL') await this.pull(op.peerId);
        this.emit('queue-flushed', op);
      } catch (e: any) {
        if (op.retryCount < 3) { op.retryCount++; this.offlineQueue.push(op); }
      }
    }
    await this.queueDb.saveQueue(this.offlineQueue);
  }

  // 启动时恢复队列
  private async restoreQueue(): Promise<void> {
    try {
      this.offlineQueue = await this.queueDb.getQueue();
      this.emit('queue-restored', this.offlineQueue.length);
    } catch (e) {
      this.offlineQueue = [];
    }
  }

  // 持久化状态
  private async persistState(): Promise<void> {
    await this.queueDb.saveQueue(this.offlineQueue);
    this.emit('state-persisted');
  }

  // ISyncEngine: 可选冲突回调
  onConflict?: (local: Chunk, remote: Chunk) => Chunk;

  private async sendManifest(peerId: PeerId): Promise<void> {
    const manifest = {
      peerId: this.dcManager.peerId,
      chunks: Array.from(this.chunkStorage.values()).map(c => ({ id: c.id, hash: c.hash, mtime: c.mtime })),
      vectorClock: {}
    };
    this.dcManager.sendText(JSON.stringify({ type: 'sync-manifest', manifest }), peerId);
  }

  private handleManifest(peerId: PeerId, manifest: any): void {
    this.emit('manifest-received', peerId, manifest);
  }

  private handleSyncRequest(peerId: PeerId, theirChunkIds: ChunkId[]): void {
    const missing = Array.from(this.chunkStorage.values()).filter(c => !theirChunkIds?.includes(c.id));
    for (const chunk of missing) {
      this.dcManager.sendText(JSON.stringify({ 
        type: 'sync-chunk', 
        chunk: { ...chunk, data: chunk.data.toString('base64'), crdtState: chunk.crdtState ? Array.from(chunk.crdtState) : undefined }
      }), peerId);
    }
    this.dcManager.sendText(JSON.stringify({ type: 'sync-complete' }), peerId);
  }

  private handleChunk(peerId: PeerId, chunkData: any): void {
    this.handleChunkWithCRDT(peerId, chunkData);
  }

  isConnected(peerId: PeerId): boolean { return this.connectedPeers.has(peerId); }
  addChunk(chunk: Chunk): void { this.chunkStorage.set(chunk.id, chunk); }
  getChunk(id: ChunkId): Chunk | undefined { return this.chunkStorage.get(id); }
  
  // 获取CRDT引擎（用于测试验证）
  getCrdtEngine(): ICrdtEngine { return this.crdtEngine; }
  // 获取队列DB（用于测试验证）
  getQueueDb(): IQueueDb { return this.queueDb; }
}

export { SyncResult, Chunk, SyncOperation, PushResult, PullResult, ICrdtEngine, IQueueDb };
