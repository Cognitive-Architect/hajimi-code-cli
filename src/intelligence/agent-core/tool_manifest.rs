//! Tool Manifest module for AGENT-PROMPT-CORE-001 Phase 2.
//!
//! Provides task-scoped tool selection and manifest generation for Planner
//! and Act LLM prompts. The manifest is filtered to ≤15 tools by default,
//! ranked by relevance to the current goal and step type.
//!
//! # Design Notes
//! - `ToolManifestGenerator` holds a reference to the runtime `ToolRegistry`
//!   and an enriched `ToolCatalog`.
//! - The scoring algorithm is intentionally left as a commented skeleton
//!   for Day 5 implementation (see `generate()`).
//! - All types are `Deserialize`-only because they are LLM output DTOs.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

// -----------------------------------------------------------------------------
// Enums
// -----------------------------------------------------------------------------

/// Functional classification of a tool.
///
/// Used by the manifest generator to match tools to task intent.
/// Each variant maps to a family of operations the Agent may need.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolCategory {
    /// Read file contents without modification.
    FileRead,
    /// Write, edit, or patch files.
    FileWrite,
    /// Search code, symbols, or text across the workspace.
    Search,
    /// Git version-control operations (status, diff, commit, etc.).
    Git,
    /// Build and compilation tools.
    Build,
    /// Test execution, collection, and reporting.
    Test,
    /// Language Server Protocol interactions (definitions, references, hover).
    Lsp,
    /// Model Context Protocol server operations.
    Mcp,
    /// Shell command execution (allow-listed).
    Shell,
    /// Static analysis, linting, and code inspection.
    Analysis,
    /// Documentation generation and reading.
    Docs,
    /// Network requests and external API calls.
    Network,
    /// Uncategorized or miscellaneous tools.
    Other,
}

/// Risk level assigned to a tool based on its potential impact.
///
/// Derived from `engine_tool_system` permission levels:
/// - `Allow`  → Low
/// - `Ask`    → Medium
/// - `Deny`   → Critical (or omitted)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Read-only or otherwise fully safe operations.
    Low,
    /// May modify workspace state but changes are recoverable.
    Medium,
    /// Destructive, wide-ranging, or hard-to-reverse changes.
    High,
    /// Irreversible, security-critical, or governance-mandatory operations.
    Critical,
}

/// Step type in the 7-step Agent Core loop.
///
/// Determines which tool categories are most relevant when generating
/// a manifest for the current operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepType {
    /// Observe current workspace and goal state.
    Observe,
    /// Retrieve relevant memory, symbols, and context.
    Retrieve,
    /// Plan decomposition and subgoal ordering.
    Plan,
    /// Execute the next approved task.
    Act,
    /// Reflect on execution outcome.
    Reflect,
    /// Persist learnings and traces.
    Store,
    /// Decide whether to continue, retry, or hand off.
    Decide,
}

// -----------------------------------------------------------------------------
// Data Transfer Objects
// -----------------------------------------------------------------------------

/// A single tool entry in the manifest presented to the LLM.
///
/// Contains everything the Agent needs to decide whether and how to use
/// the tool, including risk metadata and recovery hints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifestEntryV1 {
    /// Tool name as registered in the runtime `ToolRegistry`.
    pub name: String,
    /// Concise description of what the tool does.
    /// Prompt-injected descriptions should be compacted to ≤200 chars.
    pub description: String,
    /// Functional category for intent matching and filtering.
    pub category: ToolCategory,
    /// Whether the tool is enabled under the current runtime configuration.
    pub available: bool,
    /// Estimated risk level derived from the permission model.
    pub risk_level: RiskLevel,
    /// Whether governance approval is required before execution.
    pub requires_confirmation: bool,
    /// JSON Schema describing the tool's expected parameters.
    pub parameters_schema: serde_json::Value,
    /// Scenarios where this tool is the right choice.
    pub when_to_use: Vec<String>,
    /// Scenarios where this tool should be avoided.
    pub do_not_use_when: Vec<String>,
    /// Hints for recovering from known failure modes.
    pub recovery_hints: Vec<String>,
    /// Expected artifacts or evidence after successful execution.
    pub evidence_expected: Vec<String>,
    /// Known runtime failure kinds observed for this tool.
    pub known_failure_kinds: Vec<String>,
}

