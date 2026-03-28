//! HotTier - 热存储层（O(1)读写, Mutex<HashMap>, Send+Sync）
use std::collections::HashMap; use std::sync::{Arc, Mutex};
use super::{StorageStats, TierLevel};

pub struct HotTier<K, V> { data: Arc<Mutex<HashMap<K, V>>>, stats: Arc<Mutex<StorageStats>> }

impl<K, V> HotTier<K, V>
where K: std::hash::Hash + Eq + Send + Sync + Clone + 'static, V: Send + Sync + Clone + 'static,
{
    pub fn new() -> Self {
        Self { data: Arc::new(Mutex::new(HashMap::new())),
               stats: Arc::new(Mutex::new(StorageStats { entry_count: 0, total_bytes: 0, tier: TierLevel::Hot })) }
    }
    pub fn get(&self, key: &K) -> Option<V> { self.data.lock().ok()?.get(key).cloned() }
    pub fn put(&self, key: K, value: V) {
        if let (Ok(mut d), Ok(mut s)) = (self.data.lock(), self.stats.lock()) {
            let size = std::mem::size_of_val(&key) + std::mem::size_of_val(&value);
            let _ = d.insert(key, value); s.entry_count = d.len(); s.total_bytes += size;
        }
    }
    pub fn delete(&self, key: &K) -> Option<V> {
        if let (Ok(mut d), Ok(mut s)) = (self.data.lock(), self.stats.lock()) {
            let r = d.remove(key); if r.is_some() { s.entry_count = d.len(); } return r;
        } None
    }
    pub fn list_keys(&self) -> Vec<K> { self.data.lock().ok().map(|d| d.keys().cloned().collect()).unwrap_or_default() }
    pub fn stats(&self) -> StorageStats { self.stats.lock().map(|s| s.clone()).unwrap_or_else(|_| StorageStats { entry_count: 0, total_bytes: 0, tier: TierLevel::Hot }) }
}

impl<K, V> Default for HotTier<K, V>
where K: std::hash::Hash + Eq + Send + Sync + Clone + 'static, V: Send + Sync + Clone + 'static,
{ fn default() -> Self { Self::new() } }

impl<K, V> Drop for HotTier<K, V> {
    fn drop(&mut self) {
        if let Ok(mut d) = self.data.lock() { d.clear(); }
        if let Ok(mut s) = self.stats.lock() { s.entry_count = 0; s.total_bytes = 0; }
    }
}

#[cfg(test)]
mod tests {
    use super::*; use std::thread;
    #[test] fn test_basic() {
        let t = HotTier::<String, String>::new();
        t.put("k1".into(), "v1".into());
        assert_eq!(t.get(&"k1".into()), Some("v1".into()));
    }
    #[test] fn test_thread_safety() {
        let t = Arc::new(HotTier::<String, i32>::new());
        let hs: Vec<_> = (0..10).map(|i| { let c = t.clone(); thread::spawn(move || { c.put(format!("k{}", i), i); }) }).collect();
        for h in hs { h.join().unwrap(); }
        assert_eq!(t.stats().entry_count, 10);
    }
}
