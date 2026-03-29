//! SAB内存接口 - 零拷贝访问与Atomics同步
use std::sync::atomic::{Ordering, fence};
use crate::memory::{WasmMemoryError, read_f32_slice_from_memory};

/// 16字节对齐的数据结构
#[repr(C, align(16))]
pub struct AlignedData<T> {
    data: T,
}

impl<T> AlignedData<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }

    pub fn get(&self) -> &T {
        &self.data
    }
}

/// 从SAB读取f32切片 - 安全检查对齐
pub unsafe fn read_sab_f32_slice(
    ptr: *const f32,
    len: usize,
) -> Result<&'static [f32], WasmMemoryError> {
    if ptr.is_null() {
        return Err(WasmMemoryError::NullPointer);
    }
    if ptr as usize % 16 != 0 {
        return Err(WasmMemoryError::MisalignedPointer);
    }
    read_f32_slice_from_memory(ptr, len)
}

/// Atomics同步屏障 - 确保SAB可见性
pub fn memory_fence() {
    fence(Ordering::SeqCst);
}

/// 检查指针16字节对齐
pub fn is_aligned_16(ptr: *const u8) -> bool {
    ptr as usize % 16 == 0
}

/// 对齐计算
pub fn align_16(size: usize) -> usize {
    (size + 15) & !15
}

/// 验证SAB内存范围
pub fn validate_sab_range(offset: usize, length: usize, capacity: usize) -> bool {
    offset + length <= capacity
}
