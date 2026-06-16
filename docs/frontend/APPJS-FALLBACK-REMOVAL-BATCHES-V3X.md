# STONE-AUDIT-V3X-DAY08 app.js Proven Fallback Removal Batches

## Scope

- Task: `STONE-AUDIT-V3X-DAY08`
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `fde3a5e110bea802016938f772478de62143c714`
- Mode: proven fallback removal batches
- Result: `BLOCKED / NO-OP`
- Production code modified: `NO`
- Commit planned: documentation-only receipt plus debt note

Day08 was allowed to remove only fallback blocks listed as `SAFE-TO-REMOVE` by
`docs/frontend/APPJS-CLOSURE-SAMPLING-V3X.md`. The Day07 report says the only
safe items were already completed before Day08:

1. Six pure Command Palette inline fallback bodies already removed.
2. Resource Dashboard module compatibility fallback already removed.
3. Session List View compatibility fallback already removed.

Day07 also explicitly says: "Do not enter a broad Day08 deletion pass. There is
no new `app.js` fallback block with enough Node + WebView evidence to delete
safely in Day08."

## Git Baseline

| Check | Result |
|---|---|
| `git pull --ff-only origin stone-audit-v3x-controlled-demolition` | Already up to date |
| Branch | `stone-audit-v3x-controlled-demolition` |
| HEAD before | `fde3a5e110bea802016938f772478de62143c714` |
| Historical dirty files | Present, not staged |
| app.js lines before | `5568` |
| app.js lines after | `5568` |

## Day07 Allowlist Receipt

| Day07 item | Day08 action | Status |
|---|---|---|
| Command Palette six pure inline fallback bodies | Verified old fallback bodies remain absent from wrapper methods | Already removed |
| Resource Dashboard compatibility fallback | Verified `resource-dashboard.js` has no `compatView` / `compatController` fallback | Already removed |
| Session List View compatibility fallback | Verified `sessions.js` has a safe no-op missing-view guard, not a full render fallback | Already removed |
| `setupKeyboardShortcuts()` inline fallback | Preserved as required | KEEP-FOR-NOW |
| Inspector compatibility fallback | Not touched | BLOCKED |
| Feedback / Markdown / Topbar / Chat / Model Picker fallbacks | Not touched | KEEP-FOR-NOW |
| Settings / Audit wrappers | Not touched | UNKNOWN |
| Provider / Shell / Checkpoint / Agent streaming | Not touched | KEEP-FOR-NOW |

## Removed Batches

No Day08 production deletion was performed.

| Batch | Removed fallback | Reason |
|---|---|---|
| Batch 1 | None | Day07 says all SAFE-TO-REMOVE candidates were already completed |

## Verification Commands

| Command | Result |
|---|---|
| `node --check src/interface/web/app.js` | PASS |
| `node tests/frontend/day22_command_palette_catalog_smoke.js` | PASS |
| `node tests/frontend/day28_command_palette_dom_smoke.js` | PASS |
| `node tests/frontend/day30_command_palette_delegation_smoke.js` | PASS |
| `node tests/frontend/day24_resource_dashboard_smoke.js` | PASS |
| `node tests/frontend/day14_sessions_thinking_modules_smoke.js` | PASS |
| `node tests/frontend/day29_session_list_dom_smoke.js` | PASS |
| `npm run test:security-gate` | FAIL, known baseline: 97 findings, 3 failures, 94 warnings |
| `git diff --name-only -- src/interface/web/app.js src/interface/web/index.html src/interface/web/modules src/interface/web/styles src/interface/web/style.css src/interface/desktop src/engine/tool-system` | No output |
| `git diff --cached --name-only` | No output before staging |
| `git diff --cached --check` | PASS before staging |
| `git diff --check` | Only historical dirty document CRLF warnings |

## Security Gate Result

`npm run test:security-gate` remains failing on the known baseline issues:

1. `src/interface/web/views/command-palette-view.js:36`
2. `src/interface/web/views/session-list-view.js:22`
3. `src/interface/web/views/session-list-view.js:26`

No production code was changed by this task, so Day08 did not introduce new
security-gate findings. This task does not mark the security gate as cleared.

## WebView Result

WebView affected UI smoke: `BLOCKED / NOT RUN`.

Reason: Day08 did not remove production code and did not create a new affected
UI surface. Existing Day07 evidence says Command Palette, Resource Dashboard,
and Session List View WebView receipts already passed in their earlier slices.
No new WebView claim is made by this receipt.

## Forbidden Diff Receipt

Forbidden production diff count: `0`.

Checked paths:

- `src/interface/web/app.js`
- `src/interface/web/index.html`
- `src/interface/web/modules`
- `src/interface/web/styles`
- `src/interface/web/style.css`
- `src/interface/desktop`
- `src/engine/tool-system`

## Rollback Point

- Rollback point: `fde3a5e110bea802016938f772478de62143c714`
- Since no production code was changed, rollback is documentation-only if this
  receipt needs correction.

## 工单 V3X-DAY08 完成

- Commit: `docs(frontend): record appjs fallback removal stop`
- 分支: `stone-audit-v3x-controlled-demolition`
- HEAD before / after: `fde3a5e110bea802016938f772478de62143c714` / pending commit
- app.js lines before/after: `5568` / `5568`
- removed batches: none
- node --check app.js: PASS
- affected smoke: PASS for day22, day28, day30, day24, day14, day29
- security-gate: FAIL, known baseline, not cleared
- WebView: BLOCKED / NOT RUN, no Day08 affected UI changed
- forbidden diff: 0
- old dirty files staged: NO
- next: run a dedicated WebView receipt before deleting any remaining fallback outside Day07 completed items
