# MAINRS Second-Pass Sampling - V4X Day08

## 0. One-Line Result

Day08 completed as **read-only / docs-only Rust split sampling**. `src/interface/desktop/src/main.rs` was not modified. The current file remains `2782` lines and still contains workspace/bootstrap glue, checkpoint restore/compare helpers, tool permission helpers, provider/keyring helpers, LLM client creation, provider blackboard glue, the Tauri builder, and a large in-file test module.

人话版：这轮只摸清 `main.rs` 里还剩哪些职责，先画拆分地图，不直接拆 Rust 入口。

## 1. Baseline

| Item | Value |
|---|---|
| Task | `V4X-DAY08: main.rs Second-Pass Sampling` |
| Mode | READ-ONLY / DOCS-ONLY / Rust split sampling |
| Branch | `work` |
| HEAD before | `fe77d03db04574578ca323bda61a6815c9003e2e` |
| HEAD after | recorded by final git commit for this sampling document |
| Initial `git status --short` | clean |
| Main file sampled | `src/interface/desktop/src/main.rs` |
| `main.rs` line count | `2782` |
| Production code changed | NO |
| Old dirty files staged | NO |

## 2. Allowed Files / Actual Scope

Issue #17 only allowed this deliverable:

| Path | Allowed action | Actual action |
|---|---|---|
| `docs/debt/MAINRS-SECOND-PASS-SAMPLING-V4X.md` | add/update Day08 sampling | ADDED |
| `docs/debt/mainrs-second-pass-sampling-v4x-*.txt` | optional raw log | NOT USED |

Forbidden files were not modified:

- `src/interface/desktop/src/main.rs`
- `src/interface/desktop/tauri.conf.json`
- `src/interface/web/app.js`
- `src/engine/tool-system/src/shell.rs`

## 3. Verification Results

| Command | Result | Summary |
|---|---|---|
| `git branch --show-current` | PASS | `work` |
| `git rev-parse HEAD` | PASS | `fe77d03db04574578ca323bda61a6815c9003e2e` before this docs-only change |
| `git status --short` | PASS | clean before this docs-only change |
| `wc -l src/interface/desktop/src/main.rs` | PASS | `2782 src/interface/desktop/src/main.rs` |
| `cargo check --workspace` | BLOCKED | environment lacks system GTK/GLib pkg-config dependencies: `glib-2.0.pc` and `gobject-2.0.pc` not found; no Rust source fix attempted |
| `git diff --name-only -- src/interface/desktop/src/main.rs src/interface/desktop/tauri.conf.json src/interface/web/app.js src/engine/tool-system/src/shell.rs` | PASS | no output |
| `git diff --cached --check` | PASS | no output before docs staging; rerun before commit |

## 4. Current main.rs Responsibility Map

| Area | Approx lines | Current responsibility | Current extraction status |
|---|---:|---|---|
| imports / module wiring | 1-63 | top-level dependencies and extracted module declarations | keep small; cleanup later only after modules stabilize |
| workspace/checkpoint store helpers | 66-181 | workspace root, checkpoint dir, trace-to-checkpoint records, checkpoint record reads | helper candidate with checkpoint tests |
| checkpoint compare/restore helpers | 181-431 | compare records, resolve restore target, build/backup/apply/rollback restore plans | high-risk helper candidate; keep semantics unchanged |
| tool permission helpers | 503-573 | permission gate, arg summary, native dialog confirmation | helper candidate, security-sensitive |
| stream diagnostics helpers | 592-632 | diagnostic path and diagnostic writes | helper candidate |
| provider data/config/keyring helpers | 643-1069 | provider structs, config paths, workspace/profile trust, keyring migration/read/write/delete | high-risk keep-for-now; Provider/Keyring semantics are P0-sensitive |
| provider backup encryption helpers | 1080-1118 | PBKDF2 + AES-GCM backup encrypt/decrypt | helper candidate only with crypto/provider backup tests |
| LLM client factory | 1131-1188 | create Ollama/Anthropic/OpenAI/custom client and retrieve API keys | high-risk keep-for-now; Provider/Keyring/LLM semantics coupled |
| provider blackboard glue | 1195-1252 | write provider caps into Agent blackboard | command-wrapper/helper candidate with agent/provider tests |
| Tauri builder/startup glue | 1267-1428 | plugins, setup, registry build, AgentLoop construction, AppState management, invoke handler registration | startup-only / registry-only candidates, but startup path remains high-risk |
| test module | 1437-2782 | command/helper regression tests for approvals, fs, checkpoint, provider, agent trace | candidate for test-module split after production helper extraction |

