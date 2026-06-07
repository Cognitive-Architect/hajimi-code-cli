# COMMAND-PALETTE-FALLBACK-REMOVAL-V3X

## Scope

Task: STONE-AUDIT-V3X-CONTROLLED-DEMOLITION Day 3-E Command Palette inline fallback removal.

Goal: remove only the six pure Command Palette inline fallback branches from `src/interface/web/app.js`, while keeping `setupKeyboardShortcuts()` fallback.

Commit: NO

Push: NO

## Branch / HEAD

| Item | Value |
| --- | --- |
| Branch | `stone-audit-v3x-controlled-demolition` |
| HEAD before | `b6bf921be9c029a0829a13550082e7e1c68c5924` |
| Rollback point | `b6bf921be9c029a0829a13550082e7e1c68c5924` |

## Modified Files

| File | Change |
| --- | --- |
| `src/interface/web/app.js` | Removed six pure Command Palette inline fallback bodies; retained delegation wrappers. |
| `tests/frontend/day28_command_palette_dom_smoke.js` | Updated harness to load `command-palette-view.js` and `command-controller.js` before `app.js`. |
| `docs/frontend/COMMAND-PALETTE-FALLBACK-REMOVAL-V3X.md` | Added this receipt. |

## Removed Fallback List

Removed from `app.js`:

1. `setupCommandPalette()` fallback block
2. `showCommandPalette()` fallback DOM body
3. `hideCommandPalette()` fallback DOM body
4. `renderCommandList(query)` fallback DOM/render/click body
5. `navigateCommandList(dir)` fallback DOM body
6. `executeSelectedCommand()` fallback selected-command body

Removed fallback count: 6

## Kept Fallback List

Kept in `app.js`:

1. `setupKeyboardShortcuts()` inline fallback

`setupKeyboardShortcuts()` fallback kept: YES

Reason: this fallback still owns non-Command-Palette shortcuts such as Explorer, Search, Git, Agent Trace, Extensions, Settings, Chat Sessions, and sidebar toggle. It was intentionally not removed in this slice.

## Day28 Harness Change Summary

`tests/frontend/day28_command_palette_dom_smoke.js` now mirrors the real browser load order for the Command Palette path:

1. Load `src/interface/web/views/command-palette-view.js`
2. Load `src/interface/web/controllers/command-controller.js`
3. Load `src/interface/web/app.js`

This keeps day28 on the module delegation path after the old inline fallback is removed.

## Validation Results

| Check | Result | Evidence |
| --- | --- | --- |
| `node --check src/interface/web/app.js` | PASS | no output |
| `node --check src/interface/web/controllers/command-controller.js` | PASS | no output |
| `node --check src/interface/web/views/command-palette-view.js` | PASS | no output |
| `node --check tests/frontend/day28_command_palette_dom_smoke.js` | PASS | no output |
| `node tests/frontend/day22_command_palette_catalog_smoke.js` | PASS | `day22 command palette catalog smoke: PASS (7 scenarios)` |
| `node tests/frontend/day28_command_palette_dom_smoke.js` | PASS | `day28 command palette DOM smoke: PASS (5 scenarios)` |
| `node tests/frontend/day30_command_palette_delegation_smoke.js` | PASS | `day30 command palette delegation smoke: PASS (10 scenarios)` |

## WebView Smoke Result

Real Tauri WebView smoke after fallback removal: PASS.

Observed:

- app launch: PASS
- script order includes `views/command-palette-view.js`, `controllers/command-controller.js`, `app.js`
- `window.HajimiCommandPaletteView`: YES
- `window.HajimiCommandController`: YES
- `window.app._commandPaletteView`: YES
- `window.app._commandController`: YES
- Ctrl+Shift+P open: PASS
- input filter: PASS
- list render: PASS
- Escape close: PASS
- safe candidate click: PASS
- white screen: NO
- crash: NO
- no response: NO
- command-specific error: NO

## Forbidden Diff Check

Forbidden diff count: 0

Confirmed no diff for:

- `src/interface/web/index.html`
- `src/interface/web/controllers/**`
- `src/interface/web/views/**`
- `src/interface/web/modules/**`
- `src/interface/web/styles/**`
- `src/interface/web/style.css`
- `src/interface/desktop/src/main.rs`
- `package.json`

Not touched:

- Provider / Keyring
- Shell
- Checkpoint restore / export / compare / replay
- CSP / withGlobalTauri
- Agent streaming
- build configuration

## Dirty Worktree Boundary

Old dirty files staged: NO

Existing unrelated dirty files remain in the worktree and are not part of this receipt.

## Result

Day 3-E fallback removal local validation: PASS.

This receipt records the state before commit and push.
