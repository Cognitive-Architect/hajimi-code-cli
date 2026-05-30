# DEBT-AGENT-DEEPSEEK-TOOL-SCHEMA-WEB-SEARCH

> **ID**: `DEBT-AGENT-DEEPSEEK-TOOL-SCHEMA-WEB-SEARCH`  
> **Priority**: **P0**  
> **Date**: 2026-05-30  
> **Status**: `OPEN / INVESTIGATING`  
> **Scope**: Desktop `/agent`, LLM-Native tool export, DeepSeek/OpenAI-compatible tool schema

---

## 1. Problem Summary

After disabling the old legacy fallback masking path, real-machine `/agent` now exposes the primary provider error:

```text
LLM-Native Turn Error: Internal error: Network Error: HTTP 错误: 400 Bad Request -
{"error":{"message":"Invalid schema for function 'web_search': schema must be a
JSON Schema of 'type: \"object\"', got 'type: null'.","type":"invalid_request_error",
"param":null,"code":"invalid_request_error"}}
legacy fallback skipped.
```

Technical read: DeepSeek rejects the chat-completions request before the model starts because at least one exported tool definition has an invalid `function.parameters` JSON Schema.

Plain-language read: `/agent` is handing DeepSeek a toolbox list. One tool named `web_search` has a broken instruction card. DeepSeek checks the toolbox before doing any work, sees the bad card, and refuses the whole order.

---

## 2. User-Visible Impact

- `/agent 查看当前目录下有什么文件` fails before any local directory tool can run.
- The task is marked failed with `ActFailed`.
- This happens even though the user did not ask for web search.
- The visible error is now more truthful than the previous `read_file(Cargo.toml)` failure because the legacy fallback no longer hides the provider-side 400.

---

## 3. Evidence Collected

### 3.1 Real-machine UI evidence

Screenshot shows the Agent Trace contains two `ACTING` entries with the same provider error:

```text
Invalid schema for function 'web_search': schema must be a JSON Schema of
'type: "object"', got 'type: null'
```

The main panel shows:

```text
智能体在执行动作时失败: "LLM-Native Turn Error: Internal error:
Network Error: HTTP 错误: 400 Bad Request ..."
```

### 3.2 `web_search` is a real registered desktop tool

`src/interface/desktop/src/main.rs` registers the tool:

```rust
r.register(Arc::new(WebSearchTool::new()));
```

`src/engine/tool-system/src/network.rs` defines:

```rust
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError {
                message: "Missing query".into(),
                kind: ToolErrorKind::InvalidArgs,
            })?;
        self.search(query).await
    }
}
```

So the intended schema should be an object with at least a string `query` field.

### 3.3 `/agent` exports all registered tools to the model

`src/intelligence/agent-core/agent_loop.rs` exports tools from the runtime registry:

```rust
let tools_spec = if let Some(ref reg) = self.tool_registry {
    let guard = reg.lock().await;
    crate::llm_native::ToolSpecExporter::from_registry(&guard)
} else {
    vec![]
};
```

This means a bad schema in any registered tool can block unrelated goals.

Plain-language read: even if the user only wants to list files, the app still shows DeepSeek the full toolbox first. If one screwdriver has a nonsense label, DeepSeek refuses to enter the workshop.

### 3.4 Unknown tool schemas currently degrade to `{}` instead of a typed object

`src/intelligence/agent-core/llm_native/tool_spec.rs` has static schemas only for a small set:

```rust
"read_file" => { ... }
"write_file" => { ... }
"edit_file" => { ... }
"list_dir" | "list_directory" => { ... }
"grep_search" | "grep" => { ... }
_ => serde_json::json!({}),
```

`web_search` is not covered by this match, so it currently falls through to `{}`.

### 3.5 OpenAI-compatible client forwards schemas as-is

`src/intelligence/agent-core/llm_native/driver.rs` maps model-visible specs to engine tool definitions:

```rust
let tool_definitions: Vec<engine_llm_core::ToolDefinition> = tools
    .iter()
    .map(|t| engine_llm_core::ToolDefinition {
        name: t.name.clone(),
        description: t.description.clone(),
        parameters: t.parameters_schema.clone(),
    })
    .collect();
```

