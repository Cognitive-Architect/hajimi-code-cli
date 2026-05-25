# DEBT-B18 Security Hardening Closure

> Date: 2026-05-19
> Scope: residual security findings from `docs/debt/active/HAJIMI-SECURITY-RESIDUAL-FIX-GUIDE.md`
> Status: CLEARED for B-18 residuals; legacy DOM `innerHTML` allowlist warnings remain tracked separately by the security gate.

## Fixed Findings

- R-001 `withGlobalTauri=true`: closed by setting `src/interface/desktop/tauri.conf.json` to `withGlobalTauri=false` and routing frontend IPC through `src/interface/web/modules/tauri-bridge.js`.
- R-002 frontend-mintable tool confirmation token: removed the public token mint command and changed `execute_tool` to perform Rust-side native dialog confirmation for tools requiring approval.
- R-003 naked `run_command`: removed the legacy Tauri command from the invoke handler. Frontend shell paths now call the governed `powershell` / `bash` tool through `execute_tool`.
- R-004 anti-regression gate: `tests/security/security_audit_gate.js` now fails on global Tauri exposure, public token minting, naked `run_command`, missing `execute_tool` permission gate, unbound desktop file tools, and inline edit resolver bypass.
- R-005 receipt / CI: this receipt records verification output, and `.github/workflows/security.yml` now runs `npm run test:security-gate`.

## Verification Commands

```bash
cargo fmt -p hajimi-desktop
node --check src/interface/web/app.js
node --check src/interface/web/modules/tauri-bridge.js
node --check tests/security/security_audit_gate.js
npm run test:security-gate
cargo check -p hajimi-desktop
cargo test -p hajimi-desktop
cargo test -p engine-tool-system --lib
cargo check --workspace
```

Observed result:

```text
npm run test:security-gate: PASS, failures=0, warnings=106
cargo check -p hajimi-desktop: PASS
cargo test -p hajimi-desktop: PASS, 26 passed
cargo test -p engine-tool-system --lib: PASS, 75 passed
cargo check --workspace: PASS
```

WebView smoke:

```text
Started a temporary static server for src/interface/web on 127.0.0.1:3456.
Ran cargo tauri dev --no-watch.
Observed cargo launch target/debug/hajimi-desktop.exe and remain running until the smoke timeout cleanup.
No startup error was emitted.
```

## Risk Before / After

- Before: an injected frontend script could use `window.__TAURI__`, mint a confirmation token, and invoke high-risk tools.
- After: business frontend code no longer directly uses the global Tauri API, `withGlobalTauri` is disabled, high-risk tool execution requires a Rust-side native confirmation, and the old `run_command` IPC command is no longer exposed.

## Residual Risk

- `window.__TAURI_INTERNALS__` remains part of Tauri's runtime bridge for vanilla JS IPC. The residual B-18 attack chain is still broken by backend permission enforcement and native confirmation.
- The security gate still reports allowlisted legacy dangerous HTML API usage as warnings. Those are not part of B-18 but should remain visible until the DOM rendering debt is paid down.

## Rollback

- If the Tauri WebView bridge fails in a release build, temporarily restore only `withGlobalTauri=true` while keeping `tauri-bridge.js` and backend permission changes.
- If native dialog confirmation blocks a critical workflow, narrow the affected tool's permissions explicitly instead of restoring public token minting.
- Do not restore `run_command`; use the ToolRegistry shell tool with `ToolPermissions`.
