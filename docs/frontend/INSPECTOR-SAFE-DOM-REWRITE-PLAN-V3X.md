# Inspector Safe DOM Rewrite Plan V3X

Date: 2026-06-13

Task: `STONE-AUDIT-V3X-DAY01 Inspector Safe DOM Rewrite Sampling + Test Plan`

Branch: `stone-audit-v3x-controlled-demolition`

HEAD before: `788c305481cb93efbdbf4ee7772210171e76607e`

Source work order: `docs/roadmap/Hajimi ToneFix/task/task01.md`

Scope: docs/test-plan only. This report does not modify production code and does not prove WebView behavior.

人话版：今天只是把 Inspector 这块“哪里还在用老式 HTML 拼接、哪里明天能安全改”标清楚。不是今天就修车，也不是今天就宣布车已经修好。

## 1. Git Baseline

Commands run:

```powershell
git branch --show-current
git rev-parse HEAD
git status --short
git log --oneline -n 20
```

Result:

| Item | Value |
|---|---|
| Branch | `stone-audit-v3x-controlled-demolition` |
| HEAD before | `788c305481cb93efbdbf4ee7772210171e76607e` |
| Production code modified by this task | NO |
| Old dirty files staged | NO |
| WebView run | NOT RUN |

Pre-existing dirty files were present in `git status --short`; they were not staged by this task.

## 2. Search Terms

Commands run:

```powershell
rg -n "innerHTML|insertAdjacentHTML|Context Receipt|diff|trace|receipt|render" src/interface/web/modules/inspector.js src/interface/web/views/inspector-view.js src/interface/web/controllers/inspector-controller.js tests/frontend
rg -n "diff|Diff" src/interface/web/modules/inspector.js src/interface/web/views/inspector-view.js
rg -n "receipt|Receipt|trace|Trace" src/interface/web/modules/inspector.js src/interface/web/views/inspector-view.js
rg -n "HajimiInspector" src/interface/web
Select-String -Path 'src/interface/web/modules/inspector.js' -Pattern 'innerHTML|insertAdjacentHTML'
```

Summary:

- `src/interface/web/views/inspector-view.js` currently exposes only shell view helpers: tab binding, close binding, visibility, and active tab state.
- `src/interface/web/controllers/inspector-controller.js` currently delegates shell behavior and guarded render calls back into the app/module.
- `src/interface/web/modules/inspector.js` still owns the actual Inspector rendering bodies for edit summary, context files, model info, diff preview, trace inspector, and Context Receipt.
- No `insertAdjacentHTML` use was found in the Inspector target files.

## 3. Current Rendering Ownership

| Area | Current owner | Evidence |
|---|---|---|
| Inspector shell tab binding | `src/interface/web/views/inspector-view.js` / `src/interface/web/controllers/inspector-controller.js` | `HajimiInspectorView.setActiveTab`, `HajimiInspectorController.showInspectorTab` |
| app wrapper delegation | `src/interface/web/app.js` | `window.HajimiInspector.*` wrappers around app lines reported by `rg -n "HajimiInspector" src/interface/web` |
| edit summary rendering | `src/interface/web/modules/inspector.js` | `renderEditSummary(app)` |
| context files rendering | `src/interface/web/modules/inspector.js` | `renderContextFiles(app)` |
| model info rendering | `src/interface/web/modules/inspector.js` | `renderModelInfo(app)` |
| diff preview rendering | `src/interface/web/modules/inspector.js` | `renderInspectorDiffPreview(app)` |
| trace rendering | `src/interface/web/modules/inspector.js` | `renderTraceInspector(app)` |
| Context Receipt rendering | `src/interface/web/modules/inspector.js` | `renderContextReceiptPanel(app, receipt)` |

## 4. HTML Sink Inventory

`Select-String` found 12 Inspector `innerHTML` writes:

