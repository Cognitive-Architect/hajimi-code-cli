# STONE-AUDIT-V3X-DAY02 Inspector Safe DOM Rewrite Receipt

## Scope

- Task: `STONE-AUDIT-V3X-DAY02 | Inspector Safe DOM Rewrite Implementation`
- Date: 2026-06-13
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `e87a3fdf9f5664fec99905c5decb3d8de81a3fa1`
- Production target changed: `src/interface/web/modules/inspector.js`
- Test targets changed:
  - `tests/frontend/day18_inspector_smoke.js`
  - `tests/frontend/day25_inspector_safety_smoke.js`
  - `tests/frontend/day35_inspector_rendering_safe_dom_smoke.js`

## Summary

Inspector rendering for edit summary, context files, model info, diff preview, trace inspector, and context receipt now uses safe DOM construction with `createElement`, `textContent`, `appendChild`, and `replaceChildren`.

The prior Inspector module `innerHTML` sinks were removed instead of moved into `src/interface/web/views/inspector-view.js`.

人话版：原来 Inspector 是把一整段 HTML 字符串直接倒进页面；现在改成一个个创建小积木，再把文字放进积木里。这样就算文字里混进 `<script>` 或 `<img onerror=...>`，也只会被当成普通字看，不会被页面当成可执行代码。

## Changed Rendering Paths

| Path | Result | Notes |
|---|---|---|
| `renderEditSummary(app)` | Rewritten | Empty state and edit summary use text nodes. |
| `renderContextFiles(app)` | Rewritten | File names use `textContent`; basename behavior preserved. |
| `renderModelInfo(app)` | Rewritten | Provider/model strings use `textContent`. |
| `renderInspectorDiffPreview(app)` | Rewritten | Empty state, old diff button, hunk list, and diff lines use DOM nodes. |
| `renderTraceInspector(app)` | Rewritten | Trace rows and checkpoint buttons use DOM nodes and event listeners. |
| `renderContextReceiptPanel(app, receipt)` | Rewritten | Receipt rows and omitted blocks use DOM nodes. |

## Preserved Behavior

- Existing DOM IDs/classes/datasets were preserved for Inspector-owned output nodes.
- `inspectorOldDiffBtn` still calls `app.showGitDiff(app.currentDiffFile)`.
- Trace checkpoint `.trace-chk-btn.restore` still calls `app.restoreCheckpoint(btn.dataset.id)`.
- Trace checkpoint `.trace-chk-btn.compare` still opens the existing readonly guidance alert.
- Context Receipt empty state and populated state remain visible through `contextReceiptBody`.

## Test Harness Update

`day18` and `day25` test fixtures were upgraded to support a minimal safe DOM surface:

- `document.createElement`
- `appendChild`
- `replaceChildren`
- `textContent`
- recursive `.querySelectorAll('.class.another-class')`
- serialized `innerHTML` getter for assertions only

This does not reintroduce production `innerHTML` usage. The getter is only in Node test fakes so existing assertions can inspect rendered output.

## New Smoke

Added `tests/frontend/day35_inspector_rendering_safe_dom_smoke.js`.

It verifies:

- `src/interface/web/modules/inspector.js` no longer contains `innerHTML`, `outerHTML`, or `insertAdjacentHTML`.
- Diff preview treats malicious strings as escaped text.
- Trace inspector treats malicious strings as escaped text.
- Context Receipt treats malicious provider/model/role/omitted-block strings as escaped text.

## Validation Results

| Command | Result |
|---|---|
| `node --check src/interface/web/modules/inspector.js` | PASS |
| `node --check src/interface/web/views/inspector-view.js` | PASS |
| `node --check src/interface/web/controllers/inspector-controller.js` | PASS |
| `node --check tests/frontend/day18_inspector_smoke.js` | PASS |
| `node --check tests/frontend/day25_inspector_safety_smoke.js` | PASS |
| `node --check tests/frontend/day35_inspector_rendering_safe_dom_smoke.js` | PASS |
| `node tests/frontend/day18_inspector_smoke.js` | PASS |
| `node tests/frontend/day25_inspector_safety_smoke.js` | PASS |
| `node tests/frontend/day35_inspector_rendering_safe_dom_smoke.js` | PASS |
| `rg -n "innerHTML\|insertAdjacentHTML" src/interface/web/modules/inspector.js src/interface/web/views/inspector-view.js src/interface/web/controllers/inspector-controller.js` | PASS: no output |
| `git diff --name-only -- src/interface/web/app.js src/interface/web/index.html src/interface/desktop src/engine/tool-system` | PASS: no output |

## Security Gate

Command:

```powershell
npm run test:security-gate
```

Result:

- Overall status: KNOWN FAIL
- Findings: `97`
- Failures: `3`
- Warnings: `94`
- Allowlisted: `94`
- Inspector failures: `0`
- Inspector `innerHTML` warnings removed: `12`

Known failures remaining out of Day02 scope:

1. `src/interface/web/views/command-palette-view.js:36`
2. `src/interface/web/views/session-list-view.js:22`
3. `src/interface/web/views/session-list-view.js:26`

No security allowlist was changed.

## Forbidden Diff

Forbidden diff command:

```powershell
git diff --name-only -- src/interface/web/app.js src/interface/web/index.html src/interface/desktop src/engine/tool-system
```

Result: PASS, no output.

High-risk areas not touched:

- Provider / Keyring
- Shell execution
- Checkpoint restore/export/compare/replay
- Agent streaming
- CSP
- withGlobalTauri
- `src/interface/web/app.js`
- `src/interface/web/index.html`
- `src/interface/desktop`
- `src/engine/tool-system`

## WebView

- WebView: NOT RUN
- Reason: Day02 acceptance requires Node smoke and security-gate evidence. Real WebView validation should be handled by a separate browser wiring / WebView receipt task.
- This document does not claim real Tauri WebView click proof.

## Result

Day02 Inspector Safe DOM Rewrite: PASS for Node smoke and security-gate no-new-failure criteria.

Next recommended step: Day03 Inspector browser wiring + compatibility fallback removal, but only after reviewing this receipt and keeping WebView proof separate.
