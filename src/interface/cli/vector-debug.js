#!/usr/bin/env node
/**
 * HNSW 向量检索调试 CLI
 * 
 * 命令：
 *   hajimi vector-build [shardId]     构建/重建 HNSW 索引
 *   hajimi vector-search <simhash>    搜索相似向量
 *   hajimi vector-stats [shardId]     查看索引统计
 *   hajimi vector-benchmark           运行基准测试
 *   hajimi vector-test                运行单元测试
 * 
 * 环境变量：
 *   HAJIMI_HOME - 数据根目录（默认 ~/.hajimi）
 */

const fs = require('fs').promises;
const path = require('path');

// 导入模块
const { VectorAPI } = require('../api/vector-api');
const { runAllBenchmarks } = require('../test/hnsw-benchmark.test');

// 工具函数
const HOME = process.env.HAJIMI_HOME || path.join(process.env.HOME || '.', '.hajimi');

/**
 * 打印进度条
 */
function progressBar(current, total, width = 40) {
  const ratio = current / total;
  const filled = Math.floor(width * ratio);
  const empty = width - filled;
  const percentage = Math.floor(ratio * 100);
  
  const bar = '█'.repeat(filled) + '░'.repeat(empty);
  process.stdout.write(`\r[${bar}] ${percentage}% (${current}/${total})`);
  
  if (current === total) {
    process.stdout.write('\n');
  }
}

/**
 * 解析 SimHash 字符串
 */
function parseSimhash(str) {
  // 支持 0x 前缀或纯十六进制
  const hex = str.startsWith('0x') ? str.slice(2) : str;
  return BigInt('0x' + hex);
}

/**
 * 格式化 SimHash
 */
function formatSimhash(simhash) {
  return '0x' + simhash.toString(16).padStart(16, '0');
}

/**
 * 命令：构建索引
 */
async function cmdBuild(shardId = 'default') {
  console.log(`\n🔨 Building HNSW index for shard ${shardId}...\n`);
  
  const api = new VectorAPI({ shardId });
  await api.init();
  
  // 模拟批量导入（实际应从 Chunk 文件读取）
  console.log('Importing vectors...');
  
  // 生成测试数据（实际应用应替换为真实数据）
  const testCount = 10000;
  const items = [];
  
  for (let i = 0; i < testCount; i++) {
    const high = BigInt(Math.floor(Math.random() * 0x100000000));
    const low = BigInt(Math.floor(Math.random() * 0x100000000));
    items.push({
      simhash: (high << BigInt(32)) | low,
      metadata: { id: i, test: true }
    });
  }
  
  const startTime = Date.now();
  await api.putVectors(items, (c, t) => {
    if (c % 100 === 0 || c === t) {
      progressBar(c, t);
    }
  });
  
  const duration = Date.now() - startTime;
  
  console.log(`\n✅ Import complete!`);
  console.log(`   Vectors: ${testCount.toLocaleString()}`);
  console.log(`   Time: ${(duration/1000).toFixed(1)}s`);
  console.log(`   Rate: ${(testCount/(duration/1000)).toFixed(0)} vectors/s`);
  
  // 保存
  await api.save();
  console.log('\n💾 Index saved');
  
  await api.close();
}

/**
 * 命令：搜索向量
 */
async function cmdSearch(simhashStr, k = 10) {
  console.log(`\n🔍 Searching for similar vectors...\n`);
  
  const querySimhash = parseSimhash(simhashStr);
  console.log(`Query: ${formatSimhash(querySimhash)}`);
  console.log(`Top-K: ${k}\n`);
  
  const api = new VectorAPI();
  await api.init();
  
  const startTime = process.hrtime.bigint();
  const results = api.searchVector(querySimhash, k);
  const duration = Number(process.hrtime.bigint() - startTime) / 1000000;
  
  console.log(`Found ${results.length} results in ${duration.toFixed(2)}ms:\n`);
  
  console.log('┌──────┬────────────────────┬──────────┬────────────────┬────────┐');
  console.log('│  ID  │ SimHash            │ Distance │ Hamming Dist   │ Source │');
  console.log('├──────┼────────────────────┼──────────┼────────────────┼────────┤');
  
  for (const r of results) {
    const id = r.id.toString().padStart(4);
    const hash = formatSimhash(r.simhash).slice(0, 18).padEnd(18);
    const dist = r.distance.toFixed(4).padStart(8);
    const hamming = (r.hammingDistance ?? '-').toString().padStart(14);
    const src = r.source.padStart(6);
    console.log(`│ ${id} │ ${hash} │ ${dist} │ ${hamming} │ ${src} │`);
  }
  
  console.log('└──────┴────────────────────┴──────────┴────────────────┴────────┘');
  
  // 显示降级状态
  const fallbackStatus = api.getFallbackStatus();
  console.log(`\nFallback Status: ${fallbackStatus.state}`);
  console.log(`Memory Usage: ${fallbackStatus.memoryUsageMB.toFixed(1)}MB`);
  
  await api.close();
}