| Line | Function / area | Current sink | Data source | Current escaping | Classification |
|---:|---|---|---|---|---|
| 148 | `renderEditSummary` empty state | `el.innerHTML = '<span ...>'` | static text | static | SAFE-TO-REWRITE |
| 153 | `renderEditSummary` payload | template string | `app.currentEditPayload.summary`, hunk count | `app.escapeHtml(summary)` | SAFE-TO-REWRITE |
| 165 | `renderContextFiles` empty state | static `innerHTML` | static text | static | SAFE-TO-REWRITE |
| 167 | `renderContextFiles` file list | `.map(...).join('')` | `app.chatContextFiles` | `app.escapeHtml(name)` | SAFE-TO-REWRITE |
| 179 | `renderModelInfo` empty state | static `innerHTML` | static text | static | SAFE-TO-REWRITE |
| 184 | `renderModelInfo` provider/model | template string | provider config name/model/id | `app.escapeHtml(name/model)` | SAFE-TO-REWRITE |
| 200 | `renderInspectorDiffPreview` empty/fallback | template string | `app.currentDiffFile` | `app.escapeHtml(currentDiffFile)` | SAFE-TO-REWRITE |
| 242 | `renderInspectorDiffPreview` hunk list | `container.innerHTML = html` | `currentEditPayload.hunks`, file path, lines | `app.escapeHtml(...)` | SAFE-TO-REWRITE |
| 254 | `renderTraceInspector` empty state | static `innerHTML` | static text | static | SAFE-TO-REWRITE |
| 261 | `renderTraceInspector` event list | template string | `app.traceEvents` | `app.escapeHtml(step/iteration/details)` | SAFE-TO-REWRITE |
| 339 | `renderContextReceiptPanel` empty state | static `innerHTML` | static text | static | SAFE-TO-REWRITE |
| 367 | `renderContextReceiptPanel` receipt body | template string | receipt provider/model/role/mode/omitted blocks | `app.escapeHtml(...)` for text fields | SAFE-TO-REWRITE |

Meaning of `SAFE-TO-REWRITE`: the path is suitable for Day02 safe DOM construction using `createElement`, `textContent`, `appendChild`, `replaceChildren`, `className`, `dataset`, and explicit `addEventListener`. It does not mean the current code is CLEARED.

No item is marked CLEARED in this report.

## 5. Diff / Trace / Context Receipt Coverage Plan

| Path | Current evidence | Day02 target | Required smoke evidence |
|---|---|---|---|
| Diff empty state | `tests/frontend/day18_inspector_smoke.js`; `tests/frontend/day25_inspector_safety_smoke.js` | Render empty state with safe DOM nodes and optional old-diff button via `createElement('button')` | Empty state text present; old diff button click still calls `showGitDiff` when applicable |
| Diff hunk list | `day25` checks malicious summary/path/lines are escaped | Build card, hunk rows, old/new line rows as DOM nodes with `textContent` | Malicious `<img>`, `<script>`, `<svg>` strings appear as text, not executable HTML |
| Trace empty state | `day25` checks empty trace tab render | Render empty state using DOM nodes | Empty state text present |
| Trace event list | `day25` checks malicious step/iteration/details are escaped | Build event cards and checkpoint buttons with DOM nodes and dataset values | Malicious strings are text; restore/compare button handlers still bind; forbidden checkpoint actions are not executed in smoke |
| Context Receipt empty state | `day25` checks empty receipt state | Render receipt empty state using DOM nodes | Empty text present |
| Context Receipt full body | `day25` checks malicious receipt fields are escaped | Build header, grid, omitted list, disclaimer as DOM nodes | Provider/model/bridge role/omitted names/reasons are text; no executable tags |

## 6. Day02 Implementation Candidate Table

| Candidate | File | Proposed helper | Status | Notes |
|---|---|---|---|---|
| Shared DOM clearing helper | `src/interface/web/views/inspector-view.js` or `src/interface/web/modules/inspector.js` | `clearNode(node)` / `replaceChildren` | SAFE-TO-REWRITE | Keep it tiny; no broad UI framework |
| Text node helper | same touched file | `appendTextEl(parent, tag, className, text)` | SAFE-TO-REWRITE | Use `textContent` only |
| Button helper | same touched file | `appendButton(parent, className, text, onClick)` | SAFE-TO-REWRITE | Needed for old Diff / trace checkpoint buttons |
| `renderEditSummary` | `src/interface/web/modules/inspector.js` first, optional later move to view | rewrite body with DOM nodes | SAFE-TO-REWRITE | Not the primary Day6-D blocker but low-risk |
| `renderContextFiles` | same | rewrite body with DOM nodes | SAFE-TO-REWRITE | Preserve visible file basename behavior |
| `renderModelInfo` | same | rewrite body with DOM nodes | SAFE-TO-REWRITE | Preserve provider/model display |
| `renderInspectorDiffPreview` | same | rewrite body with DOM nodes | SAFE-TO-REWRITE | Primary blocker candidate |
| `renderTraceInspector` | same | rewrite body with DOM nodes | SAFE-TO-REWRITE | Primary blocker candidate; preserve restore/compare listeners |
| `renderContextReceiptPanel` | same | rewrite body with DOM nodes | SAFE-TO-REWRITE | Primary blocker candidate; preserve omitted list limit |
| Moving render bodies into `views/inspector-view.js` | `src/interface/web/views/inspector-view.js` | only after safe DOM rewrite | KEEP-FOR-NOW until safe DOM tests pass | Do not move old `innerHTML` strings into view |
| Security allowlist expansion | `tests/security/security_audit_allowlist.json` | N/A | BLOCKED | Day01/Day02 should not use allowlist as shortcut |

