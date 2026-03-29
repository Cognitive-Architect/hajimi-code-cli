/**
 * AlignedMemoryPool - 16字节对齐内存池
 * 
 * 为WebAssembly SIMD优化提供16字节对齐的Float32Array视图，
 * 实现JS→WASM零拷贝数据传递。
 * 
 * @author 唐音（Engineer）
 * @version 1.0.0
 */

class AlignedMemoryPool {
  /**
   * 创建对齐内存池
   * 
   * @param {Object} options - 配置选项
   * @param {number} options.initialSize - 初始容量（字节），默认64KB
   * @param {number} options.maxSize - 最大容量（字节），默认1MB
   * @param {number} options.growthFactor - 扩容因子，默认2.0
   */
  constructor(options = {}) {
    this.initialSize = options.initialSize || 64 * 1024;  // 64KB
    this.maxSize = options.maxSize || 1024 * 1024;        // 1MB
    this.growthFactor = options.growthFactor || 2.0;
    
    // 对齐常量：16字节（WebAssembly SIMD要求）
    this.ALIGNMENT = 16;
    
    // 内存池状态
    this.pool = null;         // ArrayBuffer
    this.floatView = null;    // Float32Array视图
    this.used = 0;            // 已使用字节数（相对对齐基址）
    this.allocationCount = 0; // 分配计数（用于调试）
    
    this._initPool();
  }
  
  /**
   * 获取16字节对齐的Float32Array
   * 
   * 分配流程：
   * 1. 计算所需字节数 = size * 4 (f32)
   * 2. 计算对齐地址 = alignUp(currentUsed)
   * 3. 检查容量，必要时扩容
   * 4. 创建Float32Array视图
   * 
   * @param {number} size - 需要f32元素数量
   * @returns {Float32Array|null} - 对齐的视图，失败返回null触发fallback
   */
  acquire(size) {
    if (size <= 0) {
      return new Float32Array(0);
    }
    
    const bytesNeeded = size * 4;  // f32 = 4 bytes
    const alignedOffset = this._alignUp(this.used);
    const endOffset = alignedOffset + bytesNeeded;
    
    // 检查是否需要扩容
    if (endOffset > this.pool.byteLength) {
      if (!this._growPool(endOffset)) {
        // 超过maxSize，触发fallback
        return null;
      }
    }
    
    // 创建16字节对齐的Float32Array视图
    const view = new Float32Array(this.pool, alignedOffset, size);
    this.used = endOffset;
    this.allocationCount++;
    
    return view;
  }
  
  /**
   * 释放内存（标记可复用）
   * 
   * 当前策略：简单重置used指针（适用于单线程顺序使用）
   * 复杂场景可扩展为free-list管理
   * 
   * @param {Float32Array} view - 之前acquire返回的视图
   */
  release(view) {
    if (!view || view.byteLength === 0) {
      return;
    }
    
    // 简单策略：如果释放的是第一个分配，重置used
    // 更复杂策略可实现free-list
    if (view.byteOffset === this._alignUp(0)) {
      this.used = 0;
      this.allocationCount = 0;
    }
  }
  
  /**
   * 16字节对齐算法
   * 
   * 数学公式：aligned = (addr + ALIGNMENT - 1) & ~(ALIGNMENT - 1)
   * 
   * 验证：
   * - _alignUp(0)  === 0
   * - _alignUp(1)  === 16
   * - _alignUp(15) === 16
   * - _alignUp(16) === 16
   * - _alignUp(17) === 32
   * 
   * @param {number} addr - 原始地址/偏移
   * @returns {number} - 16字节对齐后的地址
   */
  _alignUp(addr) {
    return (addr + this.ALIGNMENT - 1) & ~(this.ALIGNMENT - 1);
  }
  
  /**
   * 检查地址是否16字节对齐
   * 
   * @param {number} addr - 地址
   * @returns {boolean} - 对齐返回true
   */
  _isAligned(addr) {
    return (addr & (this.ALIGNMENT - 1)) === 0;
  }
  
  /**
   * 初始化内存池
   * 
   * 策略：多分配ALIGNMENT-1字节用于对齐调整
   */
  _initPool() {
    // 多分配15字节确保能对齐
    const allocSize = this.initialSize + this.ALIGNMENT - 1;
    this.pool = new ArrayBuffer(allocSize);
    
    // 计算对齐基址（ArrayBuffer总是从0开始，所以基址就是0）
    const baseAddr = 0;
    const alignedBase = this._alignUp(baseAddr);
    
    // 创建从对齐基址开始的Float32Array视图
    this.floatView = new Float32Array(
      this.pool, 
      alignedBase, 
      (this.initialSize - (alignedBase - baseAddr)) / 4
    );
    
    this.used = alignedBase;
  }
  
  /**
   * 扩容内存池
   * 
   * @param {number} minSize - 最小需要容量（字节）
   * @returns {boolean} - 成功返回true，失败返回false触发fallback
   */
  _growPool(minSize) {
    const currentSize = this.pool ? this.pool.byteLength : 0;
    const newSize = Math.min(
      Math.max(minSize * this.growthFactor, currentSize * this.growthFactor),
      this.maxSize + this.ALIGNMENT - 1
    );
    
    if (newSize < minSize) {
      return false;  // 超过maxSize，触发fallback
    }
    
    // 创建新池
    const oldPool = this.pool;
    this._initPoolWithSize(newSize);
    
    // 复制旧数据（如果需要保持数据）
    // 零拷贝路径下通常不需要，因为数据是临时填充的
    
    return true;
  }
  
  /**
   * 使用指定大小初始化池
   * 
   * @param {number} size - 分配大小（字节）
   */
  _initPoolWithSize(size) {
    const allocSize = size + this.ALIGNMENT - 1;
    this.pool = new ArrayBuffer(allocSize);
    
    const alignedBase = this._alignUp(0);
    this.floatView = new Float32Array(
      this.pool,
      alignedBase,
      (size - (alignedBase)) / 4
    );
    
    this.used = alignedBase;
  }
  
  /**
   * 获取池统计信息
   * 
   * @returns {Object} - 统计信息
   */
  getStats() {
    return {
      capacity: this.pool ? this.pool.byteLength : 0,
      used: this.used,
      available: this.pool ? this.pool.byteLength - this.used : 0,
      alignment: this.ALIGNMENT,
      allocationCount: this.allocationCount,
    };
  }
  
  /**
   * 重置池（释放所有分配）
   */
  reset() {
    this.used = this._alignUp(0);
    this.allocationCount = 0;
  }
}

module.exports = { AlignedMemoryPool };
