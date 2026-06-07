# STONE-AUDIT-V3X-CONTROLLED-DEMOLITION Day4-C

## Session List View Browser Wiring + Compatibility Fallback Removal

Date: 2026-06-07

Branch: `stone-audit-v3x-controlled-demolition`

HEAD before: `fe9e29b9a9258f4f39afc276016e1740aea71810`

Commit: NO

Push: NO

## Scope

This slice only wires the Session List View into the browser loading path and removes the complete compatibility render fallback from `sessions.js`.

Out of scope and not touched:

- `src/interface/web/app.js`
- Settings / Storage / Chat main flows
- `sendChatMessage()`
- `handleAgentEvent()`
- `exportAllCheckpoints()`
- `setupKeyboardShortcuts()`
- `showSidebar()`
- Provider / Keyring
- Shell
- Checkpoint restore / export / compare / replay
- CSP / withGlobalTauri
- Agent streaming
- Command Palette controller/view
- `src/interface/desktop/src/main.rs`
- repo history rewrite

## Modified Files

- `src/interface/web/index.html`
- `src/interface/web/modules/sessions.js`
- `docs/frontend/SESSION-LIST-VIEW-WEBVIEW-WIRING-V3X.md`

## index.html Script Order

The browser path now explicitly loads `views/session-list-view.js` before `modules/sessions.js`.

Observed order:

```html
<script defer src="modules/security-dom.js"></script>
<script defer src="modules/workspace.js"></script>
<script defer src="views/session-list-view.js"></script>
<script defer src="modules/sessions.js"></script>
```

The required relative order is satisfied:

```text
modules/security-dom.js
views/session-list-view.js
modules/sessions.js
```

`app.js` remains loaded after the existing module/view/controller scripts.

## Phase 1 Results: Browser Wiring With Fallback Still Present

### Node Results

```powershell
node --check src/interface/web/app.js
```

Result: PASS

```powershell
node --check src/interface/web/views/session-list-view.js
```

Result: PASS

```powershell
node --check src/interface/web/modules/sessions.js
```

Result: PASS

```powershell
node tests/frontend/day14_sessions_thinking_modules_smoke.js
```

Result: PASS

```text
day14 sessions/thinking modules smoke: PASS
```

```powershell
node tests/frontend/day29_session_list_dom_smoke.js
```

Result: PASS

```text
day29 sessionList DOM smoke: PASS (6 scenarios)
```

### WebView Results

Launch method:

```powershell
python -m http.server 3456 --bind 127.0.0.1
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9222'
target/debug/hajimi-desktop.exe
```

CDP target:

```text
http://localhost:3456/
```

Observed browser scripts:

```text
modules/security-dom.js
modules/workspace.js
views/session-list-view.js
modules/sessions.js
...
app.js
```

Observed globals:

| Check | Result |
| --- | --- |
| `window.app` exists | YES |
| `window.HajimiSessionListView` exists | YES |
| `window.HajimiSessionListView.renderSessionList` is function | YES |
| `window.HajimiSessions` exists | YES |

WebView smoke:

| Check | Result |
| --- | --- |
| app launch | PASS |
| session list visible | PASS |
| existing sessions render | PASS |
| session item click switches active session | PASS |
| white screen | NO |
| crash | NO |
| no response | NO |
| command-specific error | NO |
| console errors | 0 |
| exceptions | 0 |
| fallback used | NO |

Phase 1 click evidence:

```text
clickTargetId: session-1780318915050-sefjm
afterActive: session-1780318915050-sefjm
clickSwitched: true
fallbackUsed: false
```

## Phase 2 Results: Complete Fallback Removed

### Fallback Removal Summary

Removed from `src/interface/web/modules/sessions.js`:

- local compatibility `formatSessionTime(value)`
- local compatibility session list HTML render body
- local compatibility `.session-item` click binding
- `global.HajimiSessionListView = api` fallback mount

Kept:

