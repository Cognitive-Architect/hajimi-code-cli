/**
 * 向量编码器 - Vector Encoder
 * 
 * 功能：SimHash-64 (64bit sparse) → HNSW dense vector (128/256 dim float32)
 * 
 * 编码策略：
 * 1. Binary Expansion: 64bit → 64维 (0/1值)
 * 2. Hadamard Projection: 64bit → 128/256维（正交变换）
 * 3. L2 Normalization: 避免HNSW距离失真
 * 
 * 输出格式：Float32Array，支持配置维度
 */

const { normalizeL2 } = require('./distance');

// 编码配置
const ENCODER_CONFIG = {
  outputDim: 128,           // 输出维度 (64|128|256)
  normalize: true,          // 是否L2归一化
  method: 'hadamard',       // 'binary' | 'hadamard' | 'random'
  seed: 'hajimi-encoder-v1' // 随机投影种子
};

/**
 * 向量编码器类
 */
class VectorEncoder {
  /**
   * @param {Object} config 
   */
  constructor(config = {}) {
    this.config = { ...ENCODER_CONFIG, ...config };
    this.projectionMatrix = null;
    
    // 预生成投影矩阵（非binary方法）
    if (this.config.method !== 'binary') {
      this._initProjectionMatrix();
    }
  }
  
  /**
   * 初始化随机投影矩阵
   */
  _initProjectionMatrix() {
    const { outputDim, method, seed } = this.config;
    const inputDim = 64;  // SimHash-64
    
    // Hadamard矩阵生成（简化版，使用递归）
    if (method === 'hadamard') {
      const hadamardSize = this._nextPowerOf2(outputDim);
      this.projectionMatrix = this._generateHadamard(hadamardSize);
      this.projectionOutputDim = Math.min(outputDim, hadamardSize);
    } else if (method === 'random') {
      // 随机高斯投影
      this.projectionMatrix = this._generateRandomProjection(inputDim, outputDim, seed);
      this.projectionOutputDim = outputDim;
    }
  }
  
  /**
   * 获取下一个2的幂
   */
  _nextPowerOf2(n) {
    return Math.pow(2, Math.ceil(Math.log2(n)));
  }
  
  /**
   * 生成 Hadamard 矩阵（Sylvester构造）
   * H_2n = [H_n, H_n; H_n, -H_n]
   * @param {number} n - 必须是2的幂
   * @returns {Float32Array[]} - 矩阵（每行一个Float32Array）
   */
  _generateHadamard(n) {
    if (n === 1) {
      return [new Float32Array([1])];
    }
    
    const half = n / 2;
    const halfMatrix = this._generateHadamard(half);
    const matrix = [];
    
    for (let i = 0; i < n; i++) {
      matrix[i] = new Float32Array(n);
      const isBottom = i >= half;
      const srcRow = halfMatrix[i % half];
      
      for (let j = 0; j < n; j++) {
        const isRight = j >= half;
        const srcVal = srcRow[j % half];
        
        // [H, H; H, -H]
        if (isBottom && isRight) {
          matrix[i][j] = -srcVal;
        } else {
          matrix[i][j] = srcVal;
        }
      }
    }
    
    return matrix;
  }
  
  /**
   * 生成随机投影矩阵
   */
  _generateRandomProjection(inputDim, outputDim, seed) {
    const crypto = require('crypto');
    const matrix = [];
    
    for (let i = 0; i < outputDim; i++) {
      const row = new Float32Array(inputDim);
      
      for (let j = 0; j < inputDim; j++) {
        // Box-Muller 生成标准正态分布
        const hash1 = crypto.createHash('sha256')
          .update(`${seed}:row${i}:col${j}:0`)
          .digest();
        const hash2 = crypto.createHash('sha256')
          .update(`${seed}:row${i}:col${j}:1`)
          .digest();
        
        const u1 = hash1.readUInt32LE(0) / 0xFFFFFFFF;
        const u2 = hash2.readUInt32LE(0) / 0xFFFFFFFF;
        
        const z = Math.sqrt(-2 * Math.log(u1 + 0.0001)) * Math.cos(2 * Math.PI * u2);
        row[j] = z / Math.sqrt(inputDim);  // 归一化
      }
      
      matrix.push(row);
    }
    
    return matrix;
  }
  