Day02 recommended order:

1. Add or extend `tests/frontend/day35_inspector_rendering_safe_dom_smoke.js` to fail on raw executable HTML and to inspect `views/inspector-view.js` for dangerous sinks.
2. Rewrite Diff preview with safe DOM nodes.
3. Rewrite Trace inspector with safe DOM nodes.
4. Rewrite Context Receipt panel with safe DOM nodes.
5. Then rewrite small edit/context/model helpers if needed.
6. Run `npm run test:security-gate` and require no new Inspector view/module failures.

## 7. Required Day02 Allowed Files

Day02 should allow only:

```text
src/interface/web/modules/inspector.js
src/interface/web/views/inspector-view.js
src/interface/web/controllers/inspector-controller.js
tests/frontend/day18_inspector_smoke.js
tests/frontend/day25_inspector_safety_smoke.js
tests/frontend/day35_inspector_rendering_safe_dom_smoke.js
docs/frontend/INSPECTOR-SAFE-DOM-REWRITE-V3X.md
```

Day02 should continue forbidding:

```text
src/interface/web/app.js
src/interface/web/index.html
src/interface/desktop/**
src/engine/tool-system/src/shell.rs
Provider / Keyring
Shell
Checkpoint restore/export/compare/replay
Agent streaming
CSP / withGlobalTauri
security allowlist shortcut
```

## 8. Validation Results

| Command | Result |
|---|---|
| `node --check src/interface/web/modules/inspector.js` | PASS |
| `node tests/frontend/day18_inspector_smoke.js` | PASS: `day18 inspector module smoke: PASS` |
| `node tests/frontend/day25_inspector_safety_smoke.js` | PASS: `day25 inspector safety smoke: PASS (4 scenarios)` |
| `npm run test:security-gate` | KNOWN FAIL: findings 109; failures 3; warnings 106; allowlisted 106 |
| `git diff --name-only -- src/interface/web src/interface/desktop src/engine/tool-system` before docs creation | no output |

Known security-gate failures remain:

```text
src/interface/web/views/command-palette-view.js:36
src/interface/web/views/session-list-view.js:22
src/interface/web/views/session-list-view.js:26
```

Inspector `innerHTML` findings in `src/interface/web/modules/inspector.js` are currently allowlisted warnings, not new failures. Day6-D showed that moving those legacy sinks into `views/inspector-view.js` without safe DOM rewrite creates new failures. Therefore Day02 must rewrite the rendering sinks rather than move old HTML strings.

## 9. Stop-loss Table

| Trigger | Required action |
|---|---|
| `npm run test:security-gate` reports any new Inspector failure | STOP; revert production attempt or commit docs-only BLOCKED receipt |
| Day02 requires security allowlist expansion | STOP; do not continue without explicit separate approval |
| Day02 requires Provider/Keyring/Shell/Checkpoint/Agent streaming changes | STOP; record scope blocker |
| WebView is not run | Mark `WebView: NOT RUN`, not PASS |
| Any UNKNOWN cannot be resolved by Node smoke | Keep UNKNOWN; do not mark CLEARED |

## 10. Debt / Unknowns

| Item | Status | Note |
|---|---|---|
| WebView proof for safe DOM rewrite | UNKNOWN / NOT RUN | Day01 is docs/test-plan only |
| Inspector rendering still uses 12 allowlisted `innerHTML` sinks in module | KNOWN DEBT | Must be rewritten before moving to view |
| Existing command/session security-gate failures | KNOWN FAIL | Out of Day01 scope |
| Provider/Keyring/Shell/Checkpoint/Agent streaming | KEEP-FOR-NOW | Not sampled beyond forbidden boundary |

Additional debt record:

```text
docs/debt/INSPECTOR-SAFE-DOM-REWRITE-DEBT-V3X.md
```

## 11. Recommendation

Recommendation: allow Day02 to proceed only as `Inspector Safe DOM Rewrite Implementation`.

Day02 should start with tests and safe DOM helpers, not with script wiring and not with Provider work.

Allowed first production target: `src/interface/web/modules/inspector.js`.

Blocked shortcut: moving current legacy `innerHTML` strings into `src/interface/web/views/inspector-view.js`.

Final Day01 result: PASS for sampling and planning; implementation remains NOT DONE by design.
