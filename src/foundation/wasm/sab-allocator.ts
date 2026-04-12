// SAB分配器 - 16字节对齐零拷贝内存管理
export class SABAllocator {
  private sab: SharedArrayBuffer;
  private offset: number = 0;
  private readonly alignment: number = 16;
  private atomicView: Int32Array;

  constructor(size: number) {
    // 创建SAB，支持maxByteLength增长选项
    this.sab = new SharedArrayBuffer(size, { maxByteLength: size * 2 });
    this.atomicView = new Int32Array(this.sab, 0, 1);
  }

  // 分配16字节对齐的内存块
  allocate(bytes: number): number {
    const aligned = (this.offset + this.alignment - 1) & ~(this.alignment - 1);
    if (aligned + bytes > this.sab.byteLength) throw new Error('Out of memory');
    this.offset = aligned + bytes;
    Atomics.store(this.atomicView, 0, this.offset);
    return aligned;
  }

  // 获取SAB引用
  getBuffer(): SharedArrayBuffer { return this.sab; }

  // 获取当前偏移
  getOffset(): number { return this.offset; }

  // 使用Atomics原子操作重置，Worker安全
  reset(): void {
    Atomics.store(this.atomicView, 0, 0);
    this.offset = 0;
  }

  // Worker线程安全释放
  dispose(): void { this.offset = 0; }

  // 检查16字节对齐
  static isAligned(ptr: number, align: number = 16): boolean {
    return ptr % align === 0;
  }

  // 计算16字节对齐值
  static align(size: number, align: number = 16): number {
    return (size + align - 1) & ~(align - 1);
  }

  // 写入f32数组（零拷贝视图）
  writeF32Array(data: Float32Array, offset: number): void {
    if (!SABAllocator.isAligned(offset)) throw new Error('Not 16-byte aligned');
    const view = new Float32Array(this.sab, offset, data.length);
    view.set(data);
    Atomics.add(this.atomicView, 0, data.length);
  }

  // 读取f32数组（零拷贝视图）
  readF32Array(offset: number, length: number): Float32Array {
    if (!SABAllocator.isAligned(offset)) throw new Error('Not 16-byte aligned');
    return new Float32Array(this.sab, offset, length);
  }

  // Atomics.wait用于Worker线程同步
  waitForData(timeout: number = 1000): string {
    return Atomics.wait(this.atomicView, 0, 0, timeout);
  }

  // Atomics.notify通知等待的Worker
  notifyData(count: number = 1): number {
    return Atomics.notify(this.atomicView, 0, count);
  }

  // 获取分配统计信息
  getStats(): { total: number; capacity: number; available: number } {
    return {
      total: this.offset,
      capacity: this.sab.byteLength,
      available: this.sab.byteLength - this.offset
    };
  }
}

export default SABAllocator;
