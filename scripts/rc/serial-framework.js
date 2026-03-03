#!/usr/bin/env node
/**
 * RC Serial Test Framework - 串行测试框架
 * 强制串行执行，单任务5分钟熔断
 */

const DEFAULT_TIMEOUT = 5 * 60 * 1000; // 5分钟默认超时

class TimeoutError extends Error {
  constructor(message) {
    super(message);
    this.name = 'TimeoutError';
  }
}

function withTimeout(promise, ms, taskName = 'Task') {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new TimeoutError(`${taskName} timeout after ${ms}ms`));
    }, ms);
    promise
      .then((result) => {
        clearTimeout(timer);
        resolve(result);
      })
      .catch((err) => {
        clearTimeout(timer);
        reject(err);
      });
  });
}

async function serialRun(tasks, options = {}) {
  const { timeout = DEFAULT_TIMEOUT, onProgress } = options;
  const results = [];
  const total = tasks.length;
  const startTime = Date.now();

  console.log(`[Serial] Starting ${total} tasks (timeout: ${timeout}ms per task)`);

  for (let i = 0; i < tasks.length; i++) {
    const taskNum = i + 1;
    const progress = `[${taskNum}/${total}]`;
    
    console.log(`${progress} Starting task...`);
    
    const taskStart = Date.now();
    try {
      const task = typeof tasks[i] === 'function' ? tasks[i] : () => tasks[i];
      const result = await withTimeout(task(), timeout, `Task ${taskNum}`);
      const elapsed = Date.now() - taskStart;
      
      results.push({ status: 'ok', result, elapsed });
      console.log(`${progress} Completed in ${elapsed}ms`);
      
      if (onProgress) {
        onProgress({ current: taskNum, total, elapsed, result });
      }
    } catch (err) {
      const elapsed = Date.now() - taskStart;
      results.push({ status: 'error', error: err.message, elapsed });
      console.error(`${progress} Failed after ${elapsed}ms: ${err.message}`);
    }
  }

  const totalTime = Date.now() - startTime;
  const successCount = results.filter(r => r.status === 'ok').length;
  
  console.log(`[Serial] Completed: ${successCount}/${total} in ${totalTime}ms`);
  
  return {
    results,
    summary: {
      total,
      success: successCount,
      failed: total - successCount,
      duration: totalTime
    }
  };
}

// 干运行测试
async function dryRun() {
  const testTasks = [
    () => new Promise(r => setTimeout(() => r('task1'), 50)),
    () => new Promise(r => setTimeout(() => r('task2'), 50)),
    () => new Promise(r => setTimeout(() => r('task3'), 50))
  ];
  
  console.log('[DryRun] Testing serial framework...');
  const result = await serialRun(testTasks, { timeout: 1000 });
  
  if (result.summary.success === 3) {
    console.log('[DryRun] PASSED: All tasks executed serially');
    return true;
  }
  throw new Error('Dry run failed');
}

// CLI入口
if (require.main === module) {
  const args = process.argv.slice(2);
  if (args.includes('--dry-run')) {
    dryRun().then(() => process.exit(0)).catch(() => process.exit(1));
  } else {
    console.log('Usage: node serial-framework.js --dry-run');
    console.log('Module exports: { serialRun, dryRun, TimeoutError }');
  }
}

module.exports = { serialRun, dryRun, TimeoutError };
