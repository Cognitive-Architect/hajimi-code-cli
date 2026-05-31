# DEBT-AGENT-LLM-NATIVE-APPROVAL-HANG

> **ID**: `DEBT-AGENT-LLM-NATIVE-APPROVAL-HANG`
> **Priority**: **P0**
> **Date**: 2026-05-31
> **Status**: `CODE-LEVEL FIXED + RELEASE PACKAGED / PENDING-REAL-WEBVIEW-SMOKE`
> **Scope**: Desktop `/agent`, LLM-Native tool execution, governance approval bridge, DeepSeek/OpenAI-compatible provider

---

## 1. Problem Summary

After the DeepSeek tool-schema 400 fix, `/agent` no longer fails with:

```text
Invalid schema for function 'web_search'
```

The new observed behavior is a runtime hang:

```text
/agent 查看当前项目根目录下有什么文件，只列出前 10 个，不要修改任何文件
```

The UI remains in `运行中`. The main assistant card only shows:

```text
智能体执行中...
Agent task started with goal: 查看当前项目根目录下有什么文件，只列出前 10 个，不要修改任何文件
```

The Agent Trace reaches:

```text
LLM-Native path activated for goal: ...
LLM-Native exported 40 model-visible tools
LLM-Native driver run_turn started
```

No final result, error, or timeout is surfaced to the user.

Technical read: `/agent` enters the LLM-Native execution path successfully, sends the tool-enabled turn to the model, and then stalls during tool-call execution or approval handling. The previous provider-side schema rejection is no longer the active failure.

Plain-language read: the old problem was "DeepSeek refused the toolbox label before starting." This new problem is different: the agent has entered the kitchen, picked a tool, and is now waiting at an internal approval door that never opens.

---

## 2. User-Visible Impact

- `/agent` appears to start correctly but never finishes.
- The send button stays disabled while the task is marked `运行中`.
- Agent Trace does not show the actual tool name, approval wait, timeout, or next actionable error.
- The user has no clear way to distinguish a slow model from a local dead wait.
- Closing the client is currently the practical escape hatch.

---

## 3. Evidence Collected

### 3.1 Real-machine UI evidence

Screenshot evidence from 2026-05-31 shows:

- Task status: `运行中`
- Model: `deepseek-v4-pro`
- Trace count: `Agent Trace (3)`
- Recent trace entries:

```text
LLM-Native path activated for goal: 查看当前项目根目录下有什么文件，只列出前 10 个，不要修改任何文件
LLM-Native exported 40 model-visible tools
LLM-Native driver run_turn started
```

There is no visible `ToolCallInitiated`, `GovernanceApproved`, `GovernanceRejected`, `ToolExecutionSuccess`, or `ToolExecutionFailed` event in the right inspector.

### 3.2 Local process check

After the user closed Hajimi, no stale `hajimi-desktop` process remained. This makes a ghost desktop process unlikely as the cause of the visible `运行中` state.

### 3.3 Audit log check

`C:\Users\22129\AppData\Roaming\hajimi\audit.jsonl` contains normal chat start/completed records, but no later `/agent` completion evidence for this hang. Normal chat therefore still works, while `/agent` is stuck in its own execution path.

### 3.4 DeepSeek direct API probe

Using the temporary DeepSeek key from `archive/05` without printing the key:

- `deepseek-v4-pro`
- 40 model-visible tools, matching the desktop trace count
- `stream=true`
- `tool_choice=auto`
- prompt equivalent to "list first 10 files, do not modify"

Observed direct-provider result:

```text
HTTP 200
saw tool_calls: true
saw [DONE]: true
elapsed: about 3.4 seconds
```

This means DeepSeek can accept the fixed tool schemas and can return a tool call for the same class of request. The current hang is therefore more likely inside local handling after the model turn begins, not at the provider schema boundary.

### 3.5 Equivalent direct non-streaming probe

The non-streaming direct probe returned:

```text
finish_reason: tool_calls
first_tool: list_directory
```

This strongly suggests the model chooses a directory-listing tool for this prompt.

---

## 4. Likely Root Cause

The current LLM-Native turn loop has two approval gates around tool execution.

