//! Unified error degradation abstraction for agent-core.
//! Provides consistent warn logging and graceful fallback across all modules.

use tracing::warn;

/// Degrade a Result<T, E> to Option<T>, logging a warning with context on error.
pub fn degrade_warn<T, E: std::fmt::Display>(result: Result<T, E>, ctx: &str) -> Option<T> {
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            warn!("{} failed (continuing): {}", ctx, e);
            None
        }
    }
}
