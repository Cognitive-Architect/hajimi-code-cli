#!/usr/bin/env node
/**
 * DEBT-LSH-001-FIXED: LSH 假阳性率模拟验证（生产级SimHash版）
 * 
 * 修正内容：
 * - 使用生产级 SimHash-64 实现（随机超平面投影）
 * - 汉明距离分布峰值在32附近（符合理论）
 * - 与简化版实现差异显式声明
 * 
 * 使用方法: node lsh-collision-sim-v2.js [--vectors N] [--queries M] [--verbose]
 */

const crypto = require('crypto');
const { simhash64, hammingDistance } = require('../utils/simhash64');

// ==================== 配置 ====================
const CONFIG = {
  // LSH 参数 (来自设计文档)
  dim: 768,                    // 向量维度
  numTables: 8,                // LSH 表数量
  numHashes: 8,                // 每个表的哈希函数数量
  bucketWidth: 4,              // 桶宽度 (汉明距离 < 4)
  
  // 测试参数
  defaultVectorCount: 100000,  // 默认向量数
  defaultQueryCount: 1000,     // 默认查询数
  
  // 目标假阳性率
  targetFPR: 0.001             // 0.1%
};

// ==================== LSH 实现 ====================

class LSHIndex {
  constructor(config) {
    this.config = config;
    this.tables = Array(config.numTables).fill(null).map(() => new Map());
    this.vectors = new Map();  // id -> vector
    this.vectorHashes = new Map();  // id -> simhash
  }
  
  /**
   * 获取桶 key: 将 64bit hash 分成 numHashes 段
   */
  _getBucketKey(simhash, tableIndex) {
    const bitsPerHash = Math.floor(64 / this.config.numHashes);
    const buckets = [];
    
    for (let i = 0; i < this.config.numHashes; i++) {
      // 每个表使用不同的比特位组合
      const bitStart = (tableIndex * 8 + i) % (64 - bitsPerHash);
      const mask = (BigInt(1) << BigInt(bitsPerHash)) - BigInt(1);
      const bucket = Number((simhash >> BigInt(bitStart)) & mask);
      buckets.push(bucket);
    }
    
    return buckets.join('_');
  }
  
  /**
   * 添加向量到索引
   */
  add(id, vector) {
    const simhash = simhash64(vector);
    this.vectors.set(id, vector);
    this.vectorHashes.set(id, simhash);
    
    // 添加到所有 LSH 表
    for (let t = 0; t < this.config.numTables; t++) {
      const bucketKey = this._getBucketKey(simhash, t);
      if (!this.tables[t].has(bucketKey)) {
        this.tables[t].set(bucketKey, []);
      }
      this.tables[t].get(bucketKey).push(id);
    }
  }
  
  /**
   * 查询候选集 (不含重排序)
   */
  queryCandidates(queryVector) {
    const queryHash = simhash64(queryVector);
    const candidates = new Set();
    
    for (let t = 0; t < this.config.numTables; t++) {
      const bucketKey = this._getBucketKey(queryHash, t);
      const bucket = this.tables[t].get(bucketKey);
      
      if (bucket) {
        // 检查汉明距离
        for (const id of bucket) {
          const vectorHash = this.vectorHashes.get(id);
          const distance = hammingDistance(queryHash, vectorHash);
          
          if (distance < this.config.bucketWidth) {
            candidates.add(id);
          }
        }
      }
    }
    
    return Array.from(candidates);
  }
  
  /**
   * 计算余弦相似度
   */
  _cosineSimilarity(v1, v2) {
    let dot = 0, norm1 = 0, norm2 = 0;
    for (let i = 0; i < v1.length; i++) {
      dot += v1[i] * v2[i];
      norm1 += v1[i] * v1[i];
      norm2 += v2[i] * v2[i];
    }
    return dot / (Math.sqrt(norm1) * Math.sqrt(norm2));
  }
  
