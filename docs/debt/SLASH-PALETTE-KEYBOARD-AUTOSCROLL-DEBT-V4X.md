# SLASH Palette Keyboard Autoscroll Debt V4X

## Status

- Source task: `STONE-AUDIT-V4X-DAY4：Slash / Command / Chat Basic WebView Receipt`
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD at receipt: `e1c05c40a2b06e90fe32b1996de6fd937ff66d77`
- Production code changed: NO
- Classification: UX debt

## Observation

Manual release WebView smoke confirmed:

- Slash Palette opens when `/` is typed.
- Candidates are visible.
- Mouse scroll works.
- Low-risk `/tools` candidate can be selected/submitted and returns `可用工具（40个）`.
- No white screen or crash was observed.

However, keyboard arrow navigation is only `PARTIAL`: moving selection with arrow keys does not auto-scroll the menu position to keep the selected candidate in view.

## Not Verified

- DevTools console was NOT CHECKED.
- No production fix was attempted in Day4.
- No high-risk Slash candidate was clicked.

## Required Future Fix Scope

A future UX-only task may inspect `src/interface/web/modules/slash-palette.js` and add focused smoke coverage for:

1. ArrowDown / ArrowUp selection movement.
2. Selected candidate scrolls into view.
3. Mouse scroll behavior remains unchanged.
4. Low-risk direct candidate behavior remains unchanged.
5. High-risk fill-only candidate still does not auto-submit.

Do not combine this UX fix with Provider, Keyring, Shell, Checkpoint, Agent streaming, CSP, or withGlobalTauri work.
