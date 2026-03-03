# CRDT Strategy

## Decision: Yjs (Recommended)

| Option | Pros | Cons | Verdict |
|--------|------|------|---------|
| **Yjs** | Mature, fast, good docs, YDB persistence | Extra dependency | ✅ **Selected** |
| Automerge | Better merge semantics, Rust core | Heavier, slower JS | ❌ Rejected |
| Custom | Minimal, tailored to .hctx | High dev cost, risky | ❌ Rejected |

**Rationale:** Yjs has proven scalability, active maintenance, and direct compatibility with binary chunk storage.

## Integration with .hctx Format

```
Yjs Document (YDbf / updates)
    ↓
Binary encoding (Uint8Array)
    ↓
ChunkStorage.writeChunk(simhash, data)
    ↓
.hctx file (magic + metadata + yjs-data)
```

**Key Mapping:**
- Yjs `doc.guid` → Simhash high 64bits
- Yjs state vector → Vector clock for sync
- `.hctx` metadata stores `ydoc` type + version

## Merge Logic

```typescript
import * as Y from 'yjs';

function mergeChunks(local: Buffer, remote: Buffer): Buffer {
  const ydocLocal = new Y.Doc();
  const ydocRemote = new Y.Doc();
  
  Y.applyUpdate(ydocLocal, new Uint8Array(local));
  Y.applyUpdate(ydocRemote, new Uint8Array(remote));
  
  // Automatic CRDT merge
  const mergedUpdate = Y.encodeStateAsUpdate(ydocLocal, Y.encodeStateVector(ydocRemote));
  return Buffer.from(mergedUpdate);
}
```

## Testing Strategy

**Conflict Tests:**
- Concurrent edits on same chunk → verify merge consistency
- Offline divergent edits → verify eventual consistency
- Large document sync (>10MB) → verify performance

**Merge Tests:**
- Property-based testing with fast-check
- Fuzz testing: random ops, verify no data loss
- Cross-device sync simulation

```bash
npm test -- --grep "crdt-merge"
npm test -- --grep "conflict-resolution"
```

## Debt Declaration

- **DEBT-P2P-001**: CRDT选型风险（Yjs/Automerge/自研权衡，可能需返工）
- **DEBT-P2P-002**: NAT穿透失败fallback策略（TURN服务器依赖）
- **DEBT-P2P-003**: 大规模分片同步性能未验证（>1000 chunks）

**Mitigation:**
- DEBT-P2P-001: Yjs选型可回退，抽象CRDT接口
- DEBT-P2P-002: TURN配置为optional，纯LAN无需TURN
- DEBT-P2P-003: Benchmark suite planned for Sprint 6
