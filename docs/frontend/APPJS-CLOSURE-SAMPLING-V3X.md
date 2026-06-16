# STONE-AUDIT-V3X-DAY07 app.js Closure Sampling

日期：2026-06-16

分支：`stone-audit-v3x-controlled-demolition`

HEAD before：`f800fcad6fc2d6346616c07c4fb7ebf8e8e8c8c4`

任务性质：docs-only sampling。本文只重新测量 `src/interface/web/app.js`，盘点 wrappers / fallbacks / delegation paths，并给 Day08 删除候选做证据分层。不删除代码，不修改生产文件。

人话版：`app.js` 还是一个很大的杂物间。今天只给箱子贴标签：哪些已经有新柜子能搬，哪些只是看起来能搬，哪些箱子里可能有煤气罐，先别碰。

## Baseline

| Item | Value |
|---|---|
| Branch | `stone-audit-v3x-controlled-demolition` |
| HEAD before | `f800fcad6fc2d6346616c07c4fb7ebf8e8e8c8c4` |
| `app.js` line count | `5568` |
| Production code changed | NO |
| WebView run in this task | NO |
| Old dirty files staged | NO at sampling time |

## Commands Used

```powershell
git branch --show-current
git rev-parse HEAD
git status --short
(Get-Content src/interface/web/app.js).Count
rg -n "setup|render|show|hide|fallback|compatibility|Hajimi.*Controller|Hajimi.*View|_.*Controller|_.*View" src/interface/web/app.js
rg -n "fallback|compatibility|Hajimi|_controller|_view" src/interface/web/app.js
rg -n "WebView|fallback|PASS|BLOCKED|known fail|Fallback|fallback removed|compatibility" docs/frontend tests/frontend
node --check src/interface/web/app.js
node tests/frontend/day18_inspector_smoke.js
node tests/frontend/day24_resource_dashboard_smoke.js
node tests/frontend/day28_command_palette_dom_smoke.js
node tests/frontend/day29_session_list_dom_smoke.js
node tests/frontend/day34_model_picker_smoke.js
npm run test:security-gate
git diff --name-only -- src/interface/web src/interface/desktop src/engine/tool-system
git diff --cached --check
```

## Current Wrapper / Fallback Inventory

| Area | app.js location | Current shape | Evidence | Classification |
|---|---:|---|---|---|
| Command catalog | `app.js:90` plus `getSlashCommands()` `2357-2375` | Catalog module preferred; inline slash command fallback still exists. | `day16`, `day21`; V1.5 slash command catalog smoke history. | KEEP-FOR-NOW |
| Command Palette setup/show/hide/render/nav/execute | `app.js:5094-5126` | Thin wrappers to `_commandPaletteView` / `_commandController`; six old inline DOM fallback bodies already removed. | `COMMAND-PALETTE-FALLBACK-REMOVAL-V3X.md`; day22/day28/day30 PASS; WebView PASS. | SAFE-TO-REMOVE? NO Fallback remains |
| Command Palette keyboard shortcuts | `app.js:5127-5188` | Delegates to controller if present; keeps inline shortcut fallback for many non-palette shortcuts. | Day3-E explicitly kept it. | KEEP-FOR-NOW |
| Session List render | `app.js:3561-3567`; module fallback in `sessions.js` removed | app.js is a thin wrapper; complete `sessions.js` render fallback removed. | `SESSION-LIST-VIEW-WEBVIEW-WIRING-V3X.md`; day14/day29 PASS; WebView PASS. | SAFE-TO-REMOVE? NO app.js wrapper remains public seam |
| Session core | `app.js:3541-3560` | Thin wrappers to `HajimiSessions`; storage/chat state remains in module/app state. | Day4 docs; day14/day29 PASS. | KEEP-FOR-NOW |
| Settings controller wrappers | `app.js:2032-2058` | Thin delegation to `HajimiSettingsController`. | Day4/Day5 docs; day19 PASS in prior closure docs. | UNKNOWN for removal |
| Settings shell / tabs | `app.js:470-481` | Thin delegation to `HajimiSettingsPanel`; shared sidebar routing. | Day4-D says shared shell, do not move. | KEEP-FOR-NOW |
| Inspector wrappers | `app.js:498-566`, `4110-4129` | Thin delegation to `HajimiInspector`. | `INSPECTOR-WEBVIEW-WIRING-V3X.md` says WebView BLOCKED and fallback not removed in module. | BLOCKED |
| Resource Dashboard wrappers | `app.js:4998-5001`, `5431-5435` | Thin delegation to `HajimiResourceDashboard`; module fallback removed. | `DASHBOARD-WEBVIEW-WIRING-V3X.md`; day24 PASS; WebView PASS. | SAFE-TO-REMOVE? NO app.js wrappers are public seam |
| Audit Log wrappers | `app.js:4690-4697` | Thin delegation to `HajimiAuditLog`. | day23 smoke exists; no dedicated WebView proof in Day07. | UNKNOWN |
| Feedback / Error toast | `app.js:5002-5025` | Delegates to `HajimiFeedbackView`, inline fallback remains. | `DAY5-B...`; day31 PASS; WebView deferred. | KEEP-FOR-NOW |
| Markdown service wrappers | `app.js:5026-5093` | Delegates to `HajimiMarkdownService`, inline fallback remains. | `DAY5-B...`; day31 PASS; WebView deferred. | KEEP-FOR-NOW |
| Topbar render wrappers | `app.js:226-254`, `786-811` | Delegates when view exists; inline fallback remains. | `DAY5-B...`; day31 PASS; WebView deferred. | KEEP-FOR-NOW |
| Chat controller | `app.js:2267-2356` | Delegates `setupChat()` if controller exists; large inline fallback remains. | `DAY5-D...`; day33 PASS; WebView deferred. | KEEP-FOR-NOW |
| Chat view/rendering | `app.js:2841-3054`, `3447-3478` | Delegates to `HajimiChatView` when present; inline fallback remains. | `DAY5-C...`; day32 PASS; WebView deferred. | KEEP-FOR-NOW |
| Thinking / trace wrappers | `app.js:1894-1914`, `2810-2823`, `3479-3536`, `5394-5430` | Mostly thin delegation to `HajimiThinkingUI`. | Trace/Agent streaming adjacent. | KEEP-FOR-NOW |
| Model Picker | `app.js:3583-3683` | Delegates to model picker modules; inline fallback remains. | day34 PASS; no Day07 WebView proof. | KEEP-FOR-NOW |
| Provider management | `app.js:3568-4403`, post-init provider test `5530-5563` | Provider CRUD/probe/keyring-adjacent logic remains in app.js. | Day06 says Provider boundary. | KEEP-FOR-NOW |
| Checkpoint/session browser | `app.js:4849-4995` | Checkpoint list/restore/export/compare/replay in app.js. | Day06 says Checkpoint boundary. | KEEP-FOR-NOW |
| Shell/tool/git/search/LSP | `app.js:170-205`, `567-811`, `1597-1810`, `4602-4667` | Tauri tool / shell / git / lsp execution paths. | Day06 says execution risk. | KEEP-FOR-NOW |
| Agent streaming/governance | `app.js:3055-3435`, `4698-4847`, `5334-5343` | Agent task, stream chat, approval, governance, agent command. | Day06 says streaming/execution risk. | KEEP-FOR-NOW |

