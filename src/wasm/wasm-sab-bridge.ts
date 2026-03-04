// WASM-SAB零拷贝桥接 - 消除FFI开销
import { SABAllocator } from './sab-allocator';

export class WASMSABBridge {
  private memory: WebAssembly.Memory;
  private exports: any;
  private dimension: number = 128;

  async init(wasmModule: BufferSource): Promise<void> {
    this.memory = new WebAssembly.Memory({
      initial: 256,
      maximum: 4096,
      shared: true
    });

    const instance = await WebAssembly.instantiate(wasmModule, {
      env: { memory: this.memory }
    });
    this.exports = instance.exports;
  }

  setDimension(dim: number): void {
    this.dimension = dim;
  }

  // 零拷贝搜索：直接传递SAB指针
  search(sab: SharedArrayBuffer, offset: number, count: number, k: number): any {
    const ptr = offset / 4;
    Atomics.wait(new Int32Array(sab), 0, 0);
    return this.exports.searchBatchMemory(ptr, count, this.dimension, k);
  }

  // 使用SAB分配器直接搜索
  searchWithAllocator(allocator: SABAllocator, count: number, k: number): any {
    const sab = allocator.getBuffer();
    return this.search(sab, 0, count, k);
  }

  // 写入数据到WASM内存
  writeData(data: Float32Array, sabOffset: number): void {
    const wasmArray = new Float32Array(this.memory.buffer);
    wasmArray.set(data, sabOffset / 4);
  }

  // 获取内存统计
  getMemoryStats(): { bufferSize: number; pageSize: number } {
    return {
      bufferSize: this.memory.buffer.byteLength,
      pageSize: 64 * 1024
    };
  }

  isInitialized(): boolean {
    return !!this.exports;
  }
}
