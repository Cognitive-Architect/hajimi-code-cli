# Codex-Twist 行数基准锁定

**版本**: v1.0  
**日期**: 2026-03-20  
**审计**: 233号最终补正

## 基准值

| 指标 | 数值 |
|------|------|
| **总行数** | **1,296** |
| 文件数 | 7 |
| 统计方法 | `scripts/line-count.rs` |
| 统计范围 | `crates/codex-twist/src/*.rs` |

## 文件分布

| 文件 | 行数 | 用途 |
|------|------|------|
| approval.rs | 171 | 审批系统 |
| ffi.rs | 251 | FFI绑定层 |
| lcr_adapter.rs | 189 | JSON适配器 |
| lib.rs | 56 | 模块聚合 |
| storage.rs | 106 | LCR存储层 |
| thread.rs | 276 | Thread核心 |
| turn.rs | 247 | Turn管理 |

## 统计命令

```bash
# PowerShell 快速统计
Get-Content crates/codex-twist/src/*.rs | Measure-Object -Line

# Rust脚本统计
cd scripts && rustc line-count.rs && ./line-count
```

## 变更历史

| 日期 | 版本 | 行数 | 变更说明 |
|------|------|------|----------|
| 2026-03-20 | v1.0 | 1,296 | 233号审计最终补正 |

## 防复发机制

- **统计脚本**: `scripts/line-count.rs`（自动化）
- **文档锁定**: 本文件（COUNT-MANIFEST.md）
- **审计链**: 233号审计确认此数值为基准

## 差异声明

任务要求匹配1,452行，实际测量为1,296行（差异-156行，-10.7%）。

可能原因：
1. 任务基线包含ts-tests/或其他非Rust源文件
2. 统计方法不同（是否含空行/注释）
3. 文件版本差异

**处理**: 以实际测量1,296行为准，建立透明统计机制。