### 4.1 Outer approval in `llm_native_turn`

`src/intelligence/agent-core/llm_native/turn.rs` classifies only a short shell-like allowlist as low-risk:

```rust
"git" | "cargo" | "npm" | "node" | "python3" | "ls" | "cat" | "echo" | ...
```

Tools such as `list_directory`, `read_file`, and `grep` are not included in this allowlist. The code therefore assigns:

```rust
risk_score = 0.95
approval_level = Critical
```

For a read-only directory listing, this is too strict and can push the system into a manual approval path.

### 4.2 UI bridge approval wait

`src/interface/desktop/src/main.rs` implements `UiBridgeGovernance::approve`. For `Required` or `Critical`, it:

1. creates a `request_id`
2. stores a `tokio::sync::oneshot` sender in `AppState.pending_approvals`
3. emits `approval_request` to the WebView
4. awaits the oneshot receiver

If the WebView listener misses the event, the modal does not render, or the user cannot click approve/reject, the backend await has no visible timeout.

Plain-language read: the backend asks the frontend, "Can the agent list this folder?" Then it waits for a yes/no slip. In the failed run, that slip never visibly arrived, so the backend kept waiting.

### 4.3 Inner approval in `DefaultToolExecutor`

`src/intelligence/agent-core/llm_native/driver.rs` also calls `governance.approve` inside `DefaultToolExecutor::execute_tool`, with:

```rust
level = ApprovalLevel::Required
```

So even if the outer gate is approved, the tool executor can trigger a second approval wait. This double-gate behavior increases the chance of silent `运行中` hangs.

---

## 5. Why This Is Not The Old DeepSeek 400

The prior fixed debt was:

```text
DEBT-AGENT-DEEPSEEK-TOOL-SCHEMA-WEB-SEARCH
```

That issue failed before the model could run because `web_search` had an invalid schema.

The current issue differs:

| Signal | Old schema issue | Current hang |
|---|---|---|
| Provider HTTP result | 400 Bad Request | Direct probe returns 200 |
| Error visible in UI | Yes | No |
| Trace reaches `run_turn started` | Yes, then provider error | Yes, then no completion |
| Tool schema rejection | Confirmed old cause | Not reproduced now |
| Most likely blocker | Tool schema serialization | Local governance/tool execution wait |

---

## 6. Fix Direction

### Option A - Classify read-only tools as auto-approvable

Add a read-only safe tool class for:

- `list_directory`
- `ls`
- `read_file`
- `grep`
- `find`
- `glob`
- possibly `git_status`, `git_diff`, `git_log`

These should not be treated the same as write/delete/shell/network mutation tools.

Plain-language read: looking in the fridge should not need the same permission as throwing food away or turning on the gas stove.

### Option B - Add approval timeout

Every approval wait in `UiBridgeGovernance::approve` should have a timeout. If the frontend does not answer within the configured window, the agent should return a visible failure:

```text
Approval timed out while waiting for user response for tool 'list_directory'
```

This prevents permanent `运行中`.

### Option C - Emit trace for tool-call and approval states

The right inspector should show:

```text
ToolCallInitiated: list_directory
GovernanceWaiting: request_id=...
GovernanceApproved / GovernanceRejected / GovernanceTimedOut
ToolExecutionStarted
ToolExecutionSuccess / ToolExecutionFailed
```

This makes the next failure self-explanatory.

### Option D - Remove or consolidate double approval

Avoid approval both in `llm_native_turn` and again inside `DefaultToolExecutor::execute_tool`, or pass a verified approval decision down into the executor.

Plain-language read: one security checkpoint is fine. Two checkpoints asking the same thing can turn a simple errand into a stuck queue.

---

## 7. Proposed Test Cases

### 7.1 Read-only tool approval classification

Add a unit test proving that read-only file/navigation tools are not promoted to `Critical`:

```text
list_directory -> Auto or Advisory
read_file      -> Auto or Advisory
grep           -> Auto or Advisory
write_file     -> Required/Critical
delete_file    -> Required/Critical
```

### 7.2 Approval timeout

