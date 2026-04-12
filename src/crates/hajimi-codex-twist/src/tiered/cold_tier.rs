//! ColdTier - 冷存储层（zstd压缩，计算密集型，非O(1)）
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::task::spawn_blocking;
use super::{StorageStats, TierLevel, TieredStorage};

/// Storage 错误类型
#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
    Compression(String),
    Other(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io(e) => write!(f, "IO error: {}", e),
            StorageError::Compression(s) => write!(f, "Compression error: {}", s),
            StorageError::Other(s) => write!(f, "Storage error: {}", s),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err)
    }
}

/// 冷存储层 - zstd压缩，计算密集型
pub struct ColdTier { base_path: PathBuf, level: i32 }
impl ColdTier {
    pub fn new(p: impl AsRef<Path>) -> Self { Self { base_path: p.as_ref().to_path_buf(), level: 3 } }
    pub fn with_level(p: impl AsRef<Path>, l: i32) -> Self { Self { base_path: p.as_ref().to_path_buf(), level: l.clamp(1,22) } }
    fn path(&self, k: &str) -> PathBuf { self.base_path.join(format!("{k}.zst")) }
}
impl TieredStorage for ColdTier {
    type Error = StorageError; type Key = String; type Value = String;
    async fn get(&self, k: &Self::Key) -> Result<Option<Self::Value>, Self::Error> {
        let c = match fs::read(self.path(k)).await { Ok(d)=>d, Err(e) if e.kind()==std::io::ErrorKind::NotFound=>return Ok(None), Err(e)=>return Err(StorageError::from(e)) };
        let d = spawn_blocking(move||zstd::decode_all(&c[..])).await.map_err(|e| StorageError::Other(e.to_string()))?;
        let d = d.map_err(|e| StorageError::Compression(e.to_string()))?;
        Ok(Some(String::from_utf8_lossy(&d).to_string()))
    }
    async fn put(&self, k: Self::Key, v: Self::Value) -> Result<(), Self::Error> {
        fs::create_dir_all(&self.base_path).await?;
        let l = self.level; 
        let c = spawn_blocking(move||zstd::encode_all(v.as_bytes(),l)).await.map_err(|e| StorageError::Other(e.to_string()))?;
        let c = c.map_err(|e| StorageError::Compression(e.to_string()))?;
        fs::write(self.path(&k), c).await.map_err(StorageError::from)
    }
    async fn delete(&self, k: &Self::Key) -> Result<(), Self::Error> {
        match fs::remove_file(self.path(k)).await { Ok(())=>Ok(()), Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(()), Err(e)=>Err(StorageError::from(e)) }
    }
    async fn list_keys(&self) -> Result<Vec<Self::Key>, Self::Error> {
        let mut keys = Vec::new(); let mut e = fs::read_dir(&self.base_path).await?;
        while let Some(f) = e.next_entry().await? { 
            if let Some(n)=f.file_name().to_str() { 
                if let Some(k)=n.strip_suffix(".zst") { keys.push(k.to_string()); }
            }
        } Ok(keys)
    }
    async fn stats(&self) -> Result<StorageStats, Self::Error> {
        let mut e = fs::read_dir(&self.base_path).await?; let (mut c,mut b)=(0,0usize);
        while let Some(f)=e.next_entry().await? { let m=f.metadata().await?; if m.is_file(){c+=1;b+=m.len() as usize;} }
        Ok(StorageStats{ tier: TierLevel::Cold, entry_count: c, total_bytes: b })
    }
    fn tier_level(&self) -> TierLevel { TierLevel::Cold }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test] async fn test_basic() -> Result<(), StorageError> {
        let t = std::env::temp_dir().join("ct-test"); let tier = ColdTier::new(t.as_path());
        tier.put("k1".into(), "v1".into()).await?; assert_eq!(tier.get(&"k1".into()).await?, Some("v1".into())); tier.delete(&"k1".into()).await?;
        Ok(())
    }
    #[tokio::test] async fn test_level() -> Result<(), StorageError> {
        let t = std::env::temp_dir().join("ct-lvl"); let tier = ColdTier::with_level(t.as_path(), 1);
        tier.put("k1".into(), "v1".into()).await?; assert_eq!(tier.get(&"k1".into()).await?, Some("v1".into()));
        Ok(())
    }
}
