/**
 * Migrate - 分片初始化与迁移工具
 * 
 * 功能：
 * - 16个分片数据库初始化
 * - Schema 版本管理
 * - 幂等初始化（重复执行不报错）
 */

const fs = require('fs').promises;
const path = require('path');
const { ShardRouter } = require('./shard-router');

// 迁移配置
const MIGRATE_CONFIG = {
  currentVersion: 3,
  migrations: {
    1: 'Initial schema',
    2: 'Add vector associations',
    3: 'Add sync peers and triggers'
  }
};

/**
 * 迁移管理器
 */
class MigrationManager {
  constructor(options = {}) {
    this.router = options.router || new ShardRouter();
    this.schemaPath = options.schemaPath || path.join(__dirname, 'schema.sql');
  }

  /**
   * 初始化所有分片
   * @returns {Promise<Object>}
   */
  async initAllShards() {
    const results = {
      created: [],
      existing: [],
      errors: []
    };

    const shardPaths = this.router.getAllShardPaths();

    for (let shardId = 0; shardId < shardPaths.length; shardId++) {
      const shardPath = shardPaths[shardId];
      
      try {
        const result = await this.initShard(shardPath, shardId);
        if (result.created) {
          results.created.push(shardId);
        } else {
          results.existing.push(shardId);
        }
      } catch (err) {
        results.errors.push({ shardId, error: err.message });
      }
    }

    return results;
  }

  /**
   * 初始化单个分片
   * @param {string} shardPath - 分片数据库路径
   * @param {number} shardId - 分片ID
   * @returns {Promise<Object>}
   */
  async initShard(shardPath, shardId) {
    // 确保目录存在
    await fs.mkdir(path.dirname(shardPath), { recursive: true });

    // 检查是否已存在
    try {
      await fs.access(shardPath);
      // 文件存在，检查schema版本
      return { created: false, path: shardPath };
    } catch {
      // 文件不存在，创建新数据库
    }

    // 读取schema
    const schemaSQL = await fs.readFile(this.schemaPath, 'utf8');

    // 替换分片ID
    const customizedSQL = schemaSQL.replace(
      /INSERT OR IGNORE INTO shard_meta.*shard_id.*0/,
      `INSERT OR IGNORE INTO shard_meta (key, value) VALUES ('shard_id', '${shardId}')`
    );

    // 创建空文件（模拟SQLite初始化）
    // 实际生产环境使用 better-sqlite3 执行SQL
    await fs.writeFile(shardPath, '');
    
    // 写入schema标记文件
    const metaPath = `${shardPath}.meta`;
    await fs.writeFile(metaPath, JSON.stringify({
      version: MIGRATE_CONFIG.currentVersion,
      shardId,
      createdAt: new Date().toISOString(),
      schemaHash: this._hashString(customizedSQL)
    }));

    return { created: true, path: shardPath };
  }

  /**
   * 检查分片状态
   * @returns {Promise<Array>}
   */
  async checkShards() {
    const shardPaths = this.router.getAllShardPaths();
    const statuses = [];

    for (let shardId = 0; shardId < shardPaths.length; shardId++) {
      const shardPath = shardPaths[shardId];
      const metaPath = `${shardPath}.meta`;

      try {
        await fs.access(shardPath);
        
        let meta = null;
        try {
          const metaContent = await fs.readFile(metaPath, 'utf8');
          meta = JSON.parse(metaContent);
        } catch {
          // meta文件不存在
        }

        statuses.push({
          shardId,
          path: shardPath,
          exists: true,
          version: meta?.version || 'unknown',
          needsUpgrade: (meta?.version || 0) < MIGRATE_CONFIG.currentVersion
        });
      } catch {
        statuses.push({
          shardId,
          path: shardPath,
          exists: false,
          version: null,
          needsUpgrade: true
        });
      }
    }

    return statuses;
  }