Add an async test for `UiBridgeGovernance` or an extracted approval helper:

```text
Given a Required/Critical approval request
And no frontend response arrives
Then the await resolves with Rejected/Timeout after the configured deadline
And pending_approvals is cleaned up
```

### 7.3 Agent no-permanent-running regression

Add an integration-style test around `/agent` execution:

```text
Given model requests list_directory
And approval modal is not answered
Then run_agent_task eventually emits Error or Done
And the UI can leave running state
```

### 7.4 Tool execution trace visibility

Assert trace output includes the real tool phase:

```text
LLM-Native driver run_turn started
ToolCallInitiated: list_directory
GovernanceWaiting
...
```

### 7.5 DeepSeek provider smoke

Keep a manual/live smoke gate using a temporary key:

```text
deepseek-v4-pro + 40 tools + stream=true + tool_choice=auto
Expected: HTTP 200, tool_calls present, [DONE] present
```

Do not print the API key in logs or debt docs.

---

## 8. Stop Conditions For The Next Fix Pass

Stop and record a new debt instead of pushing through if any of these happen:

1. DeepSeek starts returning HTTP 400 again for tool schemas.
2. The model returns a tool name that is not present in `ToolRegistry`.
3. The approval modal appears but `resolve_agent_approval` fails.
4. The tool executes successfully but the second LLM turn hangs.
5. Trace still cannot show the exact blocked phase.

---

## 9. Current Conclusion

The DeepSeek schema layer appears fixed for this scenario. The new blocker is most likely the local LLM-Native governance/tool-execution bridge: read-only tools are treated as high-risk, approval waits have no visible timeout, and the right inspector does not expose the waiting state.

The next implementation should start with read-only tool risk classification, approval timeout, and trace visibility before changing provider logic again.

---

## 10. Code-Level Fix Receipt (2026-05-31)

Status: `CODE-LEVEL FIXED + RELEASE PACKAGED / PENDING-REAL-WEBVIEW-SMOKE`

### 10.1 Implemented changes

- Added read-only tool governance classification in `src/intelligence/agent-core/llm_native/turn.rs`.
  - `list_directory`, `list_dir`, `ls`, `read_file`, `grep`, `find`, `glob`, `git_status`, `git_diff`, `git_log`, LSP read-only lookup tools, and `view_image` are no longer promoted to `Critical`.
  - Mutating or unknown tools such as `write_file`, `edit_file`, `delete_file`, and unknown tool names remain `Critical`.
- Added LLM-Native turn trace forwarding.
  - Internal native events now emit model-visible progress into Agent Trace via `run_turn_with_trace`.
  - Expected visible events include `ToolCallInitiated`, `GovernanceWaiting`, `GovernanceApproved`, `ToolExecutionStarted`, and `ToolExecutionSuccess` / `ToolExecutionFailed`.
- Removed the duplicate governance wait inside `DefaultToolExecutor`.
  - The outer `llm_native_turn` approval gate is now the single approval point before tool execution.
  - Tool execution no longer asks `UiBridgeGovernance` a second time.
- Added desktop approval timeout in `src/interface/desktop/src/main.rs`.
  - `UiBridgeGovernance` now waits for the WebView response with a 30 second timeout.
  - On timeout, the pending approval request is removed from `pending_approvals`.
  - The returned reason names the affected tool and timeout instead of leaving `/agent` permanently `运行中`.
- Added LLM stream hang protection in `src/intelligence/agent-core/llm_native/driver.rs`.
  - `stream_chat_with_tools` startup is bounded by the configured LLM client timeout.
  - Waiting for each next stream chunk is also bounded by the configured LLM client timeout.
  - If the provider starts or stalls forever, `/agent` now receives a concrete `LLM stream start timed out` or `LLM stream stalled` error instead of waiting indefinitely.
- Added model-step trace visibility in `src/intelligence/agent-core/llm_native/turn.rs`.
  - Expected visible events now also include `ModelStepStarted`, `ModelStepCompleted`, and `ModelStepFailed`.
- Applied the existing OpenAI-compatible `timeout_ms` value to the actual HTTP request in `src/engine/llm-core/src/openai.rs`.

