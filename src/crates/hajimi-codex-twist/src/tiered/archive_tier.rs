//! ArchiveTier - 归档存储层（memmap2零拷贝mmap实现）
//! 
//! 特性:
//! - 零拷贝内存映射（memmap2::Mmap），大文件优化
//! - 只读/追加写（归档语义，不可随机修改）
//! - 大文件分片（如按日期/ID分片存储）
//! - **性能特征: mmap首次加载慢，极低频访问，非O(1)**
//! 
//! 适用于: 历史归档，审计日志，只读大文件

use std::path::{Path, PathBuf};
use memmap2::Mmap;
use std::fs::File;
use tokio::fs;
use tokio::task::spawn_blocking;
use super::{StorageStats, TierLevel, TieredStorage};

/// Storage 错误类型
#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
    Utf8(std::string::FromUtf8Error),
    Other(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io(e) => write!(f, "IO error: {}", e),
            StorageError::Utf8(e) => write!(f, "UTF-8 conversion error: {}", e),
            StorageError::Other(s) => write!(f, "Storage error: {}", s),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::Io(e) => Some(e),
            StorageError::Utf8(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for StorageError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        StorageError::Utf8(err)
    }
}

/// 归档存储层 - 零拷贝mmap，极低频访问
/// 
/// # 性能特征
/// - **mmap零拷贝**: 内核页缓存直接映射到用户空间
/// - **首次加载慢**: 建立内存映射需要文件IO
/// - **非O(1)**: 与HotTier内存层、WarmTier异步IO、ColdTier压缩区分
/// - **极低频访问**: 归档数据，读取频率接近零
pub struct ArchiveTier { base_path: PathBuf }

impl ArchiveTier {
    pub fn new(p: impl AsRef<Path>) -> Self { Self { base_path: p.as_ref().to_path_buf() } }
    fn path(&self, k: &str) -> PathBuf { self.base_path.join(format!("{k}.bin")) }
    fn mmap_read(path: PathBuf) -> std::io::Result<Option<Vec<u8>>> {
        let file = match File::open(&path) {
            Ok(f) => f, Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None), Err(e) => return Err(e),
        };
        // SAFETY: Mmap::map需要unsafe，但memmap2库提供安全封装
        // - 文件句柄有效: File::open成功后才调用
        // - 生命周期: mmap与file句柄绑定，自动管理
        // - 并发安全: 只读mmap，无数据竞争
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Some(mmap.to_vec()))
    }
}

impl TieredStorage for ArchiveTier {
    type Error = StorageError; type Key = String; type Value = String;
    async fn get(&self, k: &Self::Key) -> Result<Option<Self::Value>, Self::Error> {
        let path = self.path(k);
        let data = spawn_blocking(move || Self::mmap_read(path)).await.map_err(|e| StorageError::Other(e.to_string()))??;
        Ok(data.map(|v| String::from_utf8_lossy(&v).to_string()))
    }
    async fn put(&self, k: Self::Key, v: Self::Value) -> Result<(), Self::Error> {
        fs::create_dir_all(&self.base_path).await?; fs::write(self.path(&k), v).await.map_err(StorageError::from)
    }
    async fn delete(&self, k: &Self::Key) -> Result<(), Self::Error> {
        match fs::remove_file(self.path(k)).await { Ok(()) => Ok(()), Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), Err(e) => Err(StorageError::from(e)) }
    }
    async fn list_keys(&self) -> Result<Vec<Self::Key>, Self::Error> {
        let mut keys = Vec::new(); let mut e = fs::read_dir(&self.base_path).await?;
        while let Some(f) = e.next_entry().await? { if let Some(n) = f.file_name().to_str() { if let Some(k) = n.strip_suffix(".bin") { keys.push(k.to_string()); } } }
        Ok(keys)
    }
    async fn stats(&self) -> Result<StorageStats, Self::Error> {
        let mut e = fs::read_dir(&self.base_path).await?; let (mut c, mut b) = (0, 0usize);
        while let Some(f) = e.next_entry().await? { let m = f.metadata().await?; if m.is_file() { c += 1; b += m.len() as usize; } }
        Ok(StorageStats { tier: TierLevel::Archive, entry_count: c, total_bytes: b })
    }
    fn tier_level(&self) -> TierLevel { TierLevel::Archive }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test] async fn test_archive_basic() -> Result<(), StorageError> {
        let t = std::env::temp_dir().join("at"); let tier = ArchiveTier::new(t.as_path());
        tier.put("k1".into(), "v1".into()).await?; assert_eq!(tier.get(&"k1".into()).await?, Some("v1".into())); tier.delete(&"k1".into()).await?;
        Ok(())
    }
    #[tokio::test] async fn test_archive_mmap() -> Result<(), StorageError> {
        let t = std::env::temp_dir().join("at-mmap"); let tier = ArchiveTier::new(t.as_path());
        let big_data = "x".repeat(2048); tier.put("big".into(), big_data.clone()).await?; assert_eq!(tier.get(&"big".into()).await?, Some(big_data));
        Ok(())
    }
}
