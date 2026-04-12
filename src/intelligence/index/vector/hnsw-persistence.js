/**
 * HNSW 持久化模块 - Persistence Layer
 * 
 * 职责：
 * 1. 索引序列化/反序列化（JSON/Binary）
 * 2. 增量保存与 WAL（Write-Ahead Logging）
 * 3. 与 Chunk 格式（.hctx v3）集成
 * 4. 备份与恢复
 * 
 * 存储格式：
 * - 主索引: hnsw-index-{shardId}.json
 * - WAL: hnsw-wal-{shardId}.log
 * - Chunk扩展: 在 .hctx 文件中嵌入 HNSW 元数据
 */

const fs = require('fs').promises;
const path = require('path');
const crypto = require('crypto');
const { HNSWIndex } = require('./hnsw-core');

// 持久化配置
const PERSISTENCE_CONFIG = {
  format: 'json',           // 'json' | 'binary'
  compress: false,          // 是否压缩
  walEnabled: true,         // 是否启用WAL
  walFlushInterval: 5000,   // WAL 刷盘间隔
  maxBackups: 5,            // 最大备份数
  syncOnWrite: false        // 是否同步写入（牺牲性能换安全）
};

/**
 * WAL（预写日志）
 */
class WriteAheadLog {
  constructor(filePath) {
    this.filePath = filePath;
    this.buffer = [];
    this.sequence = 0;
    this.flushing = false;
  }
  
  /**
   * 记录操作
   */
  log(operation, data) {
    const entry = {
      seq: ++this.sequence,
      time: Date.now(),
      op: operation,
      data
    };
    this.buffer.push(entry);
  }
  
  /**
   * 刷盘
   */
  async flush() {
    if (this.flushing || this.buffer.length === 0) return;
    
    this.flushing = true;
    
    try {
      const lines = this.buffer.map(e => JSON.stringify(e)).join('\n') + '\n';
      await fs.appendFile(this.filePath, lines);
      this.buffer = [];
    } catch (err) {
      console.error('[WAL] Flush failed:', err);
    } finally {
      this.flushing = false;
    }
  }
  
  /**
   * 回放日志
   */
  async replay(index) {
    try {
      const content = await fs.readFile(this.filePath, 'utf8');
      const lines = content.split('\n').filter(l => l.trim());
      
      for (const line of lines) {
        try {
          const entry = JSON.parse(line);
          if (entry.op === 'insert') {
            index.insert(entry.data.id, entry.data.vector);
          } else if (entry.op === 'delete') {
            index.delete(entry.data.id);
          }
        } catch (err) {
          console.warn('[WAL] Replay entry failed:', err.message);
        }
      }
      
      return true;
    } catch (err) {
      if (err.code !== 'ENOENT') {
        console.error('[WAL] Replay failed:', err);
      }
      return false;
    }
  }
  
  /**
   * 清空
   */
  async clear() {
    this.buffer = [];
    try {
      await fs.unlink(this.filePath);
    } catch (err) {
      if (err.code !== 'ENOENT') throw err;
    }
  }
}

/**
 * HNSW 持久化管理器
 */
class HNSWPersistence {
  /**
   * @param {Object} options
   * @param {string} options.basePath - 存储路径
   * @param {string} options.shardId - 分片ID
   * @param {Object} options.config - 持久化配置
   */
  constructor(options = {}) {
    this.config = { ...PERSISTENCE_CONFIG, ...options.config };
    this.basePath = options.basePath || path.join(process.env.HOME || '.', '.hajimi/storage/v3/hnsw');
    this.shardId = options.shardId || 'default';
    
    // 文件路径
    this.indexPath = path.join(this.basePath, `hnsw-index-${this.shardId}.json`);
    this.walPath = path.join(this.basePath, `hnsw-wal-${this.shardId}.log`);
    this.backupDir = path.join(this.basePath, 'backups');
    
    // WAL
    this.wal = this.config.walEnabled ? new WriteAheadLog(this.walPath) : null;
    
    // 自动刷盘定时器
    this.flushTimer = null;
    
    if (this.wal) {
      this._startWalFlushTimer();
    }
  }
  
