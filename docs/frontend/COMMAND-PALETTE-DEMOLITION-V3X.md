# COMMAND-PALETTE-DEMOLITION-V3X

## Scope

Task: STONE-AUDIT-V3X-CONTROLLED-DEMOLITION Day 3-B Command Palette small-step migration.

Branch: `stone-audit-v3x-controlled-demolition`

HEAD before: `fd0ca7f0877ed8d7eaaf235f3b239f9fd2166acd`

Commit: NO

Push: NO

## Baseline

| Item | Value |
| --- | --- |
| Branch | `stone-audit-v3x-controlled-demolition` |
| HEAD before | `fd0ca7f0877ed8d7eaaf235f3b239f9fd2166acd` |
| Pull result | `Already up to date.` |
| app.js before line count (PowerShell) | 5084 |
| app.js after line count (PowerShell) | 5102 |
| app.js net change | +18 lines (wrapper overhead; original inline code retained as fallback) |

Note: line count increased because the migration uses a delegation-with-fallback pattern. When `HajimiCommandPaletteView` and `HajimiCommandController` modules are loaded in the browser, the new module code runs. When modules are absent (e.g., Node smoke test harness), the identical inline fallback executes. Future Day 4+ slices will remove the inline fallback and wire `<script>` tags in `index.html`, at which point app.js line count will decrease.

## Migration Structure

### `src/interface/web/views/command-palette-view.js`

Responsible for DOM query, open/close, filter render, active class, list item render.

Exported factory: `createCommandPaletteView(app)` → returns `{ show, hide, renderList, navigate, executeSelected }`.

Browser global: `window.HajimiCommandPaletteView.createCommandPaletteView`.

### `src/interface/web/controllers/command-controller.js`

Responsible for initialization (binding DOM events), keyboard navigation dispatch, Ctrl+Shift+P / Escape / sidebar shortcuts.

Exported factory: `createCommandController(app, view)` → returns `{ setup, setupKeyboardShortcuts }`.

Browser global: `window.HajimiCommandController.createCommandController`.

### `src/interface/web/app.js`

Retains wrapper methods on `window.app` object:

- `setupCommandPalette()` — delegates to controller/view if modules loaded; inline fallback otherwise
- `showCommandPalette()` — delegates to `_commandPaletteView.show()` if available
- `hideCommandPalette()` — delegates to `_commandPaletteView.hide()` if available
- `renderCommandList(query)` — delegates to `_commandPaletteView.renderList()` if available
- `navigateCommandList(dir)` — delegates to `_commandPaletteView.navigate()` if available
- `executeSelectedCommand()` — delegates to `_commandPaletteView.executeSelected()` if available
- `setupKeyboardShortcuts()` — delegates to `_commandController.setupKeyboardShortcuts()` if available

## Moved Functions

| Function | Original location | New location | Wrapper retained |
| --- | --- | --- | --- |
| `setupCommandPalette` (DOM binding) | `app.js:5107-5128` | `controllers/command-controller.js:setup()` | YES |
| `showCommandPalette` | `app.js:5130-5135` | `views/command-palette-view.js:show()` | YES |
| `hideCommandPalette` | `app.js:5137-5139` | `views/command-palette-view.js:hide()` | YES |
| `renderCommandList` | `app.js:5141-5159` | `views/command-palette-view.js:renderList()` | YES |
| `navigateCommandList` | `app.js:5161-5170` | `views/command-palette-view.js:navigate()` | YES |
| `executeSelectedCommand` | `app.js:5172-5177` | `views/command-palette-view.js:executeSelected()` | YES |
| `setupKeyboardShortcuts` | `app.js:5182-5234` | `controllers/command-controller.js:setupKeyboardShortcuts()` | YES |

## Modified Files

| File | Change type |
| --- | --- |
| `src/interface/web/app.js` | Modified — added delegation wrappers with inline fallback |
| `src/interface/web/controllers/command-controller.js` | Overwritten from skeleton — real implementation |
| `src/interface/web/views/command-palette-view.js` | Overwritten from skeleton — real implementation |
| `docs/frontend/COMMAND-PALETTE-DEMOLITION-V3X.md` | Created — this document |

## Non-Modified Boundaries

This Day 3-B pass did not modify:

- `src/interface/web/index.html`
- `src/interface/web/modules/**`
- `src/interface/web/styles/**`
- `src/interface/web/style.css`
- `src/interface/desktop/src/main.rs`
- Provider / Keyring behavior
- Shell execution
- Checkpoint restore / export / compare / replay
- CSP / withGlobalTauri
- Agent streaming
- `package.json` or build configuration
- Git history

## Forbidden Diff Check

| Path | Diff count |
| --- | --- |
| `src/interface/web/index.html` | 0 |
| `src/interface/web/modules/**` | 0 |
| `src/interface/web/styles/**` | 0 |
| `src/interface/web/style.css` | 0 |
| `src/interface/desktop/src/main.rs` | 0 |

Total forbidden diff count: **0**

## Smoke Results

