/**
 * V1 (JSON) → V2 (Binary) 迁移
 * JSON to Binary Migration
 */

const fs = require('fs').promises;
const path = require('path');
const { serializeHNSW, deserializeHNSW } = require('../format/hnsw-binary');

class V1ToV2Migration {
  constructor(options = {}) {
    this.backupDir = options.backupDir || './data/backups';
    this.atomic = options.atomic !== false; // 默认原子性
  }

  /**
   * 执行迁移
   */
  async migrate(sourcePath, targetPath) {
    console.log(`🔄 Migrating: ${sourcePath} → ${targetPath}`);

    const startTime = Date.now();
    const stats = {
      sourcePath,
      targetPath,
      vectorsMigrated: 0,
      errors: [],
      duration: 0
    };

    try {
      // 1. 读取V1 JSON数据
      const v1Data = await this._loadV1(sourcePath);
      stats.vectorsMigrated = v1Data.nodes?.length || 0;

      // 2. 创建备份（如果启用）
      if (this.atomic) {
        await this._createBackup(sourcePath);
      }

      // 3. 转换为V2二进制格式
      const v2Data = await this._convertToV2(v1Data);

      // 4. 写入目标文件（原子写入）
      await this._atomicWrite(targetPath, v2Data);

      // 5. 验证
      const verified = await this._verify(targetPath, v1Data);
      if (!verified) {
        throw new Error('Migration verification failed');
      }

      stats.duration = Date.now() - startTime;
      console.log(`✅ Migration completed in ${stats.duration}ms, ${stats.vectorsMigrated} vectors`);

      return {
        success: true,
        ...stats
      };

    } catch (err) {
      stats.duration = Date.now() - startTime;
      stats.errors.push(err.message);
      
      console.error(`❌ Migration failed: ${err.message}`);
      
      // 尝试恢复备份
      if (this.atomic) {
        await this._restoreBackup(sourcePath);
      }

      return {
        success: false,
        ...stats,
        error: err.message
      };
    }
  }

  /**
   * 加载V1 JSON数据
   */
  async _loadV1(filePath) {
    const content = await fs.readFile(filePath, 'utf-8');
    const data = JSON.parse(content);
    
    // 验证V1格式
    if (!data.nodes || !Array.isArray(data.nodes)) {
      throw new Error('Invalid V1 format: missing nodes array');
    }

    return data;
  }

  /**
   * 转换为V2格式
   */
  async _convertToV2(v1Data) {
    // 构建HNSW索引结构
    const index = {
      nodes: new Map(),
      entryPoint: v1Data.entryPoint,
      maxLevel: v1Data.maxLevel || 0,
      elementCount: v1Data.nodes.length,
      dimension: v1Data.dimension || 128
    };

    // 转换节点
    for (const node of v1Data.nodes) {
      index.nodes.set(node.id, {
        id: node.id,
        vector: node.vector,
        level: node.level || 0,
        connections: node.connections || {},
        deleted: node.deleted || false
      });
    }

    // 序列化为二进制
    const metadata = {
      dimension: index.dimension,
      maxLevel: index.maxLevel,
      migratedFrom: 'v1',
      migratedAt: new Date().toISOString()
    };

    return serializeHNSW(index, metadata);
  }

  /**
   * 原子写入
   */
  async _atomicWrite(targetPath, data) {
    const tempPath = `${targetPath}.tmp`;
    
    // 写入临时文件
    await fs.writeFile(tempPath, data);
    
    // 重命名（原子操作）
    await fs.rename(tempPath, targetPath);
  }

  /**
   * 创建备份
   */
  async _createBackup(sourcePath) {
    await fs.mkdir(this.backupDir, { recursive: true });
    
    const backupName = `${path.basename(sourcePath)}.backup-${Date.now()}`;
    const backupPath = path.join(this.backupDir, backupName);
    
    await fs.copyFile(sourcePath, backupPath);
    console.log(`📦 Backup created: ${backupPath}`);
    
    return backupPath;
  }

  /**
   * 恢复备份
   */
  async _restoreBackup(sourcePath) {
    const backups = await fs.readdir(this.backupDir);
    const baseName = path.basename(sourcePath);
    
    // 找到最新的备份
    const latestBackup = backups
      .filter(f => f.startsWith(baseName))
      .sort()
      .pop();
    
    if (latestBackup) {
      const backupPath = path.join(this.backupDir, latestBackup);
      await fs.copyFile(backupPath, sourcePath);
      console.log(`♻️ Restored from backup: ${backupPath}`);
    }
  }

  /**
   * 验证迁移结果
   */
  async _verify(targetPath, originalData) {
    try {
      // 读取二进制文件
      const binary = await fs.readFile(targetPath);
      
      // 验证魔数
      const magic = binary.slice(0, 4).toString();
      if (magic !== 'HNSW') {
        throw new Error('Invalid binary magic number');
      }

      // 验证版本
      const version = binary.readUInt16BE(4);
      if (version !== 1) {
        throw new Error(`Unexpected version: ${version}`);
      }

      // 验证向量数量匹配
      const vectorCount = binary.readUInt32BE(8);
      if (vectorCount !== originalData.nodes.length) {
        throw new Error(`Vector count mismatch: ${vectorCount} vs ${originalData.nodes.length}`);
      }

      return true;
    } catch (err) {
      console.error(`Verification failed: ${err.message}`);
      return false;
    }
  }
}

module.exports = { V1ToV2Migration };
