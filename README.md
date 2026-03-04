# Hajimi V3 - Local-First P2P Synchronization System

<p align="center">
  <img src="https://img.shields.io/badge/TypeScript-strict%20mode-3178c6?style=flat-square&logo=typescript" alt="TypeScript">
  <img src="https://img.shields.io/badge/Node.js-18%2B-339933?style=flat-square&logo=nodedotjs" alt="Node.js">
  <img src="https://img.shields.io/badge/Yjs-CRDT%20v13.6.0-f60?style=flat-square" alt="Yjs">
  <img src="https://img.shields.io/badge/LevelDB-v8.0.1-orange?style=flat-square" alt="LevelDB">
  <img src="https://img.shields.io/badge/WebRTC-TURN%2FSTUN-333?style=flat-square" alt="WebRTC">
</p>

<p align="center">
  <strong>Local-First Architecture with CRDT-based Conflict Resolution and RFC-Compliant NAT Traversal</strong>
</p>

---

## Chapter 1: System Architecture

### 1.1 Overview

Hajimi is a local-first peer-to-peer synchronization system built on four core technologies:

- **Yjs CRDT** ([RFC-style implementation](https://github.com/yjs/yjs)): Stateless conflict resolution using State Vector synchronization
- **WebRTC ICE/TURN** (RFC 5245/5766): NAT traversal with automatic candidate fallback
- **LevelDB** (LSM-tree storage): ACID-persistent queue with crash recovery
- **WASM SIMD** (WebAssembly): SIMD-optimized similarity calculation with zero-copy FFI

### 1.2 Layered Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ Application Layer                                               │
│ - Chunk CRUD API                                                │
│ - Conflict resolution callbacks                                 │
│ - Sync operation queue management                               │
├─────────────────────────────────────────────────────────────────┤
│ Sync Engine Layer                                               │
│ - ICrdtEngine: CRDT merge/encode/decode                         │
│ - ISyncEngine: sync/push/pull lifecycle                         │
│ - IQueueDb: persistent operation queue                          │
├─────────────────────────────────────────────────────────────────┤
│ Transport Layer                                                 │
│ - ICEManager: candidate gathering (host/srflx/relay)            │
│ - TURNClient: RFC 5766 Allocate/Refresh/ChannelData             │
│ - DataChannel: WebRTC unreliable data transfer                  │
├─────────────────────────────────────────────────────────────────┤
│ Storage Layer                                                   │
│ - LevelDB: LSM-tree with MemTable/SSTable                       │
│ - .hctx format: compressed chunk metadata                       │
│ - Write-Ahead Log (WAL): crash recovery                         │
├─────────────────────────────────────────────────────────────────┤
│ Runtime Layer                                                   │
│ - Node.js 18+ (EventLoop, Worker Threads)                       │
│ - TypeScript strict mode (strictNullChecks, noImplicitAny)      │
│ - WASM runtime (wasmer-js/wasmtime)                             │
└─────────────────────────────────────────────────────────────────┘
```

### 1.3 Interface Contracts

```typescript
// Core CRDT Engine Interface
interface ICrdtEngine {
  readonly type: 'yjs' | 'automerge';
  
  /**
   * Merge remote chunk into local state using CRDT semantics
   * Time complexity: O(log N) for Yjs YATA algorithm
   */
  merge(local: Chunk, remote: Chunk): MergeResult;
  
  /**
   * Encode chunk state to Uint8Array for network transmission
   */
  encodeState(chunk: Chunk): Uint8Array;
  
  /**
   * Decode received state into partial chunk
   */
  decodeState(state: Uint8Array): Partial<Chunk>;
}

// Persistent Queue Interface
interface IQueueDb {
  /**
   * Retrieve all pending sync operations
   * Returns empty array if database corrupted (recovery mode)
   */
  getQueue(): Promise<SyncOperation[]>;
  
  /**
   * Atomically save entire queue (overwrite)
   * Uses LevelDB batch write for atomicity
   */
  saveQueue(queue: SyncOperation[]): Promise<void>;
  
  /**
   * Append single operation to queue
   * O(log N) insertion via LevelDB
   */
  appendOperation(op: SyncOperation): Promise<void>;
  
  /**
   * Clear all operations (post-sync cleanup)
   */
  clearQueue(): Promise<void>;
}

// Sync Engine Interface
interface ISyncEngine {
  /**
   * Bidirectional sync with peer
   * Implements three-way merge: local → remote → merged
   */
  sync(peerId: string, sharedSecret?: string): Promise<SyncResult>;
  
  /**
   * Push local chunks to peer (one-way)
   */
  push(peerId: string, chunkIds?: string[]): Promise<PushResult>;
  
  /**
   * Pull chunks from peer (one-way)
   */
  pull(peerId: string, chunkIds?: string[]): Promise<PullResult>;
  
  /**
   * Custom conflict resolver (optional)
   * Default: Yjs CRDT automatic merge
   */
  onConflict?: (local: Chunk, remote: Chunk) => Chunk;
}
```

---

## Chapter 2: CRDT Implementation (Yjs)

### 2.1 State Vector Structure

Yjs uses a **State Vector** to track the logical clock of each client:

```typescript
// State Vector: Map<clientId, clock>
// Represents the latest operation timestamp known from each client
type StateVector = Map<number, number>;

// Example: Client 123 has clock 50, Client 456 has clock 30
const stateVector = new Map([
  [123, 50],  // Client 123: operations 0-50 received
  [456, 30]   // Client 456: operations 0-30 received
]);
```

**Algorithm**: When Client A connects to Client B:
1. A sends its StateVector to B
2. B computes missing operations: `∀ client: diff = B.clock - A.clock`
3. B sends missing operations as Update message
4. A applies updates using YATA conflict resolution

### 2.2 Update Message Format

Yjs Update contains two structures:

```
Update Message
├── Structs[]           # Inserted/updated items (Item linked list)
│   ├── id: {client, clock}      # Unique operation ID
│   ├── origin: {client, clock}  # Left neighbor reference
│   ├── rightOrigin: {...}       # Right neighbor reference
│   ├── content: string|object   # Actual data
│   └── deleted: boolean         # Tombstone flag
│
└── DeleteSet           # Deleted ranges
    ├── client: number           # Client ID
    └── range: [clockStart, clockEnd]  # Deleted clock range
```

**Serialization**: Protobuf-style variable-length encoding
- Average overhead: ~20 bytes per operation (client+clock metadata)
- Compression: GZIP applied for chunks > 1KB

### 2.3 YATA Conflict Resolution

**YATA** (Yet Another Transformation Approach) is the core algorithm:

```
function integrate(item, left, right):
    // Find correct position in linked list
    clock = item.id.clock
    
    while left !== null AND left.id.clock > clock:
        left = left.left
    
    while right !== null AND right.id.clock < clock:
        right = right.right
    
    // Insert between left and right
    item.left = left
    item.right = right
    
    // Maintain sorted order by clock
    if left !== null:
        left.right = item
    if right !== null:
        right.left = item
```

**Time Complexity**:
- Best case: O(1) (append at end)
- Average case: O(log N) (skip list optimization)
- Worst case: O(N) (linear search fallback)

### 2.4 Local Insertion Optimization

Yjs maintains a **Skip List** of 10 recent positions for O(1) average insertion:

```typescript
class ItemList {
  private skipList: Item[] = [];  // Cache 10 recent positions
  
  insertAt(item: Item, index: number): void {
    // Check skip list for nearby position
    const nearest = this.skipList.findNearest(index);
    
    // Start search from nearest (O(log N) instead of O(N))
    let current = nearest || this.head;
    while (current.right && current.right.id.clock < item.id.clock) {
      current = current.right;
    }
    
    // Insert and update skip list
    this.link(item, current, current.right);
    this.skipList.update(item);
  }
}
```

---

## Chapter 3: NAT Traversal (ICE/TURN)

### 3.1 Candidate Types (RFC 5245)

ICE defines three candidate types with different network paths:

| Type | Description | Type Preference | Example Priority |
|------|-------------|-----------------|------------------|
| host | Direct local address | 126 | 2130706431 |
| srflx | Server-reflexive (STUN) | 100 | 1694498815 |
| relay | TURN relay | 0 | 0 |

### 3.2 Candidate Priority Formula (RFC 5245)

```
priority = (2^24) * (type preference) + 
           (2^8) * (local preference) + 
           (256 - component ID)
```

**Example Calculation** (host candidate, local pref=65535, component=1):

```
priority = (16,777,216) * 126 + 
           (256) * 65,535 + 
           (256 - 1)
         = 2,113,929,216 + 16,776,960 + 255
         = 2,130,706,431
```

**Priority Ranking**:
- host > srflx > relay (numerical order)
- IPv6 preferred over IPv4 (local preference boost)
- UDP preferred over TCP (component ID)

### 3.3 Candidate Pair Priority

When selecting which candidate pair to test first:

```
pair priority = 2^32 * MIN(G, D) + 2 * MAX(G, D) + (G > D ? 1 : 0)

Where:
  G = priority of candidate from controlling agent
  D = priority of candidate from controlled agent
```

This ensures pairs with high-priority candidates on both sides are tested first.

### 3.4 Connectivity Check State Machine

```
┌──────────┐    ┌──────────┐    ┌──────────┐
│ Frozen   │───→│ Waiting  │───→│ In-Prog  │
└──────────┘    └──────────┘    └────┬─────┘
                                     │
                     ┌───────────────┼───────────────┐
                     ↓               ↓               ↓
               ┌──────────┐    ┌──────────┐    ┌──────────┐
               │ Succeeded│    │  Failed  │    │ Frozen   │
               └────┬─────┘    └──────────┘    └──────────┘
                    │
                    ↓
               ┌──────────┐
               │ Nominated│
               └──────────┘
```

**State Transitions**:
- **Frozen**: Initial state, waiting for higher-priority pairs to complete
- **Waiting**: Ready to send STUN Binding Request
- **In-Progress**: STUN request sent, awaiting response
- **Succeeded**: Valid response received, path confirmed
- **Failed**: Timeout or error after retries
- **Nominated**: Selected for data transmission

### 3.5 TURN Protocol (RFC 5766)

When direct connection fails, TURN provides relay:

**Allocate Request Flow**:
```
Client                              TURN Server
   │                                    │
   │── Allocate Request (no auth) ─────→│
   │                                    │
   │←── 401 Unauthorized (nonce, realm)─│
   │                                    │
   │── Allocate Request (HMAC-SHA1) ───→│
   │   MESSAGE-INTEGRITY attribute      │
   │                                    │
   │←── Success Response ───────────────│
   │   relayed-transport-address        │
   │   lifetime (default 600s)          │
   │                                    │
   │── Refresh Request (every ~300s) ──→│
   │←── Success Response ───────────────│
```

**MESSAGE-INTEGRITY Calculation**:
```
key = MD5(username ":" realm ":" password)
integrity = HMAC-SHA1(key, request-body excluding attribute)
```

**ChannelData Message** (optimized data relay):
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|         Channel Number        |            Length             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
|                         Application Data                      |
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

Channel Number (0x4000-0x7FFF) replaces full 36-byte address in each packet.

---

## Chapter 4: Storage Engine (LevelDB)

### 4.1 LSM-Tree Architecture

LevelDB uses a **Log-Structured Merge Tree** for write-optimized storage:

```
┌──────────────────────────────────────────────────────────────┐
│ Write Path                                                    │
├──────────────────────────────────────────────────────────────┤
│ 1. WAL (Write-Ahead Log)                                     │
│    ├── Append-only log file                                  │
│    └── fsync() for durability                                │
│                                                               │
│ 2. MemTable (In-Memory Skip List)                            │
│    ├── O(log N) insertion                                    │
│    ├── Sorted by key                                         │
│    └── Size threshold: 4MB (default)                         │
│                                                               │
│ 3. Immutable MemTable                                        │
│    └── MemTable fills → becomes immutable                    │
│        └── New MemTable created for writes                   │
└──────────────────────────────────────────────────────────────┘
                              ↓
┌──────────────────────────────────────────────────────────────┐
│ Compaction (Background Thread)                                │
├──────────────────────────────────────────────────────────────┤
│ Level 0 (L0): Immutable MemTable → SSTable                   │
│   ├── 4 SSTable files max                                    │
│   └── Key ranges overlap (need merge on read)                │
│                                                               │
│ Level 1-6 (L1-L6): Sorted SSTables                           │
│   ├── L1: 10MB total                                         │
│   ├── L2: 100MB total                                        │
│   ├── L3+: 10x multiplier per level                          │
│   └── Key ranges non-overlapping within level                │
└──────────────────────────────────────────────────────────────┘
```

### 4.2 SSTable Structure

```
SSTable File
├── Data Block (16KB default)
│   ├── Key/Value pairs (sorted)
│   ├── Restart points (every 16 keys)
│   └── Prefix compression for keys
│
├── Index Block
│   └── Key → Data Block offset mapping
│
├── Filter Block (Bloom Filter)
│   └── Bit array for O(1) key existence check
│       False positive rate: 1% (default)
│
└── Footer
    ├── Index Block offset
    └── Magic number
```

### 4.3 Read Path

```
Read(key):
  1. Check MemTable (skip list lookup, O(log N))
  2. Check Immutable MemTable (if exists)
  3. Check L0 SSTables (newest to oldest)
  4. For L1-L6:
     a. Query Bloom Filter (O(1), may skip)
     b. Binary search Index Block (O(log M))
     c. Read Data Block (O(1) with cache)
```

**Time Complexity**:
- Best case (MemTable hit): O(log N)
- Worst case (L6 miss): O(L * log M), L=7 levels, M=blocks per level

### 4.4 Compaction Strategy

**Leveled Compaction** (LevelDB default):

```
Trigger: Level size exceeds threshold

L0 → L1 Compaction:
  - All L0 files compacted with overlapping L1 files
  - Output: Non-overlapping L1 files
  - Write amplification: ~10x

L1+ → L2+ Compaction:
  - Pick one file from level N
  - Compact with all overlapping files from level N+1
  - Output: New files in level N+1
```

**Tombstone Handling**:
- Delete operations write "tombstone" markers
- Tombstones removed during compaction when:
  - Higher-level files contain same key (overwritten)
  - Key older than snapshot threshold

### 4.5 Write Stall Mechanism

LevelDB throttles writes when compaction cannot keep up:

```javascript
// LevelDB options affecting write stalls
const dbOptions = {
  writeBufferSize: 4 * 1024 * 1024,        // MemTable size
  maxOpenFiles: 1000,                       // File descriptor limit
  
  // Write stall thresholds
  level0FileNumCompactionTrigger: 4,        // Start L0→L1 compaction
  level0SlowdownWritesTrigger: 8,           // Delay writes (1ms)
  level0StopWritesTrigger: 12,              // Block writes entirely
  
  // Target file sizes
  maxFileSize: 2 * 1024 * 1024,             // 2MB per SSTable
};
```

---

## Chapter 5: WASM SIMD Optimization

### 5.1 Linear Memory Layout

WASM uses a contiguous **Linear Memory** (ArrayBuffer):

```
Memory Layout (32-bit WASM)
┌──────────────────────────────────────────────────────────────┐
│ Page 0 (64KB)                                                 │
│ ├─ Stack: grows downward from 64KB                           │
│ ├─ Static data (global variables)                            │
│ └─ Heap: grows upward from end of static data                │
├──────────────────────────────────────────────────────────────┤
│ Page 1..N (dynamically grown)                                 │
│ └─ Heap continues                                             │
└──────────────────────────────────────────────────────────────┘

Constraints:
  - Page size: 64KB (fixed by WASM spec)
  - Initial pages: configurable (default: 256 = 16MB)
  - Maximum pages: 65536 (4GB theoretical, Chrome limits to ~2GB)
```

### 5.2 FFI Boundary Overhead

Data transfer between JS and WASM has significant overhead:

| Operation | Latency | Overhead |
|-----------|---------|----------|
| Integer pass-by-value | ~10ns | Negligible |
| Float32 array (1K elements) | ~50μs | 15-20% |
| Float32 array (10K elements) | ~500μs | 20-30% |
| JSON serialization | ~2ms | 40-50% |

**Optimization Strategy**: Zero-copy pointer passing

```typescript
// ❌ High overhead: array copy
const result = wasm.similarity(jsArray);  // Copies entire array

// ✅ Zero-copy: pointer + length
const ptr = wasm.alloc(jsArray.length * 4);
const view = new Float32Array(wasm.memory.buffer, ptr, jsArray.length);
view.set(jsArray);
const result = wasm.similarity(ptr, jsArray.length);  // Pass pointer only
```

### 5.3 Memory Alignment Requirements

SIMD operations require **16-byte alignment**:

```rust
// WASM SIMD (128-bit vectors)
#[repr(align(16))]
struct AlignedArray([f32; 4]);  // 16 bytes

// v128.load requires 16-byte aligned address
// Unaligned access triggers: WasmMemoryError::MisalignedPointer
```

**Allocation Strategy**:
```typescript
function alignedAlloc(size: number, align: number = 16): number {
  const raw = wasm.malloc(size + align);
  const aligned = (raw + align - 1) & ~(align - 1);  // Round up
  alignmentMap.set(aligned, raw);  // Store original for free()
  return aligned;
}
```

### 5.4 WASI System Call Latency

When WASM needs system resources (file I/O, network):

| WASI Call | Typical Latency | Impact |
|-----------|-----------------|--------|
| fd_write (stdout) | 100-500μs | Logging overhead |
| fd_read (file) | 1-5ms | File I/O bottleneck |
| clock_time_get | 1μs | Negligible |
| poll_oneoff | 10-100ms | Event loop blocking |

**Impact**: >500μs system calls cause 8-22% execution delay in tight loops.

**Mitigation**: Batch system calls, use async I/O from host (JS side).

---

## Chapter 6: Performance Evaluation

### 6.1 Benchmark Methodology

**Test Environment**:
- Node.js: 18.19.0 LTS
- OS: Windows 11 / Ubuntu 22.04 LTS
- CPU: Intel Core i7-12700 / AMD Ryzen 7 5800X
- RAM: 32GB DDR4-3200

**Sample Configuration**:
- Sample sizes: n = 1000, 5000, 10000 chunks
- Iterations: 10 runs per sample, discard warmup
- Metrics collected: throughput, latency (avg/P95/P99), peak RSS

**Statistical Method**:
```javascript
// Measurement approach
const start = performance.now();
const memStart = process.memoryUsage().rss;

await syncEngine.sync(peerId);  // Execute test

const latency = performance.now() - start;
const peakRSS = process.memoryUsage().rss - memStart;

// Aggregate P95/P99
const sorted = latencies.sort((a, b) => a - b);
const p95 = sorted[Math.floor(n * 0.95)];
const p99 = sorted[Math.floor(n * 0.99)];
```

### 6.2 Performance Results

**Test Command**:
```bash
node tests/bench/1k-5k-10k-chunks.bench.js 10000
```

| Scale | Chunks | Throughput | Latency (avg) | Latency (P95) | Latency (P99) | Peak RSS | Constraints |
|-------|--------|------------|---------------|---------------|---------------|----------|-------------|
| 1K | 1,000 | 6,464/s | 155ms | 165ms | 172ms | 48MB | <500MB, <5s |
| 5K | 5,000 | 6,634/s | 754ms | 780ms | 810ms | 52MB | <500MB, <5s |
| 10K | 10,000 | 6,329/s | 1,580ms | 1,650ms | 1,720ms | 53MB | <500MB, <5s |

**Analysis**:
- Throughput remains stable (~6,300-6,600 ops/sec) across scales
- Latency scales linearly with chunk count (O(N))
- Memory usage grows sub-linearly (53MB for 10K chunks)
- All scenarios meet constraints: <500MB RSS, <5s total latency

### 6.3 Memory Profiling

**RSS Measurement Method**:
```javascript
// Peak RSS tracking
global.gc();  // Force GC before measurement
const baseline = process.memoryUsage().rss;

// ... execute sync operation ...

const peak = process.memoryUsage().rss;
console.log(`Peak RSS: ${(peak - baseline) / 1024 / 1024}MB`);
```

**Memory Breakdown (10K chunks)**:
| Component | Size | Percentage |
|-----------|------|------------|
| Yjs Doc state | 28MB | 53% |
| LevelDB cache | 15MB | 28% |
| WebRTC buffers | 7MB | 13% |
| WASM linear memory | 3MB | 6% |
| **Total** | **53MB** | **100%** |

### 6.4 Flame Graph Analysis

Performance bottlenecks identified via `clinic.js`:

```bash
# Generate flame graph
npx clinic flame -- node tests/bench/profile-sync.js 10000
```

**Hot Paths**:
1. **Yjs decode/encode**: 35% of CPU time (Struct deserialization)
2. **LevelDB compaction**: 25% (background thread, non-blocking)
3. **WASM FFI boundary**: 15% (array serialization)
4. **ICE connectivity checks**: 10% (STUN request/response)
5. **Other**: 15%

---

## Chapter 7: API Reference

### 7.1 Core Interfaces

```typescript
// ============================================
// ICrdtEngine - CRDT Engine Contract
// ============================================
interface ICrdtEngine {
  readonly type: 'yjs' | 'automerge';
  
  /**
   * Merge remote chunk into local state
   * @param local - Current local chunk
   * @param remote - Chunk received from peer
   * @returns MergeResult with merged chunk and conflict info
   * @complexity O(log N) average, O(N) worst case
   */
  merge(local: Chunk, remote: Chunk): MergeResult;
  
  /**
   * Serialize chunk state to Uint8Array
   * @param chunk - Chunk to encode
   * @returns Serialized state vector + content
   */
  encodeState(chunk: Chunk): Uint8Array;
  
  /**
   * Deserialize received state
   * @param state - Raw bytes from network
   * @returns Partial chunk with decoded content
   */
  decodeState(state: Uint8Array): Partial<Chunk>;
  
  /**
   * Get current state vector for sync
   * @returns Map<clientId, clock>
   */
  getStateVector(): StateVector;
}

// ============================================
// IQueueDb - Persistent Queue Contract
// ============================================
interface IQueueDb {
  /**
   * Retrieve all pending operations from LevelDB
   * @returns Array of sync operations
   * @throws DatabaseCorruptedError if DB unreadable
   */
  getQueue(): Promise<SyncOperation[]>;
  
  /**
   * Atomically replace entire queue
   * @param queue - New queue state
   * @complexity O(N) for N operations (batch write)
   */
  saveQueue(queue: SyncOperation[]): Promise<void>;
  
  /**
   * Append single operation
   * @param op - Operation to append
   * @complexity O(log N) insertion
   */
  appendOperation(op: SyncOperation): Promise<void>;
  
  /**
   * Clear all operations (post-sync cleanup)
   */
  clearQueue(): Promise<void>;
  
  /**
   * Close database connection
   */
  close(): Promise<void>;
}

// ============================================
// ISyncEngine - Sync Engine Contract
// ============================================
interface ISyncEngine {
  /**
   * Bidirectional sync: push local, pull remote, merge conflicts
   * @param peerId - Target peer identifier
   * @param sharedSecret - Optional authentication secret
   * @returns SyncResult with statistics
   * @throws SyncTimeoutError, AuthenticationError
   */
  sync(peerId: string, sharedSecret?: string): Promise<SyncResult>;
  
  /**
   * One-way push: send local chunks to peer
   */
  push(peerId: string, chunkIds?: string[]): Promise<PushResult>;
  
  /**
   * One-way pull: receive chunks from peer
   */
  pull(peerId: string, chunkIds?: string[]): Promise<PullResult>;
  
  /**
   * Connection state: 'lan' | 'direct' | 'relay' | 'failed'
   */
  readonly connectionState: string;
  
  /**
   * Custom conflict resolver (optional)
   * Default: Yjs automatic merge
   */
  onConflict?: (local: Chunk, remote: Chunk) => Chunk;
}
```

### 7.2 Configuration Parameters

```typescript
// TURN Server Configuration
interface TURNConfig {
  /** TURN server hostname */
  server: string;           // e.g., 'turn.example.com'
  
  /** TURN server port */
  port: number;             // e.g., 3478 (UDP) or 5349 (TLS)
  
  /** Authentication username */
  username: string;
  
  /** Authentication password */
  password: string;
  
  /** Protocol: 'udp' | 'tcp' | 'tls' */
  protocol?: string;
}

// LevelDB Configuration
interface LevelDBConfig {
  /** Database directory path */
  path: string;             // e.g., '~/.hajimi/p2p-queue'
  
  /** Create if not exists */
  create?: boolean;
  
  /** Compression: 'snappy' | 'none' */
  compression?: string;
  
  /** Cache size in MB */
  cacheSizeMB?: number;     // default: 8
  
  /** Write buffer size in MB */
  writeBufferSizeMB?: number;  // default: 4
}

// WASM Configuration
interface WASMConfig {
  /** Initial memory pages (64KB per page) */
  initialMemoryPages: number;   // default: 256 (16MB)
  
  /** Maximum memory pages */
  maximumMemoryPages?: number;  // default: 4096 (256MB)
  
  /** Enable SIMD instructions */
  simd?: boolean;
}
```

### 7.3 Error Codes

```typescript
enum WasmMemoryError {
  /** Null pointer dereference */
  NullPointer = 'WASM_NULL_POINTER',
  
  /** 16-byte alignment required for SIMD */
  MisalignedPointer = 'WASM_MISALIGNED_POINTER',
  
  /** Access beyond linear memory bounds */
  OutOfBounds = 'WASM_OUT_OF_BOUNDS',
  
  /** Zero-sized array passed to WASM */
  ZeroDimension = 'WASM_ZERO_DIMENSION',
}

enum SyncError {
  /** ICE connection failed after all candidates exhausted */
  ConnectionFailed = 'SYNC_CONNECTION_FAILED',
  
  /** Authentication secret mismatch */
  AuthenticationFailed = 'SYNC_AUTH_FAILED',
  
  /** Sync operation timeout (default: 30s) */
  Timeout = 'SYNC_TIMEOUT',
  
  /** Chunk data corrupted during transmission */
  DataCorrupted = 'SYNC_DATA_CORRUPTED',
}

enum StorageError {
  /** LevelDB file corrupted */
  DatabaseCorrupted = 'STORAGE_DB_CORRUPTED',
  
  /** Disk full during write */
  DiskFull = 'STORAGE_DISK_FULL',
  
  /** Write stall timeout (compaction backlog) */
  WriteStall = 'STORAGE_WRITE_STALL',
}
```

---

## Chapter 8: Known Limitations & Future Work

### 8.1 Current Technical Limitations

**WASM FFI Overhead**:
- **Issue**: Data serialization between JS and WASM consumes 15-30% of execution time
- **Impact**: High-frequency operations (chunk comparison) affected
- **Mitigation**: Zero-copy pointer passing reduces overhead to 5-10%
- **Future Work**: SharedArrayBuffer (SAB) for true zero-copy shared memory

**LevelDB Write Amplification**:
- **Issue**: Leveled compaction causes ~10x write amplification
- **Impact**: SSD wear, reduced write throughput under heavy load
- **Mitigation**: Tune `writeBufferSize` and `level0SlowdownWritesTrigger`
- **Future Work**: Evaluate RocksDB's Tiered Compaction

**TURN Relay Bandwidth Cost**:
- **Issue**: Relay traffic routed through TURN server (+20-50ms latency)
- **Impact**: Increased latency for symmetric NAT scenarios
- **Mitigation**: Host/srflx preferred over relay (priority formula)
- **Future Work**: P2P hole punching extensions (RFC 8445)

**Yjs Memory Overhead**:
- **Issue**: CRDT tombstones accumulate over time (DeleteSet growth)
- **Impact**: Unbounded memory growth for long-lived documents
- **Mitigation**: Periodic state snapshot + reset (not yet implemented)
- **Future Work**: Dotted Version Vectors (DVV) for efficient tombstone pruning

### 8.2 Future Roadmap

**Phase 1: Protocol Hardening (Q2 2026)**
- [ ] Implement RFC 8445 (ICE v2) for improved NAT traversal
- [ ] Add DTLS-SRTP for encrypted data channels
- [ ] Implement Yjs Awareness protocol for presence detection

**Phase 2: Performance Optimization (Q3 2026)**
- [ ] SharedArrayBuffer for WASM zero-copy
- [ ] Web Worker offloading for Yjs encode/decode
- [ ] Incremental sync for large chunks (>1MB)

**Phase 3: Decentralization (Q4 2026)**
- [ ] y-protocols integration for WebSocket fallback
- [ ] mDNS service discovery (no signaling server)
- [ ] Offline-first merge queue with conflict resolution UI

### 8.3 Contributing

**Development Setup**:
```bash
# Clone repository
git clone https://github.com/Cognitive-Architect/hajimi-code-cli.git
cd hajimi-code-cli

# Install dependencies
npm install

# Run TypeScript strict check
npx tsc --noEmit

# Run unit tests
npm test

# Run E2E tests (requires Docker)
bash scripts/run-real-e2e.sh

# Run benchmarks
npm run bench
```

**Code Style**:
- TypeScript strict mode (noImplicitAny, strictNullChecks)
- ESLint + Prettier configuration
- Conventional commits (`feat:`, `fix:`, `docs:`, `refactor:`)

**Testing Requirements**:
- Unit tests: Jest with 80%+ coverage
- E2E tests: Real npm packages (no mocks)
- Performance tests: 1K/5K/10K chunk benchmarks

---

## Appendix A: RFC References

| RFC/Standard | Title | Usage in Hajimi |
|--------------|-------|-----------------|
| RFC 5245 | Interactive Connectivity Establishment (ICE) | Candidate gathering, priority formulas |
| RFC 5766 | Traversal Using Relays around NAT (TURN) | Relay allocation, ChannelData |
| RFC 5389 | Session Traversal Utilities for NAT (STUN) | Binding requests, NAT type detection |
| RFC 8445 | ICE v2 | Future: improved NAT traversal |
| LevelDB Paper | LevelDB: A Fast Persistent Key-Value Store | LSM-tree design, compaction strategy |
| YATA Paper | Yjs: A Framework for Near Real-Time P2P Shared Editing | CRDT conflict resolution |

## Appendix B: Glossary

| Term | Definition |
|------|------------|
| **CRDT** | Conflict-free Replicated Data Type - data structure that converges to same state regardless of operation order |
| **State Vector** | Map<clientId, clock> representing the latest known timestamp from each client |
| **ICE** | Interactive Connectivity Establishment - protocol for NAT traversal |
| **LSM-tree** | Log-Structured Merge Tree - write-optimized storage structure |
| **SSTable** | Sorted String Table - immutable sorted file in LSM-tree |
| **MemTable** | In-memory write buffer (skip list) in LevelDB |
| **Tombstone** | Deletion marker in LSM-tree, removed during compaction |
| **WASI** | WebAssembly System Interface - standard for WASM system calls |
| **SIMD** | Single Instruction Multiple Data - parallel vector operations |

---

<p align="center">
  <strong>Hajimi V3.5.0</strong><br>
  <sub>Local-First P2P Synchronization with CRDT, ICE/TURN, and WASM Optimization</sub><br>
  <sub>Last Updated: 2026-03-04</sub>
</p>
