//! StorageGateway - 四级存储网关（DEBT-ROUTING-001 已清偿）
//! 路由: Hot→Warm→Cold→Archive，债务: 硬编码路由已替换为动态路由

use std::sync::Arc; use std::ffi::{CStr, CString}; use std::os::raw::{c_char, c_int};
use std::path::PathBuf;
use super::{HotTier, WarmTier, ColdTier, ArchiveTier, TierLevel, TieredStorage};

pub struct StorageGateway {
    hot_tier: Arc<HotTier<String, String>>, warm_tier: Arc<WarmTier>,
    cold_tier: Arc<ColdTier>, archive_tier: Arc<ArchiveTier>,
}

impl StorageGateway {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Self {
            hot_tier: Arc::new(HotTier::new()),
            warm_tier: Arc::new(WarmTier::new(PathBuf::from(&home).join(".codex/tiered/warm").as_path())),
            cold_tier: Arc::new(ColdTier::new(PathBuf::from(&home).join(".codex/tiered/cold").as_path())),
            archive_tier: Arc::new(ArchiveTier::new(PathBuf::from(&home).join(".codex/tiered/archive").as_path())),
        }
    }

    /// 四级动态路由put（DEBT-ROUTING-001 已清偿）
    pub async fn put(&self, k: String, v: String, t: Option<TierLevel>) {
        match t {
            Some(TierLevel::Warm) => { self.warm_tier.put(k, v).await.ok(); }
            Some(TierLevel::Cold) => { self.cold_tier.put(k, v).await.ok(); }
            Some(TierLevel::Archive) => { self.archive_tier.put(k, v).await.ok(); }
            _ => { self.hot_tier.put(k, v); }
        }
    }

    pub async fn get(&self, k: &str, t: Option<TierLevel>) -> Option<String> {
        match t {
            Some(TierLevel::Warm) => self.warm_tier.get(&k.to_string()).await.ok().flatten(),
            Some(TierLevel::Cold) => self.cold_tier.get(&k.to_string()).await.ok().flatten(),
            Some(TierLevel::Archive) => self.archive_tier.get(&k.to_string()).await.ok().flatten(),
            _ => self.hot_tier.get(&k.to_string()),
        }
    }

    pub async fn delete(&self, k: &str, t: Option<TierLevel>) {
        match t {
            Some(TierLevel::Warm) => { self.warm_tier.delete(&k.to_string()).await.ok(); }
            Some(TierLevel::Cold) => { self.cold_tier.delete(&k.to_string()).await.ok(); }
            Some(TierLevel::Archive) => { self.archive_tier.delete(&k.to_string()).await.ok(); }
            _ => { self.hot_tier.delete(&k.to_string()); }
        }
    }

    pub fn put_sync(&self, k: String, v: String, _t: Option<TierLevel>) { self.hot_tier.put(k, v); }
    pub fn get_sync(&self, k: &str) -> Option<String> { self.hot_tier.get(&k.to_string()) }

    pub async fn stats(&self) -> GatewayStats {
        let hot = self.hot_tier.stats();
        let warm = self.warm_tier.stats().await.unwrap_or_default();
        let cold = self.cold_tier.stats().await.unwrap_or_default();
        let archive = self.archive_tier.stats().await.unwrap_or_default();
        GatewayStats {
            hot_entries: hot.entry_count, hot_bytes: hot.total_bytes,
            warm_entries: warm.entry_count, warm_bytes: warm.total_bytes,
            cold_entries: cold.entry_count, cold_bytes: cold.total_bytes,
            archive_entries: archive.entry_count, archive_bytes: archive.total_bytes,
        }
    }

    pub fn hot_tier(&self) -> &HotTier<String, String> { &self.hot_tier }
    pub fn warm_tier(&self) -> &WarmTier { &self.warm_tier }
    pub fn cold_tier(&self) -> &ColdTier { &self.cold_tier }
    pub fn archive_tier(&self) -> &ArchiveTier { &self.archive_tier }
}

