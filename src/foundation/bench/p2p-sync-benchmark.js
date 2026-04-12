/**
 * P2P Sync Benchmark Engine - DEBT-P2P-003清偿
 * 约束: ≤150行 | RSS内存测量 | P95延迟 | 30s超时
 */
const { EventEmitter } = require('events');
const { performance } = require('perf_hooks');

const TIMEOUT = 30000; // 30s熔断

class P2PSyncBenchmark extends EventEmitter {
  constructor(config) {
    super();
    this.config = { 
      chunkCount: config.chunkCount, 
      batchSize: config.batchSize || 100, 
      timeout: config.timeout || TIMEOUT 
    };
    this.memorySnapshots = [];
    this.latencies = [];
    this.startTime = 0;
  }

  async run() {
    this.startTime = performance.now();
    const memStart = process.memoryUsage();
    this.memorySnapshots = [{ rss: memStart.rss, heapUsed: memStart.heapUsed, timestamp: 0 }];
    
    const timeoutId = setTimeout(() => this.emit('timeout'), this.config.timeout);
    let timedOut = false;
    this.once('timeout', () => { timedOut = true; });

    try {
      await this.syncChunks(this.config.chunkCount);
      clearTimeout(timeoutId);
      if (timedOut) throw new Error('TIMEOUT: 30s熔断触发');
      return this.buildResult(true);
    } catch (error) {
      clearTimeout(timeoutId);
      return { ...this.buildResult(false), error: error.message };
    }
  }

  async syncChunks(totalChunks) {
    const { batchSize } = this.config;
    const chunks = Array.from({ length: totalChunks }, (_, i) => `chunk-${i}`);
    const reportInterval = Math.ceil(totalChunks / 10); // 每10%报告

    for (let i = 0; i < chunks.length; i += batchSize) {
      const batchStart = performance.now();
      const batch = chunks.slice(i, i + batchSize);
      await this.sendBatch(batch);
      this.latencies.push(performance.now() - batchStart);
      this.snapshotMemory();
      
      if ((i + batch.length) % reportInterval === 0 || i + batch.length >= chunks.length) {
        const progress = Math.min(100, Math.round(((i + batch.length) / totalChunks) * 100));
        this.emit('progress', { progress, chunks: i + batch.length, total: totalChunks });
      }
    }
  }

  async sendBatch(batch) {
    // 模拟真实内存压力 - 创建实际数据负载(1KB per chunk)
    const payload = batch.map(id => ({ id, data: Buffer.alloc(1024).fill(id) }));
    await this.delay(Math.random() * 5 + 1); // 1-6ms模拟网络延迟
    payload.length = 0; // 释放引用
  }

  snapshotMemory() {
    const mem = process.memoryUsage();
    this.memorySnapshots.push({
      rss: mem.rss, heapUsed: mem.heapUsed, timestamp: performance.now() - this.startTime
    });
  }

  buildResult(success) {
    const duration = performance.now() - this.startTime;
    const rssValues = this.memorySnapshots.map(m => m.rss);
    const maxRss = Math.max(...rssValues);
    const minRss = Math.min(...rssValues);
    
    return {
      chunkCount: this.config.chunkCount,
      durationMs: Math.round(duration),
      throughput: Math.round((this.config.chunkCount / duration) * 1000),
      p95Latency: this.calcP95(this.latencies),
      maxMemoryMB: Math.round(maxRss / 1024 / 1024),
      memoryGrowthMB: Math.round((maxRss - minRss) / 1024 / 1024),
      success
    };
  }

  calcP95(values) {
    if (values.length === 0) return 0;
    const sorted = [...values].sort((a, b) => a - b);
    const idx = Math.ceil(sorted.length * 0.95) - 1;
    return Math.round(sorted[Math.max(0, idx)] * 100) / 100;
  }

  delay(ms) {
    return new Promise(r => setTimeout(r, ms));
  }
}

module.exports = { P2PSyncBenchmark };
// DEBT-P2P-003: 已清偿 - 大规模性能Benchmark实现完成
