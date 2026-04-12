//! WorkingMemory - 工作内存层（滑动窗口，16000 tokens，持久化可选）
//! 
//! 性能特征: 非O(1)，滑动窗口扫描+BTreeMap操作，可选持久化延迟

use std::sync::Arc;
use std::collections::BTreeMap;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::fs;
use super::{MemoryStats, MemoryLevel, MemoryTier};

/// 工作内存条目（时间戳用于滑动窗口）
#[derive(Clone)]
pub struct WorkingEntry { pub value: String, pub tokens: usize, pub created_at: Instant }

/// 工作内存层 - 滑动窗口淘汰，16000 tokens
pub struct WorkingMemory {
    entries: Arc<RwLock<BTreeMap<Instant, (String, WorkingEntry)>>>,
    total_tokens: Arc<RwLock<usize>>,
    pub limit: usize,
    base_path: Option<std::path::PathBuf>,
}

impl WorkingMemory {
    /// per-instance limit: 16000 tokens
    /// Gateway层管理 total budget: 32000 (可创建2个WorkingMemory实例)
    pub fn new() -> Self { Self::with_limit(16000) }
    pub fn with_limit(limit: usize) -> Self {
        Self { entries: Arc::new(RwLock::new(BTreeMap::new())), total_tokens: Arc::new(RwLock::new(0)), limit, base_path: None }
    }
    pub fn with_persistence(limit: usize, base_path: impl AsRef<std::path::Path>) -> Self {
        Self { entries: Arc::new(RwLock::new(BTreeMap::new())), total_tokens: Arc::new(RwLock::new(0)), limit, base_path: Some(base_path.as_ref().to_path_buf()) }
    }
    /// 清除所有条目
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        let mut total = self.total_tokens.write().await;
        // 清理持久化文件
        if let Some(ref path) = self.base_path {
            for (_, (k, _)) in entries.iter() {
                let _ = fs::remove_file(path.join(format!("{k}.json"))).await;
            }
        }
        entries.clear();
        *total = 0;
    }
    /// 压缩/整理（触发淘汰）
    pub async fn compact(&self) {
        self.evict_if_needed().await;
    }
    /// 滑动窗口淘汰 - 删除最旧条目直到total_tokens ≤ limit
    async fn evict_if_needed(&self) {
        let mut entries = self.entries.write().await;
        let mut total = self.total_tokens.write().await;
        while *total > self.limit && !entries.is_empty() {
            if let Some((oldest, (_, entry))) = entries.pop_first() {
                *total = total.saturating_sub(entry.tokens);
                if let Some(ref path) = self.base_path { let _ = fs::remove_file(path.join(format!("{oldest:?}.json"))).await; }
            }
        }
    }
    fn estimate_tokens(s: &str) -> usize { s.len() / 4 + 1 }
}

impl Default for WorkingMemory { fn default() -> Self { Self::new() } }

impl MemoryTier for WorkingMemory {
    type Error = std::io::Error; type Key = String; type Value = String;
    
    async fn get(&self, key: &Self::Key) -> Result<Option<Self::Value>, Self::Error> {
        let entries = self.entries.read().await;
        for (_, (k, entry)) in entries.iter() { if k == key { return Ok(Some(entry.value.clone())); } }
        Ok(None)
    }
    
    async fn put(&self, key: Self::Key, value: Self::Value) -> Result<(), Self::Error> {
        let tokens = Self::estimate_tokens(&value);
        let entry = WorkingEntry { value: value.clone(), tokens, created_at: Instant::now() };
        let created_at = entry.created_at;
        self.evict_if_needed().await;
        { let mut entries = self.entries.write().await; let mut total = self.total_tokens.write().await; entries.insert(created_at, (key.clone(), entry)); *total += tokens; }
        if let Some(ref path) = self.base_path { fs::create_dir_all(path).await?; fs::write(path.join(format!("{key}.json")), value).await?; }
        Ok(())
    }
    
    async fn delete(&self, key: &Self::Key) -> Result<(), Self::Error> {
        let mut entries = self.entries.write().await; let mut total = self.total_tokens.write().await;
        let to_remove: Vec<Instant> = entries.iter().filter(|(_, (k, _))| k == key).map(|(t, _)| *t).collect();
        for t in to_remove { if let Some((_, entry)) = entries.remove(&t) { *total = total.saturating_sub(entry.tokens); if let Some(ref path) = self.base_path { let _ = fs::remove_file(path.join(format!("{key}.json"))).await; } } }
        Ok(())
    }
    
    async fn list_keys(&self) -> Result<Vec<Self::Key>, Self::Error> {
        let entries = self.entries.read().await;
        Ok(entries.values().map(|(k, _)| k.clone()).collect())
    }
    
    async fn stats(&self) -> Result<MemoryStats, Self::Error> {
        let entries = self.entries.read().await; let total = self.total_tokens.read().await; let n = entries.len();
        Ok(MemoryStats { entry_count: n, total_tokens: *total, memory_bytes: n * 512, level: MemoryLevel::Working })
    }
    
    fn memory_level(&self) -> MemoryLevel { MemoryLevel::Working }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test] async fn test_working_basic() -> Result<(), Box<dyn std::error::Error>> {
        let mem = WorkingMemory::new();
        mem.put("k1".into(), "v1".into()).await?;
        assert_eq!(mem.get(&"k1".into()).await?, Some("v1".into()));
        Ok(())
    }
    #[tokio::test] async fn test_working_sliding_window() -> Result<(), Box<dyn std::error::Error>> {
        let mem = WorkingMemory::with_limit(8); // 5+5+5=15>8 triggers eviction
        mem.put("k1".into(), "x".repeat(16)).await?; // 5 tokens
        mem.put("k2".into(), "y".repeat(16)).await?; // 5 tokens
        mem.put("k3".into(), "z".repeat(16)).await?; // 触发淘汰
        assert_eq!(mem.get(&"k1".into()).await?, None); // ✅ 验证k1被淘汰
        assert_eq!(mem.get(&"k3".into()).await?, Some("z".repeat(16)));
        Ok(())
    }
    #[tokio::test] async fn test_working_capacity_16000() {
        let mem = WorkingMemory::new();
        assert_eq!(mem.limit, 16000);
    }
}
