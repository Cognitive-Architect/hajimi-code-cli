# STONE-AUDIT-V1.5-PACKAGE-SMOKE

> Status: PASS
> Recorded: 2026-06-04
> Evidence source: user-provided real-machine package smoke result

## Scope

This record captures the separate package/WebView smoke result for STONE-AUDIT-V1.5.

Validated targets:

- Slash Palette
- Command Palette
- Audit Log
- Resource Dashboard

## Environment

- Package directory: `hajimi-code-cli-package-smoke-59a8794/target/release`
- Executable: `target/release/hajimi-desktop.exe`
- Application version shown in UI: `v3.8.0-batch-1`
- Model shown in UI: `deepseek / deepseek-v4-pro`

## Baseline Startup

- Application launch: PASS
- Main UI rendered: PASS
- No white screen or crash: PASS
- Model display: PASS

## Slash Palette

- Typing `/` opens candidates: PASS
- Candidate list rendered: PASS
- Page did not white-screen or crash: PASS

## Command Palette

- `Ctrl+Shift+P` opens command palette: PASS
- Search input accepts text: PASS
- Clicking a candidate responds: PASS
- Page did not white-screen or crash: PASS

Note: clicking `文件：打开文件` opened the `tauri.localhost` file-path prompt. No file operation was continued.

## Audit Log

- Entry is reachable: PASS
- UI opens: PASS
- Log list renders: PASS
- Refresh button is clickable: PASS
- Clicking refresh did not white-screen, crash, or show an error: PASS

Note: after refresh there was no obvious visual change, but the list stayed normal.

## Resource Dashboard

- Resource monitor cards render: PASS
- Metrics/empty state render normally: PASS
- Page did not white-screen or crash: PASS

Note: resource monitor shows `迭代`, `Blackboard`, and `Edits`.

## Forbidden-Scope Confirmation

- Production code was not modified: YES
- Provider Keyring high-risk write path was not tested: YES
- Shell execution was not tested: YES
- Checkpoint restore/export/compare/replay were not executed: YES
- No opportunistic fix was made: YES

## Final Result

PACKAGE-SMOKE: PASS

`PENDING-SEPARATE-PACKAGE-SMOKE` may be treated as having a PASS package smoke receipt for the four scoped frontend targets above, based on this user-provided real-machine/WebView record.