  /**
   * 将 SimHash-64 (bigint) 展开为 64维 binary vector
   * @param {bigint} simhash 
   * @returns {Float32Array}
   */
  _expandBinary(simhash) {
    const vector = new Float32Array(64);
    for (let i = 0; i < 64; i++) {
      // 从低位到高位提取
      vector[i] = (simhash & (BigInt(1) << BigInt(i))) !== BigInt(0) ? 1.0 : -1.0;
    }
    return vector;
  }
  
  /**
   * 应用 Hadamard 变换
   * @param {Float32Array} binary64 - 64维 binary vector
   * @returns {Float32Array}
   */
  _applyHadamard(binary64) {
    // 将64维扩展到256维（Hadamard要求2的幂）
    const padded = new Float32Array(256);
    for (let i = 0; i < 64; i++) {
      padded[i] = binary64[i];
    }
    // 其余补0
    
    // 应用 Hadamard 矩阵
    const output = new Float32Array(this.projectionOutputDim);
    const hadamardMatrix = this.projectionMatrix;
    
    for (let i = 0; i < this.projectionOutputDim; i++) {
      let sum = 0;
      const row = hadamardMatrix[i];
      // 256维点积，但后192维是0，只需算前64
      for (let j = 0; j < 64; j++) {
        sum += row[j] * binary64[j];
      }
      output[i] = sum;
    }
    
    return output;
  }
  
  /**
   * 应用随机投影
   * @param {Float32Array} binary64 
   * @returns {Float32Array}
   */
  _applyRandomProjection(binary64) {
    const output = new Float32Array(this.projectionOutputDim);
    
    for (let i = 0; i < this.projectionOutputDim; i++) {
      let sum = 0;
      const row = this.projectionMatrix[i];
      for (let j = 0; j < 64; j++) {
        sum += row[j] * binary64[j];
      }
      output[i] = sum;
    }
    
    return output;
  }
  
  /**
   * 编码单个 SimHash
   * @param {bigint} simhash - SimHash-64 值
   * @returns {Float32Array} - dense vector
   */
  encode(simhash) {
    if (typeof simhash !== 'bigint') {
      throw new TypeError('simhash must be bigint');
    }
    
    let vector;
    
    switch (this.config.method) {
      case 'binary':
        // 64维 0/1 向量
        vector = this._expandBinary(simhash);
        break;
        
      case 'hadamard':
        // 64维 → Hadamard → 128/256维
        const binary = this._expandBinary(simhash);
        vector = this._applyHadamard(binary);
        break;
        
      case 'random':
        // 64维 → 随机投影 → 128/256维
        const binary2 = this._expandBinary(simhash);
        vector = this._applyRandomProjection(binary2);
        break;
        
      default:
        throw new Error(`Unknown encoding method: ${this.config.method}`);
    }
    
    // L2 归一化
    if (this.config.normalize) {
      normalizeL2(vector);
    }
    
    return vector;
  }
  
  /**
   * 批量编码
   * @param {Array<bigint>} simhashes 
   * @returns {Array<Float32Array>}
   */
  encodeBatch(simhashes) {
    return simhashes.map(h => this.encode(h));
  }
  
  /**
   * 获取输出维度
   */
  getOutputDimension() {
    switch (this.config.method) {
      case 'binary':
        return 64;
      case 'hadamard':
      case 'random':
        return this.projectionOutputDim;
      default:
        return this.config.outputDim;
    }
  }
  
  /**
   * 编码配置序列化
   */
  toJSON() {
    return {
      config: { ...this.config },
      outputDim: this.getOutputDimension()
    };
  }
}

// 静态工具方法

/**
 * 快速编码（使用默认配置）
 * @param {bigint} simhash 
 * @returns {Float32Array}
 */
function quickEncode(simhash) {
  const encoder = new VectorEncoder({ method: 'hadamard', outputDim: 128 });
  return encoder.encode(simhash);
}

/**
 * 批量快速编码
 * @param {Array<bigint>} simhashes 
 * @returns {Array<Float32Array>}
 */
function quickEncodeBatch(simhashes) {
  const encoder = new VectorEncoder({ method: 'hadamard', outputDim: 128 });
  return encoder.encodeBatch(simhashes);
}

module.exports = {
  VectorEncoder,
  quickEncode,
  quickEncodeBatch,
  ENCODER_CONFIG
};