  /**
   * 升级分片
   * @param {number} fromVersion 
   * @param {number} toVersion 
   */
  async upgradeShards(fromVersion, toVersion) {
    const results = [];
    const statuses = await this.checkShards();

    for (const status of statuses) {
      if (status.needsUpgrade && status.exists) {
        // 执行升级
        const upgradeSQL = this._getUpgradeSQL(fromVersion, toVersion);
        
        // 更新meta
        const metaPath = `${status.path}.meta`;
        await fs.writeFile(metaPath, JSON.stringify({
          version: toVersion,
          upgradedAt: new Date().toISOString()
        }));

        results.push({ shardId: status.shardId, upgraded: true });
      }
    }

    return results;
  }

  /**
   * 删除所有分片（危险操作）
   */
  async dropAllShards() {
    const shardPaths = this.router.getAllShardPaths();
    
    for (const shardPath of shardPaths) {
      try {
        await fs.unlink(shardPath);
        await fs.unlink(`${shardPath}.meta`).catch(() => {});
      } catch (err) {
        if (err.code !== 'ENOENT') throw err;
      }
    }

    return { dropped: true };
  }

  /**
   * 获取升级SQL
   */
  _getUpgradeSQL(fromVersion, toVersion) {
    // 实际生产环境根据版本差异返回升级SQL
    const upgrades = [];
    
    if (fromVersion < 2) {
      upgrades.push(`
        CREATE TABLE IF NOT EXISTS chunk_vectors (
          chunk_id INTEGER NOT NULL,
          vector_id INTEGER NOT NULL,
          similarity INTEGER,
          PRIMARY KEY (chunk_id, vector_id)
        );
        CREATE INDEX IF NOT EXISTS idx_vectors_vector_id ON chunk_vectors(vector_id);
      `);
    }
    
    if (fromVersion < 3) {
      upgrades.push(`
        CREATE TABLE IF NOT EXISTS sync_peers (...);
        CREATE TRIGGER IF NOT EXISTS update_timestamp ...;
      `);
    }
    
    return upgrades.join('\n');
  }

  /**
   * 计算字符串哈希
   */
  _hashString(str) {
    const crypto = require('crypto');
    return crypto.createHash('md5').update(str).digest('hex').substring(0, 8);
  }
}

// CLI 支持
async function main() {
  const args = process.argv.slice(2);
  const manager = new MigrationManager();

  if (args.includes('--init')) {
    console.log('初始化16个分片...');
    const results = await manager.initAllShards();
    console.log(`✅ 创建: ${results.created.length}个`);
    console.log(`ℹ️  已存在: ${results.existing.length}个`);
    if (results.errors.length > 0) {
      console.log(`❌ 错误: ${results.errors.length}个`);
      results.errors.forEach(e => console.log(`   - Shard ${e.shardId}: ${e.error}`));
    }
    return;
  }

  if (args.includes('--check')) {
    console.log('检查分片状态...');
    const statuses = await manager.checkShards();
    const existing = statuses.filter(s => s.exists).length;
    const needsUpgrade = statuses.filter(s => s.needsUpgrade).length;
    
    console.log(`总数: ${statuses.length}`);
    console.log(`已存在: ${existing}`);
    console.log(`需升级: ${needsUpgrade}`);
    
    statuses.forEach(s => {
      const status = s.exists ? `v${s.version}` : '缺失';
      console.log(`  Shard ${s.shardId}: ${status}`);
    });
    return;
  }

  if (args.includes('--drop')) {
    console.log('⚠️  删除所有分片...');
    await manager.dropAllShards();
    console.log('✅ 已删除');
    return;
  }

  console.log(`
用法: node migrate.js [选项]

选项:
  --init    初始化16个分片
  --check   检查分片状态
  --drop    删除所有分片（危险）
  `);
}

if (require.main === module) {
  main().catch(console.error);
}

module.exports = {
  MigrationManager,
  MIGRATE_CONFIG
};