## Deletion Readiness Table

| Candidate | Classification | Evidence | Day08 action |
|---|---|---|---|
| Six pure Command Palette inline fallback bodies | SAFE-TO-REMOVE already completed | `COMMAND-PALETTE-FALLBACK-REMOVAL-V3X.md` lists all six removed and WebView PASS. | No further deletion; verify they stay absent. |
| Dashboard module compatibility fallback | SAFE-TO-REMOVE already completed | `DASHBOARD-WEBVIEW-WIRING-V3X.md` says compat view/controller fallback removed; WebView PASS. | No further deletion in app.js. |
| Session list view compatibility fallback | SAFE-TO-REMOVE already completed | `SESSION-LIST-VIEW-WEBVIEW-WIRING-V3X.md` says complete fallback removed; WebView PASS. | No further deletion in app.js. |
| `setupKeyboardShortcuts()` inline fallback | KEEP-FOR-NOW | It owns non-palette shortcuts: Explorer/Search/Git/Agent Trace/Extensions/Settings/Chat Sessions/sidebar toggle. | Do not delete in Day08. |
| Inspector compatibility fallback in `modules/inspector.js` | BLOCKED | `INSPECTOR-WEBVIEW-WIRING-V3X.md` marks WebView BLOCKED and fallback removal NOT DONE. | Do not delete until dedicated WebView PASS. |
| Chat controller/view inline fallbacks | KEEP-FOR-NOW | Node smoke PASS but Day5-BE records WebView deferred and streaming core remains protected. | Do not delete in Day08. |
| Feedback/Markdown/Topbar inline fallbacks | KEEP-FOR-NOW | Node smoke PASS; no Day07 WebView proof. | Do not delete in Day08 unless separate WebView receipt exists. |
| Model Picker inline fallback | KEEP-FOR-NOW | day34 PASS; Provider-adjacent modal behavior and no Day07 WebView proof. | Do not delete in Day08. |
| Settings controller wrappers | UNKNOWN | Prior Node evidence exists; Settings full WebView visual receipt was deferred in Day4 closure. | Do not delete in Day08. |
| Audit log wrappers | UNKNOWN | Node day23 evidence exists; Day07 did not run WebView. | Do not delete in Day08. |
| Provider / Shell / Checkpoint / Agent streaming | KEEP-FOR-NOW | High-risk boundaries from Day06. | Forbidden in Day08 deletion slice. |

## Day08 Allowlist

Day08 should not delete production code by default based on this sampling. The only safe Day08 actions are verification or very narrow cleanup around already-removed fallback seams:

