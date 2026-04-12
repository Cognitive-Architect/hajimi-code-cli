//! EVMbench Adapter for Hajimi
//!
//! Provides Rust types and utilities for integrating EVMbench
//! vulnerability dataset with Hajimi's testing framework.

pub mod runner;
pub mod types;

pub use runner::Runner;
pub use types::{ExploitConfig, ExploitResult, Severity, Vulnerability, VulnerabilityDataset};

/// Re-export serde for convenience
pub use serde::{Deserialize, Serialize};
