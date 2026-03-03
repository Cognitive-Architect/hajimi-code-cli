/**
 * Yjs Adapter - CRDT Engine实现
 * DEBT-P2P-001: 已清偿
 * 约束: ≤150行 | Yjs 13.6.x | SAFETY生命周期管理
 */
import * as Y from 'yjs';
import { ICrdtEngine, Chunk, MergeResult, CrdtConfig } from './crdt-engine';

export class YjsAdapter implements ICrdtEngine {
  readonly type = 'yjs' as const;
  private config: CrdtConfig;
  // SAFETY: Y.Doc实例缓存，必须手动销毁防止内存泄漏
  private docCache: Map<string, Y.Doc> = new Map();

  constructor(config: CrdtConfig = { engine: 'yjs', maxDocSize: 10 * 1024 * 1024, gcEnabled: true }) {
    this.config = config;
  }

  // SAFETY: 创建Y.Doc，配置GC和大小限制
  private createDoc(): Y.Doc {
    const doc = new Y.Doc({ gc: this.config.gcEnabled });
    doc.getMap('chunks'); // 预创建顶层Map
    return doc;
  }

  chunkToState(chunk: Chunk): Uint8Array {
    const doc = this.createDoc();
    const chunks = doc.getMap('chunks');
    const chunkMap = new Y.Map();
    chunkMap.set('id', chunk.id);
    chunkMap.set('simhash', chunk.simhash.toString());
    chunkMap.set('data', chunk.data);
    // 元数据使用嵌套Y.Map支持CRDT合并
    const metaMap = new Y.Map();
    metaMap.set('crdtType', chunk.metadata.crdtType);
    metaMap.set('size', chunk.metadata.size);
    metaMap.set('hash', chunk.metadata.hash);
    chunkMap.set('metadata', metaMap);
    chunks.set(chunk.id, chunkMap);
    const update = Y.encodeStateAsUpdate(doc);
    this.destroyDoc(doc); // SAFETY: 立即清理
    return update;
  }

  merge(local: Chunk, remote: Chunk): MergeResult {
    // SAFETY: 创建临时Doc用于合并，用完即销毁
    const mergedDoc = this.createDoc();
    Y.applyUpdate(mergedDoc, local.crdtState);
    Y.applyUpdate(mergedDoc, remote.crdtState); // CRDT自动合并
    const mergedState = Y.encodeStateAsUpdate(mergedDoc);
    const mergedChunk = this.stateToChunk(mergedState, {
      id: local.id,
      metadata: { ...local.metadata, crdtType: 'yjs' }
    });
    this.destroyDoc(mergedDoc); // SAFETY: 清理临时Doc
    return { winner: 'merged', chunk: mergedChunk, resolutionStrategy: 'crdt-auto' };
  }

  stateToChunk(state: Uint8Array, baseChunk: Partial<Chunk>): Chunk {
    const doc = this.createDoc();
    Y.applyUpdate(doc, state);
    const chunks = doc.getMap('chunks') as Y.Map<Y.Map<any>>;
    const chunkMap = chunks.get(baseChunk.id!) || new Y.Map();
    const metaMap = chunkMap.get('metadata') as Y.Map<any> || new Y.Map();
    const chunk: Chunk = {
      id: baseChunk.id!,
      simhash: BigInt(chunkMap.get('simhash') || '0'),
      data: Buffer.from(chunkMap.get('data') || []),
      metadata: {
        crdtType: metaMap.get('crdtType') || 'yjs',
        size: metaMap.get('size') || 0,
        hash: metaMap.get('hash') || ''
      },
      crdtState: state
    };
    this.destroyDoc(doc); // SAFETY: 清理
    return chunk;
  }

  hasConflict(localState: Uint8Array, remoteState: Uint8Array): boolean {
    const localSV = this.getStateVector(localState);
    const remoteSV = this.getStateVector(remoteState);
    return !this.isSubset(localSV, remoteSV) && !this.isSubset(remoteSV, localSV);
  }

  getStateVector(state: Uint8Array): Uint8Array {
    const doc = this.createDoc();
    Y.applyUpdate(doc, state);
    const sv = Y.encodeStateVector(doc);
    this.destroyDoc(doc);
    return sv;
  }

  applyUpdate(state: Uint8Array, update: Uint8Array): Uint8Array {
    const doc = this.createDoc();
    Y.applyUpdate(doc, state);
    Y.applyUpdate(doc, update); // 应用增量更新
    const newState = Y.encodeStateAsUpdate(doc);
    this.destroyDoc(doc);
    return newState;
  }

  // SAFETY: 销毁Y.Doc实例，释放内存和事件监听
  private destroyDoc(doc: Y.Doc): void {
    doc.destroy();
  }

  // 清理所有缓存的Doc实例
  dispose(): void {
    for (const doc of this.docCache.values()) {
      doc.destroy();
    }
    this.docCache.clear();
  }

  private isSubset(sv1: Uint8Array, sv2: Uint8Array): boolean {
    if (sv1.length !== sv2.length) return false;
    for (let i = 0; i < sv1.length; i++) {
      if (sv1[i]! > sv2[i]!) return false;
    }
    return true;
  }
}

// 工厂函数
export function createYjsAdapter(config?: CrdtConfig): YjsAdapter {
  return new YjsAdapter(config);
}