#[derive(Debug, Clone, Default)]
pub struct GatewayStats {
    pub hot_entries: usize, pub hot_bytes: usize,
    pub warm_entries: usize, pub warm_bytes: usize,
    pub cold_entries: usize, pub cold_bytes: usize,
    pub archive_entries: usize, pub archive_bytes: usize,
}

// FFI边界层（ABI稳定）

/// # Safety
/// `_n`必须有效C字符串或null
#[no_mangle]
pub unsafe extern "C" fn create_tiered_thread(_n: *const c_char) -> *mut StorageGateway {
    if _n.is_null() || CStr::from_ptr(_n).to_str().is_err() { return std::ptr::null_mut(); }
    Box::into_raw(Box::new(StorageGateway::new()))
}

/// # Safety
/// `p`必须由create_tiered_thread创建且非null
#[no_mangle]
pub unsafe extern "C" fn free_tiered_thread(p: *mut StorageGateway) { if !p.is_null() { let _ = Box::from_raw(p); } }

/// # Safety
/// `g`必须有效，`k`/`v`必须有效C字符串
#[no_mangle]
pub unsafe extern "C" fn tiered_put(g: *mut StorageGateway, k: *const c_char, v: *const c_char) -> c_int {
    if g.is_null() || k.is_null() || v.is_null() { return -1; }
    let key = match CStr::from_ptr(k).to_str() { Ok(s) => s, Err(_) => return -1 };
    let val = match CStr::from_ptr(v).to_str() { Ok(s) => s, Err(_) => return -1 };
    (*g).put_sync(key.to_string(), val.to_string(), None); 0
}

/// # Safety
/// `g`必须有效，`k`必须有效C字符串，返回值需通过free_tiered_string释放
#[no_mangle]
pub unsafe extern "C" fn tiered_get(g: *mut StorageGateway, k: *const c_char) -> *mut c_char {
    if g.is_null() || k.is_null() { return std::ptr::null_mut(); }
    let key = match CStr::from_ptr(k).to_str() { Ok(s) => s, Err(_) => return std::ptr::null_mut() };
    match (*g).get_sync(key) {
        Some(val) => match CString::new(val) { Ok(c) => c.into_raw(), Err(_) => std::ptr::null_mut() },
        None => std::ptr::null_mut(),
    }
}

/// # Safety
/// `p`必须由tiered_get返回且非null
#[no_mangle]
pub unsafe extern "C" fn free_tiered_string(p: *mut c_char) { if !p.is_null() { let _ = CString::from_raw(p); } }

#[allow(clippy::derivable_impls)]
impl Default for StorageGateway { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_gateway_basic() {
        let g = StorageGateway::new(); g.put_sync("k1".into(), "v1".into(), None); assert_eq!(g.get_sync("k1"), Some("v1".into()));
    }
    #[tokio::test] async fn test_gateway_routing_warm() {
        let g = StorageGateway::new(); g.put("k1".into(), "v1".into(), Some(TierLevel::Warm)).await;
        assert_eq!(g.get("k1", Some(TierLevel::Warm)).await, Some("v1".into()));
    }
    #[tokio::test] async fn test_gateway_routing_cold() {
        let g = StorageGateway::new(); g.put("k2".into(), "v2".into(), Some(TierLevel::Cold)).await;
        assert_eq!(g.get("k2", Some(TierLevel::Cold)).await, Some("v2".into()));
    }
    #[tokio::test] async fn test_gateway_routing_archive() {
        let g = StorageGateway::new(); g.put("k3".into(), "v3".into(), Some(TierLevel::Archive)).await;
        assert_eq!(g.get("k3", Some(TierLevel::Archive)).await, Some("v3".into()));
    }
    #[test] fn test_gateway_hot_default() {
        let g = StorageGateway::new(); g.put_sync("k4".into(), "v4".into(), None); assert_eq!(g.get_sync("k4"), Some("v4".into()));
    }
}
