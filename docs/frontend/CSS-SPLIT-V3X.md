# CSS-SPLIT-V3X

## Scope

Task: STONE-AUDIT-V3X-CONTROLLED-DEMOLITION Day 1 CSS full split.

Branch: `stone-audit-v3x-controlled-demolition`

HEAD before split: `26f250fb8f4d530f5d0039901cb19d0597c57c2d`

Commit: NO

Push: NO

## Line Counts

| Item | Lines |
| --- | ---: |
| `src/interface/web/style.css` before line count | 3961 |
| `src/interface/web/style.css` after line count | 20 |

Note: Day 0 and the Day 1 task use 3961 as the operative before line count. A raw LF scan saw 4611 source lines because line-count tools disagree on this file; this report keeps the requested Day 0 baseline value as the acceptance baseline.

## Split Files

Created under `src/interface/web/styles/`:

| File | Lines |
| --- | ---: |
| `base.css` | 32 |
| `chat.css` | 1197 |
| `command-palette.css` | 62 |
| `dashboard.css` | 34 |
| `inspector.css` | 333 |
| `layout.css` | 168 |
| `modals.css` | 306 |
| `model-picker.css` | 71 |
| `provider.css` | 86 |
| `sessions.css` | 80 |
| `settings.css` | 262 |
| `sidebar.css` | 681 |
| `slash-palette.css` | 109 |
| `tokens.css` | 111 |
| `topbar.css` | 164 |
| `utilities.css` | 265 |

`src/interface/web/style.css` is now an import shell only. The split preserved existing selectors, class names, ids, and CSS declarations; no HTML or JavaScript selector contracts were intentionally changed.

## Git Status Baseline

Initial Day 1 command:

```powershell
git status --short
```

Observed pre-existing dirty entries remained in the worktree. They were not staged. This Day 1 change adds only CSS split files and this CSS split document.

## Non-Modified Boundaries

This Day 1 pass did not modify:

- `src/interface/web/app.js`
- `src/interface/web/index.html`
- `src/interface/web/modules/**`
- `src/interface/desktop/src/main.rs`
- Provider / Keyring behavior
- Shell execution
- Checkpoint restore / export / compare / replay
- CSP / withGlobalTauri
- Agent streaming
- `package.json` or build configuration
- Git history

## Smoke Results

| Check | Result | Evidence |
| --- | --- | --- |
| `node tests/frontend/day28_command_palette_dom_smoke.js` | PASS | `day28 command palette DOM smoke: PASS (5 scenarios)` |
| `node tests/frontend/day25_inspector_safety_smoke.js` | PASS | `day25 inspector safety smoke: PASS (4 scenarios)` |
| `node tests/frontend/day19_settings_smoke.js` | PASS | `day19 settings panel smoke: PASS` |
| `node tests/frontend/day24_resource_dashboard_smoke.js` | PASS | `day24 resource dashboard smoke: PASS (8 scenarios)` |
| `git diff --name-only -- src/interface/web/app.js src/interface/web/index.html src/interface/web/modules src/interface/desktop/src/main.rs` | PASS | no production-path output; count `0` |
| `git diff --cached --check` | PASS | no staged diff errors |
| `git diff --check` | PASS | only CRLF warnings for pre-existing dirty docs and the new CSS shell |

`tests/frontend/day29_session_list_dom_smoke.js` exists in this checkout but was not part of the Day 1 required command set, so it was not run in this pass.

## Risks And Unverified Points

- Node smoke coverage is not a real Tauri WebView visual proof.
- CSS `@import` loading must still be checked in a real WebView in Day 2.
- `chat.css` remains the largest split file because chat, Markdown, diff, thinking, and operation summary styles were interleaved in the original stylesheet.
- The split keeps selector names unchanged, but cross-file import order can still affect visual cascade if a browser handles imports differently than expected.
- Existing unrelated dirty files remain in the worktree and must not be staged with this Day 1 change.

## Day 1 Result

| Check | Result |
| --- | --- |
| `style.css` import shell <= 120 lines | PASS |
| `styles/*.css` created | PASS |
| app.js diff | NO |
| index.html diff | NO |
| modules diff | NO |
| main.rs diff | NO |
| old dirty files staged | NO |
| commit | NO |
| push | NO |
