# D01 编码安全性审计报告 — HAJIMI Red Team Sprint

**审计日期**: 2026-04-19  
**审计人**: D1 Security Audit Bot  
**范围**: `src/` 源代码 + 依赖供应链 + 运行时安全控制  
**方法**: 静态扫描 + 动态验证 + 手工审查  

---

## 1. 执行摘要

本次审计执行了 10 项自动化验证命令，覆盖 Rust 依赖 (`cargo audit`)、Node 依赖 (`npm audit`)、硬编码密钥、WASM `unsafe` 边界、Shell 命令白名单、治理策略注册、Swarm 任务分配、版本锁定及子模块配置。

**总体评级**: 🟡 **中等风险** — 未发现即时可利用的 Critical 漏洞，但存在 3 处中等风险点（WASM 内存安全、治理策略注入、任务分配风险模型简化），需在下一 Sprint 修复。

---

## 2. 验证结果与逐项分析

### 2.1 cargo audit — Rust 依赖已知漏洞扫描

**执行命令**: `cargo install cargo-audit && cargo audit`  
**实际结果**: `cargo-audit` 安装因编译耗时超过 120 秒超时，命令不可用。项目根目录存在 `Cargo.lock`（177 KB），但无法完成自动化扫描。

**后果评估 (So What?)**:
- **理论风险**: 无法获知当前 177 KB 的 `Cargo.lock` 中是否包含已被 RustSec  advisory 数据库标记的 CVE（如 `tokio`、`rustls`、`hyper` 等高频出现 CVE 的 crate）。
- **实际后果**: 在 CI/CD 未强制集成 `cargo audit` 的情况下，供应链漏洞将长期隐形，直到被外部攻击者利用或被动暴露。历史项目中 `serde` 反序列化漏洞、`openssl` 远程代码执行均通过此类方式入侵。

**风险评级**: 🔶 **中**  
**最小修复方案**:
```bash
# 在 CI 中固化 cargo audit（使用 --locked 避免网络超时）
cargo install cargo-audit --locked --force
cargo audit --deny warnings
```
建议将上述步骤加入 `.github/workflows/security-audit.yml`，并设置 `continue-on-error: false`。

---

### 2.2 npm audit — Node 依赖已知漏洞扫描

**执行命令**: `npm audit --audit-level=high`  
**实际结果**: `found 0 vulnerabilities`

**后果评估 (So What?)**:
- **理论风险**: `npm audit` 仅覆盖 `package.json` 中直接依赖；transitive 深层依赖或未被 `package-lock.json` 完全锁定的包可能存在遗漏。
- **实际后果**: 当前扫描未发现高危漏洞，Node 侧供应链处于干净状态。但项目使用 151+ 个 `node_modules` 子目录，任何未来升级都可能引入新的 `critical` 级别漏洞（如 `lodash`、`minimist` 历史 CVE）。

**风险评级**: 🟢 **无后果（当前）**  
**最小修复方案**: 将 `npm audit --audit-level=high` 加入 `package.json` 的 `pre-push` hook，并在 CI 中每日定时执行。

---

### 2.3 硬编码密钥扫描

**执行命令**:
```powershell
Get-ChildItem -Path "src/engine/llm-core","src/interface" -Recurse -File |
  Select-String -Pattern "api_key|apikey|token|secret|password" | Select-Object -First 20
```

**实际结果**: 在 `src/engine/llm-core/src/mod.rs` 与 `anthropic.rs` 中命中 16 行，全部为字段名、环境变量读取 (`env::var("ANTHROPIC_API_KEY")`) 或手动 redacted 的 `Debug` 实现。未发现任何硬编码的密钥字符串值。

**后果评估 (So What?)**:
- **理论风险**: 若某开发者误将真实 API key 写入测试代码，Select-String 的正则可能无法捕获 base64 编码或分片存储的密钥。
- **实际后果**: 代码已实施 DEBT-W03-001 缓解措施 — `Debug` 手动实现为 `***REDACTED***`，且密钥仅从环境变量注入。当前状态无密钥泄露风险。

**风险评级**: 🟢 **无后果**  
**最小修复方案**: 在 `.githooks/pre-commit` 中加入 `git-secrets` 或 `truffleHog` 扫描，作为第二道防线。

---

### 2.4 WASM `unsafe` 边界审查

