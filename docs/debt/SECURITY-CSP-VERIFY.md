# Hajimi CSP Baseline Verify (B-06/15)

> 日期: 2026-05-16  
> 分支: `v3.8.0-batch-1`  
> HEAD: `d697414f42584a0d0c9c85346a6a692e691c4dad`  
> 对应债务: `CS-HAJIMI-003` / Tauri CSP 关闭 + Global Tauri API 开启  
> 前置文档: `docs/debt/SECURITY-DOM-AUDIT.md`

---

## 1. 结论

Day 06 已将 Tauri CSP 从 `null` 推进到基础 baseline，并完成静态 CSP blocker 收尾：欢迎页原有 inline `onclick` 已改为 `data-welcome-action` + `addEventListener` 绑定。`withGlobalTauri` 本轮保持 `true`，原因是 `src/interface/web/app.js` 仍有大量 `window.__TAURI__` 调用，直接关闭会中断文件树、Git、聊天、Provider、MCP、Checkpoint、Trace 等核心流程。

本轮状态建议: `CS-HAJIMI-003` 从 `OPEN` 推进到 `PARTIAL/VERIFY`，不能标记 `CLEARED`。关闭 global API 需要后续迁移。

---

## 2. CSP Baseline

配置文件: `src/interface/desktop/tauri.conf.json`

```text
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline';
img-src 'self' asset: data:;
connect-src 'self' http://127.0.0.1:* http://localhost:*
```

说明:

- `script-src 'self'`: 不允许远程脚本或 inline script。
- `style-src 'self' 'unsafe-inline'`: 当前 `index.html` 和 `app.js` 仍有大量 inline style，需要先保留。
- `img-src 'self' asset: data:`: 支持本地 logo / icon、Tauri asset 与 data 图片。
- `connect-src`: 支持本地 dev server / localhost 调试连接；业务网络请求应继续优先走 Tauri command 后端。

---

## 3. 输入与扫描 Receipt

### Git 坐标

```text
git branch --show-current
v3.8.0-batch-1

git rev-parse HEAD
d697414f42584a0d0c9c85346a6a692e691c4dad
```

### Tauri 配置

```text
rg -n "withGlobalTauri|csp" src/interface/desktop/tauri.conf.json
13: "withGlobalTauri": true
25: "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: data:; connect-src 'self' http://127.0.0.1:* http://localhost:*"
```

### Day 5 前置

```text
Get-ChildItem -LiteralPath docs -Recurse -Filter SECURITY-DOM-AUDIT.md
F:\hajimi-code-cli\docs\debt\SECURITY-DOM-AUDIT.md
```

### Global Tauri API 使用

```text
rg -n "__TAURI__" src/interface/web/app.js
当前命中数: 52
```

高频用途:

| 区域 | 代表调用点 | 用途 | 迁移优先级 |
|---|---|---|---|
| Workspace / 文件树 | `loadFileTree`, `createNewFile`, `createNewFolder`, `renameFile`, `deleteFile` | 文件读写、目录、重命名、删除 | P0 |
| Git / Search | `executeSearch`, `loadGitStatus`, `showGitDiff`, `gitCommit` | 搜索与版本控制 | P1 |
| Chat / LLM / Context | `streamChat`, `optimize_context`, `get_cumulative_stats` | 聊天流式响应、上下文压缩、Token 统计 | P1 |
| Provider / Profile / Backup | `get_provider_configs`, `save_provider_config`, `list_profiles` | 模型配置和密钥元数据操作 | P1 |
| MCP / tools | `execute_tool`, `mcp_*` wrappers | 工具与 MCP 调用 | P1 |
| Trace / Checkpoint / History | `subscribe_agent_trace`, `list_checkpoints`, `get_edit_history` | Agent Trace、检查点、回放 | P2 |
| Metrics / governance | `get_resource_metrics`, governance commands | 资源与治理控制 | P2 |

---

## 4. Global API 迁移计划

### `DEBT-CSP-B06-001`

`withGlobalTauri` 暂不关闭，迁移步骤如下:

1. 新建 `src/interface/web/modules/tauri-api.js`，集中封装:

```text
invoke(command, args)
createChannel()
isTauriAvailable()
```

2. 优先迁移 Workspace/FileOps:

```text
list_dir, read_file, write_file, create_dir, rename_path, delete_path, get_current_workspace
```

3. 迁移 Git/Search/Tool:

```text
execute_tool, run_command, git_status, git_diff, git_commit
```

4. 迁移 Chat/LLM:

```text
stream_chat, optimize_context, get_cumulative_stats
```

5. 迁移 Settings/MCP/Checkpoint/Trace:

```text
provider/profile/backup commands, mcp commands, checkpoint commands, trace channel
```

6. 全部迁移后将:

```json
"withGlobalTauri": false
```

并用 `node --check` + Tauri UI smoke 验证核心功能。

---

## 5. CSP 报错摘要

本轮未启动完整 `cargo tauri dev` 做 WebView console 观察，因此运行期 CSP violation 仍需实机补验。静态审计发现的欢迎页 inline `onclick` blocker 已在收尾阶段修复，当前不再通过放宽 `script-src` 处理内联事件。

已完成的静态 / 编译验证:

```text
cargo check -p hajimi-desktop
结果: 通过；沙箱首次写 target 遇到 Windows 拒绝访问，提升权限后通过

node --check src/interface/web/app.js
结果: 通过

cargo clippy -p hajimi-desktop -- -D warnings
结果: 通过；沙箱首次写 target 遇到 Windows 拒绝访问，提升权限后通过

rg -n "\"csp\"\s*:\s*null" src/interface/desktop/tauri.conf.json
结果: 无命中

rg -n "onclick=" src/interface/web src/interface/desktop/tauri.conf.json
结果: 无命中

rg -n "script.*https://|cdn" src/interface/web src/interface/desktop/tauri.conf.json
结果: 无命中
```

待补实机验证:

```text
cd src/interface/desktop
cargo tauri dev
观察 WebView console 是否有 CSP violation
重点验证: 文件树、Git 面板、聊天流式响应、Provider 设置、MCP、Checkpoint 列表
```

---

## 6. 质量门禁

| 检查 | 结果 |
|---|---|
| CSP 不再为 `null` | 通过 |
| 包含 `default-src 'self'` | 通过 |
| `img-src` 包含 `asset:` / `data:` | 通过 |
| `connect-src` 包含 `127.0.0.1` / `localhost` | 通过 |
| 无 `default-src *` | 通过 |
| 未引入远程 CDN 脚本 | 通过 |
| 无 inline `onclick=` blocker | 通过 |
| Day 5 DOM audit 存在 | 通过 |
| `withGlobalTauri` 状态记录 | 通过，保持 `true` |

---

## 7. 风险与回滚

主要风险: CSP 误伤资源加载，尤其 Tauri dev、图片协议、未来直接 fetch 到远程 Provider URL 的路径。

不建议直接回到 `csp: null`。如出现 blocker，应按 violation 最小放宽具体 directive，并在本文档追加 receipt。

回滚方式:

```text
git restore src/interface/desktop/tauri.conf.json docs/debt/SECURITY-CSP-VERIFY.md
```
