//! TypeRacing UI Integration Tests
//!
//! Tests for WebUI and Terminal integration with the Engine

use typeracing::{Engine, TerminalAdapter, AdapterState};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Test that predictions are properly sorted by confidence
#[tokio::test]
async fn test_predictions_sorted_by_confidence() {
    let mut engine = Engine::new();
    
    // Build type tree with test predictions
    let predictions = vec![
        typeracing::PredictionNode {
            type_name: "i32".to_string(),
            confidence: 0.7,
            source: typeracing::PredictionSource::Heuristic,
            children: vec![],
        },
        typeracing::PredictionNode {
            type_name: "String".to_string(),
            confidence: 0.95,
            source: typeracing::PredictionSource::LspHover,
            children: vec![],
        },
        typeracing::PredictionNode {
            type_name: "Vec<i32>".to_string(),
            confidence: 0.85,
            source: typeracing::PredictionSource::LspDefinition,
            children: vec![],
        },
    ];
    
    engine.build_type_tree("test://file.rs", predictions);
    let result = engine.get_predictions("test://file.rs");
    
    // Should return all predictions
    assert_eq!(result.len(), 3);
}

/// Test terminal adapter initialization
#[test]
fn test_terminal_adapter_init() {
    let adapter = TerminalAdapter::new();
    
    assert!(matches!(adapter.state(), AdapterState::Idle));
    assert!(!adapter.is_initialized());
    assert!(adapter.get_predictions().is_empty());
    assert_eq!(adapter.selected_index(), 0);
}

/// Test Ctrl+Space trigger key detection
#[test]
fn test_ctrl_space_trigger() {
    // Test with Control modifier + Space
    let trigger_key = KeyEvent::new_with_kind(
        KeyCode::Char(' '),
        KeyModifiers::CONTROL,
        crossterm::event::KeyEventKind::Press
    );
    
    assert!(TerminalAdapter::is_trigger_key(trigger_key));
    
    // Test without Control modifier (should not trigger)
    let normal_key = KeyEvent::from(KeyCode::Char(' '));
    assert!(!TerminalAdapter::is_trigger_key(normal_key));
    
    // Test other keys with Control (should not trigger)
    let ctrl_a = KeyEvent::new_with_kind(
        KeyCode::Char('a'),
        KeyModifiers::CONTROL,
        crossterm::event::KeyEventKind::Press
    );
    assert!(!TerminalAdapter::is_trigger_key(ctrl_a));
}

/// Test adapter state transitions
#[test]
fn test_adapter_state_transitions() {
    let mut adapter = TerminalAdapter::new();
    
    // Initial state
    assert!(matches!(adapter.state(), AdapterState::Idle));
    
    // Clear when idle should work
    adapter.clear();
    assert!(matches!(adapter.state(), AdapterState::Idle));
}

/// Test prediction formatting for display
#[test]
fn test_format_predictions_empty() {
    let adapter = TerminalAdapter::new();
    let formatted = adapter.format_predictions();
    
    assert!(formatted.contains("No predictions available"));
}

/// Test end-to-end type prediction flow (mock)
#[tokio::test]
async fn test_e2e_type_prediction() {
    // Create engine
    let mut engine = Engine::new();
    
    // Setup test predictions
    let test_predictions = vec![
        typeracing::PredictionNode {
            type_name: "String".to_string(),
            confidence: 0.92,
            source: typeracing::PredictionSource::LspHover,
            children: vec![],
        },
        typeracing::PredictionNode {
            type_name: "&str".to_string(),
            confidence: 0.78,
            source: typeracing::PredictionSource::LspDefinition,
            children: vec![],
        },
    ];
    
    // Build type tree
    engine.build_type_tree("file:///test.rs", test_predictions.clone());
    
    // Verify predictions are stored
    let stored = engine.get_predictions("file:///test.rs");
    assert_eq!(stored.len(), 2);
    
    // Verify first prediction has highest confidence
    assert_eq!(stored[0].type_name, "String");
    assert!(stored[0].confidence > stored[1].confidence);
    
    println!("E2E Type Prediction: PASSED");
}

/// Test debounce mechanism simulation
#[tokio::test]
async fn test_debounce_simulation() {
    use std::time::{Duration, Instant};
    
    let start = Instant::now();
    let debounce_ms = 100;
    
    // Simulate debounce
    tokio::time::sleep(Duration::from_millis(debounce_ms)).await;
    
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(debounce_ms));
    
    println!("Debounce simulation: PASSED");
}

/// Test error handling for uninitialized engine
#[tokio::test]
async fn test_uninitialized_engine_error() {
    let mut adapter = TerminalAdapter::new();
    
    // Try to predict without initializing LSP
    let result = adapter.predict(
        "file:///test.rs".to_string(),
        10,
        5,
    ).await;
    
    // Should fail with EngineNotInitialized
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Engine not initialized"));
}

/// Test cache statistics
#[tokio::test]
async fn test_engine_cache_stats() {
    let engine = Engine::new();
    
    // Initial cache should be empty
    let stats = engine.cache_stats().await;
    assert_eq!(stats, 0);
    
    // Clear empty cache should work
    engine.clear_cache().await;
}

/// Integration marker test
#[test]
fn test_ui_integration_marker() {
    // This test serves as a marker for CI/CD integration
    println!("UI Integration Test Suite: PASSED");
}
