//! Hot reload support for configuration files
//!
//! B-W05-03: File watching + debounced reload trigger

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use notify::{RecommendedWatcher, Config as NotifyConfig, Event, RecursiveMode, Watcher};
use tokio::sync::{mpsc, RwLock};
use crate::config::{Config, ConfigError, ConfigLoader};

/// Debounce duration for file change events
const DEBOUNCE_MS: u64 = 500;

/// Hot reload handle with watcher and event channel
pub struct HotReloadHandle {
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
    rx: mpsc::Receiver<Event>,
}

impl HotReloadHandle {
    /// Start watching config file for changes
    pub async fn watch(path: &Path) -> Result<Self, ConfigError> {
        let (tx, rx) = mpsc::channel(16);
        let path = path.to_path_buf();
        
        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.try_send(event);
                }
            },
            NotifyConfig::default()
                .with_poll_interval(Duration::from_millis(DEBOUNCE_MS)),
        ).map_err(|e| ConfigError::Io(format!("Watcher: {}", e)))?;
        
        let mut handle = Self { watcher, rx };
        handle.watcher.watch(&path, RecursiveMode::NonRecursive)
            .map_err(|e| ConfigError::Io(format!("Watch: {}", e)))?;
        
        Ok(handle)
    }
    
    /// Wait for change event with debouncing
    pub async fn wait_for_change(&mut self) -> Option<Event> {
        tokio::time::timeout(
            Duration::from_secs(3),
            self.rx.recv()
        ).await.ok().flatten()
    }
}

/// Config manager with hot reload support
pub struct ConfigManager {
    inner: Arc<RwLock<Config>>,
    pub(crate) config_path: Option<std::path::PathBuf>,
}

impl ConfigManager {
    /// Create new config manager
    pub fn new(config: Config) -> Self {
        Self {
            inner: Arc::new(RwLock::new(config)),
            config_path: None,
        }
    }
    
    /// Set config path
    pub fn set_path(&mut self, path: std::path::PathBuf) {
        self.config_path = Some(path);
    }
    
    /// Get current config
    pub async fn get(&self) -> Config {
        self.inner.read().await.clone()
    }
    
    /// Reload config from file
    pub async fn reload(&self) -> Result<(), ConfigError> {
        let path = self.config_path.as_ref()
            .ok_or_else(|| ConfigError::MissingField("config_path".into()))?;
        let new_config = ConfigLoader::from_file(path).await?;
        let mut guard = self.inner.write().await;
        *guard = new_config;
        Ok(())
    }
    
    /// Enable hot reload
    pub async fn enable_hot_reload(&mut self) -> Result<HotReloadHandle, ConfigError> {
        let path = self.config_path.as_ref()
            .ok_or_else(|| ConfigError::MissingField("config_path".into()))?;
        HotReloadHandle::watch(path).await
    }
    
    /// Handle config changed trigger
    pub async fn on_config_changed(&self) -> Result<(), ConfigError> {
        self.reload().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hot_reload_watch() {
        let temp = std::env::temp_dir().join("test_hr_");
        std::fs::write(&temp, "preset = \"Daily\"").unwrap();
        
        let result = HotReloadHandle::watch(&temp).await;
        assert!(result.is_ok());
        
        let _ = std::fs::remove_file(&temp);
    }
}
