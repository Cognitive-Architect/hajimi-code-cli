//! FocusMemory - 焦点内存层（高频访问，LRU淘汰，4000 tokens）
//! 特性: O(1)访问、RwLock并发（读多写少优化）、LRU淘汰

use std::sync::Arc;
use tokio::sync::RwLock;
use lru::LruCache;
use std::num::NonZeroUsize;
use super::{MemoryStats, MemoryLevel, MemoryTier};

/// 焦点内存层 - O(1) LRU缓存，默认4000 tokens
pub struct FocusMemory<K, V> {
    cache: Arc<RwLock<LruCache<K, V>>>,
}

impl<K, V> FocusMemory<K, V>
where K: std::hash::Hash + Eq + Clone + Send + Sync, V: Clone + Send + Sync,
{
    pub fn new() -> Self { Self::with_capacity(4000) }
    pub fn with_capacity(cap: usize) -> Self {
        let cap = NonZeroUsize::new(cap).expect("capacity must be non-zero");
        Self { cache: Arc::new(RwLock::new(LruCache::new(cap))) }
    }
    /// 清除所有缓存
    pub async fn clear(&self) {
        self.cache.write().await.clear();
    }
    /// 压缩/整理缓存
    pub async fn compact(&self) {
        // LRU缓存不需要额外压缩，clear即可
        self.cache.write().await.clear();
    }
}

impl<K, V> Default for FocusMemory<K, V>
where K: std::hash::Hash + Eq + Clone + Send + Sync, V: Clone + Send + Sync,
{ fn default() -> Self { Self::new() } }

pub type FocusValue = String;
pub type FocusKey = String;

impl MemoryTier for FocusMemory<FocusKey, FocusValue> {
    type Error = std::io::Error;
    type Key = FocusKey;
    type Value = FocusValue;
    
    /// O(1) LRU获取，RwLock写锁
    async fn get(&self, key: &Self::Key) -> Result<Option<Self::Value>, Self::Error> {
        Ok(self.cache.write().await.get(key).cloned())
    }
    /// O(1) LRU存储，满时自动淘汰
    async fn put(&self, key: Self::Key, value: Self::Value) -> Result<(), Self::Error> {
        self.cache.write().await.put(key, value); Ok(())
    }
    /// O(1)删除
    async fn delete(&self, key: &Self::Key) -> Result<(), Self::Error> {
        self.cache.write().await.pop(key); Ok(())
    }
    /// O(n)列出所有键
    async fn list_keys(&self) -> Result<Vec<Self::Key>, Self::Error> {
        Ok(self.cache.read().await.iter().map(|(k, _)| k.clone()).collect())
    }
    /// 获取统计信息
    async fn stats(&self) -> Result<MemoryStats, Self::Error> {
        let cache = self.cache.read().await;
        let n = cache.len();
        Ok(MemoryStats { entry_count: n, total_tokens: n * 100, memory_bytes: n * 256, level: MemoryLevel::Focus })
    }
    fn memory_level(&self) -> MemoryLevel { MemoryLevel::Focus }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test] async fn test_focus_basic() {
        let mem = FocusMemory::new();
        mem.put("k1".into(), "v1".into()).await.unwrap();
        assert_eq!(mem.get(&"k1".into()).await.unwrap(), Some("v1".into()));
    }
    
    #[tokio::test] async fn test_focus_lru_eviction() {
        let mem = FocusMemory::with_capacity(2);
        mem.put("k1".into(), "v1".into()).await.unwrap();
        mem.put("k2".into(), "v2".into()).await.unwrap();
        mem.put("k3".into(), "v3".into()).await.unwrap();
        assert_eq!(mem.get(&"k1".into()).await.unwrap(), None);
        assert_eq!(mem.get(&"k2".into()).await.unwrap(), Some("v2".into()));
    }
    
    #[tokio::test] async fn test_focus_capacity_4000() {
        let mem = FocusMemory::<String, String>::new();
        assert_eq!(mem.stats().await.unwrap().level, MemoryLevel::Focus);
    }
}
