# CSS-SPLIT-WEBVIEW-SMOKE-V3X

## Scope

Task: STONE-AUDIT-V3X-CONTROLLED-DEMOLITION Day 2 CSS WebView visual regression.

Goal: verify the Day 1 CSS `@import` split loads in a real Tauri WebView without white screen, crash, no response, CSS import/load failure, or large layout collapse.

## Baseline

| Item | Value |
| --- | --- |
| Branch | `stone-audit-v3x-controlled-demolition` |
| HEAD | `26f250fb8f4d530f5d0039901cb19d0597c57c2d` |
| Day 1 CSS split committed | NO |
| Day 1 CSS split pushed | NO |
| Commit in this pass | NO |
| Push in this pass | NO |

## Launch Method

Started a local static web server from `src/interface/web`:

```powershell
python -m http.server 3456 --bind 127.0.0.1
```

Started the desktop app:

```powershell
F:\hajimi-code-cli\target\debug\hajimi-desktop.exe
```

WebView remote debugging was enabled with:

```powershell
WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222
```

Observed WebView target:

```text
title: Hajimi Code
url: http://localhost:3456/
readyState: complete
```

Temporary server and desktop app processes were stopped after the smoke.

## Results

| Check | Result | Evidence |
| --- | --- | --- |
| app launch | PASS | WebView target loaded `http://localhost:3456/`; process responded during smoke |
| chat/main view visible | PASS | `.main-area`, `.chat-container`, `.ai-chat-messages` existed with non-zero width/height |
| sidebar/session visible | PASS | `#sessionList` existed with non-zero width/height and session text |
| command palette open | PASS | Ctrl+Shift+P made `#commandPalette` active |
| command palette filter | PASS | input value became `设置`; filtered list count was `2` |
| command palette render | PASS | initial list count was `23`; filtered list rendered matching settings commands |
| command palette escape close | PASS | Escape removed active state from `#commandPalette` |
| settings visible | PASS | clicking `.activity-item[data-view="settings"]` activated `#settingsPanel` with non-zero size |
| inspector visible | PASS | `#rightInspector` displayed with non-zero size and task-detail content |
| dashboard visible | PASS | audit settings panel displayed resource metrics; `#metricIterationTab`, `#metricBlackboardTab`, `#metricEditCountTab` existed with non-zero size |
| white screen | NO | body text length remained above 20; body children existed; app shell visible |
| crash | NO | desktop process responded during smoke |
| no response | NO | keyboard and mouse interactions completed |
| CSS import/load error | NOT OBSERVED | no CSS-related console/network error captured |
| command-specific error | NO | no command-palette-specific error captured during open/filter/render/escape |

## CSS Import Evidence

The WebView reported `src/interface/web/style.css` as the loaded stylesheet and exposed all 16 imported CSS files through CSSOM.

| Import | CSS rules observed |
| --- | ---: |
| `./styles/tokens.css` | 5 |
| `./styles/base.css` | 9 |
| `./styles/layout.css` | 29 |
| `./styles/sidebar.css` | 121 |
| `./styles/topbar.css` | 25 |
| `./styles/chat.css` | 210 |
| `./styles/command-palette.css` | 9 |
| `./styles/slash-palette.css` | 14 |
| `./styles/sessions.css` | 15 |
| `./styles/settings.css` | 46 |
| `./styles/provider.css` | 14 |
| `./styles/model-picker.css` | 13 |
| `./styles/inspector.css` | 52 |
| `./styles/dashboard.css` | 10 |
| `./styles/modals.css` | 45 |
| `./styles/utilities.css` | 41 |

## Observed Non-Blocking Console / Network Notes

- A startup warning was observed: `Approval UI unavailable: Error: Tauri event listen unavailable`.
- This warning existed outside the CSS import path and did not block app launch, command palette interaction, settings display, inspector display, or dashboard visibility during this smoke.
- Read-only Tauri IPC requests were observed when opening settings/audit views, including resource metrics, provider config reads, checkpoints listing, and audit log reads.
- No Provider save/delete/probe action was clicked.
- No Shell execution was triggered.
- No Checkpoint restore/export/compare/replay action was clicked.

## Boundaries

This Day 2 pass did not modify:

- `src/interface/web/app.js`
- `src/interface/web/index.html`
- `src/interface/web/modules/**`
- `src/interface/web/style.css`
- `src/interface/web/styles/**`
- `src/interface/desktop/src/main.rs`
- Provider / Keyring behavior
- Shell execution
- Checkpoint restore / export / compare / replay
- CSP / withGlobalTauri
- Agent streaming
- `package.json` or build configuration
- Git history

## Production Change Check

Production changes beyond Day 1 CSS/doc: NO.

Old dirty files staged: NO.

Commit: NO.

Push: NO.

## Risk And Unverified Points

- This is a real Tauri WebView smoke using CDP observation, not a human pixel-by-pixel design review.
- No screenshot artifact was saved in this pass.
- Fine-grained visual drift may still exist and should be handled in later visual QA if needed.
- The observed startup approval warning is recorded as non-blocking and not attributed to the CSS split.

## Conclusion

Day 2 CSS WebView smoke result: PASS.