`src/engine/llm-core/src/openai.rs` sends that schema as `function.parameters`:

```rust
"function": {
    "name": t.name,
    "description": t.description,
    "parameters": t.parameters
}
```

So if `ToolSpecExporter` emits `{}` or another incomplete value, the provider receives it directly.

---

## 4. Likely Failure Chain

```text
Desktop ToolRegistry
  -> includes WebSearchTool("web_search")
AgentLoop::run
  -> ToolSpecExporter::from_registry(&guard)
ToolSpecExporter::get_tool_schema("web_search")
  -> falls through to {}
LlmNativeDriver::run_turn
  -> converts parameters_schema into ToolDefinition.parameters
OpenAiClient::stream_chat_with_tools
  -> sends tool function.parameters to DeepSeek
DeepSeek validation
  -> rejects web_search because parameters is not a JSON Schema object with type:"object"
/agent
  -> ActFailed with provider 400
```

---

## 5. Why This Was Hidden Before

Before the fallback repair, LLM-Native provider errors could fall through into the legacy path. That legacy path then failed on a default `read_file(Cargo.toml)`, hiding the real provider-side 400.

Current behavior is better for debugging: `/agent` still fails, but now it reports the real upstream reason.

---

## 6. Risk Assessment

### Confirmed

- DeepSeek rejects current tool schema for `web_search`.
- `web_search` exists in the registry and is exported to LLM-Native.
- `web_search` lacks a static schema in `ToolSpecExporter::get_tool_schema`.
- The OpenAI-compatible client does not sanitize or normalize tool schemas before sending.

### Probable

- More tools than `web_search` may have invalid or empty schemas, because most registered tools are not covered by the static match.
- Fixing only `web_search` may reveal the next invalid tool in the registry.

### Not Yet Verified

- Full HTTP request body was not captured in this pass.
- DeepSeek behavior was observed through the UI error, not a minimal isolated provider contract test.
- Whether OpenAI proper accepts `{}` while DeepSeek rejects it has not been re-tested here.

---

## 7. Suggested Repair Options For A Later Coding Pass

### Option A - Minimal unblock

Add a static schema for `web_search`:

```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "Search query"
    }
  },
  "required": ["query"],
  "additionalProperties": false
}
```

This may unblock the first visible failure, but it may expose the next invalid tool.

### Option B - Safer general fix

Normalize every exported tool schema before sending it to providers:

- If schema is missing, null, or not an object, replace it with:

```json
{
  "type": "object",
  "properties": {},
  "additionalProperties": true
}
```

- If schema is an object but lacks `"type": "object"`, add it.
- Preserve existing `properties`, `required`, and descriptions.
- Add tests proving every exported `ModelVisibleToolSpec.parameters_schema` has top-level `type: "object"`.

Plain-language read: before handing the toolbox list to DeepSeek, run every instruction card through a quick format checker. Missing card gets a safe blank card; half-written card gets filled in.

### Option C - Strict allowlist for LLM-Native tools

Only expose tools with known valid schemas until the full registry has schema coverage.

This reduces blast radius, but may hide useful tools from the agent and should be treated as a temporary compatibility gate.

---

## 8. Recommended Next Step

Do not change the UI first. The next coding pass should start in the LLM-Native tool schema layer:

1. Add a regression test that builds the desktop/tool-system registry or a representative registry with `web_search`.
2. Assert every exported tool schema has top-level `"type": "object"`.
3. Add `web_search` schema and/or a schema normalization fallback.
4. Re-run `/agent 查看当前目录下有什么文件` against DeepSeek.

Stop condition: if DeepSeek then rejects another tool name, record that tool and decide whether to implement full registry schema coverage instead of one-by-one patches.

---

## 9. Current Conclusion

This is no longer primarily a local file access problem. The current blocker is provider-side tool schema compatibility: DeepSeek refuses the request because `web_search` is exported with an invalid/incomplete JSON Schema.

The earlier fallback repair succeeded in exposing the real error; `/agent` now needs tool schema normalization or complete schema coverage before DeepSeek can accept the turn.
