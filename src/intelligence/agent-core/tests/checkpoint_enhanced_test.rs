//! Checkpoint enhanced tests: restore by ID, compare, export, reverse chronological list.

use agent_core::{Blackboard, CheckpointManager};
use agent_core::checkpoint::Checkpoint;
use chimera_repl::traits::ReplError;

#[tokio::test]
async fn test_restore_by_id_returns_correct_checkpoint() {
    let bb = Blackboard::new();
    let mgr = CheckpointManager::new();
    let chk = mgr.save(&"a".to_string(), None, vec![], vec![], &bb).await.unwrap();
    let rst = mgr.restore(&chk.id).await.unwrap();
    assert_eq!(rst.id, chk.id);
    assert_eq!(rst.agent_id, "a");
}

#[tokio::test]
async fn test_compare_same_checkpoints_returns_true() {
    let bb = Blackboard::new();
    let mgr = CheckpointManager::new();
    let chk = mgr.save(&"a".to_string(), None, vec![], vec![], &bb).await.unwrap();
    assert!(mgr.compare(&chk.id, &chk.id).await.unwrap());
}

#[tokio::test]
async fn test_compare_different_checkpoints_returns_false() {
    let bb = Blackboard::new();
    let mgr = CheckpointManager::new();
    let c1 = mgr.save(&"a".to_string(), None, vec![], vec![], &bb).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let c2 = mgr.save(&"a".to_string(), None, vec![], vec![], &bb).await.unwrap();
    assert!(!mgr.compare(&c1.id, &c2.id).await.unwrap());
}

#[tokio::test]
async fn test_export_returns_valid_json() {
    let bb = Blackboard::new();
    let mgr = CheckpointManager::new();
    let chk = mgr.save(&"a".to_string(), None, vec![], vec![], &bb).await.unwrap();
    let json = mgr.export(&chk.id).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["id"].as_str().unwrap(), chk.id);
}

#[tokio::test]
async fn test_list_returns_reverse_chronological_order() {
    let bb = Blackboard::new();
    let mgr = CheckpointManager::new();
    let c1 = mgr.save(&"a".to_string(), None, vec![], vec![], &bb).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    let c2 = mgr.save(&"a".to_string(), None, vec![], vec![], &bb).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    let c3 = mgr.save(&"a".to_string(), None, vec![], vec![], &bb).await.unwrap();
    let list = mgr.list(&"a".to_string()).await;
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].id, c3.id);
    assert_eq!(list[1].id, c2.id);
    assert_eq!(list[2].id, c1.id);
}

#[tokio::test]
async fn test_restore_nonexistent_checkpoint_fails() {
    let mgr = CheckpointManager::new();
    let result = mgr.restore("nonexistent").await;
    assert!(result.is_err());
    match result.unwrap_err() { ReplError::Session(m) => assert!(m.contains("not found")), _ => panic!("Expected Session error") }
}

#[tokio::test]
async fn test_checkpoint_goal_progress_preserved() {
    let chk = Checkpoint { id: "c".to_string(), timestamp: chrono::Utc::now(), agent_id: "a".to_string(), plan: None, reflections: vec![], swarm_workers: vec![], blackboard: std::collections::HashMap::new(), hash: "h".to_string(), version: 1, goal_progress: Some(0.75), key_reflection: None };
    assert_eq!(chk.goal_progress, Some(0.75));
    let de: Checkpoint = serde_json::from_str(&serde_json::to_string(&chk).unwrap()).unwrap();
    assert_eq!(de.goal_progress, Some(0.75));
}

#[tokio::test]
async fn test_checkpoint_key_reflection_preserved() {
    let chk = Checkpoint { id: "c".to_string(), timestamp: chrono::Utc::now(), agent_id: "a".to_string(), plan: None, reflections: vec![], swarm_workers: vec![], blackboard: std::collections::HashMap::new(), hash: "h".to_string(), version: 1, goal_progress: None, key_reflection: Some("Insight".to_string()) };
    assert_eq!(chk.key_reflection, Some("Insight".to_string()));
    let de: Checkpoint = serde_json::from_str(&serde_json::to_string(&chk).unwrap()).unwrap();
    assert_eq!(de.key_reflection, Some("Insight".to_string()));
}
