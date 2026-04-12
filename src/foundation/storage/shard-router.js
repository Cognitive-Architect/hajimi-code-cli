/**
 * ShardRouter - 分片路由核心
 * 
 * 功能：
 * - SimHash-64 高 8bit → 分片 00-15 路由
 * - 分片路径生成
 * - SimHash 格式校验
 * 
 * 路由规则：shard_id = (simhash_hi >> 56n) % 16
 */

const path = require('path');

// 分片配置
const SHARD_CONFIG = {
  count: 16,                    // 分片数量
  prefixBits: 8,                // 使用高8bit
  dbPathTemplate: 'shard_{id}.db',  // 分片文件名模板
  metaDir: 'meta'               // 元数据目录
};

/**
 * 分片路由器类
 */
class ShardRouter {
  constructor(options = {}) {
    this.config = { ...SHARD_CONFIG, ...options };
    this.basePath = options.basePath || path.join(process.env.HOME || '.', '.hajimi/storage/v3');
  }

  /**
   * 获取分片ID
   * @param {bigint} simhash_hi - SimHash高64位
   * @returns {number} - 分片ID (0-15)
   */
  getShardId(simhash_hi) {
    // 验证输入
    this._validateSimHash(simhash_hi);
    
    // 取高8bit
    const prefix = Number((simhash_hi >> 56n) & 0xFFn);
    
    // 取模得到分片ID
    return prefix % this.config.count;
  }

  /**
   * 获取分片数据库路径
   * @param {number} shardId - 分片ID (0-15)
   * @returns {string} - 完整路径
   */
  getShardPath(shardId) {
    if (shardId < 0 || shardId >= this.config.count) {
      throw new Error(`Shard ID out of range: ${shardId} (expected 0-${this.config.count - 1})`);
    }
    
    const fileName = this.config.dbPathTemplate.replace('{id}', shardId.toString(16).padStart(2, '0'));
    return path.join(this.basePath, this.config.metaDir, fileName);
  }

  /**
   * 获取所有分片路径
   * @returns {Array<string>} - 16个分片路径
   */
  getAllShardPaths() {
    const paths = [];
    for (let i = 0; i < this.config.count; i++) {
      paths.push(this.getShardPath(i));
    }
    return paths;
  }

  /**
   * 验证SimHash格式
   * @param {bigint} hash - 待验证的hash
   * @returns {boolean} - 是否有效
   */
  validateSimHash(hash) {
    if (typeof hash !== 'bigint') {
      throw new Error(`Invalid SimHash type: ${typeof hash} (expected bigint)`);
    }
    
    // SimHash应为64位无符号整数
    if (hash < 0n) {
      throw new Error(`SimHash must be non-negative: ${hash}`);
    }
    
    if (hash > 0xFFFFFFFFFFFFFFFFn) {
      throw new Error(`SimHash exceeds 64 bits: ${hash}`);
    }
    
    return true;
  }

  /**
   * 获取分片统计信息
   * @returns {Object} - 分片配置信息
   */
  getShardStats() {
    return {
      count: this.config.count,
      prefixBits: this.config.prefixBits,
      basePath: this.basePath,
      metaDir: this.config.metaDir
    };
  }

  /**
   * 内部验证方法
   */
  _validateSimHash(hash) {
    return this.validateSimHash(hash);
  }
}

/**
 * 分片分布测试工具
 * 用于验证100K记录在16分片上的分布均匀性
 */
class ShardDistributionTester {
  constructor(router) {
    this.router = router;
  }

  /**
   * 测试分布均匀性
   * @param {number} sampleSize - 样本数量（默认100K）
   * @returns {Object} - 分布统计
   */
  testDistribution(sampleSize = 100000) {
    const distribution = new Array(this.router.config.count).fill(0);
    
    // 生成随机SimHash并统计分布
    for (let i = 0; i < sampleSize; i++) {
      // 生成随机64位整数
      const high32 = BigInt(Math.floor(Math.random() * 0xFFFFFFFF));
      const low32 = BigInt(Math.floor(Math.random() * 0xFFFFFFFF));
      const simhash = (high32 << 32n) | low32;
      
      const shardId = this.router.getShardId(simhash);
      distribution[shardId]++;
    }
    
    // 计算统计指标
    const expected = sampleSize / this.router.config.count;
    const variance = distribution.reduce((sum, count) => {
      return sum + Math.pow(count - expected, 2);
    }, 0) / this.router.config.count;
    
    const stdDev = Math.sqrt(variance);
    const stdDevPercent = (stdDev / expected) * 100;
    
    return {
      sampleSize,
      distribution,
      expected: Math.round(expected),
      stdDev: stdDev.toFixed(2),
      stdDevPercent: stdDevPercent.toFixed(2) + '%',
      isUniform: stdDevPercent < 5  // 标准差<5%认为均匀
    };
  }
}

module.exports = {
  ShardRouter,
  ShardDistributionTester,
  SHARD_CONFIG
};
