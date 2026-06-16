# DEBT-AGENT-LLM-NATIVE-THINKING-LEAK

> **ID**: `DEBT-AGENT-LLM-NATIVE-THINKING-LEAK`
> **Priority**: **P1**
> **Date**: 2026-06-01
> **Status**: `OPEN / DIAGNOSED`
> **Scope**: Desktop `/agent`, LLM-Native final answer rendering, Thinking UI separation

---

## 1. Problem Summary

After `DEBT-AGENT-LLM-NATIVE-SUCCESS-RESULT-DROPPED` was code-level fixed and rebuilt, real desktop smoke testing shows `/agent` can now render the model's final content instead of only showing `Success`.

Observed prompt:

```text
/agent 查看当前工作区根目录下有哪些文件和文件夹，只列出前 10 个名字，不要读取文件内容，不要修改任何文件。
```

Observed improvement:

```text
The chat card now displays the directory-listing content.
```

Observed new problem:

```text
<thinking>...</thinking> content appears inline inside the final answer body.
```

Example from the screenshot:

```text
<thinking>用户要求只列出前10个名字。让我从结果中提取前10个:

1. .cargo-lock
2. .codex
3. .fingerprint
...

这些就是前10个。</thinking>当前工作区根目录下的前 10 个文件和文件夹:
```

Technical read: the LLM-Native path now preserves `TurnOutcome.final_message`, but `/agent` result rendering does not split `<thinking>...</thinking>` out of the final message before rendering it as user-visible answer content.

Plain-language read: the agent finally brings the grocery bag to the door, but it also leaves the shopper's private shopping notes taped on the bag. The answer is visible now, but the "thinking note" should be placed in the Thinking UI panel, not mixed into the answer.

---

## 2. Evidence Collected

### 2.1 Real desktop UI evidence

The 2026-06-01 desktop screenshot shows:

- `/agent` task completed successfully.
- The result card contains actual directory names.
- The same card also contains raw `<thinking>` and `</thinking>` tags.
- Agent Trace shows model and tool execution finished successfully.

This proves the current failure is no longer "result dropped." The failure is "result is not cleaned/split before rendering."

### 2.2 Code-path evidence

Relevant code paths:

- `src/engine/llm-core/src/openai.rs`
  - DeepSeek/OpenAI-compatible `reasoning_content`, `reasoning`, or `thinking_content` is converted into output chunks wrapped by `<thinking>...</thinking>`.
- `src/intelligence/agent-core/llm_native/driver.rs`
  - `StreamChunk::Output(text)` is appended directly into `assistant_content`.
  - This means final LLM-Native content may contain both reasoning tags and visible answer text.
- `src/intelligence/agent-core/agent_loop.rs`
  - LLM-Native success now returns `LoopOutcome::SuccessWithMessage(final_message)` when the final message is non-empty.
- `src/interface/desktop/src/main.rs`
  - `agent_outcome_output` sends `SuccessWithMessage(message)` to the frontend as raw result output.
- `src/interface/web/app.js`
  - `handleAgentEvent(... event.type === 'result' ...)` renders non-empty result output under the success header.
  - This branch does not call `parseThinkingStream` or `parseStreamEvent`.
- `src/interface/web/modules/thinking-ui.js`
  - Existing parser already supports extracting `<thinking>...</thinking>` from normal streaming chat responses.

Plain-language read: normal chat already has a sieve for separating "thinking" from "answer." The `/agent` result path takes a newer shortcut and skips that sieve.

---

## 3. User-Visible Impact

- The final answer looks broken because raw `<thinking>` tags appear in the chat card.
- Users see model reasoning content mixed with final answer content.
- The Thinking UI panel stays misleadingly empty or separate from the actual reasoning text.
- It creates confusion: Agent Trace says success, result content exists, but the result is visually polluted.
- It may create a policy/UX boundary risk because reasoning-like content is exposed as normal answer text.

---

## 4. Relationship To Existing Debts

This debt is related to but distinct from:

```text
DEBT-AGENT-LLM-NATIVE-SUCCESS-RESULT-DROPPED
```

That debt focused on:

- Preserving `TurnOutcome.final_message`.
- Carrying successful result content through `LoopOutcome::SuccessWithMessage`.
- Rendering non-empty result content in the chat card.

This new debt focuses on:

- Splitting thinking content out of successful `/agent` result output.
- Rendering thinking content inside the Thinking UI panel.
- Rendering only the user-facing final answer in the result body.

Do not reopen the old "result dropped" debt solely because `<thinking>` is visible. The current evidence shows result propagation improved and now needs content separation.

This debt is also related to:

```text
DEBT-THINKING-UI
```

The existing Thinking UI parser appears reusable. The missing integration is specifically the `/agent` result branch.

---

## 5. Working Hypothesis

Most likely implementation issue:

