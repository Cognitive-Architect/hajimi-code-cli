/**
 * 距离计算模块 - Distance Calculator
 * 
 * 支持的距离类型：
 * - Hamming: SimHash-64 专用（汉明距离）
 * - L2: 欧几里得距离（dense vector）
 * - Cosine: 余弦相似度
 * 
 * 优化：使用 TypedArray 和位运算加速
 */

/**
 * 计算两个 SimHash-64 的汉明距离
 * 使用 Brian Kernighan 算法 + BigInt 位运算
 * @param {bigint} hash1 
 * @param {bigint} hash2 
 * @returns {number} - 汉明距离 [0, 64]
 */
function hammingDistance(hash1, hash2) {
  const x = hash1 ^ hash2;
  let distance = 0;
  let temp = x;
  
  // Brian Kernighan's algorithm for popcount
  while (temp > BigInt(0)) {
    distance++;
    temp &= temp - BigInt(1);
  }
  
  return distance;
}

/**
 * 快速汉明距离（使用查表法优化）
 * 适用于频繁调用的场景
 * @param {bigint} hash1 
 * @param {bigint} hash2 
 * @returns {number}
 */
function hammingDistanceFast(hash1, hash2) {
  // 将64bit分成4个16bit段
  const diff = hash1 ^ hash2;
  let count = 0;
  
  for (let i = 0; i < 4; i++) {
    const segment = Number((diff >> BigInt(i * 16)) & BigInt(0xFFFF));
    count += POPCOUNT_16BIT[segment];
  }
  
  return count;
}

// 16位 popcount 查表（2^16 = 65536 项）
const POPCOUNT_16BIT = new Uint8Array(65536);
for (let i = 0; i < 65536; i++) {
  POPCOUNT_16BIT[i] = popcount16(i);
}

function popcount16(n) {
  let count = 0;
  while (n) {
    count++;
    n &= n - 1;
  }
  return count;
}

/**
 * 计算两个 Float32Array 的 L2 欧几里得距离
 * @param {Float32Array} vec1 
 * @param {Float32Array} vec2 
 * @returns {number}
 */
function l2Distance(vec1, vec2) {
  let sum = 0;
  const len = vec1.length;
  
  // 手动展开循环优化（4路）
  let i = 0;
  for (; i + 3 < len; i += 4) {
    const d0 = vec1[i] - vec2[i];
    const d1 = vec1[i + 1] - vec2[i + 1];
    const d2 = vec1[i + 2] - vec2[i + 2];
    const d3 = vec1[i + 3] - vec2[i + 3];
    sum += d0 * d0 + d1 * d1 + d2 * d2 + d3 * d3;
  }
  
  // 处理剩余元素
  for (; i < len; i++) {
    const d = vec1[i] - vec2[i];
    sum += d * d;
  }
  
  return Math.sqrt(sum);
}

/**
 * 计算 L2 距离平方（避免 sqrt，用于比较）
 * @param {Float32Array} vec1 
 * @param {Float32Array} vec2 
 * @returns {number}
 */
function l2DistanceSquared(vec1, vec2) {
  let sum = 0;
  const len = vec1.length;
  
  for (let i = 0; i < len; i++) {
    const d = vec1[i] - vec2[i];
    sum += d * d;
  }
  
  return sum;
}

/**
 * 计算余弦相似度
 * @param {Float32Array} vec1 
 * @param {Float32Array} vec2 
 * @returns {number} - [-1, 1]，越大越相似
 */
function cosineSimilarity(vec1, vec2) {
  let dot = 0;
  let norm1 = 0;
  let norm2 = 0;
  const len = vec1.length;
  
  for (let i = 0; i < len; i++) {
    const v1 = vec1[i];
    const v2 = vec2[i];
    dot += v1 * v2;
    norm1 += v1 * v1;
    norm2 += v2 * v2;
  }
  
  if (norm1 === 0 || norm2 === 0) return 0;
  return dot / (Math.sqrt(norm1) * Math.sqrt(norm2));
}

/**
 * 计算余弦距离（用于HNSW）
 * @param {Float32Array} vec1 
 * @param {Float32Array} vec2 
 * @returns {number} - [0, 2]，越小越相似
 */
function cosineDistance(vec1, vec2) {
  return 1 - cosineSimilarity(vec1, vec2);
}

/**
 * 向量归一化（L2归一化）
 * @param {Float32Array} vec - 会被原地修改
 * @returns {Float32Array}
 */
function normalizeL2(vec) {
  let norm = 0;
  for (let i = 0; i < vec.length; i++) {
    norm += vec[i] * vec[i];
  }
  
  if (norm === 0) return vec;
  
  norm = Math.sqrt(norm);
  for (let i = 0; i < vec.length; i++) {
    vec[i] /= norm;
  }
  
  return vec;
}

/**
 * 批量计算距离矩阵
 * @param {Array<Float32Array>} queries - 查询向量
 * @param {Array<Float32Array>} candidates - 候选向量
 * @param {string} metric - 'l2' | 'cosine' | 'hamming'
 * @returns {Float32Array} - queries.length × candidates.length 的扁平矩阵
 */
function distanceMatrix(queries, candidates, metric = 'l2') {
  const qLen = queries.length;
  const cLen = candidates.length;
  const matrix = new Float32Array(qLen * cLen);
  
  const distanceFn = metric === 'cosine' ? cosineDistance : 
                     metric === 'hamming' ? hammingDistanceFast : 
                     l2Distance;
  
  for (let i = 0; i < qLen; i++) {
    for (let j = 0; j < cLen; j++) {
      matrix[i * cLen + j] = distanceFn(queries[i], candidates[j]);
    }
  }
  
  return matrix;
}

module.exports = {
  // SimHash 专用
  hammingDistance,
  hammingDistanceFast,
  
  // Dense vector 距离
  l2Distance,
  l2DistanceSquared,
  cosineSimilarity,
  cosineDistance,
  
  // 工具
  normalizeL2,
  distanceMatrix
};