  /**
   * 精确查询 (暴力计算)
   */
  queryExact(queryVector, k) {
    const similarities = [];
    for (const [id, vector] of this.vectors) {
      const sim = this._cosineSimilarity(queryVector, vector);
      similarities.push({ id, similarity: sim });
    }
    similarities.sort((a, b) => b.similarity - a.similarity);
    return similarities.slice(0, k);
  }
}

// ==================== 测试 ====================

/**
 * 生成随机单位向量
 */
function generateRandomVector(dim) {
  const vector = new Float32Array(dim);
  let norm = 0;
  
  for (let i = 0; i < dim; i++) {
    vector[i] = (Math.random() * 2 - 1);
    norm += vector[i] * vector[i];
  }
  
  // 归一化
  norm = Math.sqrt(norm);
  for (let i = 0; i < dim; i++) {
    vector[i] /= norm;
  }
  
  return vector;
}

/**
 * 计算理论假阳性率 (泊松近似)
 */
function calculateTheoreticalFPR(config, n) {
  // 计算单个汉明距离 < bucketWidth 的概率
  let pSingleCollision = 0;
  for (let i = 0; i < config.bucketWidth; i++) {
    pSingleCollision += nCr(64, i) / Math.pow(2, 64);
  }
  
  // 单个表的假阳性率 (近似)
  const pSingleTableFP = 1 - Math.pow(1 - pSingleCollision, n);
  
  // 所有表的联合假阳性率
  const pAllTablesFP = 1 - Math.pow(1 - pSingleTableFP, config.numTables);
  
  return {
    pSingleCollision,
    pSingleTableFP,
    pAllTablesFP
  };
}

function nCr(n, r) {
  if (r > n) return 0;
  if (r === 0 || r === n) return 1;
  
  let result = 1;
  for (let i = 0; i < r; i++) {
    result = result * (n - i) / (i + 1);
  }
  return result;
}

/**
 * 主测试函数
 */
