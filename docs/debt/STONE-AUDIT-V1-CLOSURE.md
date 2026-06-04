# STONE-AUDIT-V1 Closure

> 状态: COMPLETED  
> 日期: 2026-06-03  
> 分支: `feature/toolfix-deepseek-schema`  
> HEAD: `fb6d900fc35b447162bb35e78391ad4a9f953c32`

## 1. 本轮目标

- 生成本地史山体检证据。
- 建立 DOM 合约。
- 建立巨石文件冻结规则。
- 跑基础验证。
- 明确下一阶段治理入口。

## 2. 已完成提交

| Day | Commit | 说明 |
|---|---|---|
| Day 1 | `2998c550e024407a830057936c9449d93d50a2bf` | 本地证据捕获 |
| Day 2 | `0db0cb5a0daf9e0be8214de3ab1aa8cf28c64730` | DOM 合约与冻结规则 |
| Day 3 | `fb6d900fc35b447162bb35e78391ad4a9f953c32` | Closure 与验证矩阵 |

## 3. 命令结果

| 检查项 | 命令 | 结果 | 备注 |
|---|---|---|---|
| Git branch | `git branch --show-current` | `feature/toolfix-deepseek-schema` | |
| Git HEAD | `git rev-parse HEAD` | `fb6d900fc35b447162bb35e78391ad4a9f953c32` | |
| Git status | `git status --short` | 正常 | 无任何非文档类的暂存与未暂存改动 |
| JS syntax | `node --check src/interface/web/app.js` | PASS | app.js 语法验证通过 |
| JS modules | `node --check modules/*.js` | PASS | modules 下所有 js 语法验证通过 |
| Security gate | `npm run test:security-gate` | PASS | 安全阈值扫描通过 |
| Rust check | `cargo check --workspace` | PASS | 全工作区 Rust 编译检查通过 |
| Diff check | `git diff --check` | PASS | 无格式或空白字符警告 |

## 4. 可选 smoke 结果

| Smoke | 结果 | 备注 |
|---|---|---|
| `day13_workspace_modules_smoke.js` | PASS | 运行通过 |
| `day14_sessions_thinking_modules_smoke.js` | PASS | 运行通过 |
| `day16_slash_palette_smoke.js` | PASS | 运行通过 |
| `day17_thinking_ui_v2_security_smoke.js` | PASS | 运行通过 |
| `day18_inspector_smoke.js` | PASS | 运行通过 |
| `day19_settings_smoke.js` | PASS | 运行通过 |
| `day20_tauri_channel_envelope_regression.test.js` | PASS | 运行通过 |

## 5. 高风险文件状态

| 文件 | 状态 | 下一步 |
|---|---|---|
| `src/interface/web/app.js` | FROZEN | 后续拆 command/slash/audit/resource/provider |
| `src/interface/desktop/src/main.rs` | FROZEN | 后续规划 `commands/*` |
| `src/interface/web/style.css` | FROZEN | 后续建分区 / 拆 CSS |
| `src/interface/web/index.html` | CONTRACTED | 维护 `DOM-CONTRACT.md` |
| `src/engine/tool-system/src/shell.rs` | WATCH | 不放宽白名单 |
| `src/interface/desktop/tauri.conf.json` | WATCH | 不放宽 CSP / global API |

## 6. 未完成项

- [ ] 完整 Top 30 大文件扫描。
- [ ] 复杂度工具扫描。
- [ ] 测试覆盖率扫描。
- [ ] Tauri packaged WebView smoke。
- [ ] app.js 下一切片拆分。

## 7. 下一阶段建议

如果基础验证全部通过，推荐进入 `STONE-AUDIT-V1.5`：

1. 优先拆 `command-palette` 或 `slash-palette`。
2. 不碰 Agent streaming。
3. 不碰 checkpoint restore。
4. 不碰 provider keyring。
5. 每拆一个模块，必须新增或复用 smoke。

## 8. 结论

`STONE-AUDIT-V1 completed. 项目已具备进入 V1.5 低风险模块拆分的前置条件。`

## 9. V1.5-B 状态备注

2026-06-04 执行 `STONE-AUDIT-V1.5-B: Extract Slash Command Catalog`：

- 已将 `getSlashCommands()` 的纯命令目录抽到 `src/interface/web/modules/slash-command-catalog.js`。
- `app.js` 保留 inline fallback；新模块存在时优先通过 `window.HajimiSlashCommandCatalog.createSlashCommandCatalog()` 读取。
- 新增 `tests/frontend/day21_slash_palette_app_integration_smoke.js` 覆盖 app 侧装配、feature flag、缺模块兜底、direct+low 自动发送、高风险只填充不自动发送、catalog trigger 等价。
- 未移动 `handleChatCommand()` 或任何 slash 命令执行分支。
- 未修改 DOM ID、CSS、security gate allowlist、shell、CSP、`withGlobalTauri`、provider keyring、checkpoint restore、Agent streaming。
- Tauri WebView slash 真实点击验证仍为 `PENDING-MANUAL-SMOKE`，不得标记为 `CLEARED`。

## 10. V1.5-C 状态备注

2026-06-04 执行 `STONE-AUDIT-V1.5-C: Extract Command Palette Catalog`：

- 已将 command palette 的纯命令目录抽到 `src/interface/web/modules/command-palette-catalog.js`。
- `app.js` 保留 inline fallback；新模块存在时优先通过 `window.HajimiCommandPaletteCatalog.createCommandPaletteCatalog(this)` 读取。
- 新增 `tests/frontend/day22_command_palette_catalog_smoke.js` 覆盖 catalog 导出、核心命令 id、命令顺序、label/key/action 保留、低风险 action 调用、模块缺失 fallback。
- 未移动 `setupCommandPalette()`、`showCommandPalette()`、`hideCommandPalette()` 或任何命令对应的实际函数。
- 未修改 DOM ID、CSS、shell、CSP、`withGlobalTauri`、provider keyring、checkpoint restore、Agent streaming。
- Tauri WebView command palette 真实点击验证仍为 `PENDING-MANUAL-SMOKE`，不得标记为 `CLEARED`。

## 11. V1.5-D 状态备注

2026-06-04 执行 `STONE-AUDIT-V1.5-D: Extract Audit Log Readonly Module`：

- 已将 audit log 的只读加载与渲染逻辑抽到 `src/interface/web/modules/audit-log.js`。
- `app.js` 保留 `loadAuditLogs()` / `setupAuditLog()` wrapper，并通过 `window.HajimiAuditLog` 转调新模块。
- 新增 `tests/frontend/day23_audit_log_smoke.js` 覆盖 refresh 绑定、Tauri 不可用安全返回、空日志、多日志、status class、恶意内容转义、`auditLogBodyTab` 缺失容错。
- 未新增 audit 写入、删除、清空能力。
- 未修改 `get_audit_logs` 后端命令、Tauri command、DOM ID、CSS、shell、CSP、`withGlobalTauri`、provider/keyring、checkpoint、Agent streaming。
- Tauri WebView audit log 真实点击验证仍为 `PENDING-MANUAL-SMOKE`，不得标记为 `CLEARED`。
