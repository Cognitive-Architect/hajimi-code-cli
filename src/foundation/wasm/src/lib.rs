//! HNSW WASM Interface - SPRINT2 DAY1
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

mod memory;
pub use memory::{check_memory_range, read_f32_slice_from_memory, WasmMemoryError};

/// WASM 内存上限 (256MB)
///
/// 依据: WebAssembly MVP 规范默认线性内存上限为 4GB，
/// 本系统设定 256MB 作为安全边界，防止宿主环境过度分配导致的越界读取。
const WASM_MAX_MEMORY: usize = 256 * 1024 * 1024;

/// 搜索结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Neighbor {
    pub id: u32,
    pub distance: f32,
}

/// HNSW索引
#[wasm_bindgen]
pub struct HNSWIndex {
    dimension: usize,
}

#[wasm_bindgen]
impl HNSWIndex {
    #[wasm_bindgen(constructor)]
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }

    /// 标准批量搜索（兼容性API）
    #[wasm_bindgen(js_name = searchBatch)]
    pub fn search_batch(
        &self,
        queries: Vec<f32>,
        query_count: usize,
        k: usize,
    ) -> Result<JsValue, JsValue> {
        if query_count == 0 {
            return Ok(serde_wasm_bindgen::to_value(&Vec::<Vec<Neighbor>>::new())?);
        }
        if queries.len() != query_count * self.dimension {
            return Err(JsValue::from_str("Dimension mismatch"));
        }

        let mut all = Vec::with_capacity(query_count);
        for _ in 0..query_count {
            let mut r = Vec::with_capacity(k);
            for i in 0..k {
                r.push(Neighbor {
                    id: i as u32,
                    distance: 0.1 * i as f32,
                });
            }
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
        &self,
        memory_ptr: *const f32,
        num_vectors: usize,
        dim: usize,
        k: usize,
    ) -> Result<JsValue, JsValue> {
        // 双路径策略：与search_batch共存
        if dim != self.dimension {
            return Err(JsValue::from_str("Dimension mismatch"));
        }
        if num_vectors == 0 {
            return Ok(serde_wasm_bindgen::to_value(&Vec::<Vec<Neighbor>>::new())?);
        }

        let total = match check_memory_range(num_vectors, dim) {
            Ok(t) => t,
            Err(e) => return Err(JsValue::from_str(&e.to_string())),
        };

        // 范围上限校验：必须在unsafe块之前执行
        if let Err(e) = validate_memory_access(memory_ptr, total) {
            return Err(JsValue::from_str(&e.to_string()));
        }

        // SAFETY: 从WasmMemory读取，前提条件已验证
        let slice = match read_f32_slice_from_memory(memory_ptr, total) {
            Ok(s) => s,
            Err(e) => return Err(JsValue::from_str(&e.to_string())),
        };

        let mut all = Vec::with_capacity(num_vectors);
        for i in 0..num_vectors {
            let _q = &slice[i * dim..(i + 1) * dim];
            let mut r = Vec::with_capacity(k);
            for j in 0..k {
                r.push(Neighbor {
                    id: j as u32,
                    distance: 0.1 * j as f32,
                });
            }
            all.push(r);
        }
        Ok(serde_wasm_bindgen::to_value(&all)?)
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

/// 校验 WASM 内存访问范围
///
/// 在 unsafe 内存操作前执行，确保 ptr + len 不超出 WASM_MAX_MEMORY，
/// 并处理 ptr + len 算术溢出。
fn validate_memory_access(ptr: *const f32, len: usize) -> Result<(), WasmMemoryError> {
    // 空指针检查
    if ptr.is_null() {
        return Err(WasmMemoryError::NullPointer);
    }
    // 对齐检查
    let ptr_addr = ptr as usize;
    if !ptr_addr.is_multiple_of(16) {
        return Err(WasmMemoryError::MisalignedPointer);
    }
    // 范围上限检查（防止算术溢出）
    let byte_len = len.saturating_mul(std::mem::size_of::<f32>());
    let end_addr = ptr_addr.saturating_add(byte_len);
    if end_addr > WASM_MAX_MEMORY || (len > 0 && byte_len == 0) {
        return Err(WasmMemoryError::OutOfBounds {
            ptr: ptr_addr,
            len: byte_len,
            max: WASM_MAX_MEMORY,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_memory_access_null() {
        let result = validate_memory_access(std::ptr::null(), 1);
        assert!(matches!(result, Err(WasmMemoryError::NullPointer)));
    }

    #[test]
    fn test_validate_memory_access_misaligned() {
        let unaligned = 0x1001 as *const f32;
        let result = validate_memory_access(unaligned, 1);
        assert!(matches!(result, Err(WasmMemoryError::MisalignedPointer)));
    }

    #[test]
    fn test_validate_memory_access_out_of_bounds() {
        let base = 0x1000 as *const f32;
        let result = validate_memory_access(base, WASM_MAX_MEMORY + 1);
        assert!(matches!(result, Err(WasmMemoryError::OutOfBounds { .. })));
    }

    #[test]
    fn test_validate_memory_access_boundary_pass() {
        let base = WASM_MAX_MEMORY as *const f32;
        let result = validate_memory_access(base, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_memory_access_overflow_rejected() {
        let base = 0x1000 as *const f32;
        let result = validate_memory_access(base, usize::MAX);
        assert!(matches!(result, Err(WasmMemoryError::OutOfBounds { .. })));
    }
}
