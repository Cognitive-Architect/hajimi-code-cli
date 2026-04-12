// SAB分配器 - Node.js 18+兼容零拷贝内存管理
export class SABAllocator {
  private sab: SharedArrayBuffer;
  private offset: number = 0;
  private readonly alignment: number = 16;
  private atomicView: Int32Array;
  private wasmMemory: WebAssembly.Memory | null = null;

  constructor(size: number) {
    // 预分配固定大小SAB，兼容Node.js 18
    this.sab = new SharedArrayBuffer(size);
    this.atomicView = new Int32Array(this.sab, 0, 4);
  }

  // 绑定WASM Memory实现直接内存访问
  bindWasmMemory(memory: WebAssembly.Memory): void { this.wasmMemory = memory; }

  // 获取WasmMemory实例用于FFI零拷贝
  getWasmMemory(): WebAssembly.Memory | null { return this.wasmMemory; }

  // 分配16字节对齐内存块
  allocate(bytes: number): number {
    const aligned = (this.offset + this.alignment - 1) & ~(this.alignment - 1);
    if (aligned + bytes > this.sab.byteLength) throw new Error('Out of memory');
    const currentOffset = Atomics.load(this.atomicView, 0);
    this.offset = aligned + bytes;
    Atomics.store(this.atomicView, 0, this.offset);
    Atomics.store(this.atomicView, 1, currentOffset);
    return aligned;
  }

  // 获取SAB引用（零拷贝）
  getBuffer(): SharedArrayBuffer { return this.sab; }

  // 获取当前偏移（原子读）
  getOffset(): number { return Atomics.load(this.atomicView, 0); }

  // 原子操作重置，Worker安全
  reset(): void {
    Atomics.store(this.atomicView, 0, 0);
    Atomics.store(this.atomicView, 1, 0);
    this.offset = 0;
  }

  // 释放资源
  dispose(): void { this.offset = 0; }

  // 检查16字节对齐
  static isAligned(ptr: number, align: number = 16): boolean { return ptr % align === 0; }

  // 计算对齐值
  static align(size: number, align: number = 16): number { return (size + align - 1) & ~(align - 1); }

  // 写入f32数组（零拷贝视图）
  writeF32Array(data: Float32Array, offset: number): void {
    if (!SABAllocator.isAligned(offset)) throw new Error('Not 16-byte aligned');
    new Float32Array(this.sab, offset, data.length).set(data);
    Atomics.add(this.atomicView, 2, data.length);
  }

  // 读取f32数组（零拷贝视图）
  readF32Array(offset: number, length: number): Float32Array {
    if (!SABAllocator.isAligned(offset)) throw new Error('Not 16-byte aligned');
    return new Float32Array(this.sab, offset, length);
  }

  // Atomics.wait用于Worker同步
  waitForData(timeout: number = 1000): string { return Atomics.wait(this.atomicView, 0, 0, timeout); }

  // Atomics.notify通知Worker
  notifyData(count: number = 1): number { return Atomics.notify(this.atomicView, 0, count); }

  // 获取统计信息
  getStats(): { total: number; capacity: number; available: number } {
    const used = Atomics.load(this.atomicView, 0);
    return { total: used, capacity: this.sab.byteLength, available: this.sab.byteLength - used };
  }
}

export default SABAllocator;
