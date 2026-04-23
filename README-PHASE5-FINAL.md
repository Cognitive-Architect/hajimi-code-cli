# Hajimi Phase 5 — Final Technical Whitepaper

**Date**: 2026-04-14  
**Scope**: Month 4 Week 16 final delivery and external engineering summary

---

## 1. Project Overview

Hajimi is a local-first peer-to-peer synchronization system. It combines CRDT-based conflict resolution, WebRTC NAT traversal, LevelDB persistent storage, and WASM-optimized vector search. Phase 5 completes the memory-intelligence stack, delivering approximate-nearest-neighbor search, full-text indexing, encrypted cloud persistence, and IDE tooling.

Major Phase 5 achievements:
- HNSW WASM query acceleration achieves 2.7x over brute-force baseline.
- Tantivy Chinese search module is compressed to 219 production lines while preserving Jieba tokenizer functionality.
- Cloud 10K stress test validates the 5-tier memory cascade under a 10,000-vector workload.
- VS Code extension provides dual IDE commands (`openAdr` and `gotoAdr`) for Architecture Decision Records.

---

## 2. System Architecture

The memory system is organized as a 5-tier cascade:

1. **Session Cache** — Hot in-memory vectors with immediate access.
2. **Auto Memory** — Structured, automatically summarized recall.
3. **Dream Memory** — Episodic, long-horizon narrative storage.
4. **Graph Memory** — HNSW ANN index for semantic similarity search.
5. **Cloud Memory** — End-to-end encrypted remote persistence layer.

Data flows from Session down to Cloud according to retention and relevance policies. Encryption uses X3DH key rotation with batch operations to minimize network overhead. The cascade ensures that hot data stays local while cold data is securely offloaded.

---

## 3. Four Solidified Technical Standards

The following standards are frozen for the project:

| Standard | Definition | Document Path |
|:---|:---|:---|
| **HNSW 2.7x** | WASM32 HNSW query speedup fallback threshold. The measured acceleration is 2.7x; the minimum acceptable fallback is 2.5x. | `docs/perf/HNSW-WASM-BENCHMARK-001.md` |
| **Tantivy 219 lines** | The Tantivy index manager production code is limited to 219 lines, achieved by removing auxiliary methods and compressing trait implementations. | `docs/debt/archive/DEBT-TANT-002.md` |
| **Cloud 10K** | End-to-end cloud stress benchmark specification requiring correct operation under a 10,000-vector load across all memory tiers. | `docs/perf/CLOUD-10K-BENCHMARK-001.md` |
| **IDE dual commands** | The VS Code extension must register and implement two commands: `openAdr` (open ADR by debt ID) and `gotoAdr` (jump to ADR from a code reference). | `docs/adr/IDE-INTEGRATION.md` |

These standards exist to keep the codebase maintainable, the benchmarks honest, and the developer experience consistent. External contributors should treat them as hard constraints.

---

## 4. Data Integrity and Honesty

All performance claims are cross-checked by `tools/data-validator.js`, which implements the ID-261 data honesty mechanism. The validator enforces:

- Realistic latency bounds: greater than 0 ms and less than 1000 ms.
- Realistic memory bounds: between 50 MB and 2048 MB.
- Corrected benchmark reports must contain realistic measured values.
- Original benchmark reports must not claim unqualified A+ ratings without correction notes.

This ensures that no inflated or fabricated metrics are published. Engineers should run the validator before submitting any performance-related documentation.

---

## 5. Debt Archive

All historical technical debts are permanently archived in `docs/debt/archive/`. No active debts remain. The archive contains debt declarations, clearance records, and remediation evidence for audit traceability.

---

## 6. Engineering Principles

Phase 5 follows four core engineering principles:

1. **Honest metrics** — Every performance claim is validated by automated tooling.
2. **Frozen standards** — Once a standard is solidified, it becomes a contractual boundary.
3. **Archived debts** — Closed debts are preserved for transparency, not deleted.
4. **Minimal surprises** — Code changes stay within well-defined size and complexity budgets.

These principles are enforced through automated checks and peer review.

---

## 7. Quick Start for Engineers

```bash
# Verify data honesty
node tools/data-validator.js

# Run HNSW WASM benchmark
wasm-pack build --release --target nodejs
node benches/hnsw_query.bench.js
```

---

*For the living project documentation, see `README.md`.*
