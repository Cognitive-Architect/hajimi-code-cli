# V3X-POST-02A｜Release WebView CSS / Asset Loading Investigation

## Summary

- Task: `V3X-POST-02A｜Release WebView CSS / Asset Loading Investigation`
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `96caff2837c3456416c9095ea32384854943af16`
- Release exe path: `F:\hajimi-code-cli\target\release\hajimi-desktop.exe`
- User-observed failure before fix: app launched without white screen or crash, but release UI rendered as bare HTML/default controls.
- Result after fix: release UI theme/layout restored; not a full WebView regression PASS.

## Root Cause

Release uses `src/interface/web/dist/dist` through Tauri `frontendDist`.

The source `src/interface/web/style.css` is an import shell:

```css
@import "./styles/tokens.css";
@import "./styles/base.css";
...
@import "./styles/utilities.css";
```

Before this fix, `scripts/sync-web-dist.js` copied `style.css` into `dist/dist`, but did not copy `src/interface/web/styles/`.
Therefore release loaded `style.css`, then failed to resolve all `./styles/*.css` imports.

Additional release asset loading gap found during the same investigation:
`index.html` references split frontend directories under `controllers/`, `services/`, and `views/`, but the sync script only copied `modules/`.
Those missing JS files were not the visible CSS root cause, but they are required by the current release HTML script graph.

## Investigation Evidence

Commands run:

```powershell
git branch --show-current
git rev-parse HEAD
git status --short
dir src\interface\web
dir src\interface\web\styles
Get-Content src\interface\web\style.css
Get-Content src\interface\web\index.html | Select-String -Pattern "stylesheet|style.css|styles"
Get-Content src\interface\desktop\tauri.conf.json
```

Important findings:

- `index.html` links `<link rel="stylesheet" href="style.css">`.
- Source `src/interface/web/styles/` exists and contains 16 CSS split files.
- Pre-fix `src/interface/web/dist/dist/style.css` existed.
- Pre-fix `src/interface/web/dist/dist/styles/` was missing.
- Post-fix `src/interface/web/dist/dist/styles/` exists with 16 files.
- Post-fix check of `index.html` JS references found no missing referenced JS files in `dist/dist`.

## Files Changed

- `scripts/sync-web-dist.js`

Change summary:

- Added `controllers`
- Added `services`
- Added `styles`
- Added `views`

No Chat / Provider / Keyring / Shell / Checkpoint / Agent streaming semantics were changed.

## Build / Gate Results

```powershell
npm run test:security-gate
```

Result:

- PASS
- findings: 97
- failures: 0
- warnings: 97
- allowlisted: 97

```powershell
cargo check --workspace
```

Result:

- PASS
- `hajimi-desktop` warnings: 29 existing warnings

```powershell
cargo tauri build
```

Result:

- PASS
- Built application: `F:\hajimi-code-cli\target\release\hajimi-desktop.exe`
- MSI bundle: `F:\hajimi-code-cli\target\release\bundle\msi\Hajimi_0.1.0_x64_en-US.msi`
- NSIS bundle: `F:\hajimi-code-cli\target\release\bundle\nsis\Hajimi_0.1.0_x64-setup.exe`
- Tauri bundler warning observed: `__TAURI_BUNDLE_TYPE variable not found in binary`; package still produced.

## Release WebView Smoke

Environment:

- Release exe: `F:\hajimi-code-cli\target\release\hajimi-desktop.exe`
- Screenshot directory: `C:\Users\22129\AppData\Local\Temp\v3x-post-02a-css-smoke`

| Check | Result | Evidence |
|---|---|---|
| app launch | PASS | release process launched |
| white screen | NO | `01-launch.png`, `09-focused-click.png` |
| crash | NO | process stayed alive during smoke |
| HTML body visible | YES | app UI visible |
| CSS loaded | PASS | dark theme/layout restored in screenshot |
| styles imports loaded | PASS | `dist/dist/styles` contains 16 files; layout restored |
| UI layout restored | PASS | no bare HTML/default button layout in post-fix screenshots |
| Command Palette | PASS | `10-command-palette-focused.png` shows styled command palette |
| Settings | BLOCKED / INCONCLUSIVE | coordinate/focus attempts did not reliably open settings; no Provider actions touched |
| Model Picker | BLOCKED / INCONCLUSIVE | approximate click did not reliably open picker; not marked PASS |
| console critical error | UNKNOWN | no DevTools console capture; file presence and screenshot evidence used instead |

Screenshot pointers:

- `C:\Users\22129\AppData\Local\Temp\v3x-post-02a-css-smoke\01-launch.png`
- `C:\Users\22129\AppData\Local\Temp\v3x-post-02a-css-smoke\10-command-palette-focused.png`
- `C:\Users\22129\AppData\Local\Temp\v3x-post-02a-css-smoke\13-settings-via-cp.png`
- `C:\Users\22129\AppData\Local\Temp\v3x-post-02a-css-smoke\14-model-picker-click.png`

## Forbidden Areas

Not touched:

- `src/interface/web/app.js`
- `src/interface/web/index.html`
- `src/interface/web/style.css`
- `src/interface/web/styles/**`
- `src/interface/web/modules/**`
- `src/interface/desktop/**`
- `src/engine/**`
- `Cargo.toml`
- `Cargo.lock`
- `package.json`
- `src/interface/desktop/tauri.conf.json`
- Chat behavior
- Provider / Keyring
- Shell execution
- Checkpoint restore / export / compare / replay
- Agent streaming
- repo history

## Production Diff Scope

Production logic changes: NO.

Release asset loading change: YES, limited to `scripts/sync-web-dist.js`.

Command run:

```powershell
git diff --name-only -- src Cargo.toml Cargo.lock package.json src/interface/desktop/tauri.conf.json
```

Result:

- No output.

## Known Remaining Caveat

This receipt proves the release CSS / split asset loading problem was fixed and the release UI is no longer bare HTML.

This receipt does not prove full WebView regression PASS. Settings and Model Picker were not reliably opened by this automated desktop smoke because focus/coordinate/input-method interference affected the click path. They should remain follow-up WebView validation items.

## Next Recommendation

Run a focused `V3X-POST-02B` manual receipt for Settings and Model Picker using the newly fixed release exe. Do not expand that receipt into Provider save/delete/probe.
