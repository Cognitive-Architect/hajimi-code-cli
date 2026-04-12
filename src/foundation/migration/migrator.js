/**
 * 迁移协调器
 * Migration Coordinator
 */

const fs = require('fs').promises;
const path = require('path');
const { VersionDetector, VERSIONS } = require('./version-detector');
const { V1ToV2Migration } = require('./v1-to-v2');

class Migrator {
  constructor(options = {}) {
    this.basePath = options.basePath || './data';
    this.detector = new VersionDetector({ basePath: this.basePath });
    
    // 注册迁移策略
    this.strategies = new Map();
    this._registerDefaultStrategies();
    
    this.options = {
      dryRun: options.dryRun || false,
      backup: options.backup !== false,
      parallel: options.parallel || 1
    };
  }

  /**
   * 注册默认迁移策略
   */
  _registerDefaultStrategies() {
    // V0 -> V1 (JSON -> Binary)
    this.strategies.set(
      `${VERSIONS.V0_JSON}->${VERSIONS.V1_BINARY}`,
      new V1ToV2Migration()
    );
    
    // V1 -> V2 (预留: Binary -> WASM)
    this.strategies.set(
      `${VERSIONS.V1_BINARY}->${VERSIONS.V2_WASM}`,
      {
        migrate: async () => ({
          success: false,
          error: 'V1->V2 migration not yet implemented'
        })
      }
    );
  }

  /**
   * 分析迁移需求
   */
  async analyze(dirPath) {
    console.log(`🔍 Analyzing: ${dirPath}`);
    
    const files = await this.detector.detectDirectory(dirPath);
    const report = {
      total: files.length,
      byVersion: {},
      migrationsNeeded: [],
      upToDate: []
    };

    for (const file of files) {
      const versionName = this.detector.getVersionInfo(file.version).name;
      report.byVersion[versionName] = (report.byVersion[versionName] || 0) + 1;

      if (file.version === VERSIONS.V0_JSON) {
        report.migrationsNeeded.push(file);
      } else if (file.version === VERSIONS.V1_BINARY) {
        report.upToDate.push(file);
      }
    }

    return report;
  }

  /**
   * 执行迁移
   */
  async migrate(options = {}) {
    const targetDir = options.dir || this.basePath;
    const targetVersion = options.to || VERSIONS.V1_BINARY;
    
    console.log(`🚀 Starting migration to V${targetVersion}...`);
    console.log(`   Dry run: ${this.options.dryRun}`);
    console.log(`   Backup: ${this.options.backup}`);

    // 分析
    const analysis = await this.analyze(targetDir);
    
    if (analysis.migrationsNeeded.length === 0) {
      console.log('✅ No migrations needed');
      return { success: true, migrated: 0 };
    }

    console.log(`📋 ${analysis.migrationsNeeded.length} files need migration`);

    if (this.options.dryRun) {
      console.log('🔍 Dry run mode - no changes made');
      return { 
        success: true, 
        dryRun: true,
        wouldMigrate: analysis.migrationsNeeded 
      };
    }

    // 执行迁移
    const results = [];
    for (const file of analysis.migrationsNeeded) {
      const result = await this._migrateFile(file, targetVersion);
      results.push(result);
    }

    // 汇总结果
    const successCount = results.filter(r => r.success).length;
    const failCount = results.length - successCount;

    console.log(`\n📊 Migration Summary:`);
    console.log(`   Success: ${successCount}`);
    console.log(`   Failed: ${failCount}`);

    return {
      success: failCount === 0,
      total: results.length,
      successCount,
      failCount,
      results
    };
  }

  /**
   * 迁移单个文件
   */
  async _migrateFile(fileInfo, targetVersion) {
    const sourceVersion = fileInfo.version;
    const strategyKey = `${sourceVersion}->${targetVersion}`;
    const strategy = this.strategies.get(strategyKey);

    if (!strategy) {
      return {
        success: false,
        file: fileInfo.path,
        error: `No migration strategy for ${strategyKey}`
      };
    }

    const targetPath = fileInfo.path.replace(/\.json$/, '.hnsw');
    
    try {
      const result = await strategy.migrate(fileInfo.path, targetPath);
      return {
        ...result,
        file: fileInfo.path
      };
    } catch (err) {
      return {
        success: false,
        file: fileInfo.path,
        error: err.message
      };
    }
  }

  /**
   * 检查是否需要迁移
   */
  async needsMigration(dirPath) {
    const analysis = await this.analyze(dirPath);
    return analysis.migrationsNeeded.length > 0;
  }

  /**
   * 自动迁移（如果必要）
   */
  async autoMigrate(dirPath) {
    if (await this.needsMigration(dirPath)) {
      return await this.migrate({ dir: dirPath });
    }
    return { success: true, migrated: 0, message: 'No migration needed' };
  }

  /**
   * 回滚迁移
   */
  async rollback(filePath) {
    const backupDir = path.join(this.basePath, 'backups');
    const baseName = path.basename(filePath);
    
    try {
      const backups = await fs.readdir(backupDir);
      const backup = backups
        .filter(f => f.startsWith(baseName))
        .sort()
        .pop();
      
      if (!backup) {
        return { success: false, error: 'No backup found' };
      }

      const backupPath = path.join(backupDir, backup);
      await fs.copyFile(backupPath, filePath);
      
      return { 
        success: true, 
        restoredFrom: backupPath 
      };
    } catch (err) {
      return { 
        success: false, 
        error: err.message 
      };
    }
  }
}

module.exports = { 
  Migrator, 
  VERSIONS 
};
