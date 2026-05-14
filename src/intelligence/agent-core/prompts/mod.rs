//! Prompt resources and provider for Agent Core LLM interactions.
//! Phase 1 (AGENT-PROMPT-CORE-001): Loads the stable Agent Persona as system prompt.

/// Load the stable Agent Persona system prompt from embedded resource.
/// # Safety: This is a read-only static prompt; no user input is embedded.
pub fn load_agent_persona() -> &'static str {
    include_str!("agent_persona.md")
}

/// Feature-gate for Persona injection.
/// Reads from environment variable `HAJIMI_PROMPT_PERSONA_ENABLED`.
/// Defaults to `true` if unset.
pub fn is_persona_enabled() -> bool {
    std::env::var("HAJIMI_PROMPT_PERSONA_ENABLED")
        .map(|v| v != "false" && v != "0")
        .unwrap_or(true)
}

/// Feature-gate for Planner v1 schema (ToolManifest + PlannerSubgoalPlanV1Dto).
/// Reads from environment variable `HAJIMI_PLANNER_V1_ENABLED`.
/// Defaults to `true` if unset.
pub fn is_planner_v1_enabled() -> bool {
    std::env::var("HAJIMI_PLANNER_V1_ENABLED")
        .map(|v| v != "false" && v != "0")
        .unwrap_or(true)
}

/// Feature-gate for Reflector V1 schema (ReflectorCritiqueV1Dto + plan adjustment routing).
/// Reads from environment variable `HAJIMI_REFLECTOR_V1_ENABLED`.
/// Defaults to `true` if unset. When disabled, the reflector falls back to
/// legacy `Critique` JSON parsing without structured root-cause or stop-loss fields.
pub fn is_reflector_v1_enabled() -> bool {
    std::env::var("HAJIMI_REFLECTOR_V1_ENABLED")
        .map(|v| v != "false" && v != "0")
        .unwrap_or(true)
}

/// Feature-gate for Act ToolCall V1 chain execution.
/// Reads from environment variable `HAJIMI_ACT_TOOLCALL_V1_ENABLED`.
/// Defaults to `true` if unset. When disabled, AgentLoop uses the legacy
/// swarm/local act path without ActExecutor chain execution.
pub fn is_act_toolcall_v1_enabled() -> bool {
    std::env::var("HAJIMI_ACT_TOOLCALL_V1_ENABLED")
        .map(|v| v != "false" && v != "0")
        .unwrap_or(true)
}

/// Feature-gate for ContextWindowManager integration.
/// Reads from environment variable `HAJIMI_CONTEXT_WINDOW_ENABLED`.
/// Defaults to `true` if unset. When disabled, falls back to simple 2-message path.
pub fn is_context_window_enabled() -> bool {
    std::env::var("HAJIMI_CONTEXT_WINDOW_ENABLED")
        .map(|v| v != "false" && v != "0")
        .unwrap_or(true)
}