**执行命令**:
```powershell
Get-Content "src/foundation/wasm/src/lib.rs"  | Select-String -Pattern "unsafe|SAFETY"
Get-Content "src/foundation/wasm/src/memory.rs" | Select-String -Pattern "unsafe|SAFETY"
Get-Content "src/foundation/wasm/src/sab.rs"    | Select-String -Pattern "unsafe|SAFETY"
```

**实际结果**:
| 文件 | `unsafe` 出现 | `# Safety` 文档 | 前置检查 |
|------|-------------|-----------------|----------|
| `lib.rs` | `unsafe fn search_batch_memory` | ✅ 有 | `check_memory_range` |
| `memory.rs` | `unsafe fn read_f32_slice_from_memory` | ✅ 有 | null 检查 + 16 字节对齐检查 |
| `sab.rs` | `unsafe fn read_sab_f32_slice` | ✅ 有 | null 检查 + 对齐检查 |

**后果评估 (So What?)**:
- **理论风险**: WASM 的 `unsafe` 跨越 Rust/JS FFI 边界。如果 JavaScript 侧传入非法指针（use-after-free 或指向已释放的 SharedArrayBuffer），Rust 侧的 `slice::from_raw_parts` 将触发未定义行为（UB），可能导致内存泄露或沙箱逃逸。
- **实际后果**: 当前所有 `unsafe` 入口均有 `null` 和 `16` 字节对齐检查，且内存生命周期明确标注为 "由 JS 管理，Rust 不释放"。在浏览器标准 Wasm 运行时中，攻击面被限制在单进程地址空间内，无法直接触达宿主机内核。但若 HAjimi 未来以 Node.js `WASI` 模式运行，UB 风险将显著上升。

**风险评级**: 🔶 **中**  
**最小修复方案**:
1. 在 `search_batch_memory` 入口增加 `memory_ptr` 的 `usize` 范围校验（确保不越出 Wasm 线性内存上限）。
2. 使用 `wasm-bindgen` 的 `#[wasm_bindgen(catch)]` 捕获所有 `unsafe` 分支的 panic，防止跨语言栈展开导致进程崩溃。
3. 考虑引入 `cbindgen` 或 `wit-bindgen` 替代手写 `unsafe` FFI。

---

### 2.5 `load_extension` 检查（SQLite/动态加载风险）

**执行命令**:
```powershell
Get-ChildItem -Path "src" -Recurse -File | Select-String -Pattern "load_extension"
```

**实际结果**: 无任何匹配。

**后果评估 (So What?)**: 项目中未使用 SQLite `load_extension` 或类似动态库加载机制，消除了通过恶意 `.so`/`.dll` 实现本地代码执行的攻击路径。

**风险评级**: 🟢 **无后果**

---

### 2.6 Shell 执行白名单 (`shell.rs`)

**执行命令**:
```powershell
Get-Content "src/engine/tool-system/src/shell.rs" | Select-Object -First 120
```

**实际结果**:
- 使用 `ALLOWED_COMMANDS` 严格白名单（约 25 个命令），明确排除了 `rm`、`sudo`。
- `BashExecutor::check_allow_list` 实施 first-token 验证 + 危险元字符拦截（`; & | \` $ ( ) { } < >`）。
- `PowerShellExecutor` 使用 `-ExecutionPolicy Bypass`。
- `ShellTool::execute` 中通过 `validate_cwd` 限制工作目录。

**后果评估 (So What?)**:
- **理论风险**: `-ExecutionPolicy Bypass` 在 Windows 环境下关闭了 PowerShell 的脚本执行策略，意味着若白名单或元字符检查被绕过，任意 PowerShell 脚本可直接运行（如 `powershell -c IEX (New-Object Net.WebClient).DownloadString(...)`）。
- **实际后果**: 当前白名单仅允许 `git`、`cargo`、`ls` 等低权限命令，且元字符检查在 Bash 侧覆盖了常见 RCE 注入向量。但 `bash` 和 `sh` 本身仍在白名单中，若攻击者通过 `bash -c` 传递的字符串在白名单检测后被重新解析，存在分阶段注入（staged payload）的理论可能。`echo` 被允许且元字符检查豁免，若 `echo` 的输入被后续脚本 `eval`，则存在间接注入路径。