async function runSimulation(options = {}) {
  const vectorCount = options.vectors || CONFIG.defaultVectorCount;
  const queryCount = options.queries || CONFIG.defaultQueryCount;
  const verbose = options.verbose || false;
  
  console.log('='.repeat(60));
  console.log('DEBT-LSH-001-FIXED: LSH 假阳性率模拟验证（生产级SimHash）');
  console.log('='.repeat(60));
  console.log(`\n配置:`);
  console.log(`  - 向量维度: ${CONFIG.dim}`);
  console.log(`  - LSH 表数: ${CONFIG.numTables}`);
  console.log(`  - 每表哈希数: ${CONFIG.numHashes}`);
  console.log(`  - 桶宽度 (汉明距离): ${CONFIG.bucketWidth}`);
  console.log(`  - 向量总数: ${vectorCount.toLocaleString()}`);
  console.log(`  - 查询次数: ${queryCount.toLocaleString()}`);
  console.log(`  - 目标假阳性率: ${(CONFIG.targetFPR * 100).toFixed(2)}%`);
  console.log(`\n  [FIXED] 使用生产级 SimHash-64（随机超平面投影）`);
  
  // 1. 理论计算
  console.log('\n' + '-'.repeat(60));
  console.log('1. 理论假阳性率计算 (泊松近似)');
  console.log('-'.repeat(60));
  
  const theoretical = calculateTheoreticalFPR(CONFIG, vectorCount);
  console.log(`  单个汉明碰撞概率 p: ${theoretical.pSingleCollision.toExponential(4)}`);
  console.log(`  单表假阳性率: ${theoretical.pSingleTableFP.toExponential(4)}`);
  console.log(`  联合假阳性率: ${theoretical.pAllTablesFP.toExponential(4)}`);
  console.log(`  → 理论值: ${(theoretical.pAllTablesFP * 100).toExponential(2)}%`);
  
  // 2. 构建索引
  console.log('\n' + '-'.repeat(60));
  console.log('2. 构建 LSH 索引');
  console.log('-'.repeat(60));
  
  const lsh = new LSHIndex(CONFIG);
  const startBuild = Date.now();
  
  for (let i = 0; i < vectorCount; i++) {
    const vector = generateRandomVector(CONFIG.dim);
    lsh.add(i, vector);
    
    if (verbose && (i + 1) % 10000 === 0) {
      console.log(`  已添加: ${(i + 1).toLocaleString()} 向量`);
    }
  }
  
  const buildTime = Date.now() - startBuild;
  console.log(`  构建时间: ${buildTime}ms`);
  
  // 统计表分布
  if (verbose) {
    console.log('\n  LSH 表分布:');
    for (let t = 0; t < CONFIG.numTables; t++) {
      let totalBuckets = lsh.tables[t].size;
      let totalEntries = 0;
      for (const [, ids] of lsh.tables[t]) {
        totalEntries += ids.length;
      }
      console.log(`    表 ${t}: ${totalBuckets.toLocaleString()} 桶, ${totalEntries.toLocaleString()} 条目`);
    }
  }
  
  // 3. 执行查询测试
  console.log('\n' + '-'.repeat(60));
  console.log('3. 假阳性率实测');
  console.log('-'.repeat(60));
  
  let totalCandidates = 0;
  let totalExactMatches = 0;
  let falsePositives = 0;
  let totalQueryTime = 0;
  
  // 汉明距离分布采样（用于验证生产级SimHash）
  const hammingSamples = [];
  
  for (let q = 0; q < queryCount; q++) {
    const queryVector = generateRandomVector(CONFIG.dim);
    
    // LSH 候选
    const startQuery = Date.now();
    const candidates = lsh.queryCandidates(queryVector);
    totalQueryTime += Date.now() - startQuery;
    
    // 精确查询 (仅对比候选集)
    const exactResults = lsh.queryExact(queryVector, 10);
    const exactTopIds = new Set(exactResults.map(r => parseInt(r.id)));
    
    // 统计
    totalCandidates += candidates.length;
    
    let queryFP = 0;
    for (const candidateId of candidates) {
      if (!exactTopIds.has(candidateId)) {
        queryFP++;
      }
    }
    falsePositives += queryFP;
    
    // 采样汉明距离
    if (q < 100) {
      const queryHash = simhash64(queryVector);
      for (let i = 0; i < Math.min(10, vectorCount); i++) {
        const v = lsh.vectorHashes.get(i);
        if (v) hammingSamples.push(hammingDistance(queryHash, v));
      }
    }
    
    if (verbose && (q + 1) % 100 === 0) {
      console.log(`  查询 ${q + 1}: 候选=${candidates.length}, FP=${queryFP}`);
    }
  }
  
  // 4. 计算统计结果
  const avgCandidates = totalCandidates / queryCount;
  const measuredFPR = falsePositives / totalCandidates;
  const avgQueryTime = totalQueryTime / queryCount;
  
  // 汉明距离分布
  const avgHamming = hammingSamples.reduce((a, b) => a + b, 0) / hammingSamples.length;
  
  console.log(`\n  结果统计:`);
  console.log(`  - 平均候选数/查询: ${avgCandidates.toFixed(2)}`);
  console.log(`  - 假阳性总数: ${falsePositives.toLocaleString()}`);
  console.log(`  - 候选总数: ${totalCandidates.toLocaleString()}`);
  console.log(`  - 实测假阳性率: ${(measuredFPR * 100).toFixed(4)}%`);
  console.log(`  - 平均查询时间: ${avgQueryTime.toFixed(2)}ms`);
  console.log(`\n  [FIXED验证] 汉明距离分布:`);
  console.log(`  - 采样数: ${hammingSamples.length}`);
  console.log(`  - 平均汉明距离: ${avgHamming.toFixed(2)} (预期≈32)`);
  console.log(`  - 分布状态: ${Math.abs(avgHamming - 32) < 3 ? '✅ 峰值在32附近' : '⚠️ 分布异常'}`);
  
  // 5. 结论
  console.log('\n' + '='.repeat(60));
  console.log('4. 验证结论');
  console.log('='.repeat(60));
  
  const passed = measuredFPR < CONFIG.targetFPR && Math.abs(avgHamming - 32) < 3;
  
  if (passed) {
    console.log(`\n  ✅ 通过: 实测 FPR ${(measuredFPR * 100).toFixed(4)}% < 目标 ${(CONFIG.targetFPR * 100).toFixed(2)}%`);
    console.log(`  ✅ 通过: 汉明距离分布峰值在32附近 (实际平均${avgHamming.toFixed(1)})`);
  } else {
    console.log(`\n  ❌ 未通过:`);
    if (measuredFPR >= CONFIG.targetFPR) {
      console.log(`     实测 FPR ${(measuredFPR * 100).toFixed(4)}% > 目标 ${(CONFIG.targetFPR * 100).toFixed(2)}%`);
    }
    if (Math.abs(avgHamming - 32) >= 3) {
      console.log(`     汉明距离分布异常 (平均${avgHamming.toFixed(1)}, 预期32)`);
    }
  }
  
  // 6. 输出 JSON 报告
  const report = {
    config: CONFIG,
    parameters: {
      vectorCount,
      queryCount
    },
    theoretical: {
      pSingleCollision: theoretical.pSingleCollision,
      pSingleTableFP: theoretical.pSingleTableFP,
      pAllTablesFP: theoretical.pAllTablesFP,
      fprPercent: theoretical.pAllTablesFP * 100
    },
    measured: {
      totalCandidates,
      falsePositives,
      fpr: measuredFPR,
      fprPercent: measuredFPR * 100,
      avgCandidatesPerQuery: avgCandidates,
      avgQueryTimeMs: avgQueryTime,
      buildTimeMs: buildTime,
      avgHammingDistance: avgHamming
    },
    implementation: {
      version: 'v2-production',
      simhashType: 'random-projection',
      hammingPeak: Math.abs(avgHamming - 32) < 3 ? 'at-32' : 'abnormal'
    },
    passed,
    timestamp: new Date().toISOString()
  };
  
  if (options.outputJson) {
    console.log('\n' + '-'.repeat(60));
    console.log('JSON 报告:');
    console.log('-'.repeat(60));
    console.log(JSON.stringify(report, null, 2));
  }
  
  return report;
}

