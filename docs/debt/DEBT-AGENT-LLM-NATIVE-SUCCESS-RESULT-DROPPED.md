# DEBT-AGENT-LLM-NATIVE-SUCCESS-RESULT-DROPPED

> **ID**: `DEBT-AGENT-LLM-NATIVE-SUCCESS-RESULT-DROPPED`
> **Priority**: **P1**
> **Date**: 2026-05-31
> **Status**: `OPEN / DIAGNOSED / DO-NOT-FIX-IN-THIS-PASS`
> **Scope**: Desktop `/agent`, LLM-Native final answer propagation, Agent UI result rendering

---

## 1. Problem Summary

After the AD-015 approval/hang remediation, a real desktop smoke test shows that `/agent` no longer stays permanently `运行中` for a read-only directory listing task.

Observed prompt:

```text
/agent 查看当前工作区根目录下有哪些文件和文件夹，只列出前 10 个名字，不要读取文件内容，不要修改任何文件。
```

Observed UI result:

```text
智能体任务已成功完成！(Success)
```

But the actual directory listing or model final answer is not displayed in the chat card.

Technical read: the LLM-Native path can complete model step 1, execute `list_directory`, complete model step 2, and return `LoopOutcome::Success`, but the user-visible final content is reduced to the bare enum value `Success`.

Plain-language read: DeepSeek and the local tool did the work, but Hajimi only tells the user "done" instead of showing the actual answer. It is like ordering groceries, the shopper buys them, the delivery app says "delivered", but no grocery bag appears at the door.

---

## 2. Evidence Collected

### 2.1 Real desktop Agent Trace evidence

The 2026-05-31 desktop screenshot shows:

```text
COMPLETED
LLM-Native driver completed successfully after 2 iteration(s)

ModelStepStarted: LLM step 1 started with 40 model-visible tools and 0 history messages.
ModelStepCompleted: LLM step 1 completed.
ToolCallInitiated: Tool 'list_directory' requested. Initiating governance approval gate.
GovernanceWaiting: Tool 'list_directory' waiting for Auto approval with risk score 0.10.
GovernanceApproved: Tool 'list_directory' approval granted.
ToolExecutionStarted: Tool 'list_directory' execution started.
ToolExecutionSuccess: Tool 'list_directory' executed successfully.
ModelStepStarted: LLM step 2 started with 40 model-visible tools and 2 history messages.
ModelStepCompleted: LLM step 2 completed.
```

This proves the old "no tool call / no trace / stuck running" symptom is no longer the active failure for this prompt.

### 2.2 Provider-side evidence

The DeepSeek dashboard for `deepseek-v4-pro` shows activity on 2026-05-31:

```text
API request count: 12
Tokens: 20,942
Input cache hit: 8,320 tokens
Input cache miss: 11,577 tokens
Output: 1,045 tokens
```

This supports the diagnosis that the provider did receive and process requests. It does not prove the exact final content, but it strongly contradicts "DeepSeek did nothing."

### 2.3 Code-path evidence

Current relevant code paths:

- `src/intelligence/agent-core/agent_loop.rs`
  - On LLM-Native success, the loop emits a completed trace and returns only:

```rust
return Ok(LoopOutcome::Success);
```

- `src/interface/desktop/src/main.rs`
  - `run_agent_task` converts the loop outcome to:

```rust
let outcome_str = format!("{:?}", outcome);
AgentUiEvent::Result { output: outcome_str }
```

- `src/interface/web/app.js`
  - `handleAgentEvent` maps raw `Success` to:

```text
✅ 智能体任务已成功完成！(Success)
```

Likely loss point: `TurnOutcome.final_message` is not carried through `LoopOutcome`, `AgentUiEvent::Result`, and frontend rendering.

Plain-language read: the answer probably falls out of the basket between the backend worker and the chat UI. By the time the frontend gets the event, only the word `Success` remains.

---

## 3. User-Visible Impact

- The user cannot see the actual result of a successful `/agent` run.
- The UI creates a false sense of completion because it shows green success without answer content.
- A read-only task can succeed internally while still being useless to the user.
- Debugging becomes confusing because DeepSeek usage, Agent Trace, and the chat card appear to disagree.

---

## 4. Relationship To AD-015

This is related to, but separate from:

```text
DEBT-AGENT-LLM-NATIVE-APPROVAL-HANG
```

AD-015 targeted:

- DeepSeek schema 400 no longer blocking the run.
- LLM-Native tool/governance trace visibility.
- Read-only tools not requiring Critical approval.
- Approval and model stream waits not hanging forever.

This new debt targets:

- Successful LLM-Native final answer propagation.
- Desktop event payload shape.
- Frontend rendering of real result content.

Do not reopen AD-015 solely because the chat card only says `Success`. The current symptom is a result-display debt, not the original approval hang.

---

## 5. Working Hypothesis

Most likely implementation issue:

```text
TurnOutcome.final_message
  -> dropped when AgentLoop returns LoopOutcome::Success
  -> desktop sends AgentUiEvent::Result { output: "Success" }
  -> frontend renders friendly success text only
```

Secondary issue to investigate:

```text
Agent Trace entries appear duplicated in the inspector.
```

Possible causes:

- Multiple trace subscriptions remain active after repeated `/agent` runs.
- The same event is sent through both a direct trace channel and a status/result path.
- UI stores duplicate trace events without deduping by timestamp/details/iteration.

This secondary issue should not be mixed into the first fix unless it is proven to share the same event lifecycle root cause.

---

## 6. Proposed Fix Direction

Do not change provider logic, DeepSeek schema generation, or governance approval policy for this debt.

Recommended minimal fix:

1. Extend the successful agent result contract so success can carry content.
   - Candidate: `LoopOutcome::SuccessWithMessage(String)`.
   - Alternative: keep `LoopOutcome::Success` and add a separate result payload field.
2. In the LLM-Native success path, preserve `TurnOutcome.final_message`.
3. In `run_agent_task`, send the success message to the frontend instead of only `Success`.
4. In `app.js`, render the final message below the success header.
5. Add regression tests proving a successful LLM-Native final message survives to the UI-facing event.

Plain-language read: keep the "task succeeded" sticker, but also put the actual answer underneath it.

---

## 7. Candidate File Modification List

Likely files:

- `src/intelligence/agent-core/agent_loop.rs`
  - Preserve final message in the success outcome.
- `src/interface/desktop/src/main.rs`
  - Send richer `AgentUiEvent::Result` payload or serialize success-with-message.
- `src/interface/web/app.js`
  - Render success content instead of mapping all success to a fixed sentence.

Likely tests:

- Agent Core unit test:
  - LLM-Native success with `final_message = "..."` returns a success outcome carrying the message.
- Desktop command/unit test:
  - `run_agent_task` or an extracted formatter emits a result payload containing the message.
- Frontend Node/unit smoke:
  - `handleAgentEvent({ type: "result", ... })` renders success text plus final answer content.

---

## 8. Stop Conditions

Stop and record a separate debt if any of these happen:

1. `TurnOutcome.final_message` is empty even though the model produced output.
2. The final answer exists in Agent Core but disappears only after Tauri Channel serialization.
3. The frontend receives the final message but sanitization/markdown rendering strips it.
4. Trace duplication is caused by a broader subscription lifecycle leak.

---

## 9. Current Conclusion

The current evidence supports this diagnosis:

```text
Backend LLM/tool execution is successful.
User-visible final answer propagation is incomplete.
```

The next implementation pass should focus on result payload propagation and frontend rendering, not on DeepSeek provider behavior.
