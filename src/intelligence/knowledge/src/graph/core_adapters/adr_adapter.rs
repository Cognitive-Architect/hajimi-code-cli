//! ADR→Graph适配器（Week 33→Week 34桥接）

use crate::knowledge::adr::{AdrEntry, AdrStatus};
use crate::knowledge::graph::{EntityType, Node};
use chrono::Utc;

/// ADR条目到节点的零拷贝转换
impl From<AdrEntry> for Node {
    fn from(adr: AdrEntry) -> Self {
        Self {
            id: adr.id,
            label: adr.title,
            entity_type: EntityType::ADR,
            properties: serde_json::json!({
                "status": match adr.status {
                    AdrStatus::Proposed => "proposed",
                    AdrStatus::Accepted => "accepted",
                    AdrStatus::Deprecated => "deprecated",
                    AdrStatus::Rejected => "rejected",
                },
                "date": adr.date.to_rfc3339(),
                "tags": adr.tags,
            }),
            embedding: None,
            created_at: adr.date,
            updated_at: Utc::now(),
        }
    }
}

/// 批量转换适配
pub fn adapt_adrs_to_nodes(adrs: Vec<AdrEntry>) -> Vec<Node> {
    adrs.into_iter().map(Node::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_adr() -> AdrEntry {
        AdrEntry {
            id: "ADR-0001".to_string(),
            title: "Test ADR".to_string(),
            status: AdrStatus::Accepted,
            date: Utc::now(),
            tags: vec!["test".to_string()],
            content: "Test content".to_string(),
        }
    }

    #[test]
    fn test_adr_to_node_conversion() {
        let adr = create_test_adr();
        let node: Node = adr.into();
        assert_eq!(node.id, "ADR-0001");
        assert_eq!(node.label, "Test ADR");
        assert!(matches!(node.entity_type, EntityType::ADR));
    }

    #[test]
    fn test_adapt_adrs_to_nodes() {
        let adrs = vec![create_test_adr(), create_test_adr()];
        let nodes = adapt_adrs_to_nodes(adrs);
        assert_eq!(nodes.len(), 2);
    }
}
