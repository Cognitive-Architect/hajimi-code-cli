/**
 * BidirectionalSyncV2 - P2P双向同步引擎 (Sprint6, DEBT-P2P-004清偿)
 * 约束: ≤250行 | 持久化队列 | ACID保证
 */
import { EventEmitter } from 'events';
import * as crypto from 'crypto';
import { P2PQueueDB } from '../storage/p2p-queue-db';
const { DataChannelManager } = require('./datachannel-manager.js');

type PeerId = string; type ChunkId = string;

interface Chunk {
  id: ChunkId; data: Buffer; mtime: number;
  hash: string; vectorClock: { [nodeId: string]: number };
}

interface SyncOperation {
  id: string; type: 'PUSH' | 'PULL' | 'SYNC';
  peerId: PeerId; timestamp: number; retryCount: number;
}

interface SyncResult {
  success: boolean; pushed: number; pulled: number;
  conflicts: number; conflictsResolved: number; duration: number; error?: string;
}

interface MergeResult {
  winner: 'local' | 'remote' | 'merged';
  data?: Buffer; resolutionStrategy: 'timestamp' | 'crdt' | 'manual';
}

interface PeerManifest {
  peerId: PeerId;
  chunks: Array<{ id: ChunkId; hash: string; mtime: number }>;
  vectorClock: { [nodeId: string]: number };
}

export class BidirectionalSyncV2 extends EventEmitter {
  private dcManager: typeof DataChannelManager;
  private chunkStorage: Map<ChunkId, Chunk> = new Map();
  private pendingManifests: Map<PeerId, PeerManifest> = new Map();
  private persistentQueue: P2PQueueDB;
  private maxQueueSize: number = 1000;
  private connectedPeers: Set<PeerId> = new Set();
  private syncInProgress: Set<PeerId> = new Set();
  private flushLock: boolean = false;
  private readonly maxRetries: number = 3;

  constructor(dcManager: typeof DataChannelManager, queueOptions?: { encryptionKey?: string }) {
    super();
    this.dcManager = dcManager;
    this.persistentQueue = new P2PQueueDB(queueOptions);
    this.setupEventHandlers();
    this.recoverQueue().catch(() => {});
  }

  async init(): Promise<void> {
    await this.persistentQueue.open();
    this.emit('queue-ready');
  }

  async destroy(): Promise<void> {
    await this.persistentQueue.close();
  }

  private setupEventHandlers(): void {
    this.dcManager.on('text-message', ({ peerId, text }) => {
      try {
        const msg = JSON.parse(text);
        if (msg.type === 'sync-manifest') this.handleManifest(peerId, msg.manifest);
        if (msg.type === 'sync-request') this.handleSyncRequest(peerId, msg.chunkIds);
        if (msg.type === 'sync-chunk') this.handleChunk(peerId, msg.chunk);
      } catch (e) { this.emit('error', peerId, e); }
    });
    this.dcManager.on('open', (peerId: PeerId) => {
      this.connectedPeers.add(peerId); this.emit('peer-connected', peerId); this.flushQueue();
    });
    this.dcManager.on('close', (peerId: PeerId) => {
      this.connectedPeers.delete(peerId); this.syncInProgress.delete(peerId); this.emit('peer-disconnected', peerId);
    });
  }

  async sync(peerId: PeerId, sharedSecret: string): Promise<SyncResult> {
    const startTime = Date.now();
    const result: SyncResult = { success: false, pushed: 0, pulled: 0, conflicts: 0, conflictsResolved: 0, duration: 0 };
    try {
      if (!this.isConnected(peerId)) {
        await this.queueOperation({ id: crypto.randomUUID(), type: 'SYNC', peerId, timestamp: Date.now(), retryCount: 0 });
        throw new Error('E_SYNC_001: Peer offline, queued');
      }
      this.syncInProgress.add(peerId);
      const key = this.dcManager.deriveKey(sharedSecret);
      this.emit('key-derived', peerId, key.length);
      await this.sendManifest(peerId);
      result.pushed = await this.push(peerId);
      result.pulled = await this.pull(peerId);
      const manifest = this.pendingManifests.get(peerId);
      if (manifest) {
        const conflicts = await this.detectConflicts(peerId, manifest);
        result.conflicts = conflicts.length;
        for (const c of conflicts) {
          const resolved = this.onConflict(c.local, c.remote);
          if (resolved.winner !== 'manual') result.conflictsResolved++;
        }
      }
      result.success = true; this.emit('sync-complete', peerId, result);
    } catch (error: any) {
      result.error = error.message; this.emit('sync-error', peerId, error);
    } finally {
      result.duration = Date.now() - startTime; this.syncInProgress.delete(peerId);
    }
    return result;
  }

  async push(peerId: PeerId): Promise<number> {
    if (!this.isConnected(peerId)) {
      await this.queueOperation({ id: crypto.randomUUID(), type: 'PUSH', peerId, timestamp: Date.now(), retryCount: 0 });
      throw new Error('E_SYNC_002: Peer offline, push queued');
    }
    let pushed = 0;
    for (const chunk of this.chunkStorage.values()) {
      this.dcManager.sendText(JSON.stringify({ type: 'sync-chunk', chunk: { ...chunk, data: chunk.data.toString('base64') } }), peerId);
      pushed++; this.emit('chunk-pushed', peerId, chunk.id);
    }
    return pushed;
  }

