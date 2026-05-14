use chrono::Utc;
use memory::graph::{GraphError, GraphMemory};
use memory::types::{MemoryEntry, MemoryLayerId};
use std::collections::HashSet;
use tempfile::TempDir;

#[test]
fn test_store_recall_roundtrip_with_three_entities() -> Result<(), GraphError> {
    let tmp = TempDir::new().unwrap();
    let mut gm = GraphMemory::new_with_path(&tmp.path().join("graph.db"))?;
    let entries = vec![
        MemoryEntry {
            id: "e1".into(),
            content: "Apple and Microsoft are tech companies".into(),
            tokens: 10,
            timestamp: Utc::now(),
            layer: MemoryLayerId::Graph,
        },
        MemoryEntry {
            id: "e2".into(),
            content: "Google and Amazon compete with Microsoft".into(),
            tokens: 10,
            timestamp: Utc::now(),
            layer: MemoryLayerId::Graph,
        },
        MemoryEntry {
            id: "e3".into(),
            content: "Apple iPhone is a product from Apple Inc".into(),
            tokens: 10,
            timestamp: Utc::now(),
            layer: MemoryLayerId::Graph,
        },
    ];
    for entry in &entries {
        gm.store(entry.clone())?;
    }
    assert!(
        gm.node_count() >= 3,
        "Expected >= 3 entities, got {}",
        gm.node_count()
    );

    let results = gm.recall("Apple")?;
    assert!(!results.is_empty(), "Expected results for 'Apple'");
    let names: HashSet<_> = results.iter().map(|n| n.name.as_str()).collect();
    assert!(names.contains("Apple"), "Expected 'Apple' in results");

    let multi = gm.recall("Microsoft Google")?;
    assert!(!multi.is_empty(), "Expected results for multi-keyword");

    gm.flush()?;
    Ok(())
}

#[test]
fn test_recall_empty_query() -> Result<(), GraphError> {
    let tmp = TempDir::new().unwrap();
    let gm = GraphMemory::new_with_path(&tmp.path().join("graph.db"))?;
    assert!(gm.recall("")?.is_empty());
    Ok(())
}

#[test]
fn test_close_then_recall_fails() -> Result<(), GraphError> {
    let tmp = TempDir::new().unwrap();
    let mut gm = GraphMemory::new_with_path(&tmp.path().join("graph.db"))?;
    gm.close();
    assert!(gm.recall("test").is_err(), "Expected error after close");
    Ok(())
}
