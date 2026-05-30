# DEBT-AGENT-LLM-NATIVE-FALLBACK-STILL-ABORTS

> **ID**: `DEBT-AGENT-LLM-NATIVE-FALLBACK-STILL-ABORTS`  
> **Priority**: **P0**  
> **Date**: 2026-05-30  
> **Status**: `OPEN / INVESTIGATING`  
> **Scope**: Desktop `/agent` runtime path, LLM-Native fallback, legacy `read_file("Cargo.toml")` abort

---

## 1. Problem Summary

User real-machine screenshot shows `/agent` still aborting:

```text
Stop-Loss forced handoff: Handoff: success=false, severity=High,
issues=Local execution of read_file returned error:
Access: 系统找不到指定的文件。 (os error 2),
suggestions=Retry with modified parameters
```

Trace also shows:

```text
legacy act path for goal ...
Local execution path activated for task ...
Loop completed with outcome: Aborted
```

Technical read: the desktop Agent still reaches the legacy/offline `legacy_act` path, and that path still has a default `read_file` fallback against `Cargo.toml`.

Plain-language read: the new Agent route may be installed, but when it trips, the app falls back to an old emergency route. That old route tries to read a default file from the wrong place, like checking the pantry in the garage instead of the kitchen, then gives up.

---

## 2. Current Evidence Collected

### 2.1 Current working tree is dirty

`git status --short` shows existing unrelated work before this note was added:

```text
 D docs/roadmap/Hajimi Agent/debt/DEBT-DAY-07-CHECKPOINT-DIFF-UI.md
 D docs/roadmap/Hajimi Agent/plan/AGENT-UI-INTEGRATION-SAMPLING-NOTES.md
 D docs/roadmap/Hajimi LLM/plan/AGENT-LLM-NATIVE-DESIGN.md
 D docs/roadmap/Hajimi LLM/plan/LLM-NATIVE-AGENT-MIGRATION-ROADMAP.md
 D docs/roadmap/Hajimi RealAgent/evidence/day-01/snapshot.md
 D docs/roadmap/Hajimi RealAgent/evidence/day-02/snapshot.md
 D docs/roadmap/Hajimi RealAgent/evidence/day-03/snapshot.md
 D docs/roadmap/Hajimi RealAgent/evidence/day-04/snapshot.md
 M src/ARCHITECTURE.md
 M src/INDEX.md
 M src/engine/llm-core/src/openai.rs
 M src/intelligence/agent-core/agent_loop.rs
 M src/interface/desktop/src/main.rs
```

This matters because a running desktop app may not match the current source tree unless it was rebuilt after these edits.

### 2.2 Driver injection exists in current source, but is uncommitted

Current `src/interface/desktop/src/main.rs` contains:

- `DesktopAgentTurnDriver`
- `AppState.agent_llm_client`
- `run_agent_task` writes the selected provider's `LlmClient` into the shared slot
- setup injects `.with_native_driver(Some(desktop_driver))`

This means the old debt `DEBT-AGENT-LLM-NATIVE-DRIVER-INJECTION` is at least present in the working tree. It does not prove the running binary includes it.

### 2.3 AgentLoop still falls back to legacy on LLM-Native error

Current `src/intelligence/agent-core/agent_loop.rs` still does this:

```rust
Err(e) => {
    warn!(
        "LlmNativeDriver run_turn failed: {:?}. Falling back to legacy path.",
        e
    );
    self.emit_trace(
        LoopState::Acting,
        format!("⚠️ LLM-Native Turn Error: {}. Falling back to legacy path.", e),
        0,
    );
}
```

Technical read: any `run_turn` error from the real provider path can be masked by legacy fallback.

Plain-language read: if the new route says "I failed", the app does not stop and show that real failure clearly. It switches to the old route, and then the old route creates a second, louder failure.

### 2.4 Legacy fallback still has hard-coded `read_file("Cargo.toml")`

Current `legacy_act()` still contains:

```rust
(
    "read_file".to_string(),
    serde_json::json!({ "path": "Cargo.toml" }),
)
```

Current `bootstrap_first_tool_call()` has the same default fallback:

```rust
tool_name: "read_file".to_string(),
parameters: serde_json::json!({ "path": "Cargo.toml" }),
```

