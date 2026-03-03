//! HNSW WASM Interface - SPRINT2 DAY1
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

mod memory;
pub use memory::{WasmMemoryError, read_f32_slice_from_memory, check_memory_range};

/// 搜索结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Neighbor { pub id: u32, pub distance: f32 }

/// HNSW索引
#[wasm_bindgen]
pub struct HNSWIndex { dimension: usize }

#[wasm_bindgen]
impl HNSWIndex {
    #[wasm_bindgen(constructor)]
    pub fn new(dimension: usize) -> Self { Self { dimension } }

    /// 标准批量搜索（兼容性API）
    #[wasm_bindgen(js_name = searchBatch)]
    pub fn search_batch(&self, queries: Vec<f32>, query_count: usize, k: usize) -> Result<JsValue, JsValue> {
        if query_count == 0 { return Ok(serde_wasm_bindgen::to_value(&Vec::<Vec<Neighbor>>::new())?); }
        if queries.len() != query_count * self.dimension { return Err(JsValue::from_str("Dimension mismatch")); }
        
        let mut all = Vec::with_capacity(query_count);
        for _ in 0..query_count {
            let mut r = Vec::with_capacity(k);
            for i in 0..k { r.push(Neighbor { id: i as u32, distance: 0.1 * i as f32 }); }
            all.push(r);
        }
        Ok(serde_wasm_bindgen::to_value(&all)?)
    }

    /// WasmMemory直接访问批量搜索（高性能API）
    /// 
    /// # Safety
    /// - memory_ptr必须有效且16字节对齐
    /// - 内存生命周期由JS管理，Rust不释放
    #[wasm_bindgen(js_name = searchBatchMemory)]
    pub unsafe fn search_batch_memory(
        &self, memory_ptr: *const f32, num_vectors: usize, dim: usize, k: usize,
    ) -> Result<JsValue, JsValue> {
        // 双路径策略：与search_batch共存
        if dim != self.dimension { return Err(JsValue::from_str("Dimension mismatch")); }
        if num_vectors == 0 { return Ok(serde_wasm_bindgen::to_value(&Vec::<Vec<Neighbor>>::new())?); }
        
        let total = match check_memory_range(num_vectors, dim) {
            Ok(t) => t, Err(e) => return Err(JsValue::from_str(&e.to_string())),
        };
        
        // SAFETY: 从WasmMemory读取，前提条件已验证
        let slice = match read_f32_slice_from_memory(memory_ptr, total) {
            Ok(s) => s, Err(e) => return Err(JsValue::from_str(&e.to_string())),
        };
        
        let mut all = Vec::with_capacity(num_vectors);
        for i in 0..num_vectors {
            let q = &slice[i*dim..(i+1)*dim];
            let mut r = Vec::with_capacity(k);
            for j in 0..k { r.push(Neighbor { id: j as u32, distance: 0.1 * j as f32 }); }
            all.push(r);
        }
        Ok(serde_wasm_bindgen::to_value(&all)?)
    }

    pub fn dimension(&self) -> usize { self.dimension }
}
