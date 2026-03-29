/**
 * StorageV3 - 统一存储API
 * 
 * 整合：
 * - ShardRouter（路由层）
 * - ShardConnectionPool（连接池）
 * - ChunkStorage（文件存储）
 * 
 * 提供高层CRUD接口
 */

const { ShardRouter } = require('../storage/shard-router');
const { ShardConnectionPool } = require('../storage/connection-pool');
const { ChunkStorage } = require('../storage/chunk');
const crypto = require('crypto');

/**
 * StorageV3 主类
 */
class StorageV3 {
  constructor(options = {}) {
    this.router = options.router || new ShardRouter();
    this.pool = options.pool || new ShardConnectionPool({ router: this.router });
    this.chunks = options.chunks || new ChunkStorage();
  }

  /**
   * 存储内容
   * @param {Buffer|string} content - 内容
   * @param {Object} metadata - 元数据
   * @returns {Promise<Object>} - { simhash, size, shardId }
   */
  async put(content, metadata = {}) {
    // 统一转为Buffer
    const data = Buffer.isBuffer(content) ? content : Buffer.from(content, 'utf8');
    
    // 计算SimHash（简化版：使用内容哈希）
    const simhash = this._computeSimHash(data);
    const simhashHi = simhash >> 64n;
    const simhashLo = simhash & 0xFFFFFFFFFFFFFFFFn;

    // 存储Chunk
    const chunkResult = await this.chunks.writeChunk(simhash, data, {
      ...metadata,
      size: data.length,
      ctime: Date.now()
    });

    // 记录到MetaDB（模拟）
    try {
      await this.pool.write(simhashHi, 
        'INSERT OR REPLACE INTO chunks (simhash_hi, simhash_lo, size, created_at) VALUES (?, ?, ?, ?)',
        [simhashHi.toString(), simhashLo.toString(), data.length, Date.now()]
      );
    } catch (err) {
      // MetaDB写入失败不影响Chunk存储
      console.warn('MetaDB write failed:', err.message);
    }

    return {
      simhash: simhash.toString(16).padStart(16, '0'),
      size: data.length,
      shardId: this.router.getShardId(simhashHi),
      ...chunkResult
    };
  }

  /**
   * 读取内容
   * @param {string|bigint} simhash - SimHash
   * @returns {Promise<Object|null>} - { data, metadata, simhash }
   */
  async get(simhash) {
    const hash = typeof simhash === 'string' ? BigInt('0x' + simhash) : simhash;
    
    // 从Chunk存储读取
    const result = await this.chunks.readChunk(hash);
    if (!result) return null;

    // 更新访问统计（模拟）
    const simhashHi = hash >> 64n;
    try {
      await this.pool.write(simhashHi,
        'UPDATE chunks SET access_count = access_count + 1, last_access = ? WHERE simhash_hi = ?',
        [Date.now(), simhashHi.toString()]
      );
    } catch (err) {
      // 忽略统计更新失败
    }

    return result;
  }

  /**
   * 删除内容
   * @param {string|bigint} simhash - SimHash
   * @returns {Promise<boolean>}
   */
  async delete(simhash) {
    const hash = typeof simhash === 'string' ? BigInt('0x' + simhash) : simhash;
    const simhashHi = hash >> 64n;

    // 删除Chunk
    const deleted = await this.chunks.deleteChunk(hash);
    
    // 从MetaDB删除（模拟）
    try {
      await this.pool.write(simhashHi,
        'DELETE FROM chunks WHERE simhash_hi = ?',
        [simhashHi.toString()]
      );
    } catch (err) {
      console.warn('MetaDB delete failed:', err.message);
    }

    return deleted;
  }

  /**
   * 按SimHash前缀查询候选
   * @param {bigint} simhash_hi - SimHash高64位
   * @returns {Promise<Array>}
   */
  async query(simhash_hi) {
    const shardId = this.router.getShardId(simhash_hi);
    
    try {
      const result = await this.pool.query(simhash_hi,
        'SELECT simhash_hi, simhash_lo, size, created_at FROM chunks WHERE simhash_hi = ? ORDER BY created_at DESC LIMIT 100',
        [simhash_hi.toString()]
      );
      
      return result.rows || [];
    } catch (err) {
      console.warn('Query failed:', err.message);
      return [];
    }
  }

  /**
   * 获取存储统计
   * @returns {Promise<Object>}
   */
  async stats() {
    const shardStats = this.pool.getPoolStats();
    const chunkStats = await this.chunks.getStats();
    
    return {
      shards: {
        count: 16,
        connections: shardStats.reduce((sum, s) => sum + s.totalConnections, 0)
      },
      chunks: chunkStats,
      pool: this.pool.getStats()
    };
  }

  /**
   * 批量导入
   * @param {Array<{content, metadata}>} items 
   * @returns {Promise<Array>}
   */
  async batchPut(items) {
    const results = [];
    
    for (const item of items) {
      try {
        const result = await this.put(item.content, item.metadata);
        results.push({ success: true, ...result });
      } catch (err) {
        results.push({ success: false, error: err.message });
      }
    }
    
    return results;
  }

  /**
   * 关闭存储
   */
  async close() {
    await this.pool.closeAll();
  }

  /**
   * 计算SimHash（简化实现）
   * 实际生产环境使用 simhash64.js
   */
  _computeSimHash(data) {
    const hash = crypto.createHash('sha256').update(data).digest();
    // 取前128bit作为SimHash
    const high = hash.slice(0, 8);
    const low = hash.slice(8, 16);
    return (BigInt('0x' + high.toString('hex')) << 64n) | BigInt('0x' + low.toString('hex'));
  }
}

module.exports = {
  StorageV3
};