/// Request parameters for generating a task-scoped tool manifest.
///
/// The generator uses these fields to rank and filter tools so that only
/// the most relevant subset is injected into the LLM prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifestRequest {
    /// Current goal description used for intent classification.
    pub goal_description: String,
    /// Which step of the 7-step loop is currently executing.
    pub step_type: StepType,
    /// The active task description, if available.
    pub current_task: Option<String>,
    /// Names of tools that recently failed with identical parameters.
    /// These are deprioritized to avoid retry loops.
    pub recently_failed_tools: Vec<String>,
    /// Token budget available for tool descriptions in the prompt.
    pub available_budget_tokens: usize,
    /// Maximum number of tools to include in the manifest.
    /// Default is 15; may be lowered for smaller models.
    pub max_tools: usize,
}

// -----------------------------------------------------------------------------
// Catalog
// -----------------------------------------------------------------------------

/// Lightweight catalog holding enriched tool metadata beyond runtime reflection.
///
/// Populated from manual catalog files (e.g. `docs/agent-prompt-core/tool-catalog/`)
/// and merged with live `ToolRegistry` data at generation time.
#[derive(Debug, Clone)]
pub struct ToolCatalog {
    // TODO: Phase 2 Day 5 — load from manual JSON catalog files.
}

impl ToolCatalog {
    /// Create an empty catalog.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ToolCatalog {
    fn default() -> Self {
        Self::new()
    }
}

// -----------------------------------------------------------------------------
// Generator
// -----------------------------------------------------------------------------

/// Generates a filtered, task-relevant tool manifest for Planner and Act prompts.
///
/// The generator scores each registered tool against the current request,
/// ranks by relevance, and caps the result to the configured budget and
/// maximum tool count.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ToolManifestGenerator {
    /// Reference to the runtime tool registry.
    ///
    /// TODO: Phase 2 Day 5 — replace `dyn Any` with concrete
    /// `Arc<dyn engine_tool_system::ToolRegistry>` once the trait is
    /// available in the public API.
    registry: Option<Arc<dyn std::any::Any + Send + Sync>>,
    /// Enriched metadata catalog merged from manual descriptions.
    catalog: ToolCatalog,
}

impl ToolManifestGenerator {
    /// Create a new generator with the given catalog.
    ///
    /// The registry is left unpopulated until Day 5 wiring.
    pub fn new(catalog: ToolCatalog) -> Self {
        Self {
            registry: None,
            catalog,
        }
    }

