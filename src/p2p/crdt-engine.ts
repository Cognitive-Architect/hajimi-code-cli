/**
 * CRDT Engine Interface - Sprint 6
 * DEBT-P2P-001: 已清偿（纯CRDT实现，无timestamp比较）
 * 约束: ≤80行 | 自动合并 | 无LWW
 */

export interface Chunk {
  id: string;
  simhash: bigint;
  data: Buffer;
  metadata: { crdtType: 'yjs' | 'raw'; size: number; hash: string };
  crdtState: Uint8Array; // Yjs update二进制
}

export interface MergeResult {
  winner: 'merged'; // CRDT总是返回merged
  chunk: Chunk;
  resolutionStrategy: 'crdt-auto';
}

/** CRDT引擎接口契约 - 实现必须提供自动合并能力（非winner选择） */
export interface ICrdtEngine {
  readonly type: 'yjs' | 'automerge';
  /** 将Chunk转换为CRDT内部表示 */
  chunkToState(chunk: Chunk): Uint8Array;
  /** 自动合并两个并发修改的Chunk，返回包含双方修改的结果 */
  merge(local: Chunk, remote: Chunk): MergeResult;
  /** 从CRDT状态恢复Chunk */
  stateToChunk(state: Uint8Array, baseChunk: Partial<Chunk>): Chunk;
  /** 检测两个Chunk是否存在冲突（非LWW判断） */
  hasConflict(localState: Uint8Array, remoteState: Uint8Array): boolean;
  /** 获取State Vector用于增量同步 */
  getStateVector(state: Uint8Array): Uint8Array;
  /** 应用增量更新 */
  applyUpdate(state: Uint8Array, update: Uint8Array): Uint8Array;
}

/** CRDT引擎配置 */
export interface CrdtConfig {
  engine: 'yjs';
  maxDocSize: number; // 最大文档大小(bytes)
  gcEnabled: boolean; // Yjs垃圾回收
}