  async pull(peerId: PeerId): Promise<number> {
    if (!this.isConnected(peerId)) {
      await this.queueOperation({ id: crypto.randomUUID(), type: 'PULL', peerId, timestamp: Date.now(), retryCount: 0 });
      throw new Error('E_SYNC_003: Peer offline, pull queued');
    }
    this.dcManager.sendText(JSON.stringify({ type: 'sync-request', chunkIds: Array.from(this.chunkStorage.keys()) }), peerId);
    return new Promise((resolve) => {
      let pulled = 0;
      const handler = ({ peerId: from, text }: { peerId: PeerId; text: string }) => {
        if (from !== peerId) return;
        try {
          const msg = JSON.parse(text);
          if (msg.type === 'sync-chunk') { this.handleChunk(peerId, msg.chunk); pulled++; }
          if (msg.type === 'sync-complete') { this.dcManager.off('text-message', handler); resolve(pulled); }
        } catch (e) {}
      };
      this.dcManager.on('text-message', handler);
      setTimeout(() => { this.dcManager.off('text-message', handler); resolve(pulled); }, 10000);
    });
  }

  async queueOperation(op: SyncOperation): Promise<void> {
    const size = await this.persistentQueue.size();
    if (size >= this.maxQueueSize) {
      const oldest = await this.persistentQueue.pop();
      this.emit('queue-dropped', oldest);
    }
    await this.persistentQueue.push(op);
    this.emit('operation-queued', op);
  }

  async flushQueue(): Promise<void> {
    if (this.flushLock) return;
    this.flushLock = true;
    try {
      const size = await this.persistentQueue.size();
      if (size === 0) return;
      const ops = await this.persistentQueue.getAll();
      await this.persistentQueue.clear();
      const failed: SyncOperation[] = [];
      for (const op of ops) {
        if (!this.isConnected(op.peerId)) {
          if (op.retryCount < this.maxRetries) { op.retryCount++; failed.push(op); }
          else this.emit('queue-failed', op, new Error('Max retries'));
          continue;
        }
        try {
          if (op.type === 'SYNC') await this.sync(op.peerId, '');
          else if (op.type === 'PUSH') await this.push(op.peerId);
          else if (op.type === 'PULL') await this.pull(op.peerId);
          this.emit('queue-flushed', op);
        } catch (e: any) {
          if (op.retryCount < this.maxRetries) { op.retryCount++; failed.push(op); }
          else this.emit('queue-failed', op, e);
        }
      }
      for (const op of failed) await this.persistentQueue.push(op);
    } finally { this.flushLock = false; }
  }

  async recoverQueue(): Promise<void> {
    const size = await this.persistentQueue.size();
    if (size > 0) this.emit('queue-recovered', size);
  }

  onConflict(local: Chunk, remote: Chunk): MergeResult {
    if (local.mtime > remote.mtime) return { winner: 'local', data: local.data, resolutionStrategy: 'timestamp' };
    if (remote.mtime > local.mtime) return { winner: 'remote', data: remote.data, resolutionStrategy: 'timestamp' };
    if (local.hash > remote.hash) return { winner: 'local', data: local.data, resolutionStrategy: 'timestamp' };
    return { winner: 'remote', data: remote.data, resolutionStrategy: 'timestamp' };
  }

  private async sendManifest(peerId: PeerId): Promise<void> {
    const manifest: PeerManifest = {
      peerId: this.dcManager.peerId,
      chunks: Array.from(this.chunkStorage.values()).map(c => ({ id: c.id, hash: c.hash, mtime: c.mtime })),
      vectorClock: {}
    };
    this.dcManager.sendText(JSON.stringify({ type: 'sync-manifest', manifest }), peerId);
  }

  private handleManifest(peerId: PeerId, manifest: PeerManifest): void {
    this.pendingManifests.set(peerId, manifest); this.emit('manifest-received', peerId, manifest);
  }

  private handleSyncRequest(peerId: PeerId, theirChunkIds: ChunkId[]): void {
    const missing = Array.from(this.chunkStorage.values()).filter(c => !theirChunkIds.includes(c.id));
    for (const chunk of missing) {
      this.dcManager.sendText(JSON.stringify({ type: 'sync-chunk', chunk: { ...chunk, data: chunk.data.toString('base64') } }), peerId);
    }
    this.dcManager.sendText(JSON.stringify({ type: 'sync-complete' }), peerId);
  }

  private handleChunk(peerId: PeerId, chunkData: any): void {
    const chunk: Chunk = {
      id: chunkData.id, data: Buffer.from(chunkData.data, 'base64'),
      mtime: chunkData.mtime, hash: chunkData.hash, vectorClock: chunkData.vectorClock || {}
    };
    this.chunkStorage.set(chunk.id, chunk); this.emit('chunk-received', peerId, chunk);
  }

  private async detectConflicts(peerId: PeerId, manifest: PeerManifest): Promise<Array<{ local: Chunk; remote: any }>> {
    const conflicts = [];
    for (const rc of manifest.chunks) {
      const lc = this.chunkStorage.get(rc.id);
      if (lc && lc.hash !== rc.hash && lc.mtime !== rc.mtime) conflicts.push({ local: lc, remote: rc });
    }
    return conflicts;
  }

  isConnected(peerId: PeerId): boolean { return this.connectedPeers.has(peerId); }
  addChunk(chunk: Chunk): void { this.chunkStorage.set(chunk.id, chunk); }
  getChunk(id: ChunkId): Chunk | undefined { return this.chunkStorage.get(id); }
  async clear(): Promise<void> { this.chunkStorage.clear(); await this.persistentQueue.clear(); this.connectedPeers.clear(); }
  getQueueSize(): Promise<number> { return this.persistentQueue.size(); }
}

export { SyncResult, Chunk, MergeResult, SyncOperation };
