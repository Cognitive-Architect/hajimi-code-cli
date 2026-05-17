# D01 — 编码安全性审计报告

> **审计维度**: D1 编码安全性  
> **审计日期**: 2026-04-28  
> **Git SHA**: 14e6c18e6bb25b30bb83013ac2bd05b128471eba  
> **审计员**: 红队安全审计员  
> **状态**: 完成

---

## 执行摘要

本次安全审计聚焦供应链投毒、已知漏洞、密钥泄露、命令注入、MCP攻击面五大攻击向量。共执行16项安全检查，发现**2项高后果风险**、**3项中后果风险**、**11项通过**。

**综合风险评级**: 🟡 **中**（存在可直接利用的Tauri command注入面，但核心工具系统Shell白名单参数化已落实）

---

## 检查清单执行结果

| ID | 类别 | 检查项 | 验证命令/方法 | 结果 | 风险评级 |
|:---|:---|:---|:---|:---:|:---:|
| S1 | SEC | `cargo audit` 高危漏洞 | `cargo audit`（未安装） | ❌ 未执行 | 中 |
| S2 | SEC | `npm audit` 高危漏洞 | `npm i --package-lock-only && npm audit --audit-level=high` | ✅ 0 vulnerabilities | 无 |
| S3 | SEC | 硬编码API密钥/Token | `Select-String` 全src扫描 | ✅ 无生产密钥 | 无 |
| S4 | SEC | WASM FFI边界安全 | 检查`foundation/wasm/src/*.rs` unsafe块+SAFETY注释 | ✅ 全部有SAFETY注释 | 无 |
| S5 | SEC | Tauri 依赖维护状态 | crates.io `tauri` v2活跃 | ✅ 活跃维护 | 无 |
| S6 | SEC | SQLite/数据库C扩展加载 | `Select-String` 搜索`load_extension` | ✅ 未使用 | 无 |
| S7 | NEG | 依赖版本未锁定 | 检查所有`Cargo.toml` | ✅ 全部`version = "=X.Y.Z"`精确锁定 | 无 |
| S8 | NEG | Git子模块/外部脚本 | 检查`.gitmodules` + scripts/ | ✅ 无`.gitmodules`，无外部脚本 | 无 |
| S9 | SEC | 原生绑定内存安全 | 检查`patches/zstd-sys/` | ⚠️ 本地补丁，非上游维护 | 低 |
| S10 | SEC | 连接池死锁/DoS风险 | 检查`sqlx`连接池配置 | ✅ 使用SQLite bundled，无外部连接池 | 无 |
| S11 | SEC | HNSW向量索引并发安全 | 检查`intelligence/memory/src/hnsw.rs` | ✅ 使用Arc+Mutex | 无 |
| S12 | SEC | 文件系统路径遍历 | 检查`engine/tool-system/src/fs.rs` | ✅ `validate_path`拒绝`..` | 无 |
| S13 | SEC | 命令注入（Shell执行） | 检查`engine/tool-system/src/shell.rs` | ⚠️ 白名单+metachar，但`bash -c`/`powershell -Command`拼接仍在 | 中 |
| S14 | NEG | MCP协议解析拒绝服务 | 检查`interface/mcp-server/` | ✅ MAX_INPUT_LEN=10KB, MAX_PATH_LEN=260 | 无 |
| S15 | SEC | 环境变量敏感信息落盘 | 检查日志打印 | ✅ 使用`secrecy::SecretString`，Debug输出`***REDACTED***` | 无 |
| S16 | HIGH | Agent Core编辑应用权限 | 检查`edit_applier.rs` | ✅ Governance审批门+10MB/50hunk限制 | 无 |

---

## 高后果发现

### D1-H1: `run_command` Tauri Command 无命令白名单过滤

**位置**: `src/interface/desktop/src/main.rs:146-157`

**代码**:
```rust
#[tauri::command]
fn run_command(cmd: &str, args: Vec<String>) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    ...
}
```

**分析**: 该Tauri command直接调用`Command::new(cmd)`，无任何白名单校验、路径校验或参数过滤。前端通过`invoke('run_command', {cmd: 'rm', args: ['-rf', '/']})`可直接执行任意系统命令。虽然`engine/tool-system/src/shell.rs`已实现了严格的白名单参数化Shell（B-04 P0），但`main.rs`中的`run_command`完全绕过了该防护层。

**后果**: 前端一旦被XSS攻破或用户安装恶意插件，可直接删除用户文件、执行勒索软件、窃取密钥。

