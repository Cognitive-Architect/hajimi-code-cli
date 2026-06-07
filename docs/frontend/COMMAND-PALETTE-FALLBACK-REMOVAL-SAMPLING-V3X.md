# COMMAND-PALETTE-FALLBACK-REMOVAL-SAMPLING-V3X

## Scope

Task: STONE-AUDIT-V3X-CONTROLLED-DEMOLITION Day 3-D Command Palette inline fallback removal sampling.

Goal: read-only sampling of `src/interface/web/app.js` Command Palette wrapper / fallback / delegation paths, to decide whether old inline fallback code has removal candidates.

This pass does not remove fallback code and does not move Sessions / Settings / Provider logic.

## Branch / HEAD

| Item | Value |
| --- | --- |
| Branch | `stone-audit-v3x-controlled-demolition` |
| HEAD | `160f9025907654e23fb10840afe242e989180268` |
| Pull result | `Already up to date.` |
| Production code changed | NO |
| Commit | NO |
| Push | NO |

## Search Terms

The sampling searched `src/interface/web/app.js` for:

- `commandPalette`
- `commandInput`
- `commandList`
- `openCommandPalette`
- `closeCommandPalette`
- `renderCommandPalette`
- `executeCommand`
- `_commandPaletteView`
- `_commandController`
- `HajimiCommandPaletteView`
- `HajimiCommandController`
- `setupCommandPalette`
- `showCommandPalette`
- `hideCommandPalette`
- `renderCommandList`
- `navigateCommandList`
- `executeSelectedCommand`
- `setupKeyboardShortcuts`

Notes:

- No `openCommandPalette`, `closeCommandPalette`, `renderCommandPalette`, or standalone `executeCommand` function name was found in `app.js` at this HEAD.
- The active names are `showCommandPalette`, `hideCommandPalette`, `renderCommandList`, `navigateCommandList`, and `executeSelectedCommand`.

## Entry Evidence

| Evidence | Location | Notes |
| --- | --- | --- |
| `init()` calls Command Palette setup first, then keyboard shortcuts | `app.js:51-56` | `setupCommandPalette()` runs before `setupKeyboardShortcuts()`. |
| command catalog includes `palette` action | `app.js:92-100` | `palette` action calls `this.showCommandPalette()`. |
| top bar search opens Command Palette | `app.js:206-208` | `topBarSearchBtn` click calls `this.showCommandPalette()`. |
| Day 3-C browser script order | `index.html:642-644` | `views/command-palette-view.js`, then `controllers/command-controller.js`, then `app.js`. |
| Day 3-C WebView globals | `docs/frontend/COMMAND-PALETTE-WEBVIEW-WIRING-V3X.md` | `window.HajimiCommandPaletteView`, `window.HajimiCommandController`, `window.app._commandPaletteView`, and `window.app._commandController` observed as YES. |

## Wrapper / Fallback Inventory

