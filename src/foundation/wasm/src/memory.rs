//! WASM内存安全访问模块
use std::slice;

/// WasmMemory访问错误
#[derive(Debug, Clone, PartialEq)]
pub enum WasmMemoryError {
    NullPointer,
    MisalignedPointer,
    InvalidMemoryRange,
    ZeroDimension,
    /// 内存访问越界：ptr/len 超出 WASM_MAX_MEMORY 上限
    OutOfBounds {
        ptr: usize,
        len: usize,
        max: usize,
    },
}

impl std::fmt::Display for WasmMemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmMemoryError::NullPointer => write!(f, "WasmMemory pointer is null"),
            WasmMemoryError::MisalignedPointer => write!(f, "Pointer not 16-byte aligned"),
            WasmMemoryError::InvalidMemoryRange => write!(f, "Invalid memory range"),
            WasmMemoryError::ZeroDimension => write!(f, "Dimension cannot be zero"),
            WasmMemoryError::OutOfBounds { ptr, len, max } => {
                write!(
                    f,
                    "Memory access out of bounds: ptr=0x{:x}, len={}, max={}",
                    ptr, len, max
                )
            }
        }
    }
}

impl std::error::Error for WasmMemoryError {}

/// 从WasmMemory读取f32切片
///
/// # Safety
/// - ptr必须有效且16字节对齐
/// - 内存生命周期由JS管理，此函数不释放
/// - 调用者必须已通过 validate_memory_access 进行范围校验
pub unsafe fn read_f32_slice_from_memory(
    ptr: *const f32,
    len: usize,
) -> Result<&'static [f32], WasmMemoryError> {
    // SAFETY: 空指针检查，确保ptr不是null
    if ptr.is_null() {
        return Err(WasmMemoryError::NullPointer);
    }

    // SAFETY: 16字节对齐检查，SIMD优化要求
    if !(ptr as usize).is_multiple_of(16) {
        return Err(WasmMemoryError::MisalignedPointer);
    }

    if len == 0 {
        return Ok(&[]);
    }

    // SAFETY: 创建切片，前提条件已验证
    Ok(slice::from_raw_parts(ptr, len))
}

/// 检查内存范围计算是否溢出
pub fn check_memory_range(num_vectors: usize, dim: usize) -> Result<usize, WasmMemoryError> {
    if dim == 0 {
        return Err(WasmMemoryError::ZeroDimension);
    }
    num_vectors
        .checked_mul(dim)
        .ok_or(WasmMemoryError::InvalidMemoryRange)
}