    /// Generate a filtered tool manifest for the given request.
    ///
    /// # Skeleton Status
    /// This is a Day 4 skeleton. The scoring and filtering logic will be
    /// implemented in Day 5.
    ///
    /// # Scoring Algorithm (Day 5)
    /// 1. Load all registered tools from `ToolRegistry.list()`.
    /// 2. For each tool, merge runtime fields (`name`, `description`,
    ///    `available`, `risk_level`) with catalog fields (`when_to_use`,
    ///    `recovery_hints`, `evidence_expected`).
    /// 3. Classify intent from `goal_description` + `current_task` +
    ///    `step_type`.
    /// 4. Score each tool:
    ///    - `+4` if `category` matches inferred intent.
    ///    - `+3` if tool name or description contains keywords from the goal.
    ///    - `+2` if the tool is commonly needed by the current `step_type`.
    ///    - `+2` if the tool provides `evidence_expected` by the current task.
    ///    - `-3` if the tool appears in `recently_failed_tools` with
    ///      identical parameters.
    ///    - `-2` if the tool is `High` risk and a lower-risk alternative
    ///      exists.
    ///    - `-99` if the tool is unavailable or disallowed by governance.
    /// 5. Sort by score descending, then by risk ascending
    ///    (`Low < Medium < High < Critical`).
    /// 6. Keep required tools first, then best-scored optional tools.
    /// 7. Cap to `max_tools` (default 15).
    /// 8. Compact descriptions and hints to fit `available_budget_tokens`.
    /// 9. Record omitted tools and reason in trace metadata.
    ///
    /// Generate a filtered tool manifest based on StepType → ToolCategory mapping.
    /// B-06-FIX-001: Minimal scoring algorithm using hardcoded category priorities.
    pub fn generate(&self, request: &ToolManifestRequest) -> Vec<ToolManifestEntryV1> {
        let categories = match request.step_type {
            StepType::Plan => vec![
                ToolCategory::Search,
                ToolCategory::Analysis,
                ToolCategory::FileRead,
            ],
            StepType::Act => vec![
                ToolCategory::FileWrite,
                ToolCategory::Shell,
                ToolCategory::Git,
            ],
            StepType::Observe => vec![
                ToolCategory::FileRead,
                ToolCategory::Search,
                ToolCategory::Lsp,
            ],
            StepType::Retrieve => vec![ToolCategory::Search, ToolCategory::Lsp, ToolCategory::Mcp],
            StepType::Reflect => vec![
                ToolCategory::Test,
                ToolCategory::Analysis,
                ToolCategory::FileRead,
            ],
            StepType::Store => vec![ToolCategory::Git, ToolCategory::Docs, ToolCategory::Network],
            StepType::Decide => vec![
                ToolCategory::FileRead,
                ToolCategory::Analysis,
                ToolCategory::Test,
            ],
        };
        categories
            .into_iter()
            .enumerate()
            .map(|(i, category)| ToolManifestEntryV1 {
                name: format!("tool-{}-{:?}", i, category),
                description: format!(
                    "Recommended {:?} tool for {:?} step",
                    category, request.step_type
                ),
                category,
                available: true,
                risk_level: RiskLevel::Low,
                requires_confirmation: false,
                parameters_schema: serde_json::json!({}),
                when_to_use: vec![format!("During {:?} step", request.step_type)],
                do_not_use_when: vec![],
                recovery_hints: vec![],
                evidence_expected: vec![],
                known_failure_kinds: vec![],
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// B-06/14: ToolManifestEntryV1 full serialize/deserialize roundtrip.
    #[test]
    fn test_tool_manifest_entry_v1_serialize() {
        let entry = ToolManifestEntryV1 {
            name: "read_file".to_string(),
            description: "Read a file".to_string(),
            category: ToolCategory::FileRead,
            available: true,
            risk_level: RiskLevel::Low,
            requires_confirmation: false,
            parameters_schema: serde_json::json!({"path": {"type": "string"}}),
            when_to_use: vec!["need file contents".to_string()],
            do_not_use_when: vec!["binary files".to_string()],
            recovery_hints: vec!["check path".to_string()],
            evidence_expected: vec!["file content".to_string()],
            known_failure_kinds: vec!["not found".to_string()],
        };
        let json = serde_json::to_string(&entry).expect("should serialize");
        let deserialized: ToolManifestEntryV1 =
            serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(entry.name, deserialized.name);
        assert_eq!(entry.category, deserialized.category);
        assert_eq!(entry.risk_level, deserialized.risk_level);
    }

    /// B-06/14: ToolManifestRequest serialize/deserialize roundtrip.
    #[test]
    fn test_tool_manifest_request_serialize() {
        let req = ToolManifestRequest {
            goal_description: "Implement feature".to_string(),
            step_type: StepType::Plan,
            current_task: Some("design".to_string()),
            recently_failed_tools: vec!["lint".to_string()],
            available_budget_tokens: 1600,
            max_tools: 15,
        };
        let json = serde_json::to_string(&req).expect("should serialize");
        let deserialized: ToolManifestRequest =
            serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(req.goal_description, deserialized.goal_description);
        assert_eq!(req.step_type, deserialized.step_type);
        assert_eq!(req.max_tools, deserialized.max_tools);
    }

    /// B-06/14: ToolCatalog default creates an empty catalog.
    #[test]
    fn test_tool_catalog_default() {
        let catalog = ToolCatalog::default();
        let catalog2 = ToolCatalog::new();
        // Both should create the same empty state.
        assert_eq!(
            std::mem::size_of_val(&catalog),
            std::mem::size_of_val(&catalog2)
        );
    }

    /// B-06-FIX-001: generate produces non-empty manifest for Plan step.
    #[test]
    fn test_generate_non_empty_plan_step() {
        let generator = ToolManifestGenerator::new(ToolCatalog::new());
        let request = ToolManifestRequest {
            goal_description: "test".to_string(),
            step_type: StepType::Plan,
            current_task: None,
            recently_failed_tools: vec![],
            available_budget_tokens: 1600,
            max_tools: 15,
        };
        let manifest = generator.generate(&request);
        assert_eq!(manifest.len(), 3, "Plan step should produce 3 tools");
        assert!(manifest.iter().any(|e| e.category == ToolCategory::Search));
    }

    /// B-06-FIX-001: generate produces correct ToolCategory mapping for all StepType variants.
    #[test]
    fn test_generate_all_step_types() {
        let generator = ToolManifestGenerator::new(ToolCatalog::new());
        let expected = vec![
            (
                StepType::Plan,
                vec![
                    ToolCategory::Search,
                    ToolCategory::Analysis,
                    ToolCategory::FileRead,
                ],
            ),
            (
                StepType::Act,
                vec![
                    ToolCategory::FileWrite,
                    ToolCategory::Shell,
                    ToolCategory::Git,
                ],
            ),
            (
                StepType::Observe,
                vec![
                    ToolCategory::FileRead,
                    ToolCategory::Search,
                    ToolCategory::Lsp,
                ],
            ),
            (
                StepType::Retrieve,
                vec![ToolCategory::Search, ToolCategory::Lsp, ToolCategory::Mcp],
            ),
            (
                StepType::Reflect,
                vec![
                    ToolCategory::Test,
                    ToolCategory::Analysis,
                    ToolCategory::FileRead,
                ],
            ),
            (
                StepType::Store,
                vec![ToolCategory::Git, ToolCategory::Docs, ToolCategory::Network],
            ),
            (
                StepType::Decide,
                vec![
                    ToolCategory::FileRead,
                    ToolCategory::Analysis,
                    ToolCategory::Test,
                ],
            ),
        ];
        for (step_type, expected_cats) in expected {
            let request = ToolManifestRequest {
                goal_description: "test".to_string(),
                step_type,
                current_task: None,
                recently_failed_tools: vec![],
                available_budget_tokens: 1600,
                max_tools: 15,
            };
            let manifest = generator.generate(&request);
            assert_eq!(
                manifest.len(),
                expected_cats.len(),
                "{:?} step should produce {} entries",
                step_type,
                expected_cats.len()
            );
            for (i, expected_cat) in expected_cats.iter().enumerate() {
                assert_eq!(
                    manifest[i].category, *expected_cat,
                    "{:?} step entry {} should be {:?}",
                    step_type, i, expected_cat
                );
            }
        }
    }

    /// B-06-FIX-001: generated entries have correct default flags.
    #[test]
    fn test_generate_entry_defaults() {
        let generator = ToolManifestGenerator::new(ToolCatalog::new());
        let request = ToolManifestRequest {
            goal_description: "test".to_string(),
            step_type: StepType::Act,
            current_task: None,
            recently_failed_tools: vec![],
            available_budget_tokens: 1600,
            max_tools: 15,
        };
        let manifest = generator.generate(&request);
        for entry in &manifest {
            assert!(entry.available, "generated tool should be marked available");
            assert_eq!(
                entry.risk_level,
                RiskLevel::Low,
                "default risk should be Low"
            );
            assert!(
                !entry.requires_confirmation,
                "default should not require confirmation"
            );
        }
    }

    /// B-06/14: RiskLevel equality and ordering semantics.
    #[test]
    fn test_risk_level_equality() {
        assert_eq!(RiskLevel::Low, RiskLevel::Low);
        assert_ne!(RiskLevel::Low, RiskLevel::High);
        assert_ne!(RiskLevel::Medium, RiskLevel::Critical);
    }

    /// B-06/14: StepType equality across all variants.
    #[test]
    fn test_step_type_equality() {
        assert_eq!(StepType::Plan, StepType::Plan);
        assert_ne!(StepType::Plan, StepType::Act);
        assert_ne!(StepType::Observe, StepType::Reflect);
    }
}
