# MAINRS Helper / Startup Split - V4X Day09

## 0. One-Line Result

Day09 stopped at **Phase A / Cargo preflight gate**. No Rust production code was moved because the cargo baseline is not green: `cargo fmt --check` reports existing formatting drift, and both `cargo check -p hajimi-desktop` and `cargo check --workspace` are blocked by missing GTK/GLib pkg-config system dependencies.

人话版：电梯没通过预检，所以没有搬 `main.rs` 家具；只记录阻塞债务。

## 1. Baseline

| Item | Value |
|---|---|
| Task | `V4X-DAY09: main.rs Helper Split with Cargo Preflight Gate` |
| Mode | Cargo preflight gated / docs-only after STOP |
| Branch | `work` |
| HEAD before | `160e640e54eb1bca12075ee518eef9f103803b93` |
| HEAD after | recorded by final git commit for this debt document |
| Initial `git status --short` | clean |
| Phase result | `PREFLIGHT BLOCKED / STOPPED` |
| Entered Phase B | NO |
| Production code changed | NO |
| Old dirty files staged | NO |

## 2. main.rs Line Count

| Item | Count |
|---|---:|
| `main.rs` before | `2782` |
| `main.rs` after | `2782` |
| Delta | `0` |
| `main.rs <=2200` | NOT MET |

Command:

```bash
wc -l src/interface/desktop/src/main.rs
```

## 3. Moved Modules

Moved modules: **NONE**.

No Rust source file was modified, no new Rust module was added, and no Tauri command name, argument, return shape, or ordering was changed.

## 4. Verification Results

| Command | Result | Summary |
|---|---|---|
| `git branch --show-current` | PASS | `work` |
| `git rev-parse HEAD` | PASS | `160e640e54eb1bca12075ee518eef9f103803b93` before this docs-only debt record |
| `git status --short` | PASS | clean before this docs-only debt record |
| `wc -l src/interface/desktop/src/main.rs` | PASS | `2782 src/interface/desktop/src/main.rs` |
| `cargo fmt --check` | FAIL | Existing Rust formatting drift reported in `src/interface/desktop/src/commands/agent.rs`, `commands/fs.rs`, `commands/info.rs`, `commands/provider.rs`, `commands/tool.rs`, and `main.rs`; no formatting changes were applied because Phase A is a gate and source edits were not allowed after STOP. |
| `cargo check -p hajimi-desktop` | BLOCKED | Missing GTK/GLib pkg-config dependencies, including `glib-2.0.pc`; command stopped in `glib-sys`. |
| `cargo check --workspace` | BLOCKED | Missing GTK/GLib pkg-config dependencies, including `glib-2.0.pc`, `gio-2.0.pc`, and `gobject-2.0.pc`; command stopped in sys crates. |
| `npm run test:security-gate` | PASS | `failures: 0`, `warnings: 97`, `allowlisted: 97` |
| `git diff --name-only -- src/interface/web/app.js src/interface/web/style.css package.json src/interface/desktop/tauri.conf.json src/engine/tool-system/src/shell.rs` | PASS | no output |
| `git diff --cached --check` | PASS | no output before docs staging; rerun before commit after staging |

## 5. Production Changes Scope

Production changes scope: **NONE**.

The following files were intentionally not modified:

- `src/interface/desktop/src/main.rs`
- `src/interface/desktop/src/*.rs`
- `src/interface/desktop/src/*/*.rs`
- `src/interface/web/app.js`
- `src/interface/web/style.css`
- `package.json`
- `src/interface/desktop/tauri.conf.json`
- `src/engine/tool-system/src/shell.rs`

## 6. Forbidden Scope Receipt

Forbidden diff command produced no output:

```bash
git diff --name-only -- src/interface/web/app.js src/interface/web/style.css package.json src/interface/desktop/tauri.conf.json src/engine/tool-system/src/shell.rs
```

Forbidden areas remained untouched:

- Tauri command external names and parameters
- Shell policy / shell allowlist
- Provider / Keyring behavior
- Checkpoint restore / export / replay / compare behavior
- Shell execution / CSP / `withGlobalTauri`
- Agent streaming main path
- Web `app.js` and `style.css`
- Git history rewrite

## 7. Debt / UNKNOWN / BLOCKED

| Item | Status | Notes |
|---|---|---|
| Day09 split completion | BLOCKED | Phase B was not entered. |
| Cargo formatting baseline | FAIL | `cargo fmt --check` reports existing drift; not fixed in this gated STOP task. |
| `cargo check -p hajimi-desktop` | BLOCKED | Missing native GTK/GLib pkg-config dependencies. |
| `cargo check --workspace` | BLOCKED | Missing native GTK/GLib pkg-config dependencies. |
| Rust move-only helper extraction | NOT RUN | No Rust code moved because cargo baseline was not green. |
| `main.rs <=2200` | NOT MET | `main.rs` remains `2782` lines. |
| Real Tauri desktop/WebView evidence | NOT RUN | Not needed after Phase A STOP; no WebView PASS claimed. |

## 8. Rollback Plan

Since this task only adds this debt document, rollback is simple:

1. Revert the commit that adds `docs/debt/MAINRS-HELPER-STARTUP-SPLIT-V4X.md`.
2. No Rust source rollback is required because no Rust source files were changed.
3. No command registry, Tauri startup, Provider/Keyring, Shell, Checkpoint, Agent streaming, or web files need rollback.

## 9. Next Step

Before retrying Day09 Phase B:

1. Restore a green formatting baseline or intentionally run a separate formatting-only task with explicit approval.
2. Install/provide the GTK/GLib development pkg-config dependencies required by Tauri on Linux (`glib-2.0.pc`, `gio-2.0.pc`, `gobject-2.0.pc`, and related GTK stack files).
3. Re-run `cargo fmt --check`, `cargo check -p hajimi-desktop`, and `cargo check --workspace`.
4. Only if those cargo gates pass, perform a narrow move-only helper extraction such as stream diagnostics or another Day08 low/medium-risk helper.

Final judgement: **Day09 blocked by cargo preflight environment and formatting baseline; no Rust production code changed.**
