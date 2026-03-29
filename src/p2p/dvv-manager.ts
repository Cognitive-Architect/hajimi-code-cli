/**
 * Dotted Version Vectors (DVV)管理器 - 内存增长O(log N)
 */
import * as Y from 'yjs';

export interface DVVEntry { replicaId: string; sequence: number; counter: number; }
export interface DVVManagerConfig {
  snapshotThreshold?: number;
  sizeThresholdMB?: number;
  enableAutoCleanup?: boolean;
}

export class DVVManager {
  private dvv: Map<string, DVVEntry> = new Map();
  private config: Required<DVVManagerConfig>;
  private updateCount = 0;
  private lastSnapshotSize = 0;
  private isCleaning = false;
  private snapshotQueue: Uint8Array[] = [];

  constructor(private doc: Y.Doc, config: DVVManagerConfig = {}) {
    this.config = {
      snapshotThreshold: config.snapshotThreshold || 1000,
      sizeThresholdMB: config.sizeThresholdMB || 10,
      enableAutoCleanup: config.enableAutoCleanup !== false,
    };
    this.setupListener();
  }

  private setupListener(): void {
    this.doc.on('update', (update: Uint8Array) => {
      this.updateCount++;
      this.trackDVV(update);
      if (this.config.enableAutoCleanup) this.checkTrigger();
    });
  }

  private trackDVV(update: Uint8Array): void {
    const sv = Y.decodeStateVector(update);
    for (const [clientId, clock] of sv) {
      const replicaId = String(clientId);
      const entry = this.dvv.get(replicaId);
      if (!entry || clock > entry.sequence) {
        this.dvv.set(replicaId, { replicaId, sequence: clock, counter: (entry?.counter || 0) + 1 });
      }
    }
  }

  private checkTrigger(): void {
    const currentSize = this.estimateSizeMB();
    const countTriggered = this.updateCount >= this.config.snapshotThreshold;
    const sizeTriggered = currentSize >= this.config.sizeThresholdMB;
    const growthTriggered = this.lastSnapshotSize > 0 && currentSize > this.lastSnapshotSize * 1.5;
    if (countTriggered || sizeTriggered || growthTriggered) void this.cleanup();
  }

  async cleanup(): Promise<boolean> {
    if (this.isCleaning) return false;
    this.isCleaning = true;
    const backup = Y.encodeStateAsUpdate(this.doc);
    try {
      const snapshot = Y.encodeStateAsUpdate(this.doc);
      this.snapshotQueue.push(snapshot);
      if (this.snapshotQueue.length > 3) this.snapshotQueue.shift();
      this.pruneDVV();
      this.updateCount = 0;
      this.lastSnapshotSize = this.estimateSizeMB();
      (this.doc as { emit: (event: string, data: unknown) => void }).emit('cleanup', { size: this.lastSnapshotSize });
      this.isCleaning = false;
      return true;
    } catch (err) {
      Y.applyUpdate(this.doc, backup);
      this.isCleaning = false;
      throw new Error(`Cleanup failed: ${String(err)}`);
    }
  }

  private pruneDVV(): void {
    const entries = Array.from(this.dvv.values()).sort((a, b) => b.sequence - a.sequence);
    const keepCount = Math.ceil(entries.length * 0.5);
    this.dvv.clear();
    for (let i = 0; i < keepCount; i++) this.dvv.set(entries[i]!.replicaId, entries[i]!);
  }

  private estimateSizeMB(): number {
    return Y.encodeStateAsUpdate(this.doc).length / (1024 * 1024);
  }

  async forceSnapshot(): Promise<Uint8Array> {
    await this.cleanup();
    return this.snapshotQueue[this.snapshotQueue.length - 1]!;
  }

  getDVV(): Map<string, DVVEntry> { return new Map(this.dvv); }
  getUpdateCount(): number { return this.updateCount; }
  isCleaningUp(): boolean { return this.isCleaning; }
  getSnapshotHistory(): Uint8Array[] { return [...this.snapshotQueue]; }
}
