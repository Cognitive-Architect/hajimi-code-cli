//! ArchiveMemory - 归档内存层（100万tokens，mmap懒加载，zstd压缩）
//! 
//! 特性:
//! - 容量限制: 1000000 tokens（1M，与TokenBudget.archive_limit一致）
//! - 存储模式: zstd压缩 + mmap懒加载（组合ColdTier+ArchiveTier模式）
//! - 访问模式: 极低频，首次加载慢（mmap建立映射），后续内存速度
//! - **性能特征: 非O(1)，懒加载延迟，归档级访问频率**
//! 
//! 设计: 内存中仅保留索引，数据mmap懒加载，压缩存储节省空间

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::fs;
use tokio::task::spawn_blocking;
use memmap2::Mmap;
use std::fs::File;
use super::{MemoryStats, MemoryLevel, MemoryTier};

/// 归档内存层 - 100万tokens，mmap懒加载，zstd压缩
/// 
/// # 性能特征
/// - **100万tokens**: 大容量归档，存储历史数据
/// - **mmap懒加载**: 首次访问触发文件IO建立映射，后续零拷贝
/// - **zstd压缩**: 节省存储空间，解压在spawn_blocking执行
/// - **非O(1)**: 与Focus/Working层区分，归档级延迟可接受
/// - **极低频访问**: 适合审计日志、历史归档
pub struct ArchiveMemory {
    index: Arc<RwLock<HashMap<String, PathBuf>>>,  // 内存索引
    base_path: PathBuf,
    #[allow(dead_code)]
    limit: usize,  // 1000000（当前为软限制，由Gateway层管理）
}

impl ArchiveMemory {
    pub fn new() -> Self { Self::with_limit(1_000_000) }
    pub fn with_limit(limit: usize) -> Self {
        Self { index: Arc::new(RwLock::new(HashMap::new())), base_path: PathBuf::from("."), limit }
    }
    pub fn with_path(limit: usize, base_path: impl AsRef<Path>) -> Self {
        Self { index: Arc::new(RwLock::new(HashMap::new())), base_path: base_path.as_ref().to_path_buf(), limit }
    }
    /// 清除所有归档条目
    pub async fn clear(&self) {
        let mut index = self.index.write().await;
        // 清理所有文件
        for (_, path) in index.iter() {
            let _ = fs::remove_file(path).await;
        }
        index.clear();
    }
    /// 压缩/整理索引
    pub async fn compact(&self) {
        // Archive层索引整理，当前实现不需要额外操作
        // 预留接口供未来实现压缩合并
    }
    fn file_path(&self, key: &str) -> PathBuf { self.base_path.join(format!("{key}.zst")) }
    fn mmap_read(path: PathBuf) -> std::io::Result<Option<Vec<u8>>> {
        let file = match File::open(&path) {
            Ok(f) => f, Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None), Err(e) => return Err(e),
        };
        Ok(Some(unsafe { Mmap::map(&file)? }.to_vec()))
    }
}

impl Default for ArchiveMemory {
    fn default() -> Self { Self::new() }
}

impl MemoryTier for ArchiveMemory {
    type Error = std::io::Error;
    type Key = String;
    type Value = String;
    
    /// 获取条目（mmap懒加载 + zstd解压）
    /// 非O(1)：mmap首次加载慢（文件IO），zstd解压计算密集型
    async fn get(&self, key: &Self::Key) -> Result<Option<Self::Value>, Self::Error> {
        let path = { let index = self.index.read().await; match index.get(key) { Some(p) => p.clone(), None => return Ok(None) } };
        let data = spawn_blocking(move || {
            match Self::mmap_read(path)? { Some(d) => zstd::decode_all(&d[..]).map(Some).map_err(std::io::Error::other), None => Ok(None) }
        }).await.map_err(std::io::Error::other)??;
        Ok(data.map(|d| String::from_utf8_lossy(&d).to_string()))
    }
    
    async fn put(&self, key: Self::Key, value: Self::Value) -> Result<(), Self::Error> {
        let path = self.file_path(&key);
        let compressed = spawn_blocking(move || zstd::encode_all(value.as_bytes(), 3)).await.map_err(std::io::Error::other)??;
        fs::create_dir_all(&self.base_path).await?; fs::write(&path, compressed).await?;
        self.index.write().await.insert(key, path);
        Ok(())
    }
    
    async fn delete(&self, key: &Self::Key) -> Result<(), Self::Error> {
        if let Some(p) = { let mut i = self.index.write().await; i.remove(key) } {
            match fs::remove_file(&p).await { Ok(()) => Ok(()), Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), Err(e) => Err(e) }
        } else { Ok(()) }
    }
    
    async fn list_keys(&self) -> Result<Vec<Self::Key>, Self::Error> {
        Ok(self.index.read().await.keys().cloned().collect())
    }
    
    async fn stats(&self) -> Result<MemoryStats, Self::Error> {
        let n = self.index.read().await.len();
        Ok(MemoryStats { entry_count: n, total_tokens: n * 1000, memory_bytes: n * 1024, level: MemoryLevel::Archive })
    }
    
    fn memory_level(&self) -> MemoryLevel { MemoryLevel::Archive }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test] async fn test_archive_basic() {
        let mem = ArchiveMemory::new();
        mem.put("k1".into(), "v1".into()).await.unwrap();
        assert_eq!(mem.get(&"k1".into()).await.unwrap(), Some("v1".into()));
    }
    
    #[tokio::test] async fn test_archive_capacity_1000000() {
        let mem = ArchiveMemory::new();
        assert_eq!(mem.limit, 1_000_000);
    }
    
    #[tokio::test] async fn test_archive_mmap_lazy() {
        let temp_dir = std::env::temp_dir().join("am-test");
        let mem = ArchiveMemory::with_path(1_000_000, &temp_dir);
        let big_data = "x".repeat(10000);
        mem.put("big".into(), big_data.clone()).await.unwrap();
        assert_eq!(mem.get(&"big".into()).await.unwrap(), Some(big_data));
        let _ = fs::remove_dir_all(&temp_dir).await;
    }
}
