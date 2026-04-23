# 红队对抗审计综合报告：执行摘要与风险热力图

**项目**: Hajimi V3 (Phase 5 收官代码)  
**审计编号**: HAJIMI-REDTEAM-SRC-AUDIT-001  
**审计日期**: 2026-04-16  
**审计方法**: 静态源码分析 + 依赖漏洞扫描 + 架构违规检测 + UX 可达性分析  
**总体风险评级**: **🔴 HIGH（高）**

---

## 1. 执行摘要（后果优先）

本次红队审计对 `src/` 全源码（445 文件 / 55,048 行）实施了三线饱和攻击：供应链安全（R-01）、维护性灾难（R-02）、用户体验死区（R-03）。结论如下：

- **R-01 供应链与基础设施**：发现 4 项高危 npm CVE（含 CVSS 8.8 的 `tar` 漏洞）、`curl | bash` 远程脚本投毒向量、无认证 WebRTC 信令服务器、以及可被绕过的 Shell 黑名单注入防护。Rust 侧因缺失 `cargo audit` 完全盲飞。
- **R-02 维护性灾难**：发现 **1,292 个 TODO/DEBT 标记**、两处核心 `simhash64` 算法复制粘贴、`codex-twist` 代码双轨制维护、`p2p-sync` Rust 存根与 TS 实现割裂、以及多处 `unsafe` 块缺少 `SAFETY` 注释。
- **R-03 用户体验死区**：发现大量“僵尸功能”（ADR 搜索、安全审计、TypeRacing）被锁在库代码中无任何入口；MCP 服务器仅暴露 3/40+ 工具（覆盖率 7.5%）；VSCode 插件 60 条命令中 56 条为空壳 stub；五层记忆系统零可视化。

**最关键的结论**：Hajimi V3 的引擎层极度丰富，但“安全边界薄弱 + 维护成本高昂 + 用户可及表面积极小”的三重赤字，意味着工程投入大量沉淀在不可见、不可达、不可维护的 dark code 中。若被攻击者利用，不仅可导致单点沦陷，更可沿“信令 MITM → Shell 注入 → 密钥窃取 → 记忆密文离线爆破”的链路完成全系统接管。

---

## 2. 风险热力图（模块 × 风险维度）

| 模块 | 供应链/安全 (R-01) | 维护性 (R-02) | UX/可达性 (R-03) | 综合 |
|:---|:---:|:---:|:---:|:---:|
| `foundation/wasm` | 🟠 中 (unsafe FFI) | 🟠 中 (WASM/JS 边界) | 🟢 低 (底层模块) | 🟠 中 |
| `foundation/storage` | 🟢 低 | 🟢 低 (设计稳定) | 🟢 低 | 🟢 低 |
| `foundation/scripts` | 🔴 高 (`curl \| bash`) | 🟢 低 | 🟢 低 | 🔴 高 |
| `engine/p2p-sync` | 🔴 高 (无认证信令) | 🔴 高 (Rust 存根割裂) | 🔴 高 (配置地狱) | 🔴 高 |
| `engine/tool-system` | 🔴 高 (Shell 注入) | 🔴 高 (29 文件巨兽) | 🟠 中 (help 缺失) | 🔴 高 |
| `engine/search` | 🟠 中 (Jieba C++ 绑定) | 🟠 中 (219 行标准略超) | 🟢 低 | 🟠 中 |
| `engine/llm-core` | 🟢 低 (密钥走环境变量) | 🟢 低 | 🟢 低 | 🟢 低 |
| `intelligence/memory` | 🟠 中 (Scrypt 降级) | 🟢 低 (结构清晰) | 🔴 高 (零可视化) | 🟠 中 |
| `intelligence/knowledge` | 🟢 低 | 🟢 低 (185 行目标略超) | 🔴 高 (search.rs 僵尸) | 🟠 中 |
| `intelligence/typeracing` | 🟢 低 | 🟢 低 | 🔴 高 (无 TUI 接线) | 🟠 中 |
| `interface/mcp-server` | 🟠 中 (输入校验有但工具 Desert) | 🟢 低 (165 行清晰) | 🔴 高 (仅 3 工具) | 🔴 高 |
| `interface/terminal` | 🟢 低 | 🟢 低 | 🔴 高 (功能面板缺失) | 🟠 中 |
| `interface/vscode` | 🟢 低 | 🟠 中 (56/60 stub) | 🔴 高 (openAdr 缺失) | 🔴 高 |
| `interface/web` | 🟢 低 | 🟢 低 | 🔴 高 (850B 骨架) | 🟠 中 |
| `crates/codex-twist` | 🟠 中 (FFI unsafe) | 🔴 高 (与 intelligence 重复) | 🟢 低 | 🔴 高 |
| **全局依赖树** | 🔴 高 (npm 4 high CVE) | 🟠 中 (版本未 pin) | 🟢 低 | 🔴 高 |

