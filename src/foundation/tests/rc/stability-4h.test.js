#!/usr/bin/env node
/**
 * B-35/03: 4小时长时稳定性测试
 * 串行第三关 - 4h × 30s采样 × 内存熔断
 * Duration: 4 * 60 * 60 * 1000 = 14400 seconds
 */

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { execSync } = require('child_process');

// V8堆限制自检（37号工程实施）
const v8 = require('v8');
const heapStats = v8.getHeapStatistics();
const heapLimitMB = heapStats.heap_size_limit / 1024 / 1024;

if (heapLimitMB > 600) {
  console.warn(`⚠️  当前V8堆限制: ${heapLimitMB.toFixed(0)}MB（建议<600MB）`);
  console.warn('💡 使用优化参数重新运行: npm run test:rc');
  console.warn('   或手动: node --max-old-space-size=512 tests/rc/stability-4h.test.js');
  console.warn('');
}

// Constants: 4h duration, 30s interval
const DURATION = 4 * 60 * 60 * 1000; // 14,400,000 ms (4h)
const SAMPLE_INTERVAL = 30000; // 30,000 ms (30s)
const FUSE_RSS = 300 * 1024 * 1024; // 300MB fuse
const FUSE_TIME = 4.5 * 60 * 60 * 1000; // 4.5h max
const EXPECTED_SAMPLES = 288; // 4h / 30s = 288

// Paths
const SOURCE_FILE = path.join(__dirname, '../fixtures/100mb.bin');
const TEMP_DIR = path.join(__dirname, '../fixtures/temp-stability');
const LOG_DIR = path.join(process.cwd(), 'logs', 'rc');
const MEMORY_CSV = path.join(LOG_DIR, 'stability-4h-memory.csv');
const TEMP_LOG = path.join(LOG_DIR, 'temperature-log.txt');
const EXPECTED_SHA256 = '79853691df6293731487bdf0ac8f3f965602adc9db145aa00d487fb8415f01bf';

// Ensure directories exist
if (!fs.existsSync(TEMP_DIR)) fs.mkdirSync(TEMP_DIR, { recursive: true });
if (!fs.existsSync(LOG_DIR)) fs.mkdirSync(LOG_DIR, { recursive: true });

// Temperature monitoring
function getTemperature() {
  try {
    if (process.platform === 'linux') {
      const output = execSync('sensors 2>/dev/null | grep -i "core\|cpu\|tdie" | head -5', { encoding: 'utf8', timeout: 5000 });
      return output.trim() || 'N/A (no sensors data)';
    }
    if (process.platform === 'win32') {
      try {
        const output = execSync('wmic /namespace:\\\root\\wmi PATH MSAcpi_ThermalZoneTemperature get CurrentTemperature 2>nul', { encoding: 'utf8', timeout: 5000 });
        const temps = output.match(/\d+/g);
        if (temps) {
          const celsius = temps.map(t => ((parseInt(t) - 2732) / 10).toFixed(1));
          return `CPU: ${celsius.join('°C, ')}°C`;
        }
      } catch (e) {}
      return 'N/A (WMIC not available)';
    }
    if (process.platform === 'darwin') {
      try {
        const output = execSync('powermetrics --samplers smc -n1 2>/dev/null | grep -i "temperature\|thermal" | head -3', { encoding: 'utf8', timeout: 5000 });
        return output.trim() || 'N/A (powermetrics unavailable)';
      } catch (e) {}
    }
    return 'N/A (unsupported platform)';
  } catch (e) {
    return `N/A (error: ${e.message})`;
  }
}

// Initialize logs
fs.writeFileSync(MEMORY_CSV, 'timestamp,rss,heapUsed,heapTotal,transferCount\n');
fs.writeFileSync(TEMP_LOG, `# Temperature Log - 4h Stability Test\n# Started: ${new Date().toISOString()}\n\n`);

// Sample memory
function sampleMemory(transferCount) {
  const usage = process.memoryUsage();
  const line = `${new Date().toISOString()},${usage.rss},${usage.heapUsed},${usage.heapTotal},${transferCount}\n`;
  fs.appendFileSync(MEMORY_CSV, line);
  return usage;
}

