//! Lightweight search interface over the ADR index.
//!
//! Provides a thin wrapper around `KnowledgeGraphIndex` for
//! ergonomic ID and keyword lookups.

use crate::adr_index::KnowledgeGraphIndex;

/// Search handle bound to a `KnowledgeGraphIndex`.
pub struct AdrSearch<'a> {
    index: &'a KnowledgeGraphIndex,
}

impl<'a> AdrSearch<'a> {
    /// Constructs a new search handle.
    pub fn new(index: &'a KnowledgeGraphIndex) -> Self {
        Self { index }
    }

    /// Finds an ADR path by its ID.
    pub fn find_by_id(&self, id: &str) -> Option<&str> {
        self.index.search_by_id(id)
    }

    /// Finds ADR paths matching a keyword.
    pub fn find_by_keyword(&self, keyword: &str) -> Vec<&str> {
        self.index.search_by_keyword(keyword)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adr_index::KnowledgeGraphIndex;

    #[test]
    fn test_search_interface() {
        let index = KnowledgeGraphIndex::new();
        let search = AdrSearch::new(&index);
        assert!(search.find_by_id("ADR-999").is_none());
        assert!(search.find_by_keyword("rust").is_empty());
    }
}