/**
 * 命令：查看统计
 */
async function cmdStats(shardId = 'default') {
  console.log(`\n📊 HNSW Index Statistics\n`);
  
  const api = new VectorAPI({ shardId });
  await api.init();
  
  const stats = api.getStats();
  
  console.log('=== API Statistics ===');
  console.log(`Total Puts:    ${stats.totalPuts.toLocaleString()}`);
  console.log(`Total Gets:    ${stats.totalGets.toLocaleString()}`);
  console.log(`Total Searches: ${stats.totalSearches.toLocaleString()}`);
  console.log(`Uptime:        ${(stats.uptime / 1000).toFixed(1)}s`);
  
  console.log('\n=== Retriever Statistics ===');
  const r = stats.retriever;
  console.log(`Total Queries:   ${r.totalQueries.toLocaleString()}`);
  console.log(`HNSW Queries:    ${r.hnswQueries.toLocaleString()} (${(r.hnswQueries/r.totalQueries*100).toFixed(1)}%)`);
  console.log(`LSH Queries:     ${r.lshQueries.toLocaleString()} (${(r.lshQueries/r.totalQueries*100).toFixed(1)}%)`);
  console.log(`Documents:       ${r.totalDocuments.toLocaleString()}`);
  console.log(`HNSW Coverage:   ${(r.hnswCoverage * 100).toFixed(1)}%`);
  
  if (r.hnswStats) {
    console.log('\n=== HNSW Index Statistics ===');
    console.log(`Element Count:   ${r.hnswStats.elementCount.toLocaleString()}`);
    console.log(`Active Count:    ${r.hnswStats.activeCount.toLocaleString()}`);
    console.log(`Deleted Count:   ${r.hnswStats.deletedCount.toLocaleString()}`);
    console.log(`Max Level:       ${r.hnswStats.maxLevel}`);
    console.log(`Entry Point:     ${r.hnswStats.entryPoint}`);
    console.log(`Avg Connections: ${r.hnswStats.avgConnections.toFixed(2)}`);
  }
  
  console.log('\n=== Memory Statistics ===');
  const m = stats.memory;
  console.log(`Current Memory:  ${m.memory.currentMB.toFixed(1)}MB`);
  console.log(`Pressure Level:  ${m.memory.pressureLevel}`);
  console.log(`Pool Hit Rate:   ${m.pool.hitRate}`);
  console.log(`Cache Hit Rate:  ${m.cache.hitRate}`);
  
  console.log('\n=== Fallback Status ===');
  const f = r.fallbackStatus;
  console.log(`Circuit State:   ${f.state}`);
  console.log(`Failure Count:   ${f.failureCount}`);
  console.log(`Success Count:   ${f.successCount}`);
  console.log(`Memory Usage:    ${f.memoryUsageMB.toFixed(1)}MB`);
  
  await api.close();
}

/**
 * 命令：运行基准测试
 */
async function cmdBenchmark() {
  await runAllBenchmarks();
}

/**
 * 命令：运行单元测试
 */