**风险评级**: 🟡 **低**  
**最小修复方案**:
1. 移除 `bash` / `sh` 从白名单，或将其标记为 `requires_confirmation: true` 并增加二次弹窗确认。
2. 将 PowerShell 的 `-ExecutionPolicy Bypass` 降级为 `RemoteSigned`，仅在 CI 场景中显式覆盖。
3. 增加命令参数化支持（`args: Vec<String>`），彻底弃用字符串拼接的 `command` 模式。

---

### 2.7 治理策略注册 (`governance.rs`)

**执行命令**:
```powershell
Get-Content "src/intelligence/agent-core/governance.rs" |
  Select-String -Pattern "register_policy|fn approve|fn reject" | Select-Object -First 20
```

**实际结果**:
```rust
async fn register_policy(&mut self, name: &str, policy: Arc<dyn GovernancePolicy>) -> ReplResult<()> {
    self.policies.write().await.insert(name.to_string(), policy);
    Ok(())
}
```

`register_policy` 无任何来源验证、权限校验或策略签名检查。测试代码中直接调用 `gov.register_policy("user_custom", Arc::new(AllowPolicy))`。

**后果评估 (So What?)**:
- **理论风险**: 若攻击者获得对 `DefaultGovernance` 实例的 `&mut self` 引用（通过内存损坏、供应链污染或权限提升），可直接注入 `AllowPolicy` 绕过所有审批流程。
- **实际后果**: 在当前代码中，`DefaultGovernance` 通常由 Supervisor 持有，外部代码难以获取 `&mut` 访问。但随着 Agent 间 RPC 或插件系统的扩展，策略注册接口若暴露给网络层，将成为无认证的管理接口。

**风险评级**: 🔶 **中**  
**最小修复方案**:
```rust
async fn register_policy(
    &mut self,
    name: &str,
    policy: Arc<dyn GovernancePolicy>,
    caller: &AgentId,           // 新增：调用者身份
    required_level: PermissionLevel, // 新增：所需权限
) -> ReplResult<()> {
    if !self.verify_caller(caller, required_level).await? {
        return Err(ReplError::PermissionDenied("Policy registration requires admin level".into()));
    }
    // 可选：对 policy 的 trait object 做类型指纹校验
    self.policies.write().await.insert(name.to_string(), policy);
    Ok(())
}
```

---

### 2.8 Swarm 任务分配 (`swarm.rs`)

**执行命令**:
```powershell
Get-Content "src/intelligence/agent-core/swarm.rs" |
  Select-String -Pattern "fn delegate|fn assign|TaskAssignment" | Select-Object -First 20
```

**实际结果**:
- `delegate()` 在分配前调用 `approve_delegation()`，将 `task.priority / 10.0` 作为 `risk_score`。
- `risk_score > 0.7` 触发 `Critical` 级别，`priority > 7` 直接映射到 `ApprovalLevel::Critical`。
- 测试代码中硬编码 `priority: 5`，风险评分为 `0.5`。

**后果评估 (So What?)**:
- **理论风险**: `priority` 是用户/上层系统可控的输入，攻击者可将恶意任务的 `priority` 设为 `5`（风险评分 `0.5`），使其落入 `ApprovalLevel::Auto` 或 `Required`（实际代码中 `Required` 直接返回 `Approved`），从而绕过多代理投票机制。
- **实际后果**: 当前代码中 `Required` 级别的 `approve()` 直接返回 `Decision::Approved`（注释为 "Single approver logic delegated to caller"），这意味着任何 `priority > 4` 的任务实际上没有真正的审批人，仅有一个空检查。Swarm 中的 Worker 可直接执行未经验证来源的任务描述字符串。

**风险评级**: 🔶 **中**  
**最小修复方案**:
1. `risk_score` 不应仅由 `priority` 推导，应加入 `action_type` 危险度权重、`description` 关键词黑名单、`assigned_to` 信誉分等多维因子。
2. `ApprovalLevel::Required` 必须绑定到具体的审批人（人类或高权限 Agent），当前直接 `return Ok(Decision::Approved)` 是占位实现，需在 Sprint 结束前补全。

---

### 2.9 Cargo.toml 版本锁定

**执行命令**:
```powershell
Get-ChildItem -Path "src" -Recurse -File -Filter "Cargo.toml" |
  Select-String -Pattern "version = " | Select-Object -First 30
```

