# Technical Debt: Agent Checkpoint File-Level Diff & Snapshot Boundaries
ID: `DEBT-AGENT-CHECKPOINT-DIFF-UI`
Priority: **P1**
Date: 2026-05-27
Status: **EXPLORED / PARTIAL-UI-INTEGRATION**

---

## 1. Context & Architectural Challenge

Under the **Hajimi IDE**'s strict four-layer architecture (`Interface -> Intelligence -> Engine -> Foundation`), the **Agent Core**'s execution trace events (`TraceEvent`) and operation summaries are streamed asynchronously in real-time to the frontend UI.

While the desktop layer automatically persists a `CheckpointRecord` whenever a checkpoint-generating step (such as `Store`, `EditProposed`, `EditApplied`, or `EditRejected`) occurs in `subscribe_agent_trace`, these records only contain execution metadata, plan reflections, and step counts. They **do not contain raw file diff details or physical content snapshots** (i.e., `CheckpointRecord.files` is empty).

Consequently, direct line-by-line file-level comparisons inside the Checkpoint Diff Preview tab are unavailable for trace-driven checkpoints.

---

## 2. Technical Debt & Safety Concerns

To maintain absolute data honesty (Zero-Fake Redline) and respect the architectural boundaries, we recognize the following technical debts:

### A. Lack of Granular Snapshot Data in Traces
Trace events emitted from `agent-core` contain counts of files modified and commands run, but do not serialize actual file content modifications or full workspace snapshots. Performing recursive filesystem hashing or full workspace diffing during every background trace tick would degrade CPU performance and violate the lightweight communication bounds.
- *Remediation*: Keep checkpoint records generated from traces lightweight, but display an honest fallback message directing the user to the existing dedicated "Git" panel to see exact filesystem modifications against `HEAD`.

### B. Read-Only Comparison Bounds
Since `CheckpointRecord.files` is empty for these trace checkpoints, the standard compare and restore commands gracefully return structured errors rather than fabricating false comparison statistics.
- *Remediation*: The UI now links trace cards directly to their real persisted checkpoint IDs (`chk_trace_{iteration}_{step_type}_{timestamp}`) and exposes beautiful, high-fidelity glassmorphism badge elements with "Restore" and "Compare" controls. Clicking "Compare" displays an informative, honest design alert rather than showing an empty placeholder or loading loop.

---

## 3. Verification Plan

1. **Syntax & Style Check**: Run `node --check src/interface/web/app.js` to ensure the Javascript workspace compiles cleanly.
2. **Interactive Validation**:
   - Run an Agent task using the `/agent` command.
   - Inspect the **Agent Trace** tab inside the Right Inspector.
   - Verify that checkpoint-generating steps (such as `EditProposed` or `EditApplied`) display a premium glassmorphism badge showing `🔒 检查点已保存` along with the matching `chk_trace_...` checkpoint ID.
   - Click "Compare" -> Verify that a detailed, honest explanation modal is presented explaining the architectural boundaries and directing the user to the Git panel.
   - Click "Restore" -> Verify that a confirm dialogue appears, and subsequently displays a clean error indicating that no file-level restore data is stored (as expected).
