//! ColdTier - 冷存储层（zstd压缩，计算密集型，非O(1)）
use super::{StorageStats, TierLevel, TieredStorage};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::task::spawn_blocking;
/// 冷存储层 - zstd压缩，计算密集型
pub struct ColdTier {
    base_path: PathBuf,
    level: i32,
}
impl ColdTier {
    pub fn new(p: impl AsRef<Path>) -> Self {
        Self {
            base_path: p.as_ref().to_path_buf(),
            level: 3,
        }
    }
    pub fn with_level(p: impl AsRef<Path>, l: i32) -> Self {
        Self {
            base_path: p.as_ref().to_path_buf(),
            level: l.clamp(1, 22),
        }
    }
    fn path(&self, k: &str) -> PathBuf {
        self.base_path.join(format!("{k}.zst"))
    }
}
impl TieredStorage for ColdTier {
    type Error = std::io::Error;
    type Key = String;
    type Value = String;
    async fn get(&self, k: &Self::Key) -> Result<Option<Self::Value>, Self::Error> {
        let c = match fs::read(self.path(k)).await {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        let d = spawn_blocking(move || zstd::decode_all(&c[..]))
            .await
            .map_err(std::io::Error::other)??;
        Ok(Some(String::from_utf8_lossy(&d).to_string()))
    }
    async fn put(&self, k: Self::Key, v: Self::Value) -> Result<(), Self::Error> {
        fs::create_dir_all(&self.base_path).await?;
        let l = self.level;
        let c = spawn_blocking(move || zstd::encode_all(v.as_bytes(), l))
            .await
            .map_err(std::io::Error::other)??;
        fs::write(self.path(&k), c).await
    }
    async fn delete(&self, k: &Self::Key) -> Result<(), Self::Error> {
        match fs::remove_file(self.path(k)).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }
    async fn list_keys(&self) -> Result<Vec<Self::Key>, Self::Error> {
        let mut keys = Vec::new();
        let mut e = fs::read_dir(&self.base_path).await?;
        while let Some(f) = e.next_entry().await? {
            if let Some(n) = f.file_name().to_str() {
                if let Some(k) = n.strip_suffix(".zst") {
                    keys.push(k.to_string());
                }
            }
        }
        Ok(keys)
    }
    async fn stats(&self) -> Result<StorageStats, Self::Error> {
        let mut e = fs::read_dir(&self.base_path).await?;
        let (mut c, mut b) = (0, 0usize);
        while let Some(f) = e.next_entry().await? {
            let m = f.metadata().await?;
            if m.is_file() {
                c += 1;
                b += m.len() as usize;
            }
        }
        Ok(StorageStats {
            tier: TierLevel::Cold,
            entry_count: c,
            total_bytes: b,
        })
    }
    fn tier_level(&self) -> TierLevel {
        TierLevel::Cold
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_basic() {
        let t = std::env::temp_dir().join("ct-test");
        let tier = ColdTier::new(t.as_path());
        tier.put("k1".into(), "v1".into()).await.unwrap();
        assert_eq!(tier.get(&"k1".into()).await.unwrap(), Some("v1".into()));
        tier.delete(&"k1".into()).await.unwrap();
    }
    #[tokio::test]
    async fn test_level() {
        let t = std::env::temp_dir().join("ct-lvl");
        let tier = ColdTier::with_level(t.as_path(), 1);
        tier.put("k1".into(), "v1".into()).await.unwrap();
        assert_eq!(tier.get(&"k1".into()).await.unwrap(), Some("v1".into()));
    }
}