// Check fuse conditions
function checkFuse(usage, elapsed) {
  if (usage.rss > FUSE_RSS) {
    throw new Error(`FUSE: RSS ${(usage.rss/1024/1024).toFixed(2)}MB > ${FUSE_RSS/1024/1024}MB`);
  }
  if (elapsed > FUSE_TIME) {
    throw new Error(`FUSE: Runtime ${(elapsed/3600000).toFixed(2)}h > ${FUSE_TIME/3600000}h`);
  }
}

// Simulate one transfer
async function simulateTransfer(index) {
  const destFile = path.join(TEMP_DIR, `stability-${index}.bin`);
  const data = fs.readFileSync(SOURCE_FILE);
  fs.writeFileSync(destFile, data);
  const sha256 = crypto.createHash('sha256').update(data).digest('hex');
  try { fs.unlinkSync(destFile); } catch (e) {}
  return sha256 === EXPECTED_SHA256;
}

// Main test runner
async function runStabilityTest() {
  console.log('[B-35/03] 4-Hour Stability Test Starting...');
  console.log(`[Config] Duration: ${DURATION/3600000}h, Interval: ${SAMPLE_INTERVAL/1000}s, Expected samples: ${EXPECTED_SAMPLES}`);
  
  const startTime = Date.now();
  let transferCount = 0;
  let successCount = 0;
  let lastHourLog = 0;
  let lastTempLog = Date.now();
  
  // Initial sample
  sampleMemory(0);
  fs.appendFileSync(TEMP_LOG, `[${new Date().toLocaleTimeString()}] Temp: ${getTemperature()}\n`);
  
  // Main loop: setInterval for 30s samples
  return new Promise((resolve, reject) => {
    const interval = setInterval(async () => {
      try {
        const elapsed = Date.now() - startTime;
        const hoursElapsed = Math.floor(elapsed / 3600000);
        
        // Hourly progress log
        if (hoursElapsed > lastHourLog) {
          console.log(`[Hour ${hoursElapsed}] ${transferCount} transfers, ${successCount} success`);
          lastHourLog = hoursElapsed;
        }
        
        // Temperature log every 5 minutes
        if (Date.now() - lastTempLog > 300000) {
          fs.appendFileSync(TEMP_LOG, `[${new Date().toLocaleTimeString()}] Temp: ${getTemperature()}\n`);
          lastTempLog = Date.now();
        }
        
        // Run transfers continuously between samples
        const transferStart = Date.now();
        while (Date.now() - transferStart < SAMPLE_INTERVAL - 100) {
          transferCount++;
          const success = await simulateTransfer(transferCount);
          if (success) successCount++;
        }
        
        // Memory sample
        const usage = sampleMemory(transferCount);
        
        // Check fuses
        checkFuse(usage, elapsed);
        
        // Completion check: 4h = 14,400 seconds
        if (elapsed >= DURATION) {
          clearInterval(interval);
          resolve({ transferCount, successCount, duration: elapsed });
        }
      } catch (err) {
        clearInterval(interval);
        reject(err);
      }
    }, SAMPLE_INTERVAL);
  });
}

// Execute
if (require.main === module) {
  runStabilityTest()
    .then(result => {
      fs.appendFileSync(TEMP_LOG, `# Completed: ${new Date().toISOString()}\n`);
      console.log('\n=== B-35/03 Results ===');
      console.log(`4小时完成: ${result.transferCount}次传输, ${result.successCount}次成功`);
      console.log(`Success rate: ${(result.successCount/result.transferCount*100).toFixed(2)}%`);
      console.log(`Duration: ${(result.duration/3600000).toFixed(2)}h`);
      console.log(`Memory CSV: ${MEMORY_CSV}`);
      console.log(`Temp Log: ${TEMP_LOG}`);
      process.exit(0);
    })
    .catch(err => {
      fs.appendFileSync(TEMP_LOG, `# Failed: ${new Date().toISOString()} - ${err.message}\n`);
      console.error('[FUSE TRIGGERED]', err.message);
      process.exit(1);
    });
}

module.exports = { runStabilityTest, DURATION, SAMPLE_INTERVAL, EXPECTED_SAMPLES };