  /**
   * 保存索引
   * @param {HNSWIndex} index 
   * @param {Object} metadata - 额外元数据
   */
  async save(index, metadata = {}) {
    const startTime = Date.now();
    
    // 确保目录存在
    await fs.mkdir(this.basePath, { recursive: true });
    await fs.mkdir(this.backupDir, { recursive: true });
    
    // 序列化
    const data = {
      version: 1,
      shardId: this.shardId,
      timestamp: Date.now(),
      metadata,
      checksum: '',
      index: index.toJSON()
    };
    
    // 计算校验和
    const jsonStr = JSON.stringify(data.index);
    data.checksum = crypto.createHash('sha256').update(jsonStr).digest('hex');
    
    // 原子写入
    const tempPath = `${this.indexPath}.tmp`;
    await fs.writeFile(tempPath, JSON.stringify(data, null, 2));
    await fs.rename(tempPath, this.indexPath);
    
    // 清空 WAL
    if (this.wal) {
      await this.wal.clear();
    }
    
    const duration = Date.now() - startTime;
    console.log(`[Persistence] Saved shard ${this.shardId} in ${duration}ms`);
    
    return true;
  }
  
  /**
   * 加载索引
   * @returns {Promise<{index: HNSWIndex, metadata: Object}|null>}
   */
  async load() {
    try {
      // 读取主索引
      const content = await fs.readFile(this.indexPath, 'utf8');
      const data = JSON.parse(content);
      
      // 校验
      if (!data.index) {
        throw new Error('Invalid index file: missing index data');
      }
      
      // 验证校验和
      if (data.checksum) {
        const jsonStr = JSON.stringify(data.index);
        const computedChecksum = crypto.createHash('sha256').update(jsonStr).digest('hex');
        if (computedChecksum !== data.checksum) {
          throw new Error('Index file corrupted: checksum mismatch');
        }
      }
      
      // 恢复索引
      const index = HNSWIndex.fromJSON(data.index);
      
      // 回放 WAL
      if (this.wal) {
        await this.wal.replay(index);
      }
      
      console.log(`[Persistence] Loaded shard ${this.shardId}`);
      
      return {
        index,
        metadata: data.metadata || {}
      };
    } catch (err) {
      if (err.code === 'ENOENT') {
        console.log(`[Persistence] No existing index for shard ${this.shardId}`);
        return null;
      }
      throw err;
    }
  }
  
  /**
   * 记录插入（WAL）
   */
  async logInsert(id, vector) {
    if (!this.wal) return;
    
    // 确保目录存在
    await fs.mkdir(this.basePath, { recursive: true }).catch(() => {});
    
    // 向量序列化
    let vectorData;
    if (typeof vector === 'bigint') {
      vectorData = { type: 'bigint', value: vector.toString() };
    } else {
      vectorData = { type: 'Float32Array', value: Array.from(vector) };
    }
    
    this.wal.log('insert', { id, vector: vectorData });
  }
  
  /**
   * 记录删除（WAL）
   */
  async logDelete(id) {
    if (!this.wal) return;
    this.wal.log('delete', { id });
  }
  
  /**
   * 创建备份
   */
  async createBackup() {
    await fs.mkdir(this.backupDir, { recursive: true });
    
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const backupPath = path.join(this.backupDir, `hnsw-${this.shardId}-${timestamp}.json`);
    
    try {
      await fs.copyFile(this.indexPath, backupPath);
      console.log(`[Persistence] Backup created: ${backupPath}`);
      
      // 清理旧备份
      await this._cleanupOldBackups();
      
      return backupPath;
    } catch (err) {
      if (err.code === 'ENOENT') {
        console.log('[Persistence] No index to backup');
        return null;
      }
      throw err;
    }
  }
  
  /**
   * 恢复备份
   */
  async restoreBackup(backupPath) {
    if (!backupPath) {
      // 使用最新的备份
      const backups = await this._listBackups();
      if (backups.length === 0) {
        throw new Error('No backups available');
      }
      backupPath = backups[backups.length - 1];
    }
    
    await fs.copyFile(backupPath, this.indexPath);
    console.log(`[Persistence] Restored from: ${backupPath}`);
    
    return this.load();
  }
  
  /**
   * 列出备份
   */
  async _listBackups() {
    try {
      const files = await fs.readdir(this.backupDir);
      return files
        .filter(f => f.startsWith(`hnsw-${this.shardId}`))
        .map(f => path.join(this.backupDir, f))
        .sort();
    } catch {
      return [];
    }
  }
  
  /**
   * 清理旧备份
   */
  async _cleanupOldBackups() {
    const backups = await this._listBackups();
    
    while (backups.length > this.config.maxBackups) {
      const oldBackup = backups.shift();
      await fs.unlink(oldBackup);
      console.log(`[Persistence] Removed old backup: ${oldBackup}`);
    }
  }
  