| Check | Result | Evidence |
| --- | --- | --- |
| `node --check src/interface/web/app.js` | PASS | no output |
| `node --check src/interface/web/controllers/command-controller.js` | PASS | no output |
| `node --check src/interface/web/views/command-palette-view.js` | PASS | no output |
| `node tests/frontend/day22_command_palette_catalog_smoke.js` | PASS | `day22 command palette catalog smoke: PASS (7 scenarios)` |
| `node tests/frontend/day28_command_palette_dom_smoke.js` | PASS | `day28 command palette DOM smoke: PASS (5 scenarios)` |
| `git diff --name-only -- forbidden paths` | PASS | empty output, count 0 |
| `git diff --check` | PASS | only LF→CRLF warnings for new files (Windows standard) |
| `git diff --cached --check` | PASS | no staged diff errors |

## Risks And Unverified Points

- Node smoke tests run the inline fallback path (no `HajimiCommandPaletteView` / `HajimiCommandController` in test harness). The delegation path is not yet exercised by automated tests.
- The delegation path will be exercised when `<script>` tags for `command-controller.js` and `command-palette-view.js` are added to `index.html` in a future slice. That change is intentionally deferred to keep this slice minimal.
- Real Tauri WebView visual verification is deferred to a future Day 3-C or Day 4 WebView smoke pass.
- `setupKeyboardShortcuts` was migrated as a whole function (including non-Command-Palette shortcuts like Ctrl+Shift+E, Ctrl+B, etc.) because it is a single `document.addEventListener('keydown', ...)` block that cannot be partially split without double-registering the event listener.
- Pre-existing unrelated dirty files remain in the worktree and were not staged.

## Old Dirty Files Staged

NO

## Commit

NO

## Push

NO

---

## Day 3-B-1: Delegation Path Smoke

### Goal

Prove the new module delegation path works end-to-end under Node without modifying `index.html` or entering WebView.

### Test

Created `tests/frontend/day30_command_palette_delegation_smoke.js`.

Load order inside the VM context:

1. `views/command-palette-view.js` → sets `window.HajimiCommandPaletteView`
2. `controllers/command-controller.js` → sets `window.HajimiCommandController`
3. `app.js` (up to `// D3-MINIMAL-FIX` marker) → `setupCommandPalette` takes delegation branch

### Scenarios Covered (10)

| # | Scenario | Assertion |
| --- | --- | --- |
| 1 | Module globals exist | `window.HajimiCommandPaletteView` and `window.HajimiCommandController` present with factory functions |
| 2 | Delegation branch taken | `app._commandPaletteView` and `app._commandController` exist after `setupCommandPalette()` |
| 3 | showCommandPalette delegation | `commandPalette` gets `active` class, input cleared, input focused, command items rendered |
| 4 | renderCommandList delegation | Filter query `"设置"` returns 1 result with correct id and label |
| 5 | hideCommandPalette delegation | `active` class removed from `commandPalette` |
| 6 | navigateCommandList delegation | Arrow navigation moves `selected` class between items |
| 7 | executeSelectedCommand delegation | Correct command action fires, palette closes |
| 8 | setupKeyboardShortcuts delegation | Ctrl+Shift+P opens palette, Escape closes palette, Ctrl+Shift+E calls `showSidebar('explorer')` |
| 9 | Click-to-execute via delegation | Clicking a filtered item triggers action and closes palette |
| 10 | Controller input events | Typing into `commandInput` triggers filter via controller binding; Enter executes selected |

### Results

| Check | Result | Evidence |
| --- | --- | --- |
| `node --check tests/frontend/day30_command_palette_delegation_smoke.js` | PASS | no output |
| `node tests/frontend/day30_command_palette_delegation_smoke.js` | PASS | `day30 command palette delegation smoke: PASS (10 scenarios)` |
| `node tests/frontend/day22_command_palette_catalog_smoke.js` | PASS | `day22 command palette catalog smoke: PASS (7 scenarios)` |
| `node tests/frontend/day28_command_palette_dom_smoke.js` | PASS | `day28 command palette DOM smoke: PASS (5 scenarios)` |
| `node --check src/interface/web/app.js` | PASS | no output |
| `node --check src/interface/web/controllers/command-controller.js` | PASS | no output |
| `node --check src/interface/web/views/command-palette-view.js` | PASS | no output |
| `git diff --name-only -- forbidden paths` | PASS | empty output, count 0 |
| `git diff --check` | PASS | 0 errors (only LF→CRLF warnings on new files) |
| `git diff --cached --check` | PASS | nothing staged |

### Key Evidence

- **Delegation path proven**: day30 smoke loads modules before app.js → `setupCommandPalette()` creates `_commandPaletteView` and `_commandController` → all subsequent wrapper calls delegate to module code.
- **Inline fallback still works**: day28 smoke loads app.js without modules → inline fallback executes → PASS.
- **Catalog compatibility preserved**: day22 smoke verifies command catalog order and actions → PASS.

### Boundaries

- `index.html` not modified: YES
- WebView test performed: NO (deferred to future slice)
- Forbidden diff count: **0**
- Old dirty files staged: NO
- Source fixes required: NONE (no changes to controller/view/app.js in this sub-slice)

### Commit

NO

### Push

NO
