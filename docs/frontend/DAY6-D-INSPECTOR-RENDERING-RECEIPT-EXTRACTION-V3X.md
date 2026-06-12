# DAY6-D Inspector Rendering / Receipt Guarded Extraction V3X

## Scope

- Task: `STONE-AUDIT-V3X Day6-D Inspector Rendering / Receipt Guarded Extraction`
- Source issue: `https://github.com/Cognitive-Architect/hajimi-code-cli/issues/8`
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `8672c81ec60aef5246436908b90f0db80aaf7025`
- Date: `2026-06-12`
- Result: `BLOCKED`

## Goal

Attempt to migrate the read-only Inspector rendering and Context Receipt paths after Day6-C:

- diff preview rendering
- Context Receipt rendering
- `get_latest_receipt` read-only invoke path

Forbidden areas remained out of scope:

- Checkpoint restore / export / compare / replay
- Provider save / delete / probe / validate / keyring / backup
- Shell execution
- Agent streaming
- generalized Tauri invoke service
- `main.rs`
- `tauri.conf.json`

## Attempted Local Change

A local Day6-D attempt moved diff / Context Receipt rendering into:

- `src/interface/web/views/inspector-view.js`
- `src/interface/web/controllers/inspector-controller.js`

The attempt preserved `day18` and `day25` behavior locally:

```text
node --check src/interface/web/modules/inspector.js
Result: PASS

node --check src/interface/web/views/inspector-view.js
Result: PASS

node --check src/interface/web/controllers/inspector-controller.js
Result: PASS

node tests/frontend/day18_inspector_smoke.js
Result: PASS
Output: day18 inspector module smoke: PASS

node tests/frontend/day25_inspector_safety_smoke.js
Result: PASS
Output: day25 inspector safety smoke: PASS (4 scenarios)
```

## Blocking Evidence

`npm run test:security-gate` failed with new non-legacy failures after the attempted migration:

```text
Security Audit Gate V1 summary
findings: 113
failures: 7
warnings: 106
allowlisted: 106

New failures introduced by the attempted Day6-D migration:
- src/interface/web/views/inspector-view.js:62
- src/interface/web/views/inspector-view.js:104
- src/interface/web/views/inspector-view.js:112
- src/interface/web/views/inspector-view.js:140
```

The new failures were caused by moving legacy `innerHTML` rendering from an allowlisted legacy location into a new view file without a safe DOM rewrite or allowlist update.

Per issue stop condition `SECURITY-001`, this cannot be submitted as a normal migration.

## Rollback / Stop Action

The local Day6-D production-code attempt was rolled back before commit.

Verification after rollback:

```text
git diff --name-only -- src/interface/web/modules/inspector.js src/interface/web/views/inspector-view.js src/interface/web/controllers/inspector-controller.js src/interface/desktop/src/main.rs src/interface/desktop/tauri.conf.json src/engine/tool-system/src/shell.rs
Result: <no output>

npm run test:security-gate
Result: KNOWN FAIL
Summary: findings 109; failures 3; warnings 106; allowlisted 106
Allowed legacy failures:
- src/interface/web/views/command-palette-view.js:36
- src/interface/web/views/session-list-view.js:22
- src/interface/web/views/session-list-view.js:26
```

## Final Day6-D State

- Day6-D production code committed: NO
- Day6-D production code left in worktree: NO
- Day6-D docs-only BLOCKED receipt committed: pending at write time
- New security-gate failure committed: NO
- Forbidden production diff count after rollback: `0`
- Old dirty files staged at doc write time: NO

## Decision

- DECISION-001: Do not continue Day6-D migration by moving `innerHTML` rendering into `inspector-view.js` without a safe DOM rewrite.
- DECISION-002: Do not update the security allowlist as a shortcut for this batch.
- DECISION-003: Stop before Day6-E because the serial work order requires each batch to complete or be explicitly blocked before continuing.

## Debt

- `DEBT-SCOPE-DAY6-D-INSPECTOR-RENDERING-BLOCKED`: Inspector diff / Context Receipt rendering requires either a true safe DOM rewrite or a separate approved allowlist/debt process before migration.
- `DEBT-TEST-DAY6-D`: Existing `day25` proves escaping behavior but does not satisfy the security-gate location policy once rendering moves to a new view file.

## Next Recommended Step

Open a separate Day6-D follow-up plan focused on safe DOM rendering for Inspector diff / Context Receipt before attempting Provider Day6-E.
