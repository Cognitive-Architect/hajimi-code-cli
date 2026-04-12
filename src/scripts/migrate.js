#!/usr/bin/env node

/**
 * 迁移CLI工具
 * Migration CLI Tool
 * 
 * Usage:
 *   node scripts/migrate.js ./data        # 迁移目录
 *   node scripts/migrate.js --dry-run     # 模拟运行
 *   node scripts/migrate.js --analyze     # 仅分析
 *   node scripts/migrate.js --rollback    # 回滚
 */

const path = require('path');
const { Migrator } = require('../src/migration/migrator');

function printUsage() {
  console.log(`
Usage: node migrate.js [options] <directory>

Options:
  --dry-run      模拟运行，不实际执行迁移
  --analyze      仅分析，不执行迁移
  --rollback     回滚最后一次迁移
  --help         显示帮助信息

Examples:
  node migrate.js ./data
  node migrate.js --dry-run ./data
  node migrate.js --analyze ./data
`);
}

async function main() {
  const args = process.argv.slice(2);
  
  if (args.length === 0 || args.includes('--help')) {
    printUsage();
    process.exit(0);
  }

  const options = {
    dryRun: args.includes('--dry-run'),
    analyze: args.includes('--analyze'),
    rollback: args.includes('--rollback')
  };

  // 找到目录参数
  const dirArg = args.find(arg => !arg.startsWith('--'));
  const targetDir = dirArg ? path.resolve(dirArg) : './data';

  const migrator = new Migrator({
    basePath: targetDir,
    dryRun: options.dryRun,
    backup: !options.dryRun
  });

  try {
    if (options.analyze) {
      // 仅分析
      const analysis = await migrator.analyze(targetDir);
      console.log('\n📊 Analysis Report:');
      console.log(JSON.stringify(analysis, null, 2));
      process.exit(0);
    }

    if (options.rollback) {
      // 回滚
      console.log('⚠️ Rollback not yet implemented for directory');
      console.log('Use: node migrate.js --rollback <specific-file>');
      process.exit(1);
    }

    // 执行迁移
    const result = await migrator.migrate({ dir: targetDir });
    
    if (result.success) {
      console.log('\n✅ Migration completed successfully');
      process.exit(0);
    } else {
      console.log('\n❌ Migration completed with errors');
      process.exit(1);
    }

  } catch (err) {
    console.error('❌ Error:', err.message);
    process.exit(1);
  }
}

main();
