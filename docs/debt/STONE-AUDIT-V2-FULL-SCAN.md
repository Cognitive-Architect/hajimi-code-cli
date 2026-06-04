# STONE-AUDIT-V2-FULL-SCAN

> Task: `STONE-AUDIT-V2-FULL-SCAN-READONLY`
> Date: 2026-06-04
> Branch: `feature/toolfix-deepseek-schema`
> HEAD: `2b50333a5b76d559545d40c7cb7d91890f17fed8`
> Raw log: `docs/debt/stone-audit-v2-full-scan-20260604-173902.txt`

## 1. Result

`PASS / READONLY SCAN COMPLETED`

This scan did not modify production code. It only records repository facts and leaves all discovered issues unfixed.

Out of scope and untouched:

- Provider / Keyring
- Shell execution
- Checkpoint restore / export / compare / replay
- Agent streaming
- CSP / `withGlobalTauri`
- Production files under `src/interface/web/*.js`, `src/interface/web/*.css`, `src/interface/desktop/*.rs`, and Tauri config

## 2. Git Baseline

| Item | Evidence |
|---|---|
| Branch | `git branch --show-current` -> `feature/toolfix-deepseek-schema` |
| HEAD | `git rev-parse HEAD` -> `2b50333a5b76d559545d40c7cb7d91890f17fed8` |
| Initial dirty files | `git status --short` output recorded in the raw log |
| Existing old dirty files | YES |
| Production code modified by this task | NO |

