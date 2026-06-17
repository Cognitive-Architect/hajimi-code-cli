# STONE-AUDIT-V3X｜Controlled Demolition Closure

## 0. One-line Conclusion

V3X controlled demolition closes as `PARTIAL SUCCESS / NOT FULLY MET`: CSS split, frontend Node smoke, security gate, and cargo workspace checks pass; `app.js <=1200`, `main.rs <=900`, repo volume reduction, and Day14 real WebView full regression are not met.

Human summary: the renovation made several rooms cleaner, but the two biggest rooms and the storage basement are still too large, and the final real-window walkthrough was not run in this closure pass.

## 1. Current Git Coordinate

| Field | Value |
|---|---|
| Branch | `stone-audit-v3x-controlled-demolition` |
| HEAD before closure doc | `b56f764e378f7fe07cebcfe8b481889acd822bf1` |
| Git status before closure doc | clean |
| Old dirty files staged | `NO` |
| Production changes in this closure | `NO` |

Commands:

```powershell
git branch --show-current
git rev-parse HEAD
git status --short
git log --oneline -n 12
```

Recent commits:

```text
b56f764e docs(debt): add repo volume surgery dry run
d51182f1 docs(roadmap): add Codex handoff for STONE-AUDIT-V3X v0.14
3618dea2 test(security/frontend): adapt security gate and dom contract smoke tests to split structure
1d723870 refactor(desktop/commands): split Tauri commands into submodules
945b6ec2 refactor(desktop): split tauri startup skeleton
cfc2443d docs(debt): sample mainrs split plan
425263dd docs(roadmap): update v3x handoff receipt
70be189a docs(roadmap): add v3x handoff
d83fc297 docs(debt): record appjs high risk remainder
22efbaf9 docs(frontend): record appjs fallback removal stop
fde3a5e1 docs(frontend): sample appjs closure candidates
f800fcad docs(debt): record tauri invoke sampling debt
```

## 2. Hard Target Table

| Target | Current measurement | Goal | Result | Evidence |
|---|---:|---:|---|---|
| `src/interface/web/app.js` | `5568` lines | `<=1200` | `NOT MET` | `(Get-Content src/interface/web/app.js).Count` |
| `src/interface/web/style.css` | `21` lines | `<=120` | `MET` | `(Get-Content src/interface/web/style.css).Count` |
| `src/interface/desktop/src/main.rs` | `2782` lines | `<=900` | `NOT MET` | `(Get-Content src/interface/desktop/src/main.rs).Count` |
| repo pack size | `3.81 GiB` | reduced or approved surgery | `NOT MET / APPROVAL REQUIRED` | `git count-objects -vH` |
| Day14 real WebView full regression | not run in this pass | PASS receipt | `BLOCKED / NOT RUN` | no real Tauri window receipt produced in this closure |

Command output:

```text
app.js lines: 5568
style.css lines: 21
main.rs lines: 2782
```

Repo volume:

```text
count: 7412
size: 52.18 MiB
in-pack: 25411
packs: 2
size-pack: 3.81 GiB
prune-packable: 2910
garbage: 0
size-garbage: 0 bytes
```

## 3. Frontend Syntax and Smoke Results

| Command | Result | Notes |
|---|---|---|
| `node --check src/interface/web/app.js` | `PASS` | no output |
| controllers/views/services `node --check` loop | `PASS` | no output |
| `node tests/frontend/day16_slash_palette_smoke.js` | `PASS` | 8 scenarios |
| `node tests/frontend/day21_slash_palette_app_integration_smoke.js` | `PASS` | 8 scenarios |
| `node tests/frontend/day22_command_palette_catalog_smoke.js` | `PASS` | 7 scenarios |
| `node tests/frontend/day25_inspector_safety_smoke.js` | `PASS` | 4 scenarios |
| `node tests/frontend/day26_dom_contract_smoke.js` | `PASS` | 162 HTML ids; 78 uncovered ids still recorded |
| `node tests/frontend/day27_handle_chat_command_invoke_smoke.js` | `PASS` | `/chat`, `/search`, `/compact` have local invoke source |
| `node tests/frontend/day28_command_palette_dom_smoke.js` | `PASS` | 5 scenarios |
| `node tests/frontend/day29_session_list_dom_smoke.js` | `PASS` | 6 scenarios |
| `node tests/frontend/day34_model_picker_smoke.js` | `PASS` | command output says PASS |
| `node tests/frontend/day14_sessions_thinking_modules_smoke.js` | `PASS` | session list render skipped when view missing in that fixture |

Interpretation:

- Node smoke coverage is healthy for the selected Day14 set.
- Node smoke is not a substitute for a real Tauri WebView click pass.
- `day26` still reports uncovered DOM ids; this is coverage debt, not a failing assertion.

## 4. Security Gate

Command:

```powershell
npm run test:security-gate
```

Observed result:

```text
Security Audit Gate V1 summary
findings: 97
failures: 0
warnings: 97
allowlisted: 97
Security Audit Gate V1: PASS
```

Interpretation:

- Current security-gate command exits PASS.
- This is not a clean security state: all 97 findings are accepted/allowlisted warnings, mostly legacy dangerous HTML API usage.
- Earlier V3X documents recorded previous known failures, but the current Day14 command result is PASS after Day12-CLEANUP allowlist adaptation.

