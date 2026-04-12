#!/usr/bin/env node
/**
 * 迁移脚本: 内存队列(v1) -> LevelDB队列(v2)
 * 用法: node migrate-queue-v1-to-v2.js [--dry-run] [--backup <path>]
 */

const fs = require('fs').promises;
const path = require('path');
const os = require('os');

const DB_PATH = path.join(os.homedir(), '.hajimi', 'p2p-queue');

async function migrate(options = {}) {
  console.log('[Migrate] v1 -> v2 队列迁移开始');
  
  // 检查是否有内存队列备份文件
  const backupPaths = [
    path.join(os.homedir(), '.hajimi', 'offline-queue-backup.json'),
    path.join(process.cwd(), 'offline-queue.json'),
    options.backup
  ].filter(Boolean);

  let sourceData = null;
  for (const bp of backupPaths) {
    try {
      const content = await fs.readFile(bp, 'utf8');
      sourceData = JSON.parse(content);
      console.log(`[Migrate] 找到备份: ${bp}`);
      break;
    } catch { continue; }
  }

  if (!sourceData) {
    console.log('[Migrate] 无v1队列备份，跳过迁移');
    return { migrated: 0, skipped: true };
  }

  if (!Array.isArray(sourceData)) {
    console.error('[Migrate] 无效数据格式');
    return { error: 'INVALID_FORMAT' };
  }

  console.log(`[Migrate] 发现 ${sourceData.length} 条v1队列记录`);

  if (options.dryRun) {
    console.log('[Migrate] 干运行模式，仅预览');
    return { dryRun: true, wouldMigrate: sourceData.length };
  }

  // 初始化LevelDB
  const { Level } = require('level');
  await fs.mkdir(DB_PATH, { recursive: true });
  
  const db = new Level(DB_PATH, { valueEncoding: 'utf8' });
  await db.open();

  let seq = 1;
  try { seq = parseInt(await db.get('__seq__'), 10) + 1; } catch {}

  const batch = db.batch();
  for (const op of sourceData) {
    if (!op.id) op.id = require('crypto').randomUUID();
    if (!op.timestamp) op.timestamp = Date.now();
    if (!op.retryCount) op.retryCount = 0;
    const entry = { id: op.id, op, seq: seq++ };
    batch.put(String(entry.seq), JSON.stringify(entry));
  }
  batch.put('__seq__', String(seq));
  await batch.write();
  await db.close();

  // 重命名备份文件
  const backupPath = path.join(os.homedir(), '.hajimi', `offline-queue-backup-v1-${Date.now()}.json`);
  for (const bp of backupPaths) {
    try {
      await fs.rename(bp, backupPath);
      console.log(`[Migrate] 已归档: ${backupPath}`);
      break;
    } catch {}
  }

  console.log(`[Migrate] ✅ 成功迁移 ${sourceData.length} 条记录`);
  return { migrated: sourceData.length, archived: backupPath };
}

async function rollback() {
  console.log('[Rollback] 回滚到v1开始');
  try {
    const { Level } = require('level');
    const db = new Level(DB_PATH, { valueEncoding: 'utf8' });
    await db.open();
    
    const ops = [];
    for await (const [key, val] of db.iterator()) {
      if (key === '__seq__') continue;
      ops.push(JSON.parse(val).op);
    }
    await db.close();

    const backupPath = path.join(os.homedir(), '.hajimi', 'offline-queue-backup.json');
    await fs.writeFile(backupPath, JSON.stringify(ops, null, 2));
    
    await fs.rm(DB_PATH, { recursive: true, force: true });
    console.log(`[Rollback] ✅ 已恢复v1备份: ${backupPath}`);
    return { restored: ops.length };
  } catch (err) {
    console.error('[Rollback] 失败:', err.message);
    return { error: err.message };
  }
}

// CLI
async function main() {
  const args = process.argv.slice(2);
  
  if (args.includes('--help') || args.includes('-h')) {
    console.log(`
用法: node migrate-queue-v1-to-v2.js [选项]

选项:
  --dry-run       预览迁移，不写入
  --rollback      回滚到v1
  --backup <path> 指定v1备份路径
  --help          显示帮助
    `);
    return;
  }

  if (args.includes('--rollback')) {
    await rollback();
    return;
  }

  const backupIndex = args.indexOf('--backup');
  const options = {
    dryRun: args.includes('--dry-run'),
    backup: backupIndex >= 0 ? args[backupIndex + 1] : null
  };

  await migrate(options);
}

if (require.main === module) {
  main().catch(console.error);
}

module.exports = { migrate, rollback };
