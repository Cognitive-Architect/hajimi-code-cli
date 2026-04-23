//! TypeRacing Terminal Adapter - Ctrl+Space trigger and Engine integration
//!
//! Provides terminal input handling for TypeRacing predictions with:
//! - Ctrl+Space hotkey binding
//! - Async Engine prediction calls
//! - Result display in terminal UI

use crate::engine::{Engine, PredictionNode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Terminal adapter errors
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("Engine not initialized")]
    EngineNotInitialized,
    #[error("Prediction failed: {0}")]
    PredictionFailed(String),
    #[error("LSP error: {0}")]
    LspError(String),
}

pub type AdapterResult<T> = Result<T, AdapterError>;

/// Terminal adapter state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterState {
    Idle,
    Predicting,
    ShowingResults,
}

/// Terminal adapter for TypeRacing integration
pub struct TerminalAdapter {
    engine: Arc<Mutex<Engine>>,
    state: AdapterState,
    last_predictions: Vec<PredictionNode>,
    selected_index: usize,
    lsp_initialized: bool,
}

impl TerminalAdapter {
    /// Create new terminal adapter instance
    pub fn new() -> Self {
        Self {
            engine: Arc::new(Mutex::new(Engine::new())),
            state: AdapterState::Idle,
            last_predictions: Vec::new(),
            selected_index: 0,
            lsp_initialized: false,
        }
    }

    /// Initialize LSP connection
    pub async fn init_lsp(&mut self, cmd: &str, args: Vec<String>) -> AdapterResult<()> {
        let mut engine = self.engine.lock().await;
        engine
            .init(cmd, args)
            .await
            .map_err(|e| AdapterError::LspError(e.to_string()))?;
        self.lsp_initialized = true;
        Ok(())
    }

    /// Check if Ctrl+Space was pressed
    pub fn is_trigger_key(key: KeyEvent) -> bool {
        key.code == KeyCode::Char(' ')
            && key.modifiers.contains(KeyModifiers::CONTROL)
    }

    /// Handle key events for prediction navigation
    pub fn handle_key(&mut self, key: KeyEvent) -> AdapterResult<Option<String>> {
        if Self::is_trigger_key(key) {
            return Ok(Some(String::new())); // Signal to spawn prediction
        }

        match self.state {
            AdapterState::ShowingResults => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    Ok(None)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected_index < self.last_predictions.len().saturating_sub(1) {
                        self.selected_index += 1;
                    }
                    Ok(None)
                }
                KeyCode::Enter => {
                    if let Some(pred) = self.last_predictions.get(self.selected_index) {
                        let result = pred.type_name.clone();
                        self.state = AdapterState::Idle;
                        Ok(Some(result))
                    } else {
                        Ok(None)
                    }
                }
                KeyCode::Esc => {
                    self.state = AdapterState::Idle;
                    Ok(None)
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    /// Spawn prediction task (non-blocking)
    pub async fn spawn_predict(
        &mut self,
        uri: String,
        line: u32,
        character: u32,
    ) -> AdapterResult<()> {
        if !self.lsp_initialized {
            return Err(AdapterError::EngineNotInitialized);
        }

        self.state = AdapterState::Predicting;
        
        let engine = Arc::clone(&self.engine);
        
        // Spawn async prediction task
        tokio::spawn(async move {
            let engine_guard = engine.lock().await;
            let handle = engine_guard.predict(uri, line, character);
            drop(engine_guard); // Release lock while awaiting
            
            match handle.await {
                Ok(Ok(predictions)) => Ok(predictions),
                Ok(Err(e)) => Err(AdapterError::PredictionFailed(e.to_string())),
                Err(e) => Err(AdapterError::PredictionFailed(e.to_string())),
            }
        });

        Ok(())
    }

    /// Execute prediction and wait for results
    pub async fn predict(
        &mut self,
        uri: String,
        line: u32,
        character: u32,
    ) -> AdapterResult<Vec<PredictionNode>> {
        if !self.lsp_initialized {
            return Err(AdapterError::EngineNotInitialized);
        }

        self.state = AdapterState::Predicting;

        let engine = self.engine.lock().await;
        let handle = engine.predict(uri.clone(), line, character);
        drop(engine);

        match handle.await {
            Ok(Ok(predictions)) => {
                self.last_predictions = predictions.clone();
                self.selected_index = 0;
                self.state = if predictions.is_empty() {
                    AdapterState::Idle
                } else {
                    AdapterState::ShowingResults
                };
                Ok(predictions)
            }
            Ok(Err(e)) => {
                self.state = AdapterState::Idle;
                Err(AdapterError::PredictionFailed(e.to_string()))
            }
            Err(e) => {
                self.state = AdapterState::Idle;
                Err(AdapterError::PredictionFailed(e.to_string()))
            }
        }
    }

    /// Get current predictions for display
    pub fn get_predictions(&self) -> &[PredictionNode] {
        &self.last_predictions
    }

    /// Get selected prediction index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get current adapter state
    pub fn state(&self) -> AdapterState {
        self.state
    }

    /// Check if LSP is initialized
    pub fn is_initialized(&self) -> bool {
        self.lsp_initialized
    }

    /// Format predictions for terminal display
    pub fn format_predictions(&self) -> String {
        if self.last_predictions.is_empty() {
            return "No predictions available".to_string();
        }

        let mut output = String::from("TypeRacing Predictions:\n");
        for (i, pred) in self.last_predictions.iter().enumerate() {
            let marker = if i == self.selected_index { "> " } else { "  " };
            let source_icon = match pred.source {
                crate::engine::PredictionSource::LspHover => "H",
                crate::engine::PredictionSource::LspDefinition => "D",
                crate::engine::PredictionSource::LspReferences => "R",
                crate::engine::PredictionSource::Heuristic => "?",
                crate::engine::PredictionSource::Historical => "@",
            };
            output.push_str(&format!(
                "{}{} {} ({:.0}%)\n",
                marker, source_icon, pred.type_name, pred.confidence * 100.0
            ));
        }
        output.push_str("\n[Enter] Select  [Esc] Cancel  [↑↓] Navigate");
        output
    }

    /// Clear prediction results
    pub fn clear(&mut self) {
        self.last_predictions.clear();
        self.selected_index = 0;
        self.state = AdapterState::Idle;
    }
}

impl Default for TerminalAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_key_detection() {
        let trigger = KeyEvent::from(KeyCode::Char(' '));
        // Note: This test would need actual modifier setup
        // Just verifying the function exists and compiles
        assert!(!TerminalAdapter::is_trigger_key(trigger));
    }

    #[test]
    fn test_initial_state() {
        let adapter = TerminalAdapter::new();
        assert!(matches!(adapter.state(), AdapterState::Idle));
        assert!(!adapter.is_initialized());
        assert!(adapter.get_predictions().is_empty());
    }
}
