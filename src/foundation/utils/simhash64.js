/**
 * SimHash-64 生产级实现
 * 
 * 特性：
 * - 使用随机超平面投影
 * - 汉明距离分布峰值在32附近（随机向量）
 * - 支持768维float32向量输入
 * 
 * 与简化版差异：
 * - 简化版使用MD5派生权重（确定性但分布不均）
 * - 生产版使用正态分布随机投影（符合LSH理论）
 */

const crypto = require('crypto');

// 预生成随机投影向量（维度768 × 64个超平面）
// 使用标准正态分布 N(0,1) 的随机权重
let PROJECTION_VECTORS = null;

/**
 * 初始化投影向量（延迟加载，可复用）
 */
function initProjectionVectors(dim = 768) {
  if (PROJECTION_VECTORS) return PROJECTION_VECTORS;
  
  PROJECTION_VECTORS = [];
  
  // 使用确定性种子生成，确保跨环境一致性
  const seed = 'hajimi-simhash-v1';
  
  for (let i = 0; i < 64; i++) {
    const weights = new Float64Array(dim);
    
    for (let j = 0; j < dim; j++) {
      // Box-Muller变换生成正态分布
      const u1 = randomFloat(seed, i, j, 0);
      const u2 = randomFloat(seed, i, j, 1);
      
      const z = Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
      weights[j] = z;
    }
    
    PROJECTION_VECTORS.push(weights);
  }
  
  return PROJECTION_VECTORS;
}

/**
 * 确定性伪随机数生成（基于seed）
 */
function randomFloat(seed, i, j, k) {
  const hash = crypto.createHash('sha256')
    .update(`${seed}:${i}:${j}:${k}`)
    .digest();
  
  // 取前4字节作为Uint32，转为[0,1)浮点数
  const uint32 = hash.readUInt32LE(0);
  return (uint32 % 1000000) / 1000000;
}

/**
 * 计算 SimHash-64
 * @param {Float32Array|Array} vector - 输入向量（已归一化）
 * @returns {bigint} - 64位SimHash值
 */
function simhash64(vector) {
  const dim = vector.length;
  const projections = initProjectionVectors(dim);
  
  // 计算与64个超平面的点积
  const bits = new Array(64).fill(0);
  
  for (let i = 0; i < 64; i++) {
    let dot = 0;
    const proj = projections[i];
    
    for (let j = 0; j < dim; j++) {
      dot += vector[j] * proj[j];
    }
    
    bits[i] = dot >= 0 ? 1 : 0;
  }
  
  // 组装64位BigInt
  let hash = BigInt(0);
  for (let i = 0; i < 64; i++) {
    if (bits[i]) {
      hash |= (BigInt(1) << BigInt(i));
    }
  }
  
  return hash;
}

/**
 * 计算两个SimHash的汉明距离
 * @param {bigint} hash1 
 * @param {bigint} hash2 
 * @returns {number} - 汉明距离 [0, 64]
 */
function hammingDistance(hash1, hash2) {
  const x = hash1 ^ hash2;
  let distance = 0;
  let temp = x;
  
  // Brian Kernighan's algorithm
  while (temp > BigInt(0)) {
    distance++;
    temp &= temp - BigInt(1);
  }
  
  return distance;
}

/**
 * 汉明距离分布测试（用于验证实现正确性）
 * 随机向量之间的汉明距离应该近似正态分布，峰值在32
 * @returns {Object} 分布统计
 */
function testHammingDistribution(sampleSize = 1000) {
  const dim = 768;
  const distances = [];
  
  // 生成随机单位向量
  function randomUnitVector() {
    const v = new Float32Array(dim);
    let norm = 0;
    
    for (let i = 0; i < dim; i++) {
      v[i] = Math.random() * 2 - 1;
      norm += v[i] * v[i];
    }
    
    norm = Math.sqrt(norm);
    for (let i = 0; i < dim; i++) {
      v[i] /= norm;
    }
    
    return v;
  }
  
  // 预生成样本
  const samples = [];
  for (let i = 0; i < sampleSize; i++) {
    samples.push(simhash64(randomUnitVector()));
  }
  
  // 计算两两汉明距离
  for (let i = 0; i < sampleSize; i++) {
    for (let j = i + 1; j < Math.min(i + 10, sampleSize); j++) {
      distances.push(hammingDistance(samples[i], samples[j]));
    }
  }
  
  // 统计分布
  const stats = {
    min: Math.min(...distances),
    max: Math.max(...distances),
    mean: distances.reduce((a, b) => a + b, 0) / distances.length,
    median: distances.sort((a, b) => a - b)[Math.floor(distances.length / 2)],
    distribution: {}
  };
  
  // 分桶统计
  for (let d = 0; d <= 64; d++) {
    const count = distances.filter(x => x === d).length;
    if (count > 0) {
      stats.distribution[d] = count;
    }
  }
  
  return stats;
}

/**
 * 批量计算SimHash（性能优化版）
 * @param {Array<Float32Array>} vectors 
 * @returns {Array<bigint>}
 */
function simhash64Batch(vectors) {
  return vectors.map(v => simhash64(v));
}

module.exports = {
  simhash64,
  hammingDistance,
  testHammingDistribution,
  simhash64Batch
};