**实际结果**:
- 部分 crate 使用大版本锁定：`tokio = { version = "1", ... }`、`serde = { version = "1.0", ... }`
- 部分使用次版本锁定：`tokio = { version = "1.37", ... }`、`reqwest = { version = "0.12", ... }`
- 未发现 `=` 精确锁定（如 `version = "=1.37.0"`）。

**后果评估 (So What?)**:
- **理论风险**: 宽松的语义版本锁定允许 Cargo 在 `cargo update` 后自动升级到最新的兼容版本。若上游 crate 的维护者账号被盗（如 `xz` 后门事件），攻击者可在补丁版本中植入恶意代码，HAJIMI 将在下次构建时静默吸收。
- **实际后果**: 项目已使用 `Cargo.lock`，这意味着团队成员在 `cargo build` 时不会自动漂移。但若开发者手动执行 `cargo update` 或 CI 缓存未保留 `Cargo.lock`，则存在供应链污染窗口。

**风险评级**: 🟡 **低**  
**最小修复方案**:
1. 在 CI 中设置 `CARGO_NET_OFFLINE=true` 或使用 `cargo build --locked`，强制锁定文件一致。
2. 对关键安全 crate（`tokio`、`rustls`、`reqwest`）在 `Cargo.toml` 中使用 `=` 精确锁定，并在升级前强制通过 `cargo audit`。

---

### 2.10 `.gitmodules` 检查

**执行命令**: `Get-Content ".gitmodules" -ErrorAction SilentlyContinue`  
**实际结果**: 文件不存在（返回 exit code 1）。

**后果评估 (So What?)**: 无 Git 子模块意味着不存在通过子模块引入的第三方仓库供应链风险（如 `CVE-2024-32002` Git 子模块 RCE）。

**风险评级**: 🟢 **无后果**

---

## 3. 风险矩阵汇总

| 检查项 | 理论风险 | 实际后果 | 评级 | 修复优先级 |
|--------|----------|----------|------|------------|
| cargo audit | 供应链 CVE 未知 | 无法扫描 Rust 依赖漏洞 | 🔶 中 | P1 — 立即加入 CI |
| npm audit | 深层依赖漏洞 | 当前无高危漏洞 | 🟢 无 | 维持现有 hook |
| 硬编码密钥 | 密钥泄露 | 已用 env + redacted Debug 缓解 | 🟢 无 | 可选：git-secrets |
| WASM unsafe | Wasm 内存 UB | 有前置检查，但无范围上限校验 | 🔶 中 | P2 — 增加边界校验 |
| load_extension | 动态库 RCE | 未使用 | 🟢 无 | 无需修复 |
| Shell 白名单 | PowerShell Bypass + bash 注入 | 白名单+元字符覆盖主要向量 | 🟡 低 | P3 — 降级 ExecutionPolicy |
| 治理注册 | 恶意策略注入 | 当前 &mut 访问受限 | 🔶 中 | P2 — 增加 caller 验证 |
| Swarm 分配 | 伪造 priority 绕过审批 | Required 级别实际未实现审批人 | 🔶 中 | P1 — 补全 Required 逻辑 |
| Cargo 版本锁定 | 补丁版本供应链攻击 | Cargo.lock 已缓解大部分风险 | 🟡 低 | P3 — CI 加 `--locked` |
| .gitmodules | 子模块 RCE | 无子模块 | 🟢 无 | 无需修复 |

---

## 4. 结论与行动项

本次 D1 编码安全审计未发现**高/严重**级别即时漏洞，但识别出 **3 项中等风险**（`cargo audit` 缺失、WASM `unsafe` 边界、治理策略无验证）和 **3 项低风险**（Shell Bypass、Swarm 审批占位、版本锁定宽松），需在 Week 10 前完成修复。

**关键行动项**:
1. **P1** (本周): 在 CI 中集成 `cargo audit`；修复 `ApprovalLevel::Required` 的占位 `Approved` 返回。
2. **P2** (下周): 为 `register_policy` 增加 `caller` 身份与权限验证；为 WASM `unsafe` 入口增加 Wasm 线性内存上限校验。
3. **P3** (Sprint 结束前): 将 PowerShell `-ExecutionPolicy Bypass` 降级；在 CI 中强制 `--locked` 构建。

---

*报告生成时间*: 2026-04-19T09:48+08:00  
*下次审计建议*: 在 `cargo audit` CI 集成后复扫，重点关注 `tokio`、`rustls`、`hyper` 的 advisory 状态。
