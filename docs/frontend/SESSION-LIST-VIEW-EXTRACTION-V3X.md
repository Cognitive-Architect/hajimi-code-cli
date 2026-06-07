# STONE-AUDIT-V3X-CONTROLLED-DEMOLITION Day4-B

## Session List View Extraction

Date: 2026-06-07

Branch: `stone-audit-v3x-controlled-demolition`

HEAD before: `a7e74faf21dbfb06db9c0e410b60fa97e933bad4`

Commit: NO

Push: NO

## Scope

This slice extracts the low-risk Sessions list DOM rendering path into the view layer.

Moved into `src/interface/web/views/session-list-view.js`:

- `formatSessionTime(value)`
- `renderSessionList(app)` DOM render body
- `.session-item` click binding

Kept in `src/interface/web/modules/sessions.js`:

- `newChatSession(app)`
- `loadChatSessions(app)`
- `saveChatSessions(app)`
- `switchSession(app, id)`
- `renderChatMessages(app)`
- `localStorage` get/set for `hajimi_chat_sessions`
- public `renderSessionList(app)` wrapper

## Modified Files

- `src/interface/web/views/session-list-view.js`
- `src/interface/web/modules/sessions.js`
- `tests/frontend/day29_session_list_dom_smoke.js`
- `docs/frontend/SESSION-LIST-VIEW-EXTRACTION-V3X.md`

## Preserved DOM Contract

The extracted view preserves the existing DOM structure and selectors:

| DOM / selector | Status |
| --- | --- |
| `#sessionList` | PRESERVED |
| `.session-empty` | PRESERVED |
| `.session-item` | PRESERVED |
| `.session-item.active` | PRESERVED |
| `.session-item-main` | PRESERVED |
| `.session-title` | PRESERVED |
| `.session-preview` | PRESERVED |
| `.session-time` | PRESERVED |
| `data-session` | PRESERVED |

Escaping behavior is preserved through the existing app helpers:

- `app.escapeAttr(s.id)`
- `app.escapeHtml(s.title || '会话')`
- `app.escapeHtml(s.preview || '')`
- `app.escapeHtml(formatSessionTime(...))`

## Preserved Click Behavior

The extracted view keeps the existing click path:

```text
.session-item click
  -> read el.dataset.session
  -> if id exists and id !== app.activeSessionId
  -> app.switchSession(id)
```

No backend session behavior was added.

No checkpoint, shell, provider, keyring, CSP, withGlobalTauri, or Agent streaming path was touched.

## sessions.js Compatibility Note

`src/interface/web/index.html` does not currently load `views/session-list-view.js`.

Because this Day4-B slice did not allow `index.html` wiring, `sessions.js` keeps a compatibility path that mounts `global.HajimiSessionListView` if the view is absent. The public `renderSessionList(app)` entry now delegates through `ensureSessionListView().renderSessionList(app)`.

This preserves current browser/script-order behavior and keeps old smokes passing. A future narrow slice can explicitly load `views/session-list-view.js` before `modules/sessions.js`, then remove the compatibility fallback.

## day29 Harness Change

`tests/frontend/day29_session_list_dom_smoke.js` now loads:

```text
modules/security-dom.js
views/session-list-view.js
modules/sessions.js
```

The smoke now asserts:

- `context.HajimiSessionListView` is mounted.
- `context.HajimiSessionListView.renderSessionList` is a function.

Existing day29 assertions remain:

- `index.html` contains `#sessionList`.
- Empty state renders `暂无会话`.
- LocalStorage mock data renders `.session-item` rows.
- `data-session` is preserved.
- Active item keeps `active`.
- Malicious title/preview strings are escaped.
- Clicking a session item switches active session.

## Validation Results

```powershell
git pull --ff-only origin stone-audit-v3x-controlled-demolition
```

Result: PASS

```text
Already up to date.
```

```powershell
node --check src/interface/web/app.js
```

Result: PASS

```powershell
node --check src/interface/web/modules/sessions.js
```

Result: PASS

```powershell
node --check src/interface/web/views/session-list-view.js
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

- `src/interface/web/views/session-list-view.js`: replaced Day3-A skeleton with real Session List View API.
- `src/interface/web/modules/sessions.js`: changed public `renderSessionList(app)` to delegate to `HajimiSessionListView`; kept local compatibility fallback because `index.html` wiring is not in this slice.

Production files not changed:

- `src/interface/web/app.js`
- `src/interface/web/index.html`
- `src/interface/web/controllers/command-controller.js`
- `src/interface/web/views/command-palette-view.js`
- `src/interface/desktop/src/main.rs`
- `src/engine/tool-system/src/shell.rs`
- `src/interface/desktop/tauri.conf.json`

## Rollback Point

```text
a7e74faf21dbfb06db9c0e410b60fa97e933bad4
```

## Next Recommended Step

Day4-C candidate:

1. Wire `views/session-list-view.js` in `index.html` before `modules/sessions.js`.
2. Run day14 and day29.
3. Run a real WebView smoke for session list visibility and session item click.
4. Only after that, remove the compatibility fallback from `sessions.js`.