- `ensureSessionListView()`
- safe no-op guard when `HajimiSessionListView` is absent
- public `renderSessionList(app)` wrapper
- localStorage logic
- `newChatSession/loadChatSessions/saveChatSessions/switchSession`
- `renderChatMessages`

Current missing-view behavior:

```text
HajimiSessionListView is required before HajimiSessions.renderSessionList; skipping session list render.
```

This is a safe no-op guard, not a restored render fallback.

### Node Results

```powershell
node --check src/interface/web/modules/sessions.js
```

Result: PASS

```powershell
node tests/frontend/day14_sessions_thinking_modules_smoke.js
```

Result: PASS

Output:

```text
day14 sessions/thinking modules smoke: PASS
HajimiSessionListView is required before HajimiSessions.renderSessionList; skipping session list render.
```

Note: day14 loads `sessions.js` directly without loading `views/session-list-view.js`. The warning is the expected safe no-op guard for that legacy harness path.

```powershell
node tests/frontend/day29_session_list_dom_smoke.js
```

Result: PASS

```text
day29 sessionList DOM smoke: PASS (6 scenarios)
```

`rg` verification showed no complete HTML render fallback remaining in `sessions.js`; only the safe no-op warning string remains.

### WebView Results

CDP target:

```text
http://localhost:3456/
```

Observed required script order:

```text
modules/security-dom.js
views/session-list-view.js
modules/sessions.js
```

Observed globals:

| Check | Result |
| --- | --- |
| `window.app` exists | YES |
| `window.HajimiSessionListView` exists | YES |
| `window.HajimiSessionListView.renderSessionList` is function | YES |
| `window.HajimiSessions` exists | YES |

WebView smoke:

| Check | Result |
| --- | --- |
| app launch | PASS |
| session list visible | PASS |
| existing sessions render | PASS |
| session item click switches active session | PASS |
| white screen | NO |
| crash | NO |
| no response | NO |
| command-specific error | NO |
| console errors | 0 |
| console warnings | 3 |
| exceptions | 0 |
| fallback used | NO |

Phase 2 click evidence:

```text
clickTargetId: session-1780318915050-sefjm
afterActive: session-1780318915050-sefjm
clickSwitched: true
fallbackUsed: false
```

The console warnings were observed during app reload, but no console errors or exceptions were observed and no warning was attributed to Session List View fallback usage.

## Forbidden Diff Check

```powershell
git diff --name-only -- src/interface/desktop/src/main.rs src/engine/tool-system/src/shell.rs src/interface/desktop/tauri.conf.json src/interface/web/controllers/command-controller.js src/interface/web/views/command-palette-view.js
```

Result: PASS, no output.

Forbidden diff count: 0

```powershell
git diff --cached --check
```

Result: PASS, no staged diff output at receipt creation time.

Old dirty files staged: NO

## Production Changes Summary

Changed production frontend files:

- `src/interface/web/index.html`: adds `views/session-list-view.js` before `modules/sessions.js`.
- `src/interface/web/modules/sessions.js`: removes full compatibility render fallback and keeps only a safe no-op guard.

Production files not changed:

- `src/interface/web/app.js`
- `src/interface/web/views/command-palette-view.js`
- `src/interface/web/controllers/command-controller.js`
- `src/interface/desktop/src/main.rs`
- `src/engine/tool-system/src/shell.rs`
- `src/interface/desktop/tauri.conf.json`

## Test Process Cleanup

The local WebView smoke helper processes were stopped after validation.

Observed after cleanup:

```text
PORT_3456_LISTENING=False
PORT_9222_LISTENING=False
```

## Rollback Point

```text
fe9e29b9a9258f4f39afc276016e1740aea71810
```

## Next Recommended Step

Day4-D candidate:

Sample `newChatSession/loadChatSessions/saveChatSessions/switchSession` for a future `session-controller.js` or `storage-service.js` split.

Do not combine that with Settings, Provider, Shell, Checkpoint, or Chat streaming.
