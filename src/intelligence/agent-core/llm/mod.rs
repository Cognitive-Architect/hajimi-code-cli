//! LLM adapter bridge: connects engine-llm-core clients to agent-core planner/reflector traits.

pub mod bridge;
pub use bridge::{PlannerLlmBridge, ReflectorLlmBridge, collect_stream};
