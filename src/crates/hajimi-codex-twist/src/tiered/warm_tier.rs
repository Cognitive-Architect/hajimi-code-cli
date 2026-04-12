//! WarmTier - 温存储层（SSD异步IO实现）
//! 
//! 特性:
//! - 异步IO（tokio::fs），非阻塞
//! - SSD持久化，毫秒级延迟
//! - 目录自动创建
//! - **性能特征: 异步IO，非O(1)**

use std::path::{Path, PathBuf};
use tokio::fs;
use super::{StorageStats, TierLevel, TieredStorage};

/// Storage 错误类型
#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::Io(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err)
    }
}

/// 温存储层 - 异步SSD存储，非O(1)
pub struct WarmTier { base_path: PathBuf }

impl WarmTier {
    pub fn new(p: impl AsRef<Path>) -> Self { Self { base_path: p.as_ref().to_path_buf() } }
    fn path(&self, k: &str) -> PathBuf { self.base_path.join(format!("{k}.json")) }
}

impl TieredStorage for WarmTier {
    type Error = StorageError; type Key = String; type Value = String;
    
    async fn get(&self, k: &Self::Key) -> Result<Option<Self::Value>, Self::Error> {
        match fs::read_to_string(self.path(k)).await {
            Ok(v) => Ok(Some(v)), Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None), Err(e) => Err(StorageError::from(e)),
        }
    }
    
    async fn put(&self, k: Self::Key, v: Self::Value) -> Result<(), Self::Error> {
        fs::create_dir_all(&self.base_path).await?; fs::write(self.path(&k), v).await.map_err(StorageError::from)
    }
    
    async fn delete(&self, k: &Self::Key) -> Result<(), Self::Error> {
        match fs::remove_file(self.path(k)).await {
            Ok(()) => Ok(()), Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), Err(e) => Err(StorageError::from(e)),
        }
    }
    
    async fn list_keys(&self) -> Result<Vec<Self::Key>, Self::Error> {
        let mut keys = Vec::new(); let mut e = fs::read_dir(&self.base_path).await?;
        while let Some(f) = e.next_entry().await? {
            if let Some(n) = f.file_name().to_str() { 
                if let Some(k) = n.strip_suffix(".json") { keys.push(k.to_string()); }
            }
        } Ok(keys)
    }
    
    async fn stats(&self) -> Result<StorageStats, Self::Error> {
        let mut e = fs::read_dir(&self.base_path).await?; let (mut c, mut b) = (0, 0usize);
        while let Some(f) = e.next_entry().await? { let m = f.metadata().await?; if m.is_file() { c += 1; b += m.len() as usize; } }
        Ok(StorageStats { tier: TierLevel::Warm, entry_count: c, total_bytes: b })
    }
    
    fn tier_level(&self) -> TierLevel { TierLevel::Warm }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test] async fn test_basic() -> Result<(), StorageError> {
        let t = std::env::temp_dir().join("wt"); let tier = WarmTier::new(t.as_path());
        tier.put("k1".into(), "v1".into()).await?;
        assert_eq!(tier.get(&"k1".into()).await?, Some("v1".into()));
        tier.delete(&"k1".into()).await?;
        Ok(())
    }
}