| # | Wrapper | app.js location | Delegation path | Fallback branch | Fallback directly manipulates DOM | Fallback binds events | Fallback triggers command action | Current classification |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `setupCommandPalette()` | `app.js:5107-5133` | If `window.HajimiCommandPaletteView.createCommandPaletteView` and `window.HajimiCommandController.createCommandController` exist, creates `this._commandPaletteView`, `this._commandController`, then calls `this._commandController.setup()`. | Gets `#commandPalette` and `#commandInput`; binds input, keydown, overlay click. | YES | YES: input, keydown, palette overlay click | INDIRECT: Enter calls `executeSelectedCommand()` | SAFE-TO-REMOVE-CANDIDATE |
| 2 | `showCommandPalette()` | `app.js:5135-5141` | If `this._commandPaletteView` exists, calls `this._commandPaletteView.show()`. | Adds `active` to `#commandPalette`, clears/focuses `#commandInput`, calls `renderCommandList('')`. | YES | NO | NO | SAFE-TO-REMOVE-CANDIDATE |
| 3 | `hideCommandPalette()` | `app.js:5143-5146` | If `this._commandPaletteView` exists, calls `this._commandPaletteView.hide()`. | Removes `active` from `#commandPalette`. | YES | NO | NO | SAFE-TO-REMOVE-CANDIDATE |
| 4 | `renderCommandList(query)` | `app.js:5148-5165` | If `this._commandPaletteView` exists, calls `this._commandPaletteView.renderList(query)`. | Filters `this.commands`, writes `#commandList.innerHTML`, attaches click handlers to `.command-item`. | YES | YES: click per command item | YES: click calls `cmd.action()` | SAFE-TO-REMOVE-CANDIDATE |
| 5 | `navigateCommandList(dir)` | `app.js:5167-5177` | If `this._commandPaletteView` exists, calls `this._commandPaletteView.navigate(dir)`. | Queries `.command-item`, moves `selected`, scrolls selected item. | YES | NO | NO | SAFE-TO-REMOVE-CANDIDATE |
| 6 | `executeSelectedCommand()` | `app.js:5179-5185` | If `this._commandPaletteView` exists, calls `this._commandPaletteView.executeSelected()`. | Reads `.command-item.selected`, finds command by `dataset.id`, hides palette, runs `cmd.action()`. | YES | NO | YES: Enter path calls `cmd.action()` | SAFE-TO-REMOVE-CANDIDATE |
| 7 | `setupKeyboardShortcuts()` | `app.js:5190-5247` | If `this._commandController` exists, calls `this._commandController.setupKeyboardShortcuts()`. | Adds one global `document.keydown` listener for Ctrl+Shift+P, Ctrl+Shift+E/F/G/A/X/S/C, Ctrl+B, Escape. | NO direct DOM writes, but calls UI methods | YES: global keydown | INDIRECT: opens palette and changes sidebars | KEEP-FOR-NOW |

## Deletion Readiness Table

| Area | Classification | Reason | Required condition before actual deletion |
| --- | --- | --- | --- |
| `setupCommandPalette()` inline fallback | SAFE-TO-REMOVE-CANDIDATE | Day 3-C verifies browser script order and module globals; day30 proves delegation branch creates `_commandPaletteView` and `_commandController`. | Update/keep Node smoke so it loads view/controller modules before app.js; then remove only fallback block. |
| `showCommandPalette()` fallback | SAFE-TO-REMOVE-CANDIDATE | View module has equivalent `show()` behavior and Day 3-C WebView proves open/focus/render. | Post-removal day30 and WebView Ctrl+Shift+P open must pass. |
| `hideCommandPalette()` fallback | SAFE-TO-REMOVE-CANDIDATE | View module has equivalent `hide()` behavior and Day 3-C WebView proves Escape close. | Post-removal day30 and WebView Escape close must pass. |
| `renderCommandList(query)` fallback | SAFE-TO-REMOVE-CANDIDATE | View module has equivalent render/filter/click behavior and day30 covers render/filter/click delegation. | Post-removal day28/day30 must prove `#commandList` renders candidates through module path. |
| `navigateCommandList(dir)` fallback | SAFE-TO-REMOVE-CANDIDATE | View module has equivalent arrow navigation and day30 covers navigation delegation. | Post-removal day30 must prove ArrowUp/ArrowDown still move selection. |
| `executeSelectedCommand()` fallback | SAFE-TO-REMOVE-CANDIDATE | View module has equivalent selected command execution and day30 covers execution delegation. | Post-removal smoke must use safe command only and must not execute Provider/Shell/Checkpoint/Agent actions. |
| `setupKeyboardShortcuts()` fallback | KEEP-FOR-NOW | The fallback is not Command Palette only. It also owns Explorer, Search, Git, Agent Trace, Extensions, Settings, Chat Sessions, and sidebar toggle shortcuts. Day 3-C only WebView-tested Command Palette keys. | Add dedicated shortcut coverage for non-palette keys or split keyboard shortcut ownership before deleting this fallback. |

## Required Pre-Removal Checks

Run before Day 3-E fallback removal:

1. `git pull --ff-only origin stone-audit-v3x-controlled-demolition`
2. `git status --short`
3. `node --check src/interface/web/app.js`
4. `node --check src/interface/web/controllers/command-controller.js`
5. `node --check src/interface/web/views/command-palette-view.js`
6. `node tests/frontend/day22_command_palette_catalog_smoke.js`
7. `node tests/frontend/day28_command_palette_dom_smoke.js`
8. `node tests/frontend/day30_command_palette_delegation_smoke.js`
9. Confirm `index.html` script order remains:
   - `views/command-palette-view.js`
   - `controllers/command-controller.js`
   - `app.js`
