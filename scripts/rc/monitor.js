#!/usr/bin/env node
/**
 * RC Memory Monitor - 内存监控脚本
 * 30秒采样间隔，300MB熔断阈值
 */

const fs = require('fs');
const path = require('path');

// 常量定义
const FUSE_THRESHOLD = 300 * 1024 * 1024; // 300MB熔断阈值
const SAMPLE_INTERVAL = 30000; // 30秒采样间隔
const LOG_DIR = path.join(process.cwd(), 'logs', 'rc');

// 确保日志目录存在
if (!fs.existsSync(LOG_DIR)) {
  fs.mkdirSync(LOG_DIR, { recursive: true });
}

const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
const logFile = path.join(LOG_DIR, `monitor-${timestamp}.csv`);

// CSV头部
fs.writeFileSync(logFile, 'timestamp,rss,heapUsed,heapTotal\n');
console.log(`[Monitor] Started, logging to: ${logFile}`);
console.log(`[Monitor] Fuse threshold: ${FUSE_THRESHOLD} bytes (${FUSE_THRESHOLD / 1024 / 1024}MB)`);

let isRunning = true;

function formatCSVLine() {
  const usage = process.memoryUsage();
  const now = new Date().toISOString();
  return `${now},${usage.rss},${usage.heapUsed},${usage.heapTotal}\n`;
}

function checkFuse(usage) {
  if (usage.rss > FUSE_THRESHOLD) {
    const msg = `FUSE TRIGGERED: RSS ${usage.rss} bytes (${(usage.rss / 1024 / 1024).toFixed(2)}MB) exceeds threshold ${FUSE_THRESHOLD} bytes (${FUSE_THRESHOLD / 1024 / 1024}MB)`;
    fs.appendFileSync(logFile, `# ${msg}\n`);
    throw new Error(msg);
  }
}

function sample() {
  if (!isRunning) return;
  const line = formatCSVLine();
  fs.appendFileSync(logFile, line);
  const usage = process.memoryUsage();
  console.log(`[${new Date().toLocaleTimeString()}] RSS: ${(usage.rss / 1024 / 1024).toFixed(2)}MB`);
  checkFuse(usage);
}

// 优雅退出处理
function cleanup() {
  isRunning = false;
  console.log('\n[Monitor] Cleanup complete, log saved to: ' + logFile);
  process.exit(0);
}

process.on('SIGINT', cleanup);
process.on('SIGTERM', cleanup);
process.on('exit', () => {
  if (isRunning) cleanup();
});

// 测试熔断功能（用于验证）
function testFuse(mb) {
  const bytes = mb * 1024 * 1024;
  console.log(`[Test] Simulating RSS = ${mb}MB (${bytes} bytes)`);
  const usage = { rss: bytes, heapUsed: 0, heapTotal: 0 };
  checkFuse(usage);
}

// 如果直接运行
if (require.main === module) {
  const args = process.argv.slice(2);
  if (args[0] === '--test-fuse' && args[1]) {
    testFuse(parseInt(args[1], 10));
  } else {
    // 开始监控
    sample();
    setInterval(sample, SAMPLE_INTERVAL);
  }
}

module.exports = { sample, testFuse, FUSE_THRESHOLD, SAMPLE_INTERVAL };
