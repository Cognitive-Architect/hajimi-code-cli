//! ADR Architecture Decision Records System
//! Links DEBT-XXX to code and test files
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use blake3::Hash;

/// ADR status states
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Deprecated,
    Superseded { by: String },
}

/// Architecture Decision Record entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdrEntry {
    pub id: String,
    pub title: String,
    pub status: AdrStatus,
    pub context: String,
    pub decision: String,
    pub consequences: String,
    pub linked_debt: Option<String>, // DEBT-XXX
    pub code_files: Vec<String>,
    pub test_files: Vec<String>,
    pub created_at: String,
    pub modified_at: String,
    pub content_hash: String,
}

/// ADR registry with debt linkage
pub struct AdrRegistry {
    entries: HashMap<String, AdrEntry>,
}

impl AdrRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Create new ADR entry
    pub fn create_adr(&mut self, id: &str, title: &str, linked_debt: Option<&str>) -> AdrEntry {
        let entry = AdrEntry {
            id: id.to_string(),
            title: title.to_string(),
            status: AdrStatus::Proposed,
            context: String::new(),
            decision: String::new(),
            consequences: String::new(),
            linked_debt: linked_debt.map(|s| s.to_string()),
            code_files: Vec::new(),
            test_files: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            content_hash: String::new(),
        };
        self.entries.insert(id.to_string(), entry.clone());
        entry
    }

    /// Get ADR by ID
    pub fn get_adr(&self, id: &str) -> Option<&AdrEntry> {
        self.entries.get(id)
    }

    /// List all ADRs with optional filter
    pub fn list_adrs(&self, status_filter: Option<AdrStatus>) -> Vec<&AdrEntry> {
        self.entries.values()
            .filter(|e| status_filter.as_ref().map_or(true, |s| e.status == *s))
            .collect()
    }

    /// Find ADRs by debt ID
    pub fn find_by_debt(&self, debt_id: &str) -> Vec<&AdrEntry> {
        self.entries.values()
            .filter(|e| e.linked_debt.as_ref().map_or(false, |d| d == debt_id))
            .collect()
    }

    /// Update ADR status
    pub fn update_status(&mut self, id: &str, new_status: AdrStatus) -> Result<(), String> {
        if let Some(entry) = self.entries.get_mut(id) {
            entry.status = new_status;
            entry.modified_at = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(format!("ADR {} not found", id))
        }
    }

    /// Link code file to ADR
    pub fn link_code_file(&mut self, adr_id: &str, file_path: &str) -> Result<(), String> {
        if let Some(entry) = self.entries.get_mut(adr_id) {
            entry.code_files.push(file_path.to_string());
            entry.modified_at = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(format!("ADR {} not found", adr_id))
        }
    }

    /// Link test file to ADR
    pub fn link_test_file(&mut self, adr_id: &str, file_path: &str) -> Result<(), String> {
        if let Some(entry) = self.entries.get_mut(adr_id) {
            entry.test_files.push(file_path.to_string());
            entry.modified_at = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(format!("ADR {} not found", adr_id))
        }
    }

    /// Generate Markdown for ADR
    pub fn generate_markdown(&self, id: &str) -> Option<String> {
        self.entries.get(id).map(|e| {
            format!(
                "# ADR-{}: {}\n\n## Status\n{:?}\n\n## Context\n{}\n\n## Decision\n{}\n\n## Consequences\n{}\n\n## Linked Debt\n{}\n\n## Code Files\n{}\n\n## Test Files\n{}\n",
                e.id, e.title, e.status, e.context, e.decision, e.consequences,
                e.linked_debt.as_deref().unwrap_or("None"),
                e.code_files.join("\n"), e.test_files.join("\n")
            )
        })
    }

    /// Compute content hash for integrity
    pub fn compute_hash(&self, id: &str) -> Option<String> {
        self.entries.get(id).map(|e| {
            let content = format!("{}{}{}", e.id, e.title, e.decision);
            blake3::hash(content.as_bytes()).to_string()
        })
    }
}

impl Default for AdrRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adr_create() {
        let mut reg = AdrRegistry::new();
        let adr = reg.create_adr("001", "Test ADR", Some("DEBT-TEST-001"));
        assert_eq!(adr.id, "001");
        assert_eq!(adr.linked_debt, Some("DEBT-TEST-001".to_string()));
    }

    #[test]
    fn test_adr_state_transition() {
        let mut reg = AdrRegistry::new();
        reg.create_adr("002", "State Test", None);
        reg.update_status("002", AdrStatus::Accepted).unwrap();
        let adr = reg.get_adr("002").unwrap();
        assert!(matches!(adr.status, AdrStatus::Accepted));
    }

    #[test]
    fn test_find_by_debt() {
        let mut reg = AdrRegistry::new();
        reg.create_adr("003", "Debt Link", Some("DEBT-001"));
        let results = reg.find_by_debt("DEBT-001");
        assert_eq!(results.len(), 1);
    }
}
