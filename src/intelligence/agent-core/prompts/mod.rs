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
