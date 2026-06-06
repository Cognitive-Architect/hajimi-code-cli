# STONE-AUDIT-V2G-SESSION-LIST-DOM-SMOKE

Date: 2026-06-06

Repo: Cognitive-Architect/hajimi-code-cli

Branch: feature/toolfix-deepseek-schema

Base HEAD: 1379d1119d1e578c326e698ee9e256f8bc4a6410

## Scope

This task adds a Node DOM smoke that explicitly names and verifies `#sessionList`.

V2E classified `sessionList` / `modules/sessions.js` as `NAMING-GAP` because the sessions module already had smoke coverage, but the exact DOM ID was not directly named by a focused smoke receipt.

This report records only Node DOM / localStorage mock behavior. It does not prove backend session behavior and does not add backend session behavior.

## Files Added

- `tests/frontend/day29_session_list_dom_smoke.js`
- `docs/frontend/SESSION-LIST-DOM-SMOKE-V2G.md`

## Production Boundary

Production code modified: NO

Production paths intentionally not modified:

- `src/interface/web/app.js`
- `src/interface/web/index.html`
- `src/interface/web/style.css`
- `src/interface/web/modules/*`

High-risk areas intentionally not touched:

- Provider / Keyring
- Shell
- Checkpoint restore / export / compare / replay
- CSP / withGlobalTauri
- Agent streaming

## Evidence

Readonly source evidence:

- `src/interface/web/index.html` contains `id="sessionList"` in the chat sessions sidebar.
- `src/interface/web/modules/sessions.js` writes rendered session HTML through `document.getElementById('sessionList')`.
- `docs/frontend/FRONTEND-SMOKE-COVERAGE-GAP-V2E.md` records `sessionList` as `NAMING-GAP`.

## Smoke Coverage

| Check | Result | Evidence |
| --- | --- | --- |
| `index.html` contains `#sessionList` | PASS | day29 reads `src/interface/web/index.html` and asserts `id="sessionList"`. |
| test fixture registers `#sessionList` | PASS | day29 creates an explicit fake DOM node with id `sessionList`. |
| sessions render path writes into `#sessionList` | PASS | day29 calls `HajimiSessions.renderSessionList(app)` and inspects `sessionList.innerHTML`. |
| empty state | PASS | Empty `app.chatSessions` renders `暂无会话` and no `.session-item`. |
| has-session state | PASS | localStorage mock with two sessions renders two `.session-item` rows. |
| active session state | PASS | First localStorage session becomes active and first item has `active` class. |
| localStorage/mock-only behavior | PASS | day29 uses `HajimiSessions.storageKey` and mock localStorage. No backend command is invoked. |
| click wiring | PASS | Clicking rendered second session item switches `app.activeSessionId` to `session-b` using local mock state. |
| escaping | PASS | Malicious title/preview text is escaped and does not render raw `<script>` or `<img>` HTML. |

## Validation Log

```text
node tests/frontend/day14_sessions_thinking_modules_smoke.js
day14 sessions/thinking modules smoke: PASS
```

```text
node tests/frontend/day29_session_list_dom_smoke.js
day29 sessionList DOM smoke: PASS (6 scenarios)
```

Production diff check:

```text
git diff --name-only -- src/interface/web/app.js src/interface/web/index.html src/interface/web/style.css src/interface/web/modules
<no output>
```

Remote branch check before report work:

```text
git ls-remote origin feature/toolfix-deepseek-schema
1379d1119d1e578c326e698ee9e256f8bc4a6410	refs/heads/feature/toolfix-deepseek-schema
```

`git status --short` before staging showed known historical dirty files plus the new day29 smoke file:

```text
 M docs/debt/DEBT-AGENT-LLM-NATIVE-BUDGET-MELTDOWN.md
 M docs/debt/INDEX.md
 M docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md
 D "docs/roadmap/Hajimi Agent/debt/DEBT-DAY-07-CHECKPOINT-DIFF-UI.md"
 D "docs/roadmap/Hajimi Agent/plan/AGENT-UI-INTEGRATION-SAMPLING-NOTES.md"
 D "docs/roadmap/Hajimi LLM/plan/AGENT-LLM-NATIVE-DESIGN.md"
 D "docs/roadmap/Hajimi LLM/plan/LLM-NATIVE-AGENT-MIGRATION-ROADMAP.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-01/snapshot.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-02/snapshot.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-03/snapshot.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-04/snapshot.md"
?? docs/debt/DEBT-AGENT-CHINESE-I18N.md
?? docs/debt/DEBT-AGENT-LLM-NATIVE-DRIVER-INJECTION.md
?? docs/debt/DEBT-AGENT-LLM-NATIVE-THINKING-LEAK.md
?? docs/debt/DEBT-AGENT-LOOP-LLM-NO-OP.md
?? docs/debt/DEBT-AGENT-UI-INTEGRATION.md
?? docs/debt/DEBT-TAURI-CHANNEL-ENVELOPE-REGRESSION.md
?? docs/debt/STONE-AUDIT-V1.5-PACKAGE-SMOKE.md
?? "docs/roadmap/Hajimi AgentFix/"
?? "docs/roadmap/Hajimi ToneFix/"
?? "docs/roadmap/hajimi template/"
?? src/interface/desktop/native-smoke.txt
?? tests/frontend/day29_session_list_dom_smoke.js
```

## Final Classification

`sessionList` / `modules/sessions.js`: `NAMING-GAP` -> PASS for exact Node DOM smoke.

Boundary: still not WebView proof and not backend session proof.

## Next Recommended Evidence

If real desktop evidence is required, open a separate readonly WebView smoke for the sessions sidebar. Keep it limited to visible session list behavior and do not add backend session behavior unless a later task explicitly approves it.
