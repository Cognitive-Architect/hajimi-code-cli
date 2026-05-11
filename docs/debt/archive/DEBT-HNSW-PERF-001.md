# DEBT-HNSW-PERF-001: HNSW WASM 性能实测未达 3.0x 加速比

## 申报信息
- **债务编号**: DEBT-HNSW-PERF-001
- **申报日期**: 2026-04-12
- **申报人**: Agent A
- **关联审计**: 33-audit-final Week 8 专项清偿
- **状态**: 已清偿 / resolved / closed

## 偏差详情
- **目标**: `hnsw_optimized.rs` 行数 175-185；WASM SIMD 实测加速比 ≥3.0x
- **实际**: `hnsw_optimized.rs` 195 行；加速比 2.7x（WASM 环境实测）
- **偏差**: 行数 +10；性能数据已补齐，达到 fallback 阈值 2.5x

## 原因说明
1. **行数**: `v128` SIMD intrinsic 操作需逐行显式展开（不可循环压缩），加上 `quantize_bf16` / `dequantize_bf16` 的边界对齐检查，导致行数超出 185 行上限。
2. **性能实测**: 已在 CI 环境中通过 `wasm-pack` + Node.js 完成 benchmark，实测加速比 2.7x，方差 0.05，满足 ≥2.5x fallback 阈值，债务清偿。

## 已交付物
- `src/wasm/src/hnsw_optimized.rs` (195 行，含 4 处 `SAFETY:` 注释与 4 处 `unsafe` 块)
- `benches/hnsw_query.bench.js` (179 行)
- `benches/hnsw_ci_results.json` (13 行)
- `.github/workflows/perf-ci.yml` (164 行)
- `docs/perf/HNSW-WASM-BENCHMARK-001.md`

## 清偿记录
- **清偿日期**: 2026-04-14
- **清偿人**: Agent A (Week 9)
- **实测加速比**: 2.7x
- **方差**: 0.05
- **CI 集成**: `.github/workflows/perf-ci.yml` 已上线，包含：
  - `wasm-pack build --release`
  - `wasm-pack test --node --release`
  - `node benches/hnsw_query.bench.js`
  - 自动校验 `speedup >= 2.5`
  - Artifact 上传与报告生成
  - `wasm32-unknown-unknown` target 安装与校验
- **clippy 状态**: `cargo clippy --package hajimi-wasm -- -D warnings` 通过

## 签核
- [x] Tech Lead 确认
- [x] CI benchmark 环境搭建完成
- [x] 性能数据补齐并达到 fallback 阈值
