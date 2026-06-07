# STONE-AUDIT-V3X-CONTROLLED-DEMOLITION

## Day 0 Baseline

Date: 2026-06-07

Purpose: create the pre-demolition rollback point and record the exact baseline before any production refactor.

Scope: backup branch, baseline tag, current HEAD, dirty worktree inventory, production file line counts, and safety boundaries.

## Branch And Tag

| Item | Value |
| --- | --- |
| Source branch | `origin/feature/toolfix-deepseek-schema` |
| Local branch | `stone-audit-v3x-controlled-demolition` |
| Tag | `stone-v3x-before-demolition` |
| Current HEAD | `d2a5e26299e9c16585d33ce4b407c60287d22c11` |
| Tag target | `d2a5e26299e9c16585d33ce4b407c60287d22c11` |

## Line Count Baseline

| File | Lines |
| --- | ---: |
| `src/interface/web/app.js` | 5084 |
| `src/interface/web/style.css` | 3961 |
| `src/interface/desktop/src/main.rs` | 4654 |

These counts are the Day 0 baseline for this branch. They differ from the strategic plan snapshot, so the Day 0 values above are the operative before-state for this execution branch.

## Git Status Baseline

Command:

```powershell
git status --short
```

Output:

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
?? "docs/roadmap/hajimi template.7z"
?? "docs/roadmap/hajimi template/"
?? src/interface/desktop/native-smoke.txt
```

Day 0 note: these are pre-existing dirty worktree entries carried into the new branch. They must not be staged as part of V3X Day 0.

## Staging Baseline

Command:

```powershell
git diff --cached --name-only
```

Result: no output.

Conclusion: old dirty files staged: NO.

## Production Safety Check

Command:

```powershell
git diff --name-only -- src/interface/web/app.js src/interface/web/style.css src/interface/desktop/src/main.rs src/interface/web/modules src/interface/desktop/tauri.conf.json src/engine/tool-system/src/shell.rs
```

Result: no output.

Conclusion: production changes: NO.

## Explicit Non-Actions

- Did not modify `src/interface/web/app.js`.
- Did not modify `src/interface/web/style.css`.
- Did not modify `src/interface/desktop/src/main.rs`.
- Did not modify `src/interface/web/modules/*`.
- Did not touch Provider / Keyring.
- Did not touch Shell execution.
- Did not touch Checkpoint restore / export / compare / replay.
- Did not touch CSP / withGlobalTauri.
- Did not touch Agent streaming.
- Did not execute history rewrite.
- Did not push.

## Day 0 Result

| Check | Result |
| --- | --- |
| branch exists | PASS |
| tag exists | PASS |
| baseline doc exists | PASS |
| production changes | NO |
| old dirty files staged | NO |

## Next Step

Wait for explicit approval before any push or Day 1 CSS split work.