async function cmdTest() {
  console.log('\n🧪 Running HNSW unit tests...\n');
  
  const tests = [];
  let passed = 0;
  let failed = 0;
  
  function test(name, fn) {
    tests.push({ name, fn });
  }
  
  // 测试 1: HNSW 插入与搜索
  test('HNSW-CF-001: Insert and search single vector', async () => {
    const { HNSWIndex } = require('../vector/hnsw-core');
    const { VectorEncoder } = require('../vector/encoder');
    
    const index = new HNSWIndex({ distanceMetric: 'l2' });
    const encoder = new VectorEncoder();
    
    const simhash = BigInt('0x1234567890abcdef');
    const vector = encoder.encode(simhash);
    
    index.insert(0, vector);
    const results = index.search(vector, 1);
    
    if (results.length !== 1 || results[0].id !== 0) {
      throw new Error('Search returned wrong result');
    }
  });
  
  // 测试 2: 批量插入
  test('HNSW-CF-002: Batch insert 1000 vectors', async () => {
    const { HNSWIndex } = require('../vector/hnsw-core');
    const { VectorEncoder } = require('../vector/encoder');
    
    const index = new HNSWIndex({ distanceMetric: 'l2' });
    const encoder = new VectorEncoder();
    
    const start = Date.now();
    for (let i = 0; i < 1000; i++) {
      const simhash = BigInt(i);
      const vector = encoder.encode(simhash);
      index.insert(i, vector);
    }
    const duration = Date.now() - start;
    
    if (duration > 5000) {
      throw new Error(`Too slow: ${duration}ms > 5000ms`);
    }
    
    if (index.elementCount !== 1000) {
      throw new Error(`Wrong count: ${index.elementCount}`);
    }
  });
  
  // 测试 3: 编码器
  test('HNSW-CF-003: Encode 64bit to 128-dim vector', async () => {
    const { VectorEncoder } = require('../vector/encoder');
    
    const encoder = new VectorEncoder({ method: 'hadamard', outputDim: 128 });
    const simhash = BigInt('0x1234567890abcdef');
    const vector = encoder.encode(simhash);
    
    if (vector.length !== 128) {
      throw new Error(`Wrong dimension: ${vector.length}`);
    }
  });
  
  // 测试 4: 降级切换
  test('HNSW-CF-005: Fallback to LSH when HNSW fails', async () => {
    const { HybridRetriever } = require('../vector/hybrid-retriever');
    
    const retriever = new HybridRetriever();
    
    // 添加一些文档
    for (let i = 0; i < 100; i++) {
      retriever.add(BigInt(i), { id: i });
    }
    
    // 强制熔断
    retriever.circuitBreaker.forceOpen();
    
    // 搜索应该使用 LSH
    const results = retriever.search(BigInt(50), 5);
    
    if (results.length === 0) {
      throw new Error('No results returned');
    }
    
    if (results[0].source !== 'lsh') {
      throw new Error('Should use LSH fallback');
    }
  });
  
  // 测试 5: 空索引搜索
  test('HNSW-NG-001: Search empty index returns empty', async () => {
    const { HNSWIndex } = require('../vector/hnsw-core');
    const { VectorEncoder } = require('../vector/encoder');
    
    const index = new HNSWIndex();
    const encoder = new VectorEncoder();
    const vector = encoder.encode(BigInt(0));
    
    const results = index.search(vector, 10);
    
    if (results.length !== 0) {
      throw new Error('Should return empty array');
    }
  });
  
  // 运行所有测试
  for (const { name, fn } of tests) {
    try {
      await fn();
      console.log(`  ✅ ${name}`);
      passed++;
    } catch (err) {
      console.log(`  ❌ ${name}`);
      console.log(`     ${err.message}`);
      failed++;
    }
  }
  
  console.log(`\n${passed}/${tests.length} tests passed`);
  
  if (failed > 0) {
    process.exit(1);
  }
}

/**
 * 显示帮助
 */
function showHelp() {
  console.log(`
Hajimi Vector Debug CLI

Usage:
  node vector-debug.js <command> [options]

Commands:
  build [shardId]          Build/rebuild HNSW index
  search <simhash> [k]     Search similar vectors
  stats [shardId]          Show index statistics
  benchmark                Run performance benchmarks
  test                     Run unit tests
  help                     Show this help

Examples:
  node vector-debug.js build 0
  node vector-debug.js search 0x1234567890abcdef 10
  node vector-debug.js stats
  node vector-debug.js benchmark
`);
}

/**
 * 主函数
 */
async function main() {
  const args = process.argv.slice(2);
  const command = args[0];
  
  try {
    switch (command) {
      case 'build':
      case 'b':
        await cmdBuild(args[1]);
        break;
        
      case 'search':
      case 's':
        if (!args[1]) {
          console.error('Error: simhash required');
          process.exit(1);
        }
        await cmdSearch(args[1], parseInt(args[2]) || 10);
        break;
        
      case 'stats':
        await cmdStats(args[1]);
        break;
        
      case 'benchmark':
      case 'bench':
        await cmdBenchmark();
        break;
        
      case 'test':
      case 't':
        await cmdTest();
        break;
        
      case 'help':
      case '-h':
      case '--help':
      default:
        showHelp();
        break;
    }
  } catch (err) {
    console.error('\n❌ Error:', err.message);
    if (process.env.DEBUG) {
      console.error(err.stack);
    }
    process.exit(1);
  }
}

// 运行
main();
