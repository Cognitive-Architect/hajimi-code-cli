# DEBT-AGENT-LLM-NATIVE-BUDGET-MELTDOWN

> **ID**: `DEBT-AGENT-LLM-NATIVE-BUDGET-MELTDOWN`
> **Priority**: **P1**
> **Date**: 2026-06-01
> **Last Updated**: 2026-06-01
> **Status**: `FIXED-CANDIDATE / REAL-WEBVIEW-SMOKE-PASSED / NEEDS-RECHECK`
> **Scope**: Desktop `/agent`, LLM-Native multi-turn loop, repeated read-only tool calls, token budget meltdown
> **Fix Commits**: `50681745`, `8c776e7e`, `8dcb52ea`, `54e47c21`
> **Validation Commit**: `15a64acf`

---

## 1. Current Conclusion

This debt is no longer an open implementation blocker.

The Day 1-3 code pass fixed the observed LLM-Native loop meltdown path, and the Day 4 validation pass recorded both automated regression evidence and one real desktop WebView smoke pass.

Current status is still **not marked fully closed** because the project keeps real GUI validation conservative. The debt should remain visible until a later QA pass confirms the same smoke on a clean release/install path and records durable evidence.

Plain-language read: the kitchen fire was put out, and one cooked meal came out correctly. We are not deleting the fire report yet; we are keeping it on the clipboard until another person checks the stove once more.

---

## 2. Original Failure

The failure was first observed with this real desktop prompt:

```text
/agent 查看当前工作区根目录下有哪些文件和文件夹，只列出前 10 个名字，不要读取文件内容，不要修改任何文件。
```

Observed UI result before the fix:

```text
智能体在执行动作时失败:
"Handoff: Budget Exceeded. Token/Iteration meltdown threshold reached.
Preserving execution state. Total tokens: 8611" (ActFailed)
```

Observed Agent Trace before the fix:

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

Technical read: LLM-Native successfully reached model/tool execution, but the multi-turn loop repeated the same directory-listing work and crossed the 8192 token meltdown threshold before producing a successful final answer.

Plain-language read: the agent could enter the kitchen, but it kept checking the same shelf again and again. The shopping receipt became too thick, so the safety meter tripped before dinner was served.

---

## 3. Root Cause

The failure was a backend LLM-Native convergence issue, not a provider-schema or frontend-rendering issue.

Main causes:

1. **Final answer checked too late**
   - In `src/intelligence/agent-core/llm_native/turn.rs`, token meltdown checking happened before final assistant content was inspected.
   - A model step that already returned usable final content could still be discarded if accumulated token usage crossed the threshold first.

2. **Current user intent was re-appended too often**
   - In `src/intelligence/agent-core/llm_native/driver.rs`, each model step could see the same raw user request appended again after prior tool results.
   - This made the model more likely to repeat the same read-only tool call instead of answering from the existing result.

3. **Duplicate read-only tool calls had no local guard**
   - The same read-only tool with the same arguments could be approved and executed repeatedly inside one LLM-Native turn.
   - For `list_directory`, that multiplied tool-result history and token usage.

4. **Tool results were too easy to re-feed in large form**
   - Even a simple directory listing can become expensive when repeated across model turns.

Plain-language read: the assistant was handed the same grocery list more than once, bought the same vegetables more than once, and then the receipt got copied back into the notebook until the notebook overflowed.

---

## 4. Implemented Fix

### 4.1 Day 1 Baseline Coverage

Commit:

```text
50681745 test(agent): cover llm-native budget meltdown baseline
```

Purpose:

- Add regression coverage around the meltdown pattern before changing behavior.
- Make the failure shape testable in `llm_native`.

### 4.2 Day 2 Final Answer And Intent Handling

Commit:

```text
8c776e7e fix(agent): preserve llm-native final answers and dedupe current intent
```

Implemented behavior:

- `turn.rs` now checks final assistant content before hard token meltdown failure.
- A non-empty assistant answer with no tool calls is preserved as success even if the final step pushes token accounting past the warning line.
- `driver.rs` now builds chat messages through `build_chat_messages_for_step`.
- The current `intent.text` is added exactly once when not already present.
- Existing unrelated user history is preserved without losing the current task.

### 4.3 Day 2 Empty Final Message Guard

Commit:

```text
8dcb52ea fix(agent): reject empty or whitespace assistant message without tool calls
```

Implemented behavior:

- Empty or whitespace-only assistant content is not treated as a successful final answer.
- The loop returns a clear handoff message instead of silently marking success.

### 4.4 Day 3 Duplicate Read-Only Tool Suppression

Commit:

```text
54e47c21 feat(intelligence/agent-core): duplicate read-only tool suppression in llm-native turn
```

Implemented behavior:

- `turn.rs` tracks canonical `(tool_name, arguments)` keys for read-only tools during a single turn.
- If the same read-only tool call repeats with the same arguments, it is suppressed before governance approval and before physical tool execution.
- The model receives a compact tool result telling it to use the previous result.
- Failed read-only tool calls are not recorded into the suppression set, so transient failures can still retry.
- Multibyte truncation uses `chars().take(200).collect()` to avoid slicing Chinese or emoji text in the middle of a character.

