# PHASE1-CHUNK-FORMAT: Chunk 文件格式规范

> **工单**: P1-03/05  
> **日期**: 2026-02-22  
> **状态**: ✅ 已完成

---

## 1. 文件格式 (.hctx v3)

### 1.1 整体结构

```
┌─────────────────────────────────────────────────────────┐
│                    Chunk File (.hctx)                    │
├─────────────────────────────────────────────────────────┤
│  Header (128 bytes)                                      │
│  ├── Magic (4 bytes)     │ "HCTX" (0x48435458)          │
│  ├── Version (1 byte)    │ 0x03                         │
│  ├── Flags (1 byte)      │ 0x01=compressed              │
│  ├── Reserved (2 bytes)  │ ─                            │
│  ├── Original Size (8)   │ 原始数据大小                 │
│  ├── Stored Size (8)     │ 存储大小                     │
│  ├── Data Hash (32)      │ SHA256                       │
│  ├── Meta Length (4)     │ 元数据JSON长度               │
│  └── Reserved (64)       │ 未来扩展                     │
├─────────────────────────────────────────────────────────┤
│  Metadata (variable)     │ JSON格式                     │
├─────────────────────────────────────────────────────────┤
│  Payload (variable)      │ 实际数据                     │
└─────────────────────────────────────────────────────────┘
```

### 1.2 目录结构

```
~/.hajimi/storage/v3/
└── chunks/
    ├── 00/                  # SimHash前缀00
    │   ├── 0011223344556677.hctx
    │   └── ...
    ├── 01/                  # SimHash前缀01
    │   └── ...
    └── ff/                  # SimHash前缀ff
        └── ...
```

---

## 2. 核心API

```javascript
class ChunkStorage {
  // 写入Chunk
  async writeChunk(simhash, data, metadata) → { simhash, path, size, hash }
  
  // 读取Chunk
  async readChunk(simhash) → { data, metadata, size, simhash }
  
  // 删除Chunk
  async deleteChunk(simhash) → boolean
  
  // 检查存在
  async exists(simhash) → boolean
  
  // 获取统计
  async getStats() → { totalChunks, byPrefix }
}
```

---

## 3. 自测结果

### 3.1 CHUNK-001: 读写一致性 ✅

```javascript
const data = Buffer.from('Hello, Hajimi!');
await storage.writeChunk(simhash, data);
const result = await storage.readChunk(simhash);
assert(result.data.equals(data));  // ✅
```

### 3.2 CHUNK-002: 元数据完整 ✅

支持嵌套JSON元数据完整保存和读取。

### 3.3 CHUNK-003: 大文件支持 ✅

2MB文件读写测试通过，支持>1MB文件。

### 3.4 CHUNK-004: 并发写入 ✅

10个并发写入无冲突，全部可读。

### 3.5 CHUNK-005: 不存在文件 ✅

不存在文件返回`null`，不抛出异常。

---

## 4. P4检查表

| 检查点 | 覆盖 | 用例ID | 状态 |
|--------|------|--------|------|
| CF | ✅ | CHUNK-001,002 | 通过 |
| RG | ✅ | CHUNK-005 | 通过 |
| NG | ✅ | CHUNK-003 | 通过 |
| UX | ✅ | CHUNK-004 | 通过 |
| E2E | ✅ | CHUNK-001 | 通过 |
| High | ✅ | CHUNK-001 | 通过 |
| 字段完整性 | ✅ | 全部5项 | 通过 |
| 需求映射 | ✅ | P1-03 | 通过 |
| 执行结果 | ✅ | 全部通过 | 通过 |
| 范围边界 | ✅ | 仅Chunk层 | 通过 |

**P4检查**: 10/10 ✅

---

## 5. 交付物

| 文件 | 路径 | 说明 |
|------|------|------|
| 核心实现 | `src/storage/chunk.js` | ChunkStorage类 |
| 单元测试 | `src/test/chunk.test.js` | 7项测试 |
| 格式文档 | `docs/PHASE1-CHUNK-FORMAT.md` | 本文档 |

---

**工单状态**: A级通过 ✅
