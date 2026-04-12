/**
 * Write Amplification基准测试
 * 目标：WA < 3x，吞吐量提升50%
 * 使用Worker线程进行并行测试
 */
const { Worker, isMainThread, parentPort, workerData } = require('worker_threads');

const TOTAL_WRITES = 100000;
const VALUE_SIZE = 1000;
const DB_PATH = './bench-leveldb';

if (isMainThread) {
  async function main() {
    console.log('Write Amplification Benchmark');
    console.log('================================');
    console.log(`Total writes: ${TOTAL_WRITES}`);
    console.log(`Value size: ${VALUE_SIZE} bytes`);
    
    const worker = new Worker(__filename, {
      workerData: { totalWrites: TOTAL_WRITES, valueSize: VALUE_SIZE, path: DB_PATH }
    });
    
    worker.on('message', (result) => {
      console.log(`\nWrite Amplification: ${result.wa.toFixed(2)}x`);
      console.log(`Throughput: ${result.throughput.toFixed(0)} ops/sec`);
      console.log(`Status: ${result.wa < 3 ? 'PASS (WA<3x)' : 'FAIL'}`);
    });
    
    worker.on('error', (err) => {
      console.error('Worker error:', err);
      process.exit(1);
    });
    
    worker.on('exit', (code) => {
      if (code !== 0) console.error(`Worker exited with code ${code}`);
    });
  }
  main();
} else {
  async function benchmark() {
    const { OptimizedLevelDB } = require('../../src/storage/leveldb-optimized');
    const { totalWrites, valueSize, path } = workerData;
    const db = new OptimizedLevelDB({ path, enableMonitoring: true });
    
    const start = performance.now();
    for (let i = 0; i < totalWrites; i++) {
      await db.put(`key-${i}`, 'x'.repeat(valueSize));
    }
    const elapsed = (performance.now() - start) / 1000;
    const wa = db.getWriteAmplification();
    
    await db.close();
    parentPort.postMessage({ wa, throughput: totalWrites / elapsed });
  }
  benchmark();
}