// ==================== CLI ====================

function main() {
  const args = process.argv.slice(2);
  const options = {
    vectors: CONFIG.defaultVectorCount,
    queries: CONFIG.defaultQueryCount,
    verbose: false,
    outputJson: false
  };
  
  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case '--vectors':
      case '-n':
        options.vectors = parseInt(args[++i]);
        break;
      case '--queries':
      case '-q':
        options.queries = parseInt(args[++i]);
        break;
      case '--verbose':
      case '-v':
        options.verbose = true;
        break;
      case '--json':
      case '-j':
        options.outputJson = true;
        break;
      case '--help':
      case '-h':
        console.log('用法: node lsh-collision-sim-v2.js [选项]');
        console.log('');
        console.log('选项:');
        console.log('  -n, --vectors N    向量数量 (默认: 100000)');
        console.log('  -q, --queries N    查询数量 (默认: 1000)');
        console.log('  -v, --verbose      详细输出');
        console.log('  -j, --json         输出 JSON 报告');
        console.log('  -h, --help         显示帮助');
        console.log('');
        console.log('差异声明:');
        console.log('  v2版使用生产级SimHash（随机超平面投影）');
        console.log('  v1版使用简化SimHash（MD5派生权重）');
        process.exit(0);
    }
  }
  
  runSimulation(options).then(report => {
    process.exit(report.passed ? 0 : 1);
  }).catch(err => {
    console.error('错误:', err);
    process.exit(1);
  });
}

if (require.main === module) {
  main();
}

module.exports = { runSimulation, LSHIndex };