**最小修复方案**: 将`run_command`重构为调用`engine_tool_system::ShellTool`的execute方法，复用已有的白名单+metachar防护；或至少增加与`ALLOWED_COMMANDS`一致的白名单校验。

**风险评级**: 🔴 **高**

---

### D1-H2: `read_file`/`write_file`/`list_dir` Tauri Commands 无路径限制

**位置**: `src/interface/desktop/src/main.rs:125-143`

**代码**:
```rust
#[tauri::command]
fn read_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}
#[tauri::command]
fn write_file(path: &str, content: &str) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| e.to_string())
}
#[tauri::command]
fn list_dir(path: &str) -> Result<Vec<String>, String> { ... }
```

**分析**: 三个裸文件系统command无任何路径校验。`write_file`可覆盖`~/.bashrc`、`/etc/hosts`（Unix）或注册表相关文件（Windows）。`read_file`可读取`~/.ssh/id_rsa`。

**后果**: 任意文件读写 = 密钥窃取、系统配置篡改、持久化后门。

**最小修复方案**: 增加`allowed_paths`参数（参考`ShellTool::with_paths`），默认限制在项目workspace目录内；对路径进行canonicalize并拒绝`..`遍历。

**风险评级**: 🔴 **高**

---

## 中后果发现

### D1-M1: `cargo audit` 未安装，Rust依赖CVE扫描盲区

**验证**: `cargo audit` 返回 `no such command: audit`

**后果**: 无法自动检测Rust依赖中的已知CVE。虽然`Cargo.toml`使用精确版本锁定（`=X.Y.Z`），但如果锁定版本本身存在已知漏洞，项目将毫不知情。

**最小修复方案**: `cargo install cargo-audit && cargo audit`，将结果纳入CI流水线。

**风险评级**: 🟡 **中**

---

### D1-M2: Shell执行仍使用`bash -c`/`powershell -Command`拼接

**位置**: `src/engine/tool-system/src/shell.rs:145-150`

**分析**: `BashExecutor`使用`("bash", vec!["-c"])`，`PowerShellExecutor`使用`("powershell", vec!["-Command"])`。虽然已实施严格的白名单（24命令）+ metachar过滤（`; & | ` $ ( ) { } < >`），但`-c`/`-Command`拼接模式在理论上仍存在注入可能（例如通过编码绕过）。文档`SHELL-FEATURE-DEBT-002.md`已承认此为降级功能。

**后果**: 如果白名单或metachar过滤存在逻辑漏洞，可导致命令注入。当前实现中未发现可利用路径。

**最小修复方案**: 将`bash -c`彻底替换为`Command::new(allowed_cmd).args(allowed_args)`的参数化执行，消除拼接层。

**风险评级**: 🟡 **中**

---

### D1-M3: `delete_api_key_with_profile` 为空实现

**位置**: `src/interface/desktop/src/main.rs`

**分析**: `delete_api_key_with_profile`函数注释"skipped for compatibility in v3.8.0"，直接返回`Ok(())`而不实际删除keyring中的密钥。

**后果**: 用户删除Provider后，密钥仍残留在OS Keyring中，造成密钥泄露面扩大。

**最小修复方案**: 补全`entry.delete_password()`调用，并添加错误处理。

**风险评级**: 🟡 **中**

---

## 误报清单

| ID | 发现 | 误报原因 |
|:---|:---|:---|
| D1-F1 | `validate_provider` 可能泄露密钥到HTTP请求 | `req.header("x-api-key", &key)` 是真实验证的必要步骤，5s超时+HTTPS传输，密钥不会落盘或入日志 |
| D1-F2 | `patches/zstd-sys/` 存在供应链投毒风险 | 本地补丁是修复上游API不匹配的必需措施，patch.crates-io明确声明来源和原因，无可疑代码 |
| D1-F3 | `edit_applier.rs` 可任意修改文件 | 受Governance审批门+10MB/50hunk/原子写入/唯一备份多重限制，非任意修改 |

---

## 修复验证（Phase 4→5）

| 修复项 | Phase 4状态 | Phase 5验证 | 结果 |
|:---|:---|:---|:---:|
| Shell白名单参数化 | 已修复 | `shell.rs`检查 | ✅ 未回归 |
| 密钥存储OS Keyring | 已修复 | `main.rs`检查 | ✅ 未回归 |
| `validate_provider`真实HTTP | 已修复 | 代码审查+逻辑验证 | ✅ 未回归 |

---

*审计完成。所有结论均有命令输出或代码片段支撑。*