Old dirty files present before this scan:

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
```

This task adds only:

```text
docs/debt/STONE-AUDIT-V2-FULL-SCAN.md
docs/debt/stone-audit-v2-full-scan-20260604-173902.txt
```

## 3. Top 30 Current Tracked Large Files

Command evidence: `git ls-files | ... Sort-Object Bytes -Descending | Select-Object -First 30`

| Rank | KB | Bytes | Path |
|---:|---:|---:|---|
| 1 | 88269.2 | 90387630 | `models/fast-all-MiniLM-L6-v2/model.onnx` |
| 2 | 81230.6 | 83180180 | `models/all-MiniLM-L6-v2.tar.gz` |
| 3 | 14000.4 | 14336376 | `src/intelligence/codex-twist/index.node` |
| 4 | 695.0 | 711661 | `models/fast-all-MiniLM-L6-v2/tokenizer.json` |
| 5 | 337.4 | 345541 | `src/patches/zstd-sys/zstd/lib/compress/zstd_compress.c` |
| 6 | 256.3 | 262488 | `src/patches/zstd-sys/zstd/lib/common/xxhash.h` |
| 7 | 229.6 | 235108 | `Cargo.lock` |
| 8 | 227.7 | 233184 | `src/interface/web/app.js` |
| 9 | 226.1 | 231508 | `models/fast-all-MiniLM-L6-v2/vocab.txt` |
| 10 | 224.8 | 230204 | `docs/security/examples/SECURITY_REVIEW_SAMPLE.json` |
| 11 | 179.3 | 183628 | `src/interface/desktop/src/main.rs` |
| 12 | 177.9 | 182177 | `src/patches/zstd-sys/zstd/lib/legacy/zstd_v07.c` |
| 13 | 177.5 | 181748 | `src/patches/zstd-sys/zstd/lib/zstd.h` |
| 14 | 159.7 | 163487 | `src/patches/zstd-sys/zstd/lib/legacy/zstd_v06.c` |
| 15 | 150.1 | 153712 | `src/patches/zstd-sys/zstd/lib/legacy/zstd_v05.c` |
| 16 | 140.1 | 143417 | `src/interface/desktop/Cargo.lock` |
| 17 | 129.4 | 132468 | `src/patches/zstd-sys/zstd/lib/legacy/zstd_v04.c` |
| 18 | 122.3 | 125199 | `src/patches/zstd-sys/zstd/lib/legacy/zstd_v02.c` |
| 19 | 109.3 | 111939 | `src/patches/zstd-sys/src/bindings_zstd_experimental.rs` |
| 20 | 108.7 | 111309 | `src/patches/zstd-sys/zstd/lib/legacy/zstd_v03.c` |
| 21 | 101.7 | 104126 | `src/patches/zstd-sys/src/bindings_zstd_std_experimental.rs` |
| 22 | 101.4 | 103834 | `docs/security/examples/SECURITY_REVIEW_SAMPLE.md` |
| 23 | 100.4 | 102821 | `src/patches/zstd-sys/zstd/lib/compress/zstd_lazy.c` |
| 24 | 100.0 | 102400 | `src/patches/zstd-sys/zstd/lib/decompress/zstd_decompress.c` |
| 25 | 99.1 | 101467 | `src/patches/zstd-sys/zstd/lib/decompress/zstd_decompress_block.c` |
| 26 | 90.9 | 93127 | `src/interface/web/style.css` |
| 27 | 82.1 | 84098 | `src/patches/zstd-sys/zstd/lib/compress/zstdmt_compress.c` |
| 28 | 81.9 | 83899 | `src/interface/desktop/gen/schemas/desktop-schema.json` |
| 29 | 81.9 | 83899 | `src/interface/desktop/gen/schemas/windows-schema.json` |
| 30 | 76.4 | 78215 | `src/intelligence/agent-core/agent_loop.rs` |

## 4. Source Line Count Hotspots

Command evidence: Node read-only scan over `src`, `tests`, and `scripts`, excluding build/dependency directories and `src/patches`.

| Rank | Lines | File |
|---:|---:|---|
| 1 | 5617 | `src/interface/web/app.js` |
| 2 | 5094 | `src/interface/desktop/src/main.rs` |
| 3 | 4611 | `src/interface/web/style.css` |
| 4 | 2023 | `src/intelligence/agent-core/agent_loop.rs` |
| 5 | 1957 | `src/intelligence/agent-core/llm_native/turn.rs` |
| 6 | 1715 | `src/intelligence/memory/src/dream.rs` |
| 7 | 1571 | `src/intelligence/agent-core/llm/bridge.rs` |
| 8 | 1289 | `src/intelligence/agent-core/llm_native/driver.rs` |
| 9 | 1173 | `src/engine/llm-core/src/openai.rs` |
| 10 | 1148 | `src/intelligence/agent-core/agent_loop_tests.rs` |
| 11 | 1137 | `src/intelligence/agent-core/context_budget.rs` |
| 12 | 1054 | `src/intelligence/memory/src/graph.rs` |
| 13 | 988 | `src/intelligence/agent-core/long_context_pack.rs` |
| 14 | 898 | `src/intelligence/agent-core/context_probe.rs` |
| 15 | 892 | `src/intelligence/agent-core/act_executor.rs` |
| 16 | 870 | `src/interface/web/modules/thinking-ui.js` |
| 17 | 856 | `src/intelligence/agent-core/edit_applier.rs` |
| 18 | 799 | `src/intelligence/memory/src/hnsw.rs` |
| 19 | 771 | `src/engine/tool-system/src/fs.rs` |
| 20 | 734 | `src/intelligence/agent-core/planner.rs` |
| 21 | 718 | `src/intelligence/agent-core/context_receipt.rs` |
| 22 | 708 | `src/intelligence/agent-core/security_workflow.rs` |
| 23 | 698 | `src/intelligence/codex-twist/src/ffi.rs` |
| 24 | 690 | `src/engine/tool-system/src/lsp.rs` |
| 25 | 664 | `src/engine/tool-system/src/mcp.rs` |
| 26 | 645 | `src/interface/web/index.html` |
| 27 | 639 | `src/intelligence/agent-core/orchestrator.rs` |
| 28 | 638 | `src/intelligence/agent-core/tests/agent_core_e2e.rs` |
| 29 | 580 | `src/intelligence/agent-core/tests/llm_native_e2e_tests.rs` |
| 30 | 549 | `src/intelligence/agent-core/tests/editing_e2e.rs` |

## 5. Top 30 Complexity Hotspots

No dedicated complexity tool was available (`tokei`, `cloc`, `scc`, and `radon` were not found). This is a read-only heuristic scan: it estimates hotspots from function span plus branch-like tokens. Treat this as triage, not exact cyclomatic complexity.

Command evidence: Node read-only scan over `src`, `tests`, and `scripts`, excluding build/dependency directories and `src/patches`.

| Rank | Score | Branch Tokens | Lines | Start | Function | File |
|---:|---:|---:|---:|---:|---|---|
| 1 | 114 | 110 | 270 | 3759 | `setupProviderSettings` | `src/interface/web/app.js` |
| 2 | 112 | 108 | 300 | 2572 | `handleChatCommand` | `src/interface/web/app.js` |
| 3 | 94 | 92 | 118 | 1279 | `highlightCode` | `src/interface/web/app.js` |
| 4 | 72 | 68 | 314 | 177 | `run` | `src/intelligence/agent-core/agent_loop.rs` |
| 5 | 63 | 60 | 182 | 3273 | `streamChat` | `src/interface/web/app.js` |
| 6 | 53 | 50 | 188 | 188 | `chat_and_collect` | `src/intelligence/agent-core/llm/bridge.rs` |
| 7 | 53 | 50 | 188 | 506 | `chat_and_collect` | `src/intelligence/agent-core/llm/bridge.rs` |
| 8 | 51 | 48 | 167 | 204 | `send_request` | `src/engine/tool-system/src/lsp.rs` |
| 9 | 50 | 47 | 199 | 78 | `run_edit_workflow` | `src/intelligence/agent-core/workflow_orchestrator.rs` |
| 10 | 49 | 46 | 184 | 1336 | `handle_plan_adjustment` | `src/intelligence/agent-core/agent_loop.rs` |
| 11 | 47 | 46 | 59 | 4030 | `openProviderModal` | `src/interface/web/app.js` |
| 12 | 46 | 42 | 245 | 853 | `stream_chat_with_tools` | `src/engine/llm-core/src/openai.rs` |
| 13 | 45 | 42 | 229 | 35 | `parseGateJson` | `scripts/security-report.js` |
| 14 | 45 | 42 | 186 | 624 | `parse_openai_sse_line` | `src/engine/llm-core/src/openai.rs` |
| 15 | 39 | 36 | 225 | 52 | `createSlashPalette` | `src/interface/web/modules/slash-palette.js` |
| 16 | 38 | 36 | 88 | 309 | `generate_smart_message` | `src/engine/tool-system/src/git.rs` |
| 17 | 35 | 33 | 133 | 63 | `spawn_worker` | `src/intelligence/agent-core/worker_lifecycle_manager.rs` |
| 18 | 33 | 32 | 36 | 298 | `insert_with_levels` | `src/intelligence/memory/src/hnsw.rs` |
| 19 | 32 | 28 | 246 | 2017 | `stream_chat` | `src/interface/desktop/src/main.rs` |
| 20 | 30 | 28 | 115 | 204 | `retrieve_with_ast` | `src/intelligence/agent-core/memory_retriever.rs` |
| 21 | 29 | 26 | 165 | 639 | `legacy_act` | `src/intelligence/agent-core/agent_loop.rs` |
| 22 | 29 | 27 | 102 | 3170 | `handleAgentEvent` | `src/interface/web/app.js` |
| 23 | 28 | 26 | 122 | 812 | `bootstrap_first_tool_call` | `src/intelligence/agent-core/agent_loop.rs` |
| 24 | 28 | 26 | 103 | 14 | `score_skill` | `src/intelligence/agent-core/skills/scoring.rs` |
| 25 | 28 | 26 | 86 | 169 | `parseStreamEvent` | `src/interface/web/modules/thinking-ui.js` |
| 26 | 28 | 27 | 79 | 56 | `resolve_dynamic_retrieval_budget` | `src/intelligence/agent-core/memory_retriever.rs` |
| 27 | 28 | 27 | 53 | 5180 | `setupKeyboardShortcuts` | `src/interface/web/app.js` |
| 28 | 28 | 27 | 35 | 99 | `insert` | `src/intelligence/memory/src/hnsw.rs` |
| 29 | 27 | 25 | 148 | 200 | `parse_sse` | `src/engine/llm-core/src/anthropic.rs` |
| 30 | 27 | 25 | 128 | 2443 | `sendChatMessage` | `src/interface/web/app.js` |

Full-repo complexity note: when `docs/codex-twist-source` is included, copied/reference source dominates the full list. See raw log for that full-scope evidence.

## 6. Risk Keyword Statistics

Command evidence: Node read-only regex scan over text-like files, excluding `.git`, `node_modules`, `target`, `dist`, `build`, `.next`, and `coverage`.

| Keyword | All Matches | `src` Matches | `tests` Matches | `docs` Matches | Files With Match |
|---|---:|---:|---:|---:|---:|
| `TODO` | 764 | 29 | 6 | 214 | 270 |
| `FIXME` | 107 | 2 | 0 | 9 | 57 |
| `DEBT` | 7767 | 146 | 28 | 454 | 1193 |
| `innerHTML` | 828 | 109 | 53 | 452 | 83 |
| `unsafe` | 1329 | 52 | 1 | 709 | 344 |
| `unwrap(` | 4116 | 930 | 57 | 2780 | 448 |
| `panic!(` | 1566 | 24 | 3 | 1532 | 336 |
| `expect(` | 12507 | 397 | 100 | 11818 | 797 |

Top source-facing notes:

- `src/interface/web/app.js` contains `innerHTML` matches and remains a primary frontend rendering debt hotspot.
- `src/intelligence/memory/src/dream.rs` and `src/intelligence/memory/src/graph.rs` are prominent `unwrap(` source hotspots.
- Many `unsafe`, `panic!(`, and `expect(` matches are in archived docs or copied reference source, so the table is a scan map, not a fix list.

## 7. Frontend Module Smoke Coverage Matrix

Command evidence: Node read-only scan of `src/interface/web/modules/*.js` and `tests/frontend/*.js`.

| Module | Named / Direct Smoke | Mentioned In Tests | Evidence |
|---|---:|---:|---|
| `audit-log.js` | YES | YES | `tests/frontend/day23_audit_log_smoke.js` |
| `command-palette-catalog.js` | YES | YES | `tests/frontend/day22_command_palette_catalog_smoke.js` |
| `inspector.js` | YES | YES | `tests/frontend/day18_inspector_smoke.js`, `tests/frontend/day25_inspector_safety_smoke.js` |
| `resource-dashboard.js` | YES | YES | `tests/frontend/day24_resource_dashboard_smoke.js` |
| `security-dom.js` | NO | YES | `tests/frontend/day13_workspace_modules_smoke.js`, `tests/frontend/day14_sessions_thinking_modules_smoke.js`, `tests/frontend/day17_thinking_ui_v2_security_smoke.js`, `tests/frontend/thinking_stream_parser.test.js` |
| `sessions.js` | YES | YES | `tests/frontend/day14_sessions_thinking_modules_smoke.js`, `tests/frontend/day22_command_palette_catalog_smoke.js` |
| `settings-panel.js` | NO | YES | `tests/frontend/day19_settings_smoke.js` |
| `slash-command-catalog.js` | NO | YES | `tests/frontend/day21_slash_palette_app_integration_smoke.js` |
| `slash-palette.js` | YES | YES | `tests/frontend/day16_slash_palette_smoke.js`, `tests/frontend/day21_slash_palette_app_integration_smoke.js` |
| `tauri-bridge.js` | NO | YES | `tests/frontend/day13_workspace_modules_smoke.js`, `tests/frontend/day14_sessions_thinking_modules_smoke.js`, `tests/frontend/day20_tauri_channel_envelope_regression.test.js`, `tests/frontend/day21_event_bridge.test.js` |
| `thinking-ui.js` | YES | YES | `tests/frontend/day17_thinking_ui_v2_security_smoke.js`, `tests/frontend/agent_thinking_leak_smoke.js`, `tests/frontend/day14_sessions_thinking_modules_smoke.js`, `tests/frontend/thinking_stream_parser.test.js` |
| `workspace.js` | YES | YES | `tests/frontend/day13_workspace_modules_smoke.js` |

Coverage language is intentionally conservative: this matrix only says test files exist and reference the module. It does not claim runtime PASS for this V2 scan unless a test command is separately run and recorded.

## 8. Test Coverage Gaps

Command evidence: `package.json` and `rg` scan show `npm test` uses Jest, `npm run test:security-gate` exists, but no coverage command such as `nyc`, `c8`, or a coverage script is configured.

Observed gaps:

| Gap | Evidence | Status |
|---|---|---|
| No repository-level coverage percentage | `package.json` has `test` and `test:security-gate`, no coverage script | `BLOCKED-BY-NO-LOCAL-COVERAGE-TOOL` |
| `security-dom.js` lacks exact named smoke file | Matrix above | Covered indirectly, direct smoke candidate |
| `settings-panel.js` smoke name does not match module name | `tests/frontend/day19_settings_smoke.js` | Existing smoke, naming gap only |
| `slash-command-catalog.js` lacks exact named smoke file | `tests/frontend/day21_slash_palette_app_integration_smoke.js` | Covered through slash app integration |
| `tauri-bridge.js` has envelope tests, but no real WebView claim in this scan | `tests/frontend/day20_tauri_channel_envelope_regression.test.js`, `tests/frontend/day21_event_bridge.test.js` | Keep WebView claims separate |
| Large frontend controller still has broad behavior surface | `src/interface/web/app.js` 5617 lines, complexity hotspots above | Needs future slice-specific tests |
| Desktop main remains large | `src/interface/desktop/src/main.rs` 5094 lines | Future read-only command map recommended |

## 9. Large File And Repository Volume Anomalies

Command evidence:

- `git count-objects -vH`
- current filesystem size scan with `Get-ChildItem -Recurse`
- top directory size scan
- git history object scan with `git rev-list --objects --all | git cat-file --batch-check`

Findings:

| Area | Evidence | Note |
|---|---:|---|
| Current tracked file total | 733 files, 198,164,815 bytes | From `git ls-files` size measurement |
| Git pack size | 3.81 GiB | Repository history is much larger than current tracked files |
| Current root `target` directory | 47,911.8 MB | Build output dominates current working tree |
| `src/interface/desktop/target` | 4,366.5 MB | Nested build output dominates `src/interface` size |
| Current `models` directory | 166.4 MB | Includes ONNX model and tarball tracked by git |
| Largest current tracked file | 90,387,630 bytes | `models/fast-all-MiniLM-L6-v2/model.onnx` |
| Largest historical object found | 104,865,616 bytes | `src/foundation/tests/fixtures/100mb.bin` in git history |
| Historical build artifacts | Present | `target/debug/...` and `crates/hajimi-hnsw/target/...` appear in git history object scan |

No cleanup was performed.

## 10. Next Knife Candidates Only

These are candidates, not implemented in this scan:

1. `STONE-AUDIT-V2A-DOM-CONTRACT-SMOKE`: read-only DOM ID contract scan for `index.html`, `app.js`, modules, and smoke tests.
2. `STONE-AUDIT-V2B-APPJS-HOTSPOT-SAMPLING`: sample only `setupProviderSettings`, `handleChatCommand`, `streamChat`, and `sendChatMessage`; do not extract.
3. `STONE-AUDIT-V2C-REPO-VOLUME-PLAN`: plan-only cleanup strategy for build artifacts and historical large objects; do not delete in the first pass.
4. `STONE-AUDIT-V2D-SOURCE-UNWRAP-PANIC-MAP`: read-only Rust source map for `unwrap(`, `expect(`, and `panic!(` with test/source separation.
5. `STONE-AUDIT-V2E-FRONTEND-SMOKE-NAMING-GAP`: add or rename smoke coverage only after explicit approval; current scan does not implement it.

## 11. Validation

Validation commands were run after report creation and recorded in the raw log.

| Command | Result |
|---|---|
| `git status --short` | PASS, old dirty files remain plus the two V2 report files |
| `git rev-parse HEAD` | PASS, unchanged at `2b50333a5b76d559545d40c7cb7d91890f17fed8` |
| `git diff --check` | PASS with pre-existing CRLF warnings on unrelated old dirty docs |
| `git diff --cached --check` | PASS, empty cached diff |
