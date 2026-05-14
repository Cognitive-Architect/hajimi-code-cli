//! ADR目录监听（基于notify crate）
use crate::core_adr::{AdrError, Result};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::time::Duration;

pub struct AdrWatcher {
    _watcher: RecommendedWatcher,
}

impl AdrWatcher {
    pub async fn watch_adr_dir<F>(dir: &Path, callback: F) -> Result<Self>
    where
        F: Fn(notify::Event) + Send + 'static,
    {
        if !dir.exists() {
            std::fs::create_dir_all(dir).map_err(AdrError::Io)?;
        }
        let watcher = RecommendedWatcher::new(
            move |res: std::result::Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    callback(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )
        .map_err(|e| AdrError::Io(std::io::Error::other(e)))?;
        let mut watcher = watcher;
        watcher
            .watch(dir, RecursiveMode::NonRecursive)
            .map_err(|e| AdrError::Io(std::io::Error::other(e)))?;
        Ok(Self { _watcher: watcher })
    }
}

impl Drop for AdrWatcher {
    fn drop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_watcher_new() -> Result<()> {
        let tmp = TempDir::new()?;
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        let _watcher = AdrWatcher::watch_adr_dir(tmp.path(), move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        })
        .await?;
        sleep(Duration::from_millis(100)).await;
        assert!(tmp.path().exists());
        Ok(())
    }
}