## 5. Cargo / Rust Regression

Command:

```powershell
cargo check --workspace
```

Observed result:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.96s
```

Warnings:

```text
warning count observed: 29
main sources include unused imports and dead code after command split
```

Interpretation:

- Cargo workspace check passes.
- The warnings should remain recorded as cleanup debt, not fixed in Day14.
- Day14 did not modify Rust production code.

## 6. WebView Receipt

Day14 real Tauri WebView full regression: `BLOCKED / NOT RUN`.

Reason:

- This closure pass did not operate a real Tauri desktop window or produce a fresh click receipt.
- Prior V3X docs contain partial WebView receipts for selected areas, but they do not prove the full Day14 real app path.

Therefore:

- WebView app launch: `BLOCKED / NOT RUN`
- Command Palette real click: `BLOCKED / NOT RUN for Day14 full pass`
- Chat basic real click: `BLOCKED / NOT RUN`
- Session basic real click: `BLOCKED / NOT RUN for Day14 full pass`
- Inspector real click: `BLOCKED / NOT RUN for Day14 full pass`
- Dashboard real click: `BLOCKED / NOT RUN for Day14 full pass`
- Provider read-only real click: `BLOCKED / NOT RUN`

This document does not claim WebView PASS.

## 7. Production Diff / Staging Safety

Commands:

```powershell
git diff --name-only -- src
git diff --cached --check
```

Observed before creating this closure doc:

```text
git diff --name-only -- src: no output
git diff --cached --check: PASS
```

Closure scope:

- Only this docs/debt closure document is intended to be staged and committed.
- No production source, build config, or Tauri config is changed by Day14.

## 8. Final Score

Final score: `68 / 100`

| Area | Score | Reason |
|---|---:|---|
| CSS demolition | 10 / 10 | `style.css` is 21 lines and target is met |
| Frontend Node smoke | 14 / 15 | selected smoke set passes; uncovered DOM ids remain |
| Security gate | 12 / 15 | command passes, but 97 accepted-risk warnings remain |
| Cargo / Rust compile | 12 / 15 | workspace check passes, but 29 warnings remain |
| app.js size target | 4 / 15 | target not met; Day09 `RETARGET` is honest but still incomplete |
| main.rs size target | 6 / 15 | command split improved structure, but target not met |
| repo volume | 2 / 10 | dry-run complete, pack still `3.81 GiB`; approval required |
| Day14 real WebView full regression | 0 / 5 | not run in this pass |
| Documentation honesty | 8 / 10 | receipts and debt are recorded; final WebView still missing |

Human summary: the project is much more organized than before, but the original hard numbers were not all achieved. This is a controlled partial win, not a full demolition victory.

## 9. Residual Debt

| Debt | Status | Why it remains |
|---|---|---|
| `app.js <=1200` | `OPEN / RETARGET` | high-risk paths remain coupled: Chat streaming, Provider/Keyring, Checkpoint, Shell/tool execution, Agent streaming |
| `main.rs <=900` | `OPEN` | command handlers moved, but helper types, tests, and state-bound logic still remain in main.rs |
| repo volume | `OPEN / APPROVAL REQUIRED` | pack remains `3.81 GiB`; actual shrink requires history rewrite approval |
| Day14 WebView full regression | `BLOCKED / NOT RUN` | no fresh real Tauri window click receipt was produced |
| security HTML rendering warnings | `OPEN / ACCEPTED RISK` | 97 allowlisted DOM HTML findings remain |
| cargo warnings | `OPEN` | 29 warnings after Rust split, not fixed in closure |
| DOM coverage gaps | `OPEN` | `day26` still records `UNCOVERED ids: 78` |
| Provider/Keyring high-risk path | `OPEN` | only read-only slices and smoke records exist; no semantic rewrite |
| Checkpoint restore/export/compare/replay | `OPEN` | intentionally not changed during V3X closure |
| Shell/tool execution path | `OPEN` | high-risk command path must remain separately audited |

## 10. Next Roadmap

Recommended next single action:

`Open a V3X-post closure triage task for real Tauri WebView full regression only.`

Do not combine it with app.js reduction, main.rs cleanup, or repo history surgery.

After that:

1. If WebView passes, decide whether to approve repo-volume history rewrite.
2. If WebView fails, record exact failing path and open a focused fix task.
3. Only after WebView and repo-volume decisions should app.js/main.rs size targets be revisited.

## 11. Forbidden Claims

- Do not claim `app.js <=1200` is met.
- Do not claim `main.rs <=900` is met.
- Do not claim repo volume surgery executed.
- Do not claim Day14 WebView PASS.
- Do not claim security warnings are fixed.
- Do not claim cargo warnings are fixed.

## 12. Closure Receipt

| Field | Value |
|---|---|
| Closure document | `docs/debt/STONE-AUDIT-V3X-CONTROLLED-DEMOLITION-CLOSURE.md` |
| Commit planned | `docs(debt): close v3x controlled demolition` |
| Production changes | `NO` |
| History rewrite | `NO` |
| Old dirty files staged | `NO` |
| Final conclusion | `PARTIAL SUCCESS / NOT FULLY MET` |
