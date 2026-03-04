// SAB开销基准测试 - 验证零拷贝性能
const { Worker } = require('worker_threads');
const { performance } = require('perf_hooks');

function runTest(worker, vectorCount) {
  return new Promise((resolve) => {
    worker.once('message', resolve);
    worker.postMessage({ vectorCount, dimension: 128 });
  });
}

async function benchmark() {
  const worker = new Worker('./sab-worker.js');

  for (let i = 0; i < 100; i++) {
    await runTest(worker, 1000);
  }

  const results = [];
  for (let n = 0; n < 1000; n++) {
    const start = performance.now();
    await runTest(worker, 10000);
    results.push(performance.now() - start);
  }

  const avg = results.reduce((a, b) => a + b) / results.length;
  const p95 = results.sort((a, b) => a - b)[Math.floor(results.length * 0.95)];
  const overhead = (avg / 100) * 100;

  console.log(`Overhead: ${overhead.toFixed(2)}% (P95: ${p95.toFixed(2)}ms)`);
  console.log(`Avg latency: ${avg.toFixed(3)}ms`);
  console.log(`Status: ${overhead < 5 ? 'PASS (<5%)' : 'FAIL'}`);

  worker.terminate();
  return { avg, p95, overhead };
}

benchmark().catch(console.error);