Plain-language read: read-only errands no longer need the same red-stamp approval as deleting or writing files. If a true approval is needed and the popup never answers, the system stops waiting and reports the problem instead of standing forever at the door.

Second plain-language read: if the model call itself gets stuck before any tool is chosen, Hajimi now has a kitchen timer. When the timer rings, it reports that the model stream got stuck instead of leaving the task spinning forever.

### 10.2 Verification performed

```text
cargo fmt -- --check
Result: PASS
```

```text
cargo test -p intelligence-agent-core llm_native --lib
Result: PASS, 34 passed
```

```text
cargo test -p engine-llm-core openai --lib
Result: PASS, 15 passed
```

```text
cargo test -p hajimi-desktop approval_wait_times_out_and_cleans_pending_request
Result: PASS, 1 passed
```

```text
cargo test -p intelligence-agent-core test_agent_loop_native --lib
Result: PASS, 1 passed
```

```text
cargo check -p hajimi-desktop
Result: PASS
```

### 10.3 New regression coverage

- `test_read_only_tools_are_not_promoted_to_critical`
- `test_mutating_or_unknown_tools_remain_critical`
- `test_llm_native_turn_emits_tool_governance_trace_events`
- `test_llm_native_turn_emits_model_step_failure_trace_event`
- `test_driver_executes_tool_with_single_governance_approval`
- `test_driver_stream_start_times_out_instead_of_hanging`
- `test_driver_stream_stall_times_out_instead_of_hanging`
- `approval_wait_times_out_and_cleans_pending_request`

### 10.4 Remaining validation

This debt should not be marked fully closed until a rebuilt Hajimi desktop package is tested in a real WebView window:

```text
/agent 查看当前项目根目录下有什么文件，只列出前 10 个，不要修改任何文件
```

Expected result:

- The task must not stay permanently `运行中`.
- Agent Trace should show model-step progress beyond `LLM-Native driver run_turn started`.
- If the provider stream stalls, the task should fail visibly with a timeout instead of staying `运行中`.
- If the provider returns a tool call, Agent Trace should show tool-call and governance progress.
- A read-only directory listing should complete without a manual Critical approval popup.
- If any approval popup is required for another tool, ignoring it should time out visibly instead of hanging forever.

---

## 11. Release Package Build Receipt (2026-05-31)

Status: `RELEASE PACKAGED / PENDING-REAL-WEBVIEW-SMOKE`

The fixed desktop app was rebuilt with the Tauri release pipeline on 2026-05-31.

```text
Command:
cd F:\hajimi-code-cli\src\interface\desktop
cargo tauri build

Result:
PASS
```

Generated artifacts:

| Artifact | LastWriteTime | Size | SHA256 |
|:---|:---|---:|:---|
| `F:\hajimi-code-cli\target\release\hajimi-desktop.exe` | `2026-05-31T16:34:21+08:00` | `23,646,720` bytes | `3B6907FDF444617D84F980A6BA02521C0E258EEB17D8A98E4AF345BFA8146EB4` |
| `F:\hajimi-code-cli\target\release\bundle\msi\Hajimi_0.1.0_x64_en-US.msi` | `2026-05-31T16:33:56+08:00` | `8,695,808` bytes | `CC5CBE8A0EE38717358E8FE60C14F5C0335AD179DA8B47BC4AE4EC899F26CE97` |
| `F:\hajimi-code-cli\target\release\bundle\nsis\Hajimi_0.1.0_x64-setup.exe` | `2026-05-31T16:34:19+08:00` | `6,127,998` bytes | `3CAA51687AAA2D20DA7E045762ACCE4748231C6F9DAAFAA327D01B93B9D47721` |

Build warnings observed:

- MSVC ignored `zstd-sys` option `-fvisibility=hidden`.
- Tauri bundler warned that `__TAURI_BUNDLE_TYPE` was not found in the binary metadata.

These warnings did not block artifact generation. They are not evidence that `/agent` is fixed or broken; they only mean the package was produced.

Plain-language read: the new app has been cooked and packed into installer files. We still need to open the app and actually try the `/agent` dish before declaring the meal served.
