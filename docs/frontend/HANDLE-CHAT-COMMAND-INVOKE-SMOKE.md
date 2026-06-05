# STONE-AUDIT-V2B-1-HANDLE-CHAT-COMMAND-INVOKE-SMOKE

Date: 2026-06-05
Branch: feature/toolfix-deepseek-schema
Baseline HEAD: 291937d18cfb393043be396bbb0571877a5cba1f

## Scope

Check object: `handleChatCommand(text)` in `src/interface/web/app.js`.

Branches checked:

- `/chat`
- `/search`
- `/compact`

This task adds a readonly smoke and this fact report only. It does not modify production code, does not fix `invoke`, does not refactor `handleChatCommand`, and does not start real Tauri or WebView.

## Smoke Method

`tests/frontend/day27_handle_chat_command_invoke_smoke.js` reads `src/interface/web/app.js` as text and performs static checks:

1. Locate the `handleChatCommand(text)` function body.
2. Locate the `/chat`, `/search`, and `/compact` branches.
3. Check whether each branch directly uses `invoke`.
4. Check whether `handleChatCommand(text)` contains an explicit local `invoke` source, such as `const invoke =`, `let invoke =`, `var invoke =`, `this.getTauriInvoke()`, or `getTauriInvoke()`.
5. Check whether the function signature appears to provide `invoke` as a parameter.
6. Record whether an explicit outer `invoke` declaration appears before the method.

## Smoke Result

Overall result: FAIL

Meaning: the smoke found a scope-risk pattern. This is a static finding, not a WebView/runtime proof.

| Branch | Status | Fact |
| --- | --- | --- |
| `/chat` | FAIL | Branch directly uses `invoke`; no local or parameter `invoke` source was found inside `handleChatCommand(text)`. |
| `/search` | FAIL | Branch directly uses `invoke`; no local or parameter `invoke` source was found inside `handleChatCommand(text)`. |
| `/compact` | FAIL | Branch directly uses `invoke`; no local or parameter `invoke` source was found inside `handleChatCommand(text)`. |

## Findings

- `handleChatCommand(text)` `/chat`, `/search`, and `/compact` branches appear to use `invoke`.
- The sampled function body does not show a local `invoke` declaration.
- The function signature does not provide `invoke` as a parameter.
- No explicit closure-level `invoke` source was found before the `window.app` object.
- This report records an invoke scope risk. It does not claim a confirmed runtime bug.

Observed smoke output:

```text
day27 handleChatCommand invoke smoke
method: handleChatCommand(text) lines 2572-2874
method has local invoke source: NO
method has invoke parameter source: NO
explicit closure invoke source before window.app object: NO
/chat: FAIL line=2661 usesInvoke=YES reason=branch uses invoke but handleChatCommand has no local or parameter invoke source
/search: FAIL line=2757 usesInvoke=YES reason=branch uses invoke but handleChatCommand has no local or parameter invoke source
/compact: FAIL line=2843 usesInvoke=YES reason=branch uses invoke but handleChatCommand has no local or parameter invoke source
overall: FAIL
```

## Production-Code Boundary

- Production code modified: NO
- Problem fixed: NO
- Real Tauri/WebView launched: NO
- Provider / Keyring touched: NO
- Shell touched: NO
- Checkpoint touched: NO
- CSP / withGlobalTauri touched: NO
- Agent streaming touched: NO

## Validation Commands

```powershell
node tests/frontend/day27_handle_chat_command_invoke_smoke.js
git diff --name-only -- src/interface/web/app.js src/interface/web/index.html src/interface/web/style.css src/interface/web/modules src/interface/desktop/src/main.rs src/interface/desktop/tauri.conf.json src/engine/tool-system/src/shell.rs
git diff --check
git diff --cached --check
```

## Next Recommended Step

Open a separate fix task or a deeper isolated smoke task for `handleChatCommand(text)` `/chat`, `/search`, and `/compact` behavior. Do not combine that with Provider / Keyring, Shell, Checkpoint, CSP / withGlobalTauri, or Agent streaming work.

## V2B-2 Fix Follow-Up

Task: `STONE-AUDIT-V2B-2-HANDLE-CHAT-COMMAND-INVOKE-FIX`

Date: 2026-06-05

Fix summary:

- Added a local invoke source at the start of `handleChatCommand(text)`:
  - `const invoke = this.getTauriInvoke();`
- Did not move `/chat`, `/search`, or `/compact` branches.
- Did not modify `streamChat()`.
- Did not modify `sendChatMessage()`.
- Did not modify Provider / Keyring, Shell, Checkpoint, CSP / withGlobalTauri, or Agent streaming logic.

Observed day27 smoke after fix:

```text
day27 handleChatCommand invoke smoke
method: handleChatCommand(text) lines 2572-2876
method has local invoke source: YES
method has invoke parameter source: NO
explicit closure invoke source before window.app object: NO
/chat: PASS line=2663 usesInvoke=YES reason=branch uses invoke and handleChatCommand has an explicit invoke source
/search: PASS line=2759 usesInvoke=YES reason=branch uses invoke and handleChatCommand has an explicit invoke source
/compact: PASS line=2845 usesInvoke=YES reason=branch uses invoke and handleChatCommand has an explicit invoke source
overall: PASS
```

Current follow-up result: PASS
