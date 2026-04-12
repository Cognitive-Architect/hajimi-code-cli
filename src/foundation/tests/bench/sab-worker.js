// SAB Worker - 用于基准测试的Worker线程
const { parentPort } = require('worker_threads');
const { SABAllocator } = require('../../src/wasm/sab-allocator');
const { WASMSABBridge } = require('../../src/wasm/wasm-sab-bridge');

parentPort?.on('message', async ({ vectorCount, dimension }) => {
  // 创建SAB分配器
  const allocator = new SABAllocator(vectorCount * dimension * 4 + 1024);
  
  // 写入测试数据
  const testData = new Float32Array(vectorCount * dimension);
  for (let i = 0; i < testData.length; i++) {
    testData[i] = Math.random();
  }
  
  const offset = allocator.allocate(testData.length * 4);
  allocator.writeF32Array(testData, offset);
  
  // 通知主线程完成
  parentPort?.postMessage({ done: true, offset });
});