Plain-language read: if the assistant already bought tomatoes from the same market with the same list, the cashier says, "you already have this receipt, use it," instead of charging again. If the first purchase failed because the card reader blinked, a retry is still allowed.

---

## 5. Validation Evidence

### 5.1 Automated Regression Checks

Day 4 validation commit:

```text
15a64acf docs(agent): record llm-native budget meltdown validation
```

Recorded automated checks:

```text
cargo fmt -- --check
cargo test -p intelligence-agent-core --lib llm_native
node tests/frontend/agent_result_rendering_smoke.js
node tests/frontend/agent_thinking_leak_smoke.js
node --check src/interface/web/app.js
cargo check -p hajimi-desktop
```

Important observed result:

```text
cargo test -p intelligence-agent-core --lib llm_native
running 49 tests
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 305 filtered out
```

The targeted test suite covers:

- Final answer preserved when token usage crosses the warning threshold on the final step.
- Token meltdown still fails when the model asks for more tools instead of answering.
- Current user intent seeded exactly once.
- Unrelated older user history does not suppress the current task.
- Empty and whitespace-only assistant final messages are rejected.
- Duplicate read-only tool calls with identical args are suppressed.
- Same read-only tool with different args is not suppressed.
- Duplicate write tools are not suppressed by the read-only guard.
- Existing repeated write safety remains active.
- Failed read-only calls are not suppressed on later retry.
- Multibyte result truncation does not panic.

### 5.2 Real WebView Smoke

Day 4 recorded a real desktop smoke using:

```text
/agent 查看当前工作区根目录下有哪些文件和文件夹，只列出前 10 个名字，不要读取文件内容，不要修改任何文件。
```

Expected and observed high-level outcome:

```text
Task completes in 1-2 model steps.
Directory listing tool executes once for the useful result path.
No BudgetExceeded.
Final answer lists the first 10 names.
No raw <thinking> tags in the final answer body.
```

Recorded Agent Trace pattern:

```text
ModelStepStarted: LLM step 1 started
ToolExecutionSuccess: Tool 'list_dir' executed successfully
ModelStepStarted: LLM step 2 started
LLM-Native driver completed successfully after 2 iteration(s)
```

Recorded release artifact:

```text
EXE: F:\hajimi-code-cli\target\release\hajimi-desktop.exe
LastWriteTime: 2026/06/01 17:34:00
SHA256: 9E9F49CA15FB414E9342460AD9594F2AD576995FC49D0C00B004C5A171DAB9A3
```

---

## 6. Current Source Files

Primary implementation files:

- `src/intelligence/agent-core/llm_native/turn.rs`
  - Final answer detection before token meltdown handoff.
  - Empty assistant response guard.
  - Duplicate read-only tool suppression.
  - Compact repeated result handoff.
  - Multibyte-safe truncation.

- `src/intelligence/agent-core/llm_native/driver.rs`
  - `build_chat_messages_for_step`.
  - Current intent deduplication.
  - Unrelated history preservation.

No Day 4 production-code changes were made. Day 4 only recorded validation and updated debt state.

---

## 7. Remaining Risk

This debt is kept as `FIXED-CANDIDATE` rather than fully closed for these reasons:

1. The real WebView smoke was recorded, but a later clean-install or fresh-workspace QA pass should repeat it.
2. The guard only suppresses duplicate read-only calls with identical canonical arguments. If the model loops through different tools or slightly different arguments, a separate convergence guard may still be needed.
3. The model-visible tool list still contains roughly 40 tools. That cost is acceptable for the current smoke, but broader tool-manifest compaction may be useful later.
4. `list_directory` output compaction was not deeply redesigned. The fix avoided the observed repetition first, which is the correct lower-risk move.

Plain-language read: we stopped the repeated same receipt problem. If the assistant starts visiting ten different stores instead, that is a different problem and should get a new debt note.

---

## 8. Recheck Recipe

Use this exact prompt in the release desktop app:

```text
/agent 查看当前工作区根目录下有哪些文件和文件夹，只列出前 10 个名字，不要读取文件内容，不要修改任何文件。
```

Pass criteria:

- UI reaches task complete, not `ActFailed`.
- Final answer contains a concise list of root entries.
- No raw `<thinking>` or `</thinking>` appears in the answer body.
- Agent Trace shows no `Budget Exceeded`.
- Same read-only directory call is not physically executed repeatedly with identical arguments.
- Typical flow finishes in 1-2 model steps.

Fail criteria:

- `Budget Exceeded` returns for the same simple read-only prompt.
- The model repeatedly executes the same read-only tool with identical args.
- Final answer is empty or only says `Success`.
- Raw thinking tags leak into the visible final answer.

If fail criteria reproduce, reopen this debt and attach the trace event sequence.

---

## 9. Closure Rule

This debt can be moved from `FIXED-CANDIDATE` to `CLOSED` only after a follow-up QA pass records:

```text
1. release or installer path used,
2. exact prompt used,
3. final answer screenshot or transcript,
4. Agent Trace showing no BudgetExceeded,
5. evidence that duplicate read-only calls are suppressed or absent,
6. confirmation that no raw thinking tags are visible.
```

Do not close this debt from unit tests alone.
