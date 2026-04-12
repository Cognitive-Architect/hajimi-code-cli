# Week 40 债务清理记录

## Week 41 前置清理（B级收官补救）

### DEBT-PHYSICAL-HNSW-W40 — CLOSED ✅

**Issue**: `crates/hajimi-hnsw/` 目录物理残留（732行Week 35废弃代码）  
**Resolution**: 物理删除目录，归档至 `attic/deprecated/hnsw-week35/`  
**Evidence**: 
- `ls crates/ | grep hnsw | wc -l` = 0
- Git commit: `cleanup(week41): remove hajimi-hnsw crate`
**Closed Date**: 2026-04-10

### DEBT-WORKSPACE-HNSW-W40 — CLOSED ✅

**Issue**: Cargo.toml workspace members 仍包含已废弃crate  
**Resolution**: 移除 `"crates/hajimi-hnsw",` 行  
**Evidence**: Cargo.toml L5 删除，workspace成员从8个减至7个  
**Closed Date**: 2026-04-10

### DEBT-UNWRAP-L118-W40 — CLOSED ✅

**Issue**: `tests/bench/pgvector_perf.rs:118` 存在 `unwrap()` 违反零unwrap基线  
**Resolution**: 替换为 `unwrap_or(std::cmp::Ordering::Equal)`  
**Evidence**: `grep "unwrap()" tests/bench/pgvector_perf.rs | wc -l` = 0  
**Closed Date**: 2026-04-10
