# DEBT-AGENT-LLM-NATIVE-BUDGET-MELTDOWN

> **ID**: `DEBT-AGENT-LLM-NATIVE-BUDGET-MELTDOWN`
> **Priority**: **P1**
> **Date**: 2026-06-01
> **Status**: `FIXED-CANDIDATE / NEEDS-RECHECK`
> **Scope**: Desktop `/agent`, LLM-Native multi-turn loop, repeated tool calls, token budget meltdown

---

## 1. Problem Summary

After the `/agent` final-message propagation and Thinking UI leak fixes were packaged, a real desktop smoke test shows a new failure mode:

```text
/agent 查看当前工作区根目录下有哪些文件和文件夹，只列出前 10 个名字，不要读取文件内容，不要修改任何文件。
```

Observed UI result:

```text
智能体在执行动作时失败:
"Handoff: Budget Exceeded. Token/Iteration meltdown threshold reached.
Preserving execution state. Total tokens: 8611" (ActFailed)
```

Observed Agent Trace:

```text
LLM-Native driver completed with failure:
Handoff: Budget Exceeded. Token/Iteration meltdown threshold reached.
Preserving execution state. Total tokens: 8611

ModelStepStarted: LLM step 3 started with 40 model-visible tools and 4 history messages.
ModelStepCompleted: LLM step 3 completed.
ToolExecutionSuccess: Tool 'list_directory' executed successfully.
ToolExecutionStarted: Tool 'list_directory' execution started.
GovernanceApproved: Tool 'list_directory' approval granted.
```

Technical read: LLM-Native successfully reaches model/tool execution, but the multi-turn loop repeats `list_directory` and crosses the 8192 token meltdown threshold before producing a successful final answer.

Plain-language read: the agent is no longer stuck at the door, and it can enter the kitchen. But it keeps checking the same ingredient shelf again and again. The shopping list grows too thick, so the safety meter trips before dinner is served.

---

## 2. Evidence Collected

### 2.1 Real WebView evidence

The 2026-06-01 desktop screenshot shows:

- The app is running the newly packaged release.
- `/agent` command starts successfully.
- Agent Trace reaches `LLM step 3`.
- `list_directory` is executed successfully more than once.
- Final result is `ActFailed` with `Budget Exceeded`.
- The recorded total token count is `8611`, above the current hard threshold of `8192`.

This proves the active failure is not:

- DeepSeek schema 400.
- Missing LLM call.
- Tool approval hang.
- Frontend Thinking UI leakage.
- Final result dropped as only `Success`.

The active failure is a backend LLM-Native loop/budget convergence issue.

### 2.2 Code-path evidence

Relevant code:

- `src/intelligence/agent-core/llm_native/turn.rs`
  - `history.push(next_msg.clone())` happens before token-budget checking.
  - `token_tracker.total_tokens() >= 8192` returns a failure immediately.
  - `match next_msg` that checks whether the assistant has final content happens after the meltdown check.

Risky ordering:

```text
model step returns
-> append assistant message to history
-> add token usage
-> if total >= 8192: return Budget Exceeded failure
-> only then inspect whether next_msg was final answer or tool call
```

This means a model step that already returned final content can still be discarded if the budget check fires first.

- `src/intelligence/agent-core/llm_native/driver.rs`
  - Every step maps all previous history to chat messages.
  - Every step appends the raw user intent again at the end.
  - Every step calls `stream_chat_with_tools(..., ToolChoiceMode::Auto)`.

Risky prompt shape:

```text
history so far
+ original user request again
+ all 40 model-visible tools again
+ tool_choice = auto
```

This can encourage repeated tool calls because the model sees the same user request after tool results.

- `src/engine/tool-system/src/directory.rs`
  - `list_directory` returns JSON for directory entries.
  - Even non-recursive output includes full paths, size, modified, and type fields.

Risky output shape:

```text
The user only asked for 10 names.
The tool may return richer JSON than needed.
The rich JSON is fed back into model history.
Repeated calls multiply token usage.
```

Plain-language read: each loop asks the assistant again with the full instruction book, then pastes the full receipt back into the chat. If it does that two or three times, even a small task can overflow the budget drawer.

---

## 3. User-Visible Impact

- A simple read-only `/agent` task fails even though tool execution itself succeeds.
- The UI shows an alarming `ActFailed` result for a task that should be easy.
- DeepSeek usage can increase without producing a user-visible answer.
- Users may mistake the issue for provider failure, but the trace shows the provider and tool are working.
- The test route becomes noisy because the agent can fail from budget meltdown before validating newer frontend fixes.

---

## 4. Relationship To Existing Debts

This debt is related to but distinct from:

```text
DEBT-AGENT-LLM-NATIVE-APPROVAL-HANG
DEBT-AGENT-LLM-NATIVE-SUCCESS-RESULT-DROPPED
DEBT-AGENT-LLM-NATIVE-THINKING-LEAK
```

Those debts targeted:

