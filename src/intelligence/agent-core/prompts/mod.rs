//! Prompt resources and provider for Agent Core LLM interactions.
//! Phase 1 (AGENT-PROMPT-CORE-001): Loads the stable Agent Persona as system prompt.

/// Load the stable Agent Persona system prompt from embedded resource.
/// # Safety: This is a read-only static prompt; no user input is embedded.
pub fn load_agent_persona() -> &'static str {
    include_str!("agent_persona.md")
}

/// Feature-gate for Persona injection.
/// TODO: Connect to runtime config in Phase 2 (AGENT-PROMPT-CORE-001).
pub fn is_persona_enabled() -> bool {
    // When config integration is ready, read from:
    // config.agent_core.prompt_persona_enabled.unwrap_or(true)
    true
}
