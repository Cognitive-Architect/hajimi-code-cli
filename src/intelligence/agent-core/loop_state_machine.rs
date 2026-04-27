//! DEBT-LINES-B0301A: Extracted from agent_loop.rs to reduce line count.
//! Encapsulates the 7-step agent loop state transitions.

use crate::agent_loop::{LoopOutcome, LoopState};

/// Stateless state machine for the 7-step proactive agent loop.
///
/// The loop follows: Observing → Retrieving → Planning → Acting →
/// Reflecting → Storing → Deciding → (back to Observing or terminal).
pub struct LoopStateMachine;

impl LoopStateMachine {
    /// Advance from the current state to the next state in the loop.
    pub fn next_state(current: LoopState) -> LoopState {
        match current {
            LoopState::Idle => LoopState::Observing,
            LoopState::Observing => LoopState::Retrieving,
            LoopState::Retrieving => LoopState::Planning,
            LoopState::Planning => LoopState::Acting,
            LoopState::Acting => LoopState::Reflecting,
            LoopState::Reflecting => LoopState::Storing,
            LoopState::Storing => LoopState::Deciding,
            LoopState::Deciding => LoopState::Observing,
            LoopState::Completed | LoopState::Failed => LoopState::Idle,
        }
    }

    /// Check if a state is terminal (loop should stop).
    pub fn is_terminal(state: LoopState) -> bool {
        matches!(state, LoopState::Completed | LoopState::Failed)
    }

    /// Determine if the loop should terminate based on the latest outcome.
    pub fn should_terminate(outcome: &LoopOutcome) -> bool {
        !matches!(outcome, LoopOutcome::InProgress)
    }

    /// Human-readable name for a loop state.
    pub fn state_name(state: LoopState) -> &'static str {
        match state {
            LoopState::Idle => "Idle",
            LoopState::Observing => "Observing",
            LoopState::Retrieving => "Retrieving",
            LoopState::Planning => "Planning",
            LoopState::Acting => "Acting",
            LoopState::Reflecting => "Reflecting",
            LoopState::Storing => "Storing",
            LoopState::Deciding => "Deciding",
            LoopState::Completed => "Completed",
            LoopState::Failed => "Failed",
        }
    }

    /// Convert a state to a zero-based step index for display ordering.
    pub fn step_index(state: LoopState) -> usize {
        match state {
            LoopState::Idle => 0,
            LoopState::Observing => 1,
            LoopState::Retrieving => 2,
            LoopState::Planning => 3,
            LoopState::Acting => 4,
            LoopState::Reflecting => 5,
            LoopState::Storing => 6,
            LoopState::Deciding => 7,
            LoopState::Completed => 8,
            LoopState::Failed => 9,
        }
    }
}
