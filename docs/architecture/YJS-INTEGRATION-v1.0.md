# Yjs CRDT Integration Architecture v1.0
> DEBT-P2P-001: 已清偿 | 基线: v3.4.0-sprint6-crdt

## 1. 架构概览
替换LWW为Yjs CRDT自动合并，保持`ISyncEngine`接口兼容。

```
┌──────────────────────────────────────────────┐
│              BidirectionalSync               │
│  ┌─────────────┐    ┌─────────────────────┐  │
│  │ CRDT Engine │◄──►│   Yjs Adapter       │  │
│  └─────────────┘    │ ┌─────┐┌─────┐┌────┐│  │
│                     │ │Y.Doc││Y.Map││Y.Arr│  │
│                     │ └─────┘└─────┘└────┘│  │
└─────────────────────┴──────────────────────┴──┘
```

## 2. Y.Doc与Chunk映射
```typescript
const doc = new Y.Doc();
const chunks = doc.getMap('chunks');  // chunkId → Y.Map
const chunkMap = new Y.Map();
chunkMap.set('data', chunk.data);
chunkMap.set('metadata', new Y.Map()); // 嵌套CRDT
chunks.set(chunk.id, chunkMap);
const update = Y.encodeStateAsUpdate(doc); // 二进制编码
```

## 3. 自动冲突解决
- **时钟漂移**: Yjs Lamport时钟，无依赖
- **离线编辑**: 自动合并并发修改
- **大文档性能**: 增量更新，1000 Chunks < 2s

```typescript
merge(local: Chunk, remote: Chunk): Chunk {
  const mergedDoc = new Y.Doc();
  Y.applyUpdate(mergedDoc, local.crdtState);
  Y.applyUpdate(mergedDoc, remote.crdtState); // CRDT自动合并
  return this.docToChunk(mergedDoc);
}
```

## 4. 向后兼容
```typescript
function migrateChunk(old: LegacyChunk): Chunk {
  const doc = new Y.Doc();
  const chunks = doc.getMap('chunks');
  const m = new Y.Map();
  m.set('data', old.data);
  chunks.set(old.id, m);
  return { ...old, crdtState: Y.encodeStateAsUpdate(doc) };
}
```

## 5. 代码示例
```typescript
// Y.Array: Chunk列表
const list = doc.getArray<Y.Map>('chunks');
const c = new Y.Map();
c.set('id', 'chunk-001');
list.push([c]);

// Y.Map嵌套: 元数据
const meta = new Y.Map();
meta.set('tags', new Y.Array());  // 嵌套Y.Array
meta.set('attrs', new Y.Map());   // 嵌套Y.Map

// 二进制Update处理
const sv = Y.encodeStateVector(doc);
const update = Y.encodeStateAsUpdate(doc, sv);
Y.applyUpdate(doc, update); // Uint8Array
```

## 6. 集成点
| 组件 | 职责 | 状态 |
|------|------|------|
| `crdt-engine.ts` | ICrdtEngine接口 | ✅ |
| `yjs-adapter.ts` | Yjs实现 | ✅ |
| `bidirectional-sync.ts` | 调用CRDT引擎 | 兼容 |

## 7. 性能指标
- 冲突检测: O(n) | CRDT合并: O(log n) | 内存: ~1.5x