  /**
   * 启动 WAL 刷盘定时器
   */
  _startWalFlushTimer() {
    this.flushTimer = setInterval(() => {
      if (this.wal) {
        this.wal.flush().catch(console.error);
      }
    }, this.config.walFlushInterval);
    
    if (this.flushTimer.unref) {
      this.flushTimer.unref();
    }
  }
  
  /**
   * 强制刷盘
   */
  async flush() {
    if (this.wal) {
      await this.wal.flush();
    }
  }
  
  /**
   * 检查点（保存并清空WAL）
   */
  async checkpoint(index, metadata = {}) {
    await this.save(index, metadata);
  }
  
  /**
   * 删除所有数据
   */
  async destroy() {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
      this.flushTimer = null;
    }
    
    const files = [
      this.indexPath,
      this.walPath,
      `${this.indexPath}.tmp`
    ];
    
    for (const file of files) {
      try {
        await fs.unlink(file);
      } catch (err) {
        if (err.code !== 'ENOENT') throw err;
      }
    }
    
    console.log(`[Persistence] Destroyed shard ${this.shardId}`);
  }
  
  /**
   * 获取存储统计
   */
  async getStats() {
    let indexSize = 0;
    let walSize = 0;
    
    try {
      const indexStat = await fs.stat(this.indexPath);
      indexSize = indexStat.size;
    } catch {}
    
    try {
      const walStat = await fs.stat(this.walPath);
      walSize = walStat.size;
    } catch {}
    
    const backups = await this._listBackups();
    
    return {
      shardId: this.shardId,
      indexSize,
      walSize,
      totalSize: indexSize + walSize,
      backupCount: backups.length
    };
  }
  
  /**
   * 二进制保存（DEBT-PHASE2-005）
   * @param {HNSWIndex} index
   * @param {Object} metadata
   */
  async saveBinary(index, metadata = {}) {
    const { serializeHNSW } = require('../format/hnsw-binary');
    const startTime = Date.now();
    
    await fs.mkdir(this.basePath, { recursive: true });
    
    // 二进制序列化
    const buffer = serializeHNSW(index, {
      dimension: metadata.dimension || 128,
      ...metadata
    });
    
    // 原子写入
    const binaryPath = this.indexPath.replace('.json', '.bin');
    const tempPath = `${binaryPath}.tmp`;
    await fs.writeFile(tempPath, buffer);
    await fs.rename(tempPath, binaryPath);
    
    // 清空 WAL
    if (this.wal) {
      await this.wal.clear();
    }
    
    const duration = Date.now() - startTime;
    console.log(`[Persistence] Binary saved shard ${this.shardId} in ${duration}ms (${buffer.length} bytes)`);
    
    return { duration, size: buffer.length };
  }
  
  /**
   * 二进制加载
   * @returns {Promise<{index: HNSWIndex, metadata: Object}|null>}
   */
  async loadBinary() {
    const { deserializeHNSW } = require('../format/hnsw-binary');
    const { HNSWNode } = require('./hnsw-core');
    const binaryPath = this.indexPath.replace('.json', '.bin');
    
    try {
      const buffer = await fs.readFile(binaryPath);
      const data = deserializeHNSW(buffer);
      
      // 重建索引
      const index = new (require('./hnsw-core').HNSWIndex)({
        distanceMetric: 'l2'
      });
      
      index.maxLevel = data.header.maxLevel;
      index.entryPoint = data.header.entryPoint;
      index.elementCount = data.header.vectorCount;
      
      // 恢复节点
      for (const nodeData of data.nodes) {
        const node = new HNSWNode(nodeData.id, nodeData.vector, nodeData.level);
        node.connections = nodeData.connections;
        node.deleted = nodeData.deleted;
        index.nodes.set(nodeData.id, node);
      }
      
      console.log(`[Persistence] Binary loaded shard ${this.shardId}`);
      
      return {
        index,
        metadata: {
          timestamp: data.header.timestamp,
          vectorCount: data.header.vectorCount
        }
      };
    } catch (err) {
      if (err.code === 'ENOENT') {
        return null;
      }
      throw err;
    }
  }
  
  /**
   * 智能加载（优先二进制，回退JSON）
   */
  async loadSmart() {
    // 先尝试二进制
    const binary = await this.loadBinary().catch(() => null);
    if (binary) return binary;
    
    // 回退到 JSON
    return this.load();
  }
}

module.exports = {
  HNSWPersistence,
  WriteAheadLog,
  PERSISTENCE_CONFIG
};