- Tool schema and provider 400 errors.
- Read-only tool approval hang.
- Successful result message propagation.
- Removing `<thinking>...</thinking>` from the final answer body.

This debt targets:

- LLM-Native multi-turn convergence.
- Repeated same-tool same-argument calls.
- Token meltdown threshold handling.
- Prompt/history shape after tool results.

Do not reopen the old debts solely because this smoke failed. The current symptom is later in the pipeline: execution reaches repeated model/tool turns and then trips the budget guard.

---

## 5. Working Hypothesis

Most likely chain:

```text
User asks for first 10 root entries
-> model calls list_directory
-> tool returns directory JSON
-> next LLM step sees tool result plus original user request again
-> model calls list_directory again instead of answering
-> history grows with repeated tool results and repeated 40-tool manifests
-> step 3 pushes total usage to 8611
-> token meltdown guard returns ActFailed before a usable final answer is emitted
```

Secondary hypothesis:

```text
The model may have returned final answer content in step 3,
but the current token check executes before `match next_msg`,
so final content could be discarded when total tokens exceed 8192.
```

This needs code-level instrumentation or a targeted test to confirm.

Plain-language read: the agent may already have the answer in hand, but the budget alarm rings before the answer is allowed onto the screen.

---

## 6. Proposed Fix Direction

Recommended fix order:

1. **Reorder final-answer handling before hard budget failure**
   - Inspect `next_msg` first.
   - If it is an assistant message with no tool calls and non-empty content, preserve it as a success, even if the accumulated token counter crossed the warning line on that final step.
   - Keep budget failure for cases that still request tools or have no final answer.

2. **Avoid re-appending raw user intent every step after tool results**
   - The original request should appear once as the user message.
   - Tool follow-up steps should rely on existing history and tool result messages.
   - If a reminder is needed, use a compact system/developer hint rather than duplicating the full user request.

3. **Add repeated tool-call guard**
   - Track `(tool_name, arguments)` during a single turn.
   - If the same read-only tool with the same arguments is requested again, do not execute it again.
   - Feed the model a compact message such as:

```text
Tool result already exists for this request. Use the previous result to answer.
```

4. **Compact `list_directory` result for model-facing history**
   - For non-recursive directory listing, consider returning only the fields needed by the model.
   - At minimum, cap large tool outputs before placing them in LLM history.

5. **Make token threshold configurable or context-aware**
   - The hard `8192` threshold is useful as a safety guard.
   - It may be too low for a provider/model path that repeatedly sends 40 tools.
   - Do not simply raise it as the first fix; reduce repeated work first.

Plain-language read: first let a finished answer through the door, then stop asking the same worker to buy the same vegetables twice, then make the grocery receipt shorter.

---

## 7. Candidate File Modification List

Likely files:

- `src/intelligence/agent-core/llm_native/turn.rs`
  - Reorder final-content detection relative to token meltdown checks.
  - Add repeated tool-call guard for same `(tool_name, arguments)`.
  - Add trace event for duplicate tool-call suppression.

- `src/intelligence/agent-core/llm_native/driver.rs`
  - Stop appending `intent.text` as a new user message on every step, or gate it to the first step only.
  - Consider adding a compact follow-up instruction after tool results.

- `src/engine/tool-system/src/directory.rs`
  - Optional later optimization: produce or expose a compact directory-list output for model consumption.

Likely tests:

- `cargo test -p intelligence-agent-core llm_native --lib`
  - Existing suite must remain green.

- New unit test:
  - Final assistant content is preserved even if the final step pushes token usage above 8192.

- New unit test:
  - Repeated `list_directory` with identical args in the same turn is not executed repeatedly.

- New unit test:
  - Tool-result follow-up does not append raw user intent again on every step.

- Real WebView smoke:

```text
/agent 查看当前工作区根目录下有哪些文件和文件夹，只列出前 10 个名字，不要读取文件内容，不要修改任何文件。
```

Expected:

```text
Task completes in 1-2 model steps.
`list_directory` executes at most once for identical args.
No `Budget Exceeded`.
Final answer contains the first 10 names.
No raw `<thinking>` tags in final answer body.
```

---

## 8. Stop Conditions

Stop and record a follow-up debt if any of these happen:

1. The model repeatedly calls different tools, not the same tool, and the guard does not apply.
2. The final answer is empty after duplicate tool-call suppression.
3. Removing repeated raw user intent causes the model to forget the original task.
4. Token usage still crosses 8192 in one step before any tool result exists.
5. Raising the token threshold is required before tool repetition and prompt duplication are fixed.

---

## 9. Current Conclusion

The current evidence supports this diagnosis:

```text
/agent can now reach LLM/tool execution and render failures honestly.
The active blocker is LLM-Native loop convergence and token budget handling.
```

The next implementation pass should focus on:

```text
1. preserve final content before budget meltdown failure,
2. avoid repeated raw user-intent injection,
3. suppress duplicate read-only tool calls,
4. compact model-facing tool results.
```

This is a backend agent-loop debt, not a frontend Thinking UI debt.