10. Confirm forbidden diff count is 0 before editing:
    - `src/interface/web/modules/**`
    - `src/interface/web/styles/**`
    - `src/interface/web/style.css`
    - `src/interface/desktop/src/main.rs`
    - Provider / Keyring / Shell / Checkpoint / CSP / withGlobalTauri / Agent streaming areas

## Required Post-Removal Checks

Run after Day 3-E fallback removal:

1. `node --check src/interface/web/app.js`
2. `node --check src/interface/web/controllers/command-controller.js`
3. `node --check src/interface/web/views/command-palette-view.js`
4. `node tests/frontend/day22_command_palette_catalog_smoke.js`
5. `node tests/frontend/day28_command_palette_dom_smoke.js`
   - If day28 still depends on inline fallback, update only the smoke harness to load view/controller modules. Do not reintroduce production fallback.
6. `node tests/frontend/day30_command_palette_delegation_smoke.js`
7. Real Tauri WebView smoke:
   - app launch PASS
   - `window.HajimiCommandPaletteView` YES
   - `window.HajimiCommandController` YES
   - `window.app._commandPaletteView` YES
   - `window.app._commandController` YES
   - Ctrl+Shift+P open PASS
   - input filter PASS
   - list render PASS
   - Escape close PASS
   - safe candidate click PASS
   - white screen NO
   - crash NO
   - no response NO
   - command-specific error NO
8. `git diff --name-only -- src/interface/web/modules src/interface/web/styles src/interface/web/style.css src/interface/desktop/src/main.rs`
9. `git diff --check`
10. `git diff --cached --check`

## Rollback Point

Rollback to the last pushed Day 3-C commit if removal causes regression:

`160f9025907654e23fb10840afe242e989180268`

Practical rollback file list for a Day 3-E removal attempt:

- `src/interface/web/app.js`
- `tests/frontend/day28_command_palette_dom_smoke.js` if the test harness is updated for module loading
- Any Day 3-E receipt document

## Forbidden Areas

This sampling did not modify and Day 3-E should not touch:

- `src/interface/web/modules/**`
- `src/interface/web/styles/**`
- `src/interface/web/style.css`
- `src/interface/desktop/src/main.rs`
- Provider / Keyring
- Shell
- Checkpoint restore / export / compare / replay
- CSP / withGlobalTauri
- Agent streaming
- `package.json` or build configuration

## Risk Notes

- `SAFE-TO-REMOVE-CANDIDATE` means the old inline branch has enough evidence to be a candidate for removal. It does not mean the branch has already been removed.
- `day28_command_palette_dom_smoke.js` currently proves the DOM shell behavior and historically exercised inline `app.js` behavior. After fallback removal, day28 may need a test harness adjustment so it loads `command-palette-view.js` and `command-controller.js` before `app.js`.
- `setupKeyboardShortcuts()` is a mixed shortcut owner. Its fallback includes non-Command-Palette shortcuts and should not be removed in the same small slice unless those shortcuts get explicit coverage.
- Day 3-C WebView required a no-cache reload before the new script order appeared. Future WebView smoke should use no-cache reload or a fresh app launch before recording final evidence.
- Existing unrelated dirty files remain in the worktree and were not staged.

## Recommendation

Day 3-E fallback removal is allowed only as a narrow slice.

Allowed Day 3-E removal target:

- Remove the inline fallback bodies for the six pure Command Palette wrapper paths:
  - `setupCommandPalette()` fallback block
  - `showCommandPalette()` fallback DOM body
  - `hideCommandPalette()` fallback DOM body
  - `renderCommandList(query)` fallback DOM/render/click body
  - `navigateCommandList(dir)` fallback DOM body
  - `executeSelectedCommand()` fallback selected-command body

Keep for now:

- Keep `setupKeyboardShortcuts()` fallback in app.js until non-palette shortcut coverage exists or keyboard shortcut ownership is split in a separate slice.

Do not enter Sessions / Settings / Provider migration as part of Day 3-E.
