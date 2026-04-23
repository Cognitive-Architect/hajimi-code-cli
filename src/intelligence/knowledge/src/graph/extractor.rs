//! 实体抽取逻辑（ADR/Rust AST→Node）

use crate::core_adr::AdrEntry;
use crate::graph::{Node, Result};

/// 实体抽取器
pub struct EntityExtractor;

impl EntityExtractor {
    /// 从ADR条目抽取节点
    pub fn extract_from_adr(adr: &AdrEntry) -> Result<Node> {
        let entity = adr.to_entity();
        Ok(Node::from_graph_entity(entity))
    }

    /// 批量抽取（事务优化）
    pub fn extract_batch(adrs: &[AdrEntry]) -> Result<Vec<Node>> {
        adrs.iter().map(Self::extract_from_adr).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core_adr::AdrStatus;
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
    fn test_extract_from_adr() -> Result<()> {
        let adr = create_test_adr();
        let node = EntityExtractor::extract_from_adr(&adr)?;
        assert_eq!(node.id, "ADR-0001");
        assert_eq!(node.label, "Test ADR");
        Ok(())
    }

    #[test]
    fn test_extract_batch() -> Result<()> {
        let adrs = vec![create_test_adr(), create_test_adr()];
        let nodes = EntityExtractor::extract_batch(&adrs)?;
        assert_eq!(nodes.len(), 2);
        Ok(())
    }
}
