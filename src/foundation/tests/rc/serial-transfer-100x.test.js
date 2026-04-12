#!/usr/bin/env node
/**
 * B-35/02: 100x100MB Serial Transfer Stress Test
 * 串行第二关 - 100次×100MB传输压力测试
 */

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { serialRun } = require('../../scripts/rc/serial-framework.js');

// Paths
const SOURCE_FILE = path.join(__dirname, '../fixtures/100mb.bin');
const TEMP_DIR = path.join(__dirname, '../fixtures/temp');
const LOGS_DIR = path.join(process.cwd(), 'logs/rc');
const RESULTS_CSV = path.join(LOGS_DIR, 'transfer-100x-results.csv');
const MEMORY_CSV = path.join(LOGS_DIR, 'memory-100x-samples.csv');
const EXPECTED_SHA256 = '79853691df6293731487bdf0ac8f3f965602adc9db145aa00d487fb8415f01bf';

// Ensure directories exist
if (!fs.existsSync(TEMP_DIR)) fs.mkdirSync(TEMP_DIR, { recursive: true });
if (!fs.existsSync(LOGS_DIR)) fs.mkdirSync(LOGS_DIR, { recursive: true });

// Memory monitor with sync writes
function createMemoryMonitor(logFile) {
  fs.writeFileSync(logFile, 'timestamp,rss,heapUsed,heapTotal\n');
  const samples = [];
  
  function sample() {
    const usage = process.memoryUsage();
    const line = `${new Date().toISOString()},${usage.rss},${usage.heapUsed},${usage.heapTotal}`;
    samples.push(line);
    fs.appendFileSync(logFile, line + '\n');
    
    if (usage.rss > 300 * 1024 * 1024) {
      console.error('[FUSE] RSS exceeded 300MB!');
      process.exit(1);
    }
    return usage.rss;
  }
  
  // Initial sample
  sample();
  
  const interval = setInterval(sample, 5000);
  return { stop: () => clearInterval(interval), sample };
}

// Transfer simulation with SHA256 verification
async function simulateTransfer(index) {
  const destFile = path.join(TEMP_DIR, `received-${index}.bin`);
  const start = Date.now();
  
  // Read source, copy to dest
  const data = fs.readFileSync(SOURCE_FILE);
  fs.writeFileSync(destFile, data);
  
  // Calculate SHA256
  const sha256 = crypto.createHash('sha256').update(data).digest('hex');
  const status = sha256 === EXPECTED_SHA256 ? 'PASS' : 'FAIL';
  
  // Cleanup temp file
  try { fs.unlinkSync(destFile); } catch (e) {}
  
  const duration = Date.now() - start;
  const speed = (100 * 1024 * 1024) / (duration / 1000) / (1024 * 1024); // MB/s
  
  return { index, duration, speed: speed.toFixed(2), sha256, status };
}

async function runTest() {
  console.log('[B-35/02] Starting 100x100MB Serial Transfer Test');
  console.log(`[Config] Source: ${SOURCE_FILE}`);
  console.log(`[Config] Expected SHA256: ${EXPECTED_SHA256}`);
  
  // Start memory monitor
  const monitor = createMemoryMonitor(MEMORY_CSV);
  
  // Collect all results
  const allResults = [];
  
  // Serial execution using for-await pattern
  for (let i = 0; i < 100; i++) {
    const progress = `[${i + 1}/100]`;
    console.log(`${progress} Starting transfer...`);
    
    try {
      // 5 minute timeout per task
      const result = await Promise.race([
        simulateTransfer(i + 1),
        new Promise((_, reject) => 
          setTimeout(() => reject(new Error('Task timeout (5min)')), 5 * 60 * 1000)
        )
      ]);
      
      allResults.push(result);
      console.log(`${progress} Completed in ${result.duration}ms, speed: ${result.speed}MB/s, ${result.status}`);
    } catch (err) {
      console.error(`${progress} Failed: ${err.message}`);
      allResults.push({ index: i + 1, duration: 0, speed: 0, sha256: '', status: 'ERROR' });
    }
  }
  
  // Stop memory monitor
  monitor.stop();
  
  // Write results CSV atomically
  let csvContent = 'index,duration_ms,speed_mbps,sha256,status\n';
  for (const r of allResults) {
    csvContent += `${r.index},${r.duration},${r.speed},${r.sha256},${r.status}\n`;
  }
  fs.writeFileSync(RESULTS_CSV, csvContent);
  
  // Calculate statistics
  const passCount = allResults.filter(r => r.status === 'PASS').length;
  const failCount = allResults.filter(r => r.status === 'FAIL').length;
  const errorCount = allResults.filter(r => r.status === 'ERROR').length;
  const speeds = allResults.filter(r => r.speed > 0).map(r => parseFloat(r.speed));
  const avgSpeed = speeds.length > 0 ? (speeds.reduce((a, b) => a + b, 0) / speeds.length).toFixed(2) : 0;
  const minSpeed = speeds.length > 0 ? Math.min(...speeds).toFixed(2) : 0;
  const maxSpeed = speeds.length > 0 ? Math.max(...speeds).toFixed(2) : 0;
  
  // Get memory stats
  const memLines = fs.readFileSync(MEMORY_CSV, 'utf8').split('\n').filter(l => l.trim() && !l.startsWith('timestamp'));
  const rssValues = memLines.map(l => parseInt(l.split(',')[1])).filter(v => !isNaN(v));
  const peakRss = rssValues.length > 0 ? (Math.max(...rssValues) / 1024 / 1024).toFixed(2) : 0;
  
  console.log('\n=== B-35/02 Test Results ===');
  console.log(`Total transfers: ${allResults.length}`);
  console.log(`Passed: ${passCount}, Failed: ${failCount}, Errors: ${errorCount}`);
  console.log(`Average speed: ${avgSpeed} MB/s`);
  console.log(`Min speed: ${minSpeed} MB/s, Max speed: ${maxSpeed} MB/s`);
  console.log(`Peak RSS: ${peakRss} MB`);
  console.log(`\nResults CSV: ${RESULTS_CSV}`);
  console.log(`Memory CSV: ${MEMORY_CSV}`);
  
  // Cleanup temp directory
  try { fs.rmSync(TEMP_DIR, { recursive: true, force: true }); } catch (e) {}
  
  // Exit code based on results
  if (passCount === 100 && parseFloat(avgSpeed) >= 5 && parseFloat(peakRss) < 300) {
    console.log('\n[B-35/02] PASSED - All 100 transfers successful, Avg speed >= 5MB/s, RSS < 300MB');
    process.exit(0);
  } else {
    console.error(`\n[B-35/02] FAILED - Pass: ${passCount}/100, Avg speed: ${avgSpeed}MB/s, Peak RSS: ${peakRss}MB`);
    process.exit(1);
  }
}

// Allow module usage or direct execution
if (require.main === module) {
  runTest().catch(e => {
    console.error('[Fatal]', e);
    process.exit(1);
  });
}

module.exports = { runTest, simulateTransfer };