## 5. Helper Candidates

| Candidate | Current location | Suggested target | Risk | Required verification | Rollback point |
|---|---|---|---|---|---|
| workspace root + checkpoint dir helpers | `get_workspace_dir`, `checkpoint_store_dir` | `desktop/src/workspace.rs` or `desktop/src/checkpoint_store.rs` | medium | `cargo check --workspace`; checkpoint tests; fs path tests | revert helper module and imports |
| checkpoint record conversion/read/find/compare | `trace_event_id` through `compare_checkpoint_records` | `desktop/src/checkpoint_store.rs` | medium | `cargo test -p hajimi-desktop checkpoint`; `cargo check --workspace` | restore functions to `main.rs` |
| restore planning/apply helpers | `checkpoint_restore_content` through `apply_restore_plan` | `desktop/src/checkpoint_restore.rs` | high | restore tests, symlink/path traversal tests, dry-run/write restore tests | revert module extraction only |
| tool permission helpers | `tool_requires_confirmation` through `confirm_tool_native` | `desktop/src/tool_authorization.rs` | medium-high security | tool authorization tests; native confirmation policy tests | revert module extraction only |
| stream diagnostic helpers | `stream_diag_path`, `preview_for_diagnostic`, `write_stream_diagnostic` | `desktop/src/stream_diagnostics.rs` | low-medium | `cargo check --workspace`; command info smoke if available | revert module extraction only |
| provider config path/trust helpers | `provider_config_path` through `write_configs_to_path` | `desktop/src/provider_config_store.rs` | high Provider/Keyring | provider workspace/profile tests; key leak tests; permission tests | revert extraction, do not change semantics |
| backup encryption helpers | `derive_key`, `encrypt_backup`, `decrypt_backup` | `desktop/src/provider_backup_crypto.rs` | high crypto | provider backup serialization/encryption tests | revert extraction only |
| provider caps blackboard helper | `write_provider_caps_to_blackboard` | `desktop/src/provider_blackboard.rs` | medium | agent provider tests; blackboard leak-prevention test | revert extraction only |

## 6. Registry Candidates

| Candidate | Current location | Suggested target | Risk | Required verification | Rollback point |
|---|---|---|---|---|---|
| ToolRegistry construction | already in `src/interface/desktop/src/registry.rs` | keep in `registry.rs`; optionally add registry count smoke | low-medium | `cargo check --workspace`; tool list smoke; ensure allowed workspace paths preserved | revert registry changes |
| Tauri invoke handler list | `main.rs` `tauri::generate_handler![...]` block | possible `commands::handler::handler_list!` macro later | medium-high | `cargo check --workspace`; command smoke for fs/tool/provider/checkpoint/agent | revert macro/list extraction |
| command module declarations | `mod commands;` + generated handler imports | keep in `commands/mod.rs`; do not rename commands | medium | `cargo check --workspace`; frontend command invocation smoke | revert module move |

## 7. Startup Candidates

| Candidate | Current location | Suggested target | Risk | Required verification | Rollback point |
|---|---|---|---|---|---|
| Tauri plugins + builder setup | `main()` builder chain | `desktop/src/app_builder.rs` or `startup::build_app_state` | high startup | `cargo check --workspace`; real Tauri launch/WebView smoke when possible | revert startup extraction |
| AgentLoop construction | `main()` setup closure | `startup.rs` or `agent_loop_setup.rs` | high Agent streaming/provider coupling | agent command tests; trace subscription tests; provider selection tests | revert extraction only |
| AppState construction | `main()` setup closure | `state::AppState::new(...)` constructor candidate | medium-high | state compile tests; command tests for all state fields | revert constructor extraction |
| trace sender injection | `main()` setup closure | startup helper | high Agent trace | subscribe trace tests; real app smoke when possible | revert helper extraction |

