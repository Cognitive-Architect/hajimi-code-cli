# APPJS Chat Basic Path WebView Debt V4X

## Status

- Task: `STONE-AUDIT-V4X-DAY3：Chat Basic Path Extraction`
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `aff9e9871944d06ba4b9747e4e3fc460ae01a829`
- Production code changed: NO
- Debt reason: fresh Day3 real Tauri WebView smoke was NOT RUN in this task.

## What Is Verified

- Node smoke added at `F:\hajimi-code-cli\tests\frontend\day32_chat_basic_path_smoke.js`.
- The smoke verifies Chat basic DOM wiring and confirms the controller does not call `streamChat()` or Tauri invoke paths.
- `src/interface/web/index.html` already loads `views/chat-view.js` and `controllers/chat-controller.js` before `app.js`.
- `src/interface/web/app.js` already delegates `setupChat()` to `window.HajimiChatController.init(this)` before inline fallback.

## What Is Not Verified Here

- Fresh real desktop WebView launch after this task.
- Fresh manual click of Chat input/type/delete/submit after this task.
- Fresh console inspection for Chat-specific errors after this task.
- Real Provider response after this task.

## Prior Evidence That Must Not Be Over-Claimed

The V4X plan records POST-02B manual release WebView smoke where Chat input editable, Chat send, and Provider response were PASS with assistant replying `pong`. That is useful background evidence, but this Day3 task does not re-run WebView and must not mark a fresh Day3 WebView result as PASS.

## Required Future Receipt

Run a Day4 or POST follow-up real WebView receipt:

1. Launch release or dev Tauri WebView.
2. Confirm app launch PASS, white screen NO, crash NO.
3. Type in Chat input.
4. Submit through Enter and send button.
5. Confirm no Chat-specific console error.
6. Keep Provider save/delete/test/probe/keyring, Shell, Checkpoint, and Agent streaming high-risk paths out of scope unless a separate approved task permits them.
