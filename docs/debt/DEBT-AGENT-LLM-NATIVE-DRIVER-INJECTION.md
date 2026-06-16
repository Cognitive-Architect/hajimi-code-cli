# DEBT-AGENT-LLM-NATIVE-DRIVER-INJECTION — Agent LLM-Native Driver 桌面注入缺失

> **ID**: `DEBT-AGENT-LLM-NATIVE-DRIVER-INJECTION`  
> **Priority**: **P0**  
> **Date**: 2026-05-30  
> **Status**: `FIXED` (2026-05-30)  
> **修复方案**: `DesktopAgentTurnDriver` 动态延迟绑定（详见 `docs/roadmap/hajimi interface/plan/P0-LLM-NATIVE-DRIVER-INJECTION-FIX.md`）  
> **发现者**: 压力怪（审计官）+ 用户真机验收  
> **关联审计**: B-05/25-AUDIT-REPORT-v3.md  
> **关联路线图**: LLM-NATIVE-AGENT-MIGRATION-ROADMAP.md Phase 5  
> **分支**: `release/llm-native-agent-v3`  
> **HEAD**: `bdee9ed873d80a6a97bdb1ea6569009913a2d37c`

---

## 1. 问题摘要

Hajimi IDE 桌面端 `/agent` 命令在真机环境中**完全失效**，无法走 LLM-Native 路径，静默降级到 legacy 后也因硬编码路径问题而 Aborted。

**核心矛盾**：`cargo test --test llm_native_e2e_tests` = 5 passed（全绿），但桌面应用真机执行 `/agent` 命令 = **Aborted**。

> **一句话总结**：`AgentLoopBuilder::production_ready()` 未注入 `LlmNativeDriver`，导致 25 天 LLM-Native 大重构的**核心成果在桌面端无法落地**。

---

## 2. 故障现象

### 真机复现步骤

1. 启动 Hajimi IDE（release 构建）
2. 配置 deepseek 模型（API Key 有效，普通聊天正常）
3. 输入 `/agent 创建一个名为 hello-agent.txt 的文件，内容是 "Hello from LLM-Native Agent"`
4. Agent Trace 面板显示：`legacy act path for goal ...`
5. 最终结果：`智能体任务被终止或自动退出。(Aborted)`

### Agent Trace 关键错误

```
Stop-Loss forced handoff: Handoff: success=false, severity=High
issues=Local execution of read_file returned error:
Access: 系统找不到指定的文件。(os error 2)
suggestions=Retry with modified parameters
```

---

## 3. 根因分析

### 3.1 直接原因：`native_driver` 为 `None`

`AgentLoop::run()` 的分支逻辑（`src/intelligence/agent-core/agent_loop.rs:192`）：

```rust
if crate::prompts::is_agent_llm_native_enabled() {
    if let Some(ref driver) = self.native_driver {
        // ✅ 走 LLM-Native 路径（E2E 测试走这里）
    } else {
        // ❌ native_driver 为 None → silent fallback 到 legacy（桌面端走这里）
    }
}
```

### 3.2 根本原因：`production_ready()` 未注入 `native_driver`

`AgentLoopBuilder::production_ready()`（`src/intelligence/agent-core/agent_loop_builder.rs:81`）：

```rust
pub fn production_ready(device_id: &str) -> Self {
    // ... 初始化 memory ...
    Self::new()
        .with_memory(Some(memory))
        .with_sync_gateway(Some(sync_gateway))
    // ❌ 没有 .with_native_driver(...)
}
```

桌面后端调用点（`src/interface/desktop/src/main.rs:3517`）：

```rust
AgentLoopBuilder::production_ready("hajimi-desktop")
```

### 3.3 连锁原因：legacy fallback 硬编码路径在桌面 CWD 下失效

`legacy_act()` 默认 fallback（`src/intelligence/agent-core/agent_loop.rs`）：

```rust
("read_file", json!({ "path": "Cargo.toml" }))
```

- 单元测试 CWD = 项目根目录 → `Cargo.toml` 存在 ✅
- 桌面应用 CWD = `target/release/` → `Cargo.toml` 不存在 ❌

### 3.4 故障链完整复盘

```
production_ready() 未注入 native_driver
    ↓
AgentLoop::run() 中 native_driver.is_none() → silent fallback legacy
    ↓
legacy_act() 调用 → Task 无 tool_calls → fallback read_file("Cargo.toml")
    ↓
CWD = target/release/ → os error 2 (找不到文件)
    ↓
Reflect → success=false
    ↓
第二次迭代 → 再次失败
    ↓
Stop-Loss 触发 → Handoff → Aborted
```

---

## 4. 为什么 E2E 测试通过了？

| 维度 | E2E 测试 | 桌面应用 |
|:---|:---|:---|
| `AgentLoop` 构建方式 | 测试代码手动 `AgentLoopBuilder::new()` + `.with_native_driver(...)` | `production_ready("hajimi-desktop")` |
| `native_driver` | `Some(driver)` ✅ | `None` ❌ |
| 执行路径 | LLM-Native ✅ | legacy fallback ❌ |
| CWD | 项目根目录（`Cargo.toml` 存在） | `target/release/`（`Cargo.toml` 不存在） |