## 8. Command-Wrapper Candidates

| Candidate | Current status | Suggested action | Risk | Required verification | Rollback point |
|---|---|---|---|---|---|
| fs commands | already extracted to `commands/fs.rs` | keep stable; no Day08 change | medium path safety | fs path traversal/symlink tests | revert only if regression |
| provider commands | already extracted to `commands/provider.rs`, backed by `main.rs` provider helpers | move helpers later, not command semantics | high Provider/Keyring | provider config/keyring/backup tests | revert helper extraction |
| checkpoint commands | already extracted to `commands/checkpoint.rs`, backed by `main.rs` checkpoint helpers | helper-only extraction later | high restore/export | checkpoint restore/export/compare tests | revert helper extraction |
| agent commands | already extracted to `commands/agent.rs`, backed by startup/provider helpers | keep-for-now | high Agent streaming | stream/agent/provider/trace tests | revert only with strong evidence |
| tool commands | already extracted to `commands/tool.rs`, backed by auth helpers | helper-only extraction later | high shell/tool safety | tool permission tests; shell allowlist tests | revert helper extraction |
| info/governance/profile commands | already extracted | keep stable | low-medium | command compile/smoke tests | revert only if regression |

## 9. High-Risk Keep / Keep-For-Now

| Area | Reason |
|---|---|
| Provider save/delete/test/keyring | P0 secret storage and workspace/profile isolation; do not change semantics in sampling task |
| Shell execution / shell allowlist | security-sensitive and explicitly forbidden by Issue #17 |
| Checkpoint restore/export/replay/compare semantics | destructive file-write and backup/rollback semantics need dedicated tests |
| Agent streaming / trace / resource alerts | runtime event stream and LLM provider coupling; keep stable |
| Tauri command names/signatures | frontend IPC contract risk; do not rename or wrap without command smoke |
| Tauri builder startup | desktop startup requires real app/WebView evidence for high confidence |
| `withGlobalTauri`, CSP, web `app.js` | outside Day08 scope and explicitly forbidden |

## 10. Day09-Day11 Rust Split Recommendation

| Day | Suggested focus | Scope | Required checks | Stop rule |
|---|---|---|---|---|
| Day09 | helper-only low/medium extraction | workspace/checkpoint record read/compare or stream diagnostics | `cargo check --workspace`; targeted checkpoint/info tests if available | stop on semantic changes or path/security uncertainty |
| Day10 | registry/startup seam hardening | keep `registry.rs`; optionally extract AppState construction helper without command name changes | `cargo check --workspace`; tool list and command smoke | stop if Tauri launch/WebView evidence becomes required |
| Day11 | provider/checkpoint high-risk planning or test split | add tests or split test module only; avoid Provider/Keyring semantics | provider/checkpoint tests; security gates | stop before any keyring/restore behavior change |

Recommended first implementation task: **extract a narrow checkpoint record/read/compare helper module or stream diagnostic helper**, not Provider/Keyring, Shell, Agent streaming, or Tauri startup.

## 11. Debt / UNKNOWN / BLOCKED

| Item | Status | Notes |
|---|---|---|
| `main.rs <=900` hard target | NOT MET | Current line count is `2782`. |
| `cargo check --workspace` in this environment | BLOCKED | Missing system GTK/GLib development pkg-config files (`glib-2.0.pc`, `gobject-2.0.pc`). |
| Real Tauri desktop/WebView confidence | NOT RUN | Day08 did not launch a real desktop app. |
| Safe migration of Provider/Keyring helpers | UNKNOWN / high-risk-keep | Requires dedicated tests and no secret-leak regressions. |
| Safe migration of startup builder | UNKNOWN / high-risk-keep | Requires desktop launch or equivalent startup evidence. |
| Safe migration of checkpoint restore helpers | UNKNOWN / high-risk-keep | Requires restore/rollback/path traversal tests. |

## 12. Next Step

Proceed to Day09 only with a narrow Rust helper extraction whose rollback point is simple and whose verification does not require changing forbidden semantics. Keep Provider/Keyring, Shell, Checkpoint restore writes, Agent streaming, and Tauri startup as keep-for-now unless a dedicated task adds the required tests first.
