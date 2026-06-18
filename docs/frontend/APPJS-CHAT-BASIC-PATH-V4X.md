# STONE-AUDIT-V4X-DAY3｜Chat Basic Path Extraction

## 0. One-Line Result

Day3 completed as a controlled Chat basic path slice: the browser load path already wires `views/chat-view.js` and `controllers/chat-controller.js` before `app.js`, `app.js setupChat()` already delegates to `HajimiChatController`, and this task added a focused Node smoke receipt for chat input, Enter submit, send button, model picker button, context buttons, session buttons, and slash-palette safe/high-risk selection behavior.

人话版：聊天输入框这块“门铃线”已经接到新控制器上了，本轮补了专门验电笔，确认按回车和点发送只会触发发送外壳，不会直接去动模型流式回复那根大线。

## 1. Scope

- Task: `STONE-AUDIT-V4X-DAY3：Chat Basic Path Extraction`
- Source work order: `F:\hajimi-code-cli\docs\roadmap\Hajimi ToneFix\task\task03.md`
- Source Day2 sampling: `F:\hajimi-code-cli\docs\frontend\APPJS-SECOND-PASS-SAMPLING-V4X.md`
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `aff9e9871944d06ba4b9747e4e3fc460ae01a829`
- Production code changed: NO
- WebView run in this task: NOT RUN; existing POST-02B manual receipt says Chat basic path passed, but this task does not re-claim a fresh WebView PASS.

## 2. Files

| Path | Status | Note |
|---|---|---|
| `F:\hajimi-code-cli\src\interface\web\controllers\chat-controller.js` | Existing production file, not modified | Owns Chat basic DOM event wiring. |
| `F:\hajimi-code-cli\src\interface\web\views\chat-view.js` | Existing production file, not modified | Owns Chat render shell helpers. |
| `F:\hajimi-code-cli\tests\frontend\day32_chat_basic_path_smoke.js` | Added | New focused Chat basic path smoke. |
| `F:\hajimi-code-cli\docs\frontend\APPJS-CHAT-BASIC-PATH-V4X.md` | Added | This receipt. |
| `F:\hajimi-code-cli\docs\debt\APPJS-CHAT-BASIC-PATH-WEBVIEW-DEBT-V4X.md` | Added | Records fresh WebView NOT RUN debt for this slice. |

## 3. Existing Wiring Evidence

| Check | Evidence | Result |
|---|---|---|
| `chat-view.js` script order | `src/interface/web/index.html` loads `views/chat-view.js` before `app.js`. | PASS |
| `chat-controller.js` script order | `src/interface/web/index.html` loads `controllers/chat-controller.js` before `app.js`. | PASS |
| `setupChat()` delegation | `src/interface/web/app.js` checks `window.HajimiChatController.init` before inline fallback. | PASS |
| Streaming core untouched | No edit to `streamChat()`. | PASS |
| Provider semantics untouched | No edit to Provider config/list/save/probe/keyring paths. | PASS |
| Agent streaming untouched | No edit to `invokeAgentTask()`, `handleAgentEvent()`, or streaming semantics. | PASS |

## 4. Node Smoke Coverage

New smoke: `F:\hajimi-code-cli\tests\frontend\day32_chat_basic_path_smoke.js`

It verifies:

- `index.html` loads `chat-view.js` and `chat-controller.js` before `app.js`.
- `app.js setupChat()` keeps a wrapper that delegates to `HajimiChatController` before the legacy inline fallback.
- `chat-controller.js` does not reference `streamChat`, `invokeTauri`, `getTauriInvoke`, `providerConfigs`, or `activeProviderId`.
- Chat input `input` event resizes and notifies slash palette if present.
- `Shift+Enter` does not submit.
- `Enter` submits through `app.sendChatMessage()`.
- Send button submits through `app.sendChatMessage()`.
- Model picker button calls `app.openModelPicker()`.
- Add context / clear context / edit mode / new session buttons delegate to app shell callbacks.
- Low-risk direct slash selection calls `app.sendChatMessage()`.
- High-risk slash selection fills input but does not auto-submit.
- The fake app defines `streamChat()` and `invokeTauri()` as throwing functions; the smoke passes only if the Chat basic controller does not call them.

## 5. Validation Receipt

| Command | Result |
|---|---|
| `git pull --ff-only origin stone-audit-v3x-controlled-demolition` | PASS: already up to date. |
| `git status --short` | PASS with old untracked files only before Day3 writes: `.agents/`, V4X plan markdown. |
| `node --check src/interface/web/app.js` | PASS |
| `node --check src/interface/web/controllers/chat-controller.js` | PASS |
| `node --check src/interface/web/views/chat-view.js` | PASS |
| `node tests/frontend/day32_chat_basic_path_smoke.js` | PASS |
| `npm run test:security-gate` | PASS |
| `git diff --name-only -- src/interface/desktop/src/main.rs src/engine/tool-system/src/shell.rs src/interface/desktop/tauri.conf.json` | PASS: no output. |
| `git diff --cached --check` | PASS |

## 6. Key State

| Item | Result |
|---|---|
| chat input | PASS in Node smoke |
| chat submit | PASS in Node smoke via `app.sendChatMessage()` |
| streamChat changed | NO |
| Provider semantics changed | NO |
| WebView result | NOT RUN in this task; see debt file |
| production changes | NO production code changes |
| old dirty files staged | NO |

## 7. Debt / Unknown

- Fresh Day3 WebView smoke is NOT RUN.
- Existing POST-02B manual release receipt says Chat input/edit/send/provider response passed, but that is not a replacement for a new Day3 WebView run after this smoke addition.
- `sendChatMessage()`, `handleChatCommand()`, and `streamChat()` remain high-risk and intentionally unchanged.

Debt companion:

- `F:\hajimi-code-cli\docs\debt\APPJS-CHAT-BASIC-PATH-WEBVIEW-DEBT-V4X.md`

## 8. Next

Proceed to Day4 only as a Slash / Command / Chat basic linkage receipt task. Do not expand this Day3 slice into Provider, Keyring, Shell, Checkpoint, Agent streaming, CSP, or withGlobalTauri work.