1. Verify Command Palette old fallback bodies remain absent from `setupCommandPalette()`, `showCommandPalette()`, `hideCommandPalette()`, `renderCommandList(query)`, `navigateCommandList(dir)`, and `executeSelectedCommand()`.
2. Verify `setupKeyboardShortcuts()` fallback remains present.
3. Verify `modules/resource-dashboard.js` still has no full `compatView` / `compatController` fallback.
4. Verify `modules/sessions.js` still has only the safe no-op missing-view guard, not the old full render fallback.

No new deletion target is marked fully safe in `app.js` by Day07.

人话版：Day08 不该拿大锤继续砸。最多就是拿手电筒复查：之前剪掉的旧线没有长回来，主电闸那根线还在。

## Smoke Coverage Matrix

| Area | Node smoke | WebView receipt | Status for deletion |
|---|---|---|---|
| Command Palette fallback removal | day22/day28/day30 PASS | Day3-E WebView PASS | Already removed; no further Day08 deletion |
| Resource Dashboard fallback removal | day24 PASS | Dashboard WebView PASS | Already removed in module |
| Session List fallback removal | day14/day29 PASS | Day4-C WebView PASS | Already removed in module |
| Inspector | day18/day25/day35 PASS | WebView BLOCKED | BLOCKED |
| Feedback/Markdown/Topbar | day31 PASS | WebView deferred | KEEP-FOR-NOW |
| Chat View | day32 PASS | WebView deferred | KEEP-FOR-NOW |
| Chat Controller | day33 PASS | WebView deferred | KEEP-FOR-NOW |
| Model Picker | day34 PASS | WebView not proven in Day07 | KEEP-FOR-NOW |
| Settings shell/controller | day19 historical PASS | settings visual WebView deferred | UNKNOWN |
| Audit Log | day23 historical PASS | no Day07 WebView proof | UNKNOWN |
| Provider/Shell/Checkpoint/Agent streaming | mixed or no direct safe smoke | high-risk | KEEP-FOR-NOW |

## Validation Results

| Check | Result |
|---|---|
| `node --check src/interface/web/app.js` | PASS |
| `node tests/frontend/day18_inspector_smoke.js` | PASS |
| `node tests/frontend/day24_resource_dashboard_smoke.js` | PASS |
| `node tests/frontend/day28_command_palette_dom_smoke.js` | PASS |
| `node tests/frontend/day29_session_list_dom_smoke.js` | PASS |
| `node tests/frontend/day34_model_picker_smoke.js` | PASS |
| `npm run test:security-gate` | FAIL, known baseline: 97 findings, 3 failures, 94 warnings, 94 allowlisted |

Security-gate known failures:

```text
src/interface/web/views/command-palette-view.js:36
src/interface/web/views/session-list-view.js:22
src/interface/web/views/session-list-view.js:26
```

This task does not fix these failures and does not mark them cleared.

## Forbidden Areas

Not touched and not eligible for Day08 deletion:

- Provider / Keyring
- Shell execution
- Checkpoint restore / export / compare / replay
- CSP / `withGlobalTauri`
- Agent streaming
- `src/interface/desktop/**`
- `src/engine/tool-system/**`
- `src/interface/web/app.js` production code in Day07

## Rollback

If Day08 accidentally removes a wrapper/fallback and UI behavior regresses, rollback candidates are:

- `src/interface/web/app.js`
- `src/interface/web/modules/inspector.js`
- `src/interface/web/modules/resource-dashboard.js`
- `src/interface/web/modules/sessions.js`
- `src/interface/web/index.html`
- affected smoke file under `tests/frontend/`

Primary rollback point for this Day07 sampling:

```text
f800fcad6fc2d6346616c07c4fb7ebf8e8e8c8c4
```

## Recommendation

Do not enter a broad Day08 deletion pass. There is no new `app.js` fallback block with enough Node + WebView evidence to delete safely in Day08.

Recommended next slice:

1. Run a dedicated WebView receipt for Feedback / Markdown / Topbar / Chat / Model Picker if deletion of their app.js fallbacks is desired.
2. Run a dedicated Inspector WebView retry before removing Inspector compatibility fallback.
3. Keep Provider / Shell / Checkpoint / Agent streaming out of closure deletion work.

## 工单 V3X-DAY07 完成

- Commit: `docs(frontend): sample appjs closure candidates`
- 分支: `stone-audit-v3x-controlled-demolition`
- HEAD before / after: before `f800fcad6fc2d6346616c07c4fb7ebf8e8e8c8c4`; after filled by commit receipt
- app.js lines: `5568`
- wrappers sampled: `89`
- SAFE-TO-REMOVE count: `3 already completed`
- KEEP-FOR-NOW count: `11 groups`
- UNKNOWN count: `2 groups`
- BLOCKED count: `1 group`
- security-gate: FAIL, known baseline 3 failures
- production changes: NO
- old dirty files staged: NO
- Day08 allowlist: verification-only; no new app.js deletion target