In a desktop app, current working directory may be `target/debug`, `target/release`, or another runtime directory. With workspace sandboxing enabled, this relative path can resolve outside the project root or to a non-existent file. That matches the screenshot error.

### 2.5 Local validation performed

```text
cargo check -p hajimi-desktop
```

Result:

```text
Finished `dev` profile ... target(s) in 11.25s
```

```text
cargo test -p engine-llm-core openai -- --nocapture
```

Result:

```text
11 passed; 0 failed
```

Technical read: compile and local OpenAI-compatible parser tests pass. They do not validate DeepSeek live tool-call behavior.

Plain-language read: the stove turns on, and the recipe test passes with fake ingredients. We still have not proved the real DeepSeek ingredient behaves the same in the live kitchen.

---

## 3. Likely Failure Chain

```text
/agent <goal>
  -> frontend invokes run_agent_task(providerId=deepseek or selected provider)
  -> run_agent_task creates OpenAI-compatible LlmClient
  -> AgentLoop enters LLM-Native path
  -> LlmNativeDriver.run_turn hits provider/runtime error
  -> AgentLoop catches Err and falls back to legacy path
  -> legacy_act chooses default read_file("Cargo.toml")
  -> runtime CWD/sandbox cannot find that file
  -> Reflect sees repeated failed local execution
  -> Stop-Loss handoff
  -> Aborted
```

This is a working hypothesis, not fully proven, because the live provider error body was not captured in this pass.

---

## 4. Additional Risk Found

Current uncommitted `src/interface/desktop/src/main.rs` includes a temporary-looking test:

```text
test_capture_agent_trace
```

It reads:

```text
C:\Users\22129\AppData\Roaming\hajimi\providers.json
```

and fetches a real API key from OS keyring, then calls a real provider.

Risk:

- test depends on this specific Windows machine
- test depends on private provider config and keyring state
- test may call a paid/remote API during test runs
- test should not become a normal committed unit test without explicit gating/ignore/manual marker

Plain-language read: this is like writing a smoke test that only passes if it can open your own fridge and use your own grocery card. Useful for a one-time check, dangerous as a shared test.

---

## 5. What Is Not Proven Yet

- Not proven whether the running app binary includes the current `DesktopAgentTurnDriver` injection.
- Not proven whether `HAJIMI_AGENT_LLM_NATIVE_ENABLED` was disabled in the runtime environment.
- Not proven whether DeepSeek returns an HTTP error for `tools` / `tool_choice=auto`.
- Not proven whether DeepSeek returns a stream format that current parser fails to interpret.
- Not proven whether the model returns `write_file` with a relative path, which would become a separate workspace-root context issue.

---

## 6. Recommended Next Single Action

Freshly rebuild/run the desktop app from the current working tree, then trigger one minimal task:

```text
/agent 请在当前项目根目录下创建一个名为 deepseek-smoke.txt 的文件，内容写 Hello DeepSeek Native
```

Capture both:

```text
Agent Trace panel entries containing "LLM-Native"
Tauri terminal output containing "HTTP Error Body" or "LlmNativeDriver run_turn failed"
```

Stop-loss rule:

```text
If trace shows `LLM-Native Turn Error`, stop and preserve the provider error body.
If trace goes directly to `legacy act path` with no LLM-Native trace, stop and check runtime env/build freshness.
If native path calls `write_file` but path fails, stop and open a separate workspace-root/path-normalization debt.
```

---

## 7. Candidate Fix Directions, Not Yet Applied

1. Do not silently fallback from LLM-Native provider/runtime errors to `legacy_act`; surface the real provider error to UI.
2. Remove or hard-fail the `read_file("Cargo.toml")` default fallback in normal desktop `/agent` execution.
3. Add a desktop-level smoke test with a mocked `AgentTurnDriver`, proving setup injects the driver and never reaches legacy for native-enabled runs.
4. Add a provider-live manual test behind an explicit ignored/manual gate if DeepSeek live validation is needed.
5. Provide workspace root context to the model or normalize relative tool paths inside the desktop workspace sandbox.

---

## 8. Current Conclusion

The screenshot is consistent with an LLM-Native failure being hidden by a legacy fallback, not with a simple compile failure. The visible `read_file` error is probably the second failure, not the first one.

Plain-language conclusion: the thing we can see is smoke from the backup route. The real first spark is probably earlier, when the new LLM-Native route failed and quietly handed the job to the old route.