**结论**：E2E 测试覆盖的是 Intelligence 层内部逻辑，但 Interface 层（桌面后端）的生产代码没有完成 `LlmNativeDriver` 的装配。这是 **"测试绿通但真机翻车"** 的典型场景。

---

## 5. 影响评估

| 影响面 | 严重程度 | 说明 |
|:---|:---:|:---|
| `/agent` 命令 | **致命** | 完全不可用，所有 Agent 功能失效 |
| 中文意图零篡改 | **致命** | 用户无法体验 LLM-Native 的核心价值 |
| MCP 工具动态调用 | **致命** | 无法触发 |
| 安全 Governance 拦截 | **部分** | rm-rf 拦截在 E2E 中验证通过，但桌面端无法触发 |
| 普通聊天 | **无影响** | 直接调用 `LlmClient`，绕过 `AgentLoop` |

---

## 6. 修复方向（待决策）

### 方案 A：在 `main.rs` 中补充 `with_native_driver()`（短期）

在 `src/interface/desktop/src/main.rs:3517` 之后链式调用：

```rust
AgentLoopBuilder::production_ready("hajimi-desktop")
    .with_native_driver(Arc::new(LlmNativeDriver::new(...)))
```

- **优点**：改动最小，快速修复
- **缺点**：`main.rs` 需要直接处理 Provider 配置和 `LlmNativeDriver` 构造，增加 Interface 层复杂度

### 方案 B：重构 `production_ready()` 接收 Provider 配置（中期）

```rust
pub fn production_ready(device_id: &str, provider_config: &LlmProviderConfig) -> Self {
    // ... 初始化 memory ...
    let driver = LlmNativeDriver::from_config(provider_config);
    Self::new()
        .with_memory(Some(memory))
        .with_sync_gateway(Some(sync_gateway))
        .with_native_driver(Some(Arc::new(driver)))
}
```

- **优点**：工厂方法职责完整，调用方简洁
- **缺点**：需要设计 `LlmProviderConfig` 的传递机制

### 方案 C：增加 `native_driver` 缺失时的显式错误提示（长期）

在 `AgentLoop::run()` 中：

```rust
if crate::prompts::is_agent_llm_native_enabled() {
    if let Some(ref driver) = self.native_driver {
        // LLM-Native 路径
    } else {
        // ❌ 不再 silent fallback
        return Err(ReplError::Session(
            "LLM-Native enabled but LlmNativeDriver not initialized".to_string()
        ));
    }
}
```

- **优点**：消除静默降级，用户体验明确
- **缺点**：需要同步调整所有测试和 fallback 逻辑

---

## 7. 验证清单（修复后必须执行）

- [ ] 桌面应用 release 构建后，`/agent` 命令不再走 `legacy act path`
- [ ] Agent Trace 面板显示 `LLM-Native path enabled for goal: ...`
- [ ] 中文意图 `"创建一个名为 test.txt 的文件"` 直达 LLM，零本地规则篡改
- [ ] LLM 自主返回 `write_file` ToolCall，文件被成功创建
- [ ] `cargo test --test llm_native_e2e_tests` 仍然 5 passed
- [ ] `cargo test -p intelligence-agent-core --lib` 仍然 325 passed
- [ ] `cargo check --workspace` 0 errors
- [ ] `cargo clippy --workspace -- -D warnings` 0 warnings

---

## 8. 关联上下文

| 文档 | 路径 | 说明 |
|:---|:---|:---|
| LLM-Native 路线图 | `docs/roadmap/Hajimi LLM/plan/LLM-NATIVE-AGENT-MIGRATION-ROADMAP.md` | Phase 5 标记 [COMPLETE]，但桌面注入未完成 |
| 终局清债记录 | `docs/debt/DEBT-AGENT-LLM-NATIVE-MIGRATION.md` | 宣告 SUCCESSFULLY CLOSED，但遗漏了此注入债务 |
| Day 25 审计报告 | `audit report/B-05-25-AUDIT-REPORT-v3.md` | 审计时未发现此问题（E2E 测试绿通掩盖了真机缺陷）|
| AgentLoop 主入口 | `src/intelligence/agent-core/agent_loop.rs:177` | `run()` 方法分支逻辑 |
| AgentLoopBuilder | `src/intelligence/agent-core/agent_loop_builder.rs:81` | `production_ready()` 工厂方法 |
| 桌面后端入口 | `src/interface/desktop/src/main.rs:3517` | `AgentLoopBuilder::production_ready()` 调用点 |

---

## 9. 历史教训

> **Day 25 审计的盲区**：审计验证的 `cargo test --test llm_native_e2e_tests` 和 `cargo test -p intelligence-agent-core --lib` 全部是 Intelligence 层内部测试。Interface 层的 `production_ready()` 调用点**没有任何测试覆盖**（既无单元测试也无集成测试），导致 P0 级功能缺陷在终局审计中漏网。
>
> **建议**：未来在 `main.rs` 等 Interface 层入口增加**端到端冒烟测试**，确保工厂方法装配完整性。

---

*此债务为 LLM-Native Migration 的**最后一公里的最后一公里**。25 天大重构在代码层面全部完成，但桌面端的 Driver 装配线尚未接通。*