```text
DeepSeek reasoning_content
  -> openai.rs wraps it as <thinking>...</thinking>
  -> llm_native/driver.rs appends all Output chunks into assistant_content
  -> AgentLoop returns SuccessWithMessage(raw final_message)
  -> desktop sends raw output to frontend
  -> app.js /agent result branch renders raw output without thinking parsing
```

The bug is probably not in DeepSeek, not in tool execution, and not in the governance gate.

Plain-language read: the water is clean enough, but this one pipe skipped the filter. Install the same filter on this pipe before blaming the water source.

---

## 6. Proposed Fix Direction

Recommended minimal fix:

1. In `src/interface/web/app.js`, update the `/agent` `result` branch.
2. When `event.output` is non-empty and not a known enum/failure string, pass it through the existing Thinking UI parser.
3. If parsed thinking content exists:
   - call `updateTurnThinking(turn, { state: 'done', content: parsed.thinking })`.
4. Render only parsed response content below the success header.
5. If parsed response is empty but thinking exists:
   - show a clear fallback such as `模型仅返回了思考过程，未返回最终回答。`
6. Add a frontend smoke test proving raw `<thinking>` tags are not rendered in `/agent` final result content.

Candidate helper shape:

```text
const parsed = this.parseThinkingStream(outcome);
const visibleOutcome = (parsed.response || '').trim();
const thinking = (parsed.thinking || '').trim();
```

Alternative longer-term fix:

1. Split `SuccessWithMessage` into a structured payload:
   - `final_message`
   - `thinking_content`
2. Send explicit fields through `AgentUiEvent::Result`.
3. Let the frontend render each field directly.

Recommended sequencing:

```text
V1: Frontend-only parser reuse for /agent result branch.
V2: Structured backend event payload if future agent result metadata grows.
```

Plain-language read: first add the missing strainer in the frontend bowl. If later this soup gets more ingredients, redesign the kitchen ticket.

---

## 7. Candidate File Modification List

Likely files:

- `src/interface/web/app.js`
  - Reuse `parseThinkingStream` in `/agent` result handling.
  - Update Thinking UI panel state when agent final output contains thinking tags.
  - Render cleaned final answer content.
- `tests/frontend/agent_result_rendering_smoke.js`
  - Add coverage for result output containing `<thinking>plan</thinking>answer`.
  - Assert raw thinking tags are not part of final rendered output contract.
- `src/interface/web/modules/thinking-ui.js`
  - Only touch if current parser cannot correctly parse the exact `/agent` output shape.

Possible docs after fix:

- `docs/debt/DEBT-AGENT-LLM-NATIVE-THINKING-LEAK.md`
  - Add code-level fix receipt.
- `docs/debt/INDEX.md`
  - Move status from `OPEN / DIAGNOSED` to `CODE-LEVEL FIXED / PENDING-REAL-WEBVIEW-SMOKE`.

---

## 8. Test Cases

### 8.1 Frontend parser contract

Input:

```text
<thinking>先想一下</thinking>最终答案
```

Expected:

```text
thinking = 先想一下
response = 最终答案
```

### 8.2 `/agent` final result rendering

Input event:

```json
{
  "type": "result",
  "output": "<thinking>选择前10个</thinking>1. .cargo-lock\n2. .codex"
}
```

Expected UI contract:

```text
Thinking panel content contains: 选择前10个
Final result body contains: 1. .cargo-lock
Final result body does not contain: <thinking>
Final result body does not contain: </thinking>
```

### 8.3 Existing success strings remain stable

Inputs:

```text
Success
Aborted
BudgetExceeded
ActFailed("boom")
```

Expected:

```text
Existing friendly rendering remains unchanged.
```

### 8.4 Real WebView smoke

Prompt:

```text
/agent 查看当前工作区根目录下有哪些文件和文件夹，只列出前 10 个名字，不要读取文件内容，不要修改任何文件。
```

Expected:

```text
The chat card shows the success header and the directory list.
The chat card does not show raw <thinking> or </thinking> tags.
The Thinking UI panel shows or can reveal the extracted thinking content if the model returned it.
Agent Trace still shows successful list_directory execution.
```

---

## 9. Stop Conditions

Stop and record a separate debt if any of these happen:

1. The existing parser removes legitimate answer text together with thinking content.
2. The frontend receives no `<thinking>` tags, but the UI still shows stale thinking from another turn.
3. The backend emits malformed tags that cannot be parsed by the existing `thinking-ui.js` parser.
4. The fix requires changing the Tauri event contract beyond local result rendering.
5. Real WebView smoke shows duplicate or stale trace events tied to the same result card.

---

## 10. Current Conclusion

The current evidence supports this diagnosis:

```text
/agent LLM-Native execution and final result propagation now work.
The remaining defect is result-content separation: thinking tags are carried into the final answer body.
```

The next implementation pass should focus on `/agent` frontend result parsing and Thinking UI integration, not on DeepSeek provider schema, tool execution, or governance approval.
