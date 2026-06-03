# Frontend DOM Contract

> 状态: ACTIVE
> 目标: 防止 index.html / app.js / style.css / modules 之间 DOM ID 漂移

## Contract Rules

1. 修改 DOM ID 必须同步 JS 绑定。
2. 修改 DOM ID 必须同步 CSS 选择器。
3. 修改 DOM ID 必须补 smoke test 或手动验收记录。
4. 禁止删除仍被 app.js / modules 使用的 DOM ID。
5. 新增 UI 区域必须声明 owner module。

## Core DOM IDs

| DOM ID | 所属区域 | 归属模块 | JS 绑定 | CSS 依赖 | Smoke |
|---|---|---|---|---|---|
| `activityBar` | Layout | app/settings-panel | TBD | TBD | TBD |
| `sidebar` | Layout | settings-panel | TBD | TBD | TBD |
| `sessionList` | Sessions | sessions.js | TBD | TBD | TBD |
| `fileTree` | Workspace | workspace.js | TBD | TBD | TBD |
| `rightInspector` | Inspector | inspector.js | TBD | TBD | TBD |
| `settingsPanel` | Settings | settings-panel.js | TBD | TBD | TBD |
| `providerListTab` | Provider | app.js / future provider module | TBD | TBD | TBD |
| `mcpServerListTab` | MCP | app.js / future mcp module | TBD | TBD | TBD |
| `inspectorDiffContent` | Inspector | inspector.js | TBD | TBD | TBD |
| `contextReceiptBody` | Receipt | inspector.js | TBD | TBD | TBD |

## Change Checklist

- [ ] DOM ID 是否被 JS 查询？
- [ ] DOM ID 是否被 CSS 使用？
- [ ] 是否有对应 smoke？
- [ ] 是否影响 packaged WebView？
- [ ] 是否需要更新截图或验收记录？