**图例**: 🔴 高 = 可直接导致系统沦陷/生产事故/用户流失；🟠 中 = 显著增加成本或风险；🟢 低 = 可控或在可接受范围。

---

## 3. 三线战况总结

### R-01/A：供应链与基础设施 — 战线失守
- **高危漏洞**: `tar` (CVSS 8.8), `path-to-regexp` (CVSS 7.5), `express-rate-limit` (CVSS 7.5) 未修复。
- **供应链投毒**: `src/foundation/scripts/setup/install-evm-toolchain.sh:53` 使用 `curl -L https://foundry.paradigm.xyz | bash`。
- **P2P MITM**: `signaling-server.js` 无连接认证，`clientId` 由 `Math.random()` 生成。
- **命令注入**: `shell.rs` 黑名单可被简单绕过（如 `r\m -rf /` 变体、反引号、环境变量间接调用）。
- **Rust 盲区**: `cargo audit` 未安装，无法评估 `tantivy`、`jieba-rs`、`sqlx`、`ort` 等 crates 的 CVE 状态。

### R-02/B：维护性灾难 — 结构性腐烂
- **债务规模**: 1,292 个 TODO/FIXME/DEBT 标记，35% 聚集在 `engine/` 核心层。
- **重复代码**: `simhash64` 复制于 `engine/search/src/tantivy_index.rs` 与 `intelligence/knowledge/src/adr_index.rs`。
- **双轨制**: `src/crates/hajimi-codex-twist/` 与 `src/intelligence/codex-twist/` 同时维护相同逻辑。
- **幽灵模块**: `engine/p2p-sync/src/lib.rs` 25 行纯占位，真实实现在 TS 中。
- **unsafe 文档缺口**: `vector_text_hybrid.rs:82` 等处直接调用 `from_raw_parts` 而无 SAFETY 注释。

### R-03/C：用户体验死区 — 功能沙漠
- **僵尸功能 3 具**: ADR 搜索 (`search.rs`)、安全审计 (`security.rs`)、TypeRacing (`typeracing/`) 均无用户入口。
- **MCP 覆盖率 7.5%**: 仅 `hajimi_search` / `hajimi_add` / `hajimi_stats`，40+ 引擎工具被锁死。
- **VSCode 空壳率 93%**: 60 条命令中 56 条仅弹出 `Executing: ${cmd}` toast；`openAdr` / `gotoAdr` 完全缺失。
- **五层记忆黑箱**: Session/Auto/Dream/Graph/Cloud 无任何状态查询 CLI 或可视化面板。
- **P2P 配置地狱**: 配置分散在 7+ 文件，无向导、无 QR 码、无一键配对。

---

## 4. 关键攻击链建模（Kill Chain）

攻击者可将三战线弱点串联成完整杀伤链：

```
Step 1: 供应链投毒
  └─> 篡改 foundry.paradigm.xyz 响应
      └─> 开发者执行 install-evm-toolchain.sh
          └─> 开发机沦陷，窃取 ~/.cargo/credentials 与 API 密钥

Step 2: P2P 信令劫持
  └─> 攻击者连接无认证 signaling-server
      └─> 伪造 peerId，拦截 SDP/ICE
          └─> MITM 控制 Peer 间 DataChannel

Step 3: Shell 注入执行
  └─> 通过 MITM 通道诱导目标运行工具调用
      └─> 利用 shell.rs 黑名单绕过执行任意命令
          └─> 在目标机器植入持久化后门

Step 4: 记忆密文窃取与降级爆破
  └─> 窃取 CloudMemory 加密分块
      └─> 触发或利用 degraded_mode 的 Scrypt 回退
          └─> 以低参数离线暴力破解用户端到端加密数据
```

**结论**: 单一弱点不一定致命，但串联后形成了从“开发者机器 → 网络层 → 终端用户 → 加密数据”的完整攻击路径。

---

## 5. Top 10 后果清单（按严重程度排序）

