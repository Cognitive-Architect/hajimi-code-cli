//! Core types for EVMbench adapter
use serde::{Deserialize, Serialize};

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity { Critical, High, Medium, Low }
impl Default for Severity { fn default() -> Self { Severity::Medium } }

/// Vulnerability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub contract_name: String,
    pub vulnerability: String,
    pub severity: Severity,
    pub code: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub lines: Vec<u32>,
}
fn default_source() -> String { "evmbench".to_string() }

impl Vulnerability {
    pub fn new(name: &str, vuln_type: &str, severity: Severity, code: &str) -> Self {
        Self { contract_name: name.to_string(), vulnerability: vuln_type.to_string(), severity, code: code.to_string(), description: String::new(), source: default_source(), lines: Vec::new() }
    }
}

/// Test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityTest {
    pub id: String,
    pub contract_code: String,
    pub exploit_template: Option<String>,
    pub difficulty: DifficultyLevel,
}

/// Difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifficultyLevel { Easy, Medium, Hard, Critical }

/// Exploit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploitConfig {
    pub target_contract: String,
    pub rpc_url: String,
    pub attacker_key: String,
    pub exploit_code: String,
    pub expected_drain: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}
fn default_timeout() -> u64 { 60000 }

/// Exploit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploitResult {
    pub success: bool,
    pub tx_hash: Option<String>,
    pub drained_amount: Option<String>,
    pub error: Option<String>,
    pub duration: u64,
}

impl ExploitResult {
    pub fn success_result(duration: u64, tx_hash: Option<String>) -> Self {
        Self { success: true, tx_hash, drained_amount: None, error: None, duration }
    }
    pub fn failure(error: &str, duration: u64) -> Self {
        Self { success: false, tx_hash: None, drained_amount: None, error: Some(error.to_string()), duration }
    }
}

/// Complete dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityDataset {
    pub version: String,
    pub source: String,
    pub count: usize,
    pub generated_at: String,
    pub vulnerabilities: Vec<Vulnerability>,
}

/// Bench config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchConfig {
    pub anvil_endpoint: String,
    pub timeout_ms: u64,
    pub max_gas_limit: u64,
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self { anvil_endpoint: "http://127.0.0.1:8545".to_string(), timeout_ms: 60000, max_gas_limit: 10_000_000 }
    }
}