| 排名 | 后果 | 来源 | 可利用性 |
|:---|:---|:---|:---:|
| 1 | 开发者机器完全沦陷，后门持久化 | `curl \| bash` | 高 |
| 2 | P2P 同步被 MITM，用户数据在传输中被窃/篡改 | 无认证信令 | 高 |
| 3 | 攻击者执行任意系统命令（RCE） | `shell.rs` 黑名单绕过 | 高 |
| 4 | npm 依赖漏洞在生产构建中被触发（RCE/DoS） | `tar` / `path-to-regexp` | 中 |
| 5 | 用户云端记忆密文被离线暴力破解 | Scrypt 降级模式 | 中 |
| 6 |  Rust 侧带毒 crate 引入后门而无人知晓 | 缺失 `cargo audit` | 中 |
| 7 | 工程团队核心成员离职后代码库无法维护 | 1,292 DEBT + 双轨制 | 高 |
| 8 | AI Agent / 用户只能使用 7.5% 的系统能力 | MCP 3 工具限制 | 高 |
| 9 | IDE 插件用户大量流失（点击无反应） | 56/60 stub 命令 | 高 |
| 10 | ADR 知识图谱构建后无法被检索，投入归零 | `search.rs` 僵尸 | 高 |

---

## 6. 修复优先级（P0 / P1 / P2）

### P0 — 立即修复（1–3 天，阻止沦陷）
1. **移除或替换 `curl | bash`**：下载脚本后校验 checksum/PGP 签名再执行。
2. **修复 npm 高危 CVE**：执行 `npm audit fix`，升级 `tar`、`path-to-regexp`、`express-rate-limit`。
3. **加固信令服务器**：增加连接前共享密钥验证或 token 鉴权；将 `Math.random()` 替换为 `crypto.randomBytes()`。
4. **重构 Shell 工具**：弃用字符串黑名单，改用 allow-list / 参数化命令 / 禁止 shell interpretation。

### P1 — 短期修复（1–2 周，降低风险）
5. **安装 `cargo audit` 并集成 CI**：确保 Rust CVE 零盲区。
6. **清理 `codex-twist` 双轨制**：删除 `src/crates/hajimi-codex-twist/` 中的重复代码，或彻底将其转为 thin wrapper。
7. **补充 `unsafe` SAFETY 注释**：至少覆盖 `vector_text_hybrid.rs`、`integration/src/end_to_end.rs`。
8. **MCP 工具桥接**：通过 `ffi-bridge` 将 `ReadFileTool`、`GrepTool`、`GitStatusTool`、`SecurityAuditTool` 注册到 MCP，覆盖率提升至 ≥15 个。

### P2 — 中期改进（2–4 周，提升 UX 与可维护性）
9. **为五层记忆添加 `hajimi memory status` CLI**：输出各层条目数、token 占用、同步状态。
10. **实现 VSCode 命令骨架填充**：删除或实现 56 个 stub 命令；补全 `openAdr` / `gotoAdr`。
11. **统一帮助系统**：在 Terminal UI 与 MCP 中提供全局 `help` 命令，枚举所有可用工具。
12. **建立 DEBT 清收机制**：将 1,292 个 TODO/FIXME 导入 Issue 系统，设定 owner 与 deadline。

---

## 7. 最终综合风险评级

| 维度 | 评级 | 核心理由 |
|:---|:---:|:---|
| 安全可利用性 | 🔴 高 | 存在直接 RCE、MITM、供应链投毒的高可利用路径 |
| 维护可持续性 | 🔴 高 | 1,292 债务标记 + 双轨制 + 认知割裂，新人 onboarding 成本极高 |
| 用户可及性 | 🔴 高 | 大量功能为僵尸代码，MCP/IDE/CLI 入口严重不足 |
| **综合风险** | **🔴 HIGH** | 三线同时亮红灯，弱点可串联成完整杀伤链 |

**红队建议**：在 P0 项未完成前，不建议将 Hajimi V3 作为生产级系统对外发布或大规模推广。当前代码基更适合作为“功能验证原型”而非“Phase 5 收官交付物”。

---

*本报告由 HAJIMI-REDTEAM-SRC-AUDIT-001 三线审计结果综合生成。详细技术证据请参阅：*
- *`audit report/redteam/R01-SUPPLY-CHAIN-RISK.md`*
- *`audit report/redteam/R02-MAINTENANCE-NIGHTMARE.md`*
- *`audit report/redteam/R03-UX-DEADZONE.md`*
