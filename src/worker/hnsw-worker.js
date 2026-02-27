/**
 * HNSW Worker Thread
 * 在独立线程中执行索引构建，避免阻塞主线程
 */

const { parentPort, workerData, threadId } = require('worker_threads');
const path = require('path');
const { HNSWIndex } = require(path.join(__dirname, '../vector/hnsw-core')); // Use absolute path

// Worker状态
const state = {
  isBuilding: false,
  buildStartTime: null,
  vectorsProcessed: 0,
  totalVectors: 0,
  memoryLimitMB: workerData?.memoryLimitMB || 300
};

// 内存监控
function checkMemory() {
  const usage = process.memoryUsage();
  const rssMB = usage.rss / 1024 / 1024;
  
  if (rssMB > state.memoryLimitMB) {
    parentPort.postMessage({
      type: 'memory_warning',
      rssMB: rssMB.toFixed(2),
      limitMB: state.memoryLimitMB,
      threadId
    });
    return false;
  }
  return true;
}

// 构建索引
async function buildIndex(data) {
  const { vectors, options = {} } = data;
  
  state.isBuilding = true;
  state.buildStartTime = Date.now();
  state.totalVectors = vectors.length;
  state.vectorsProcessed = 0;

  parentPort.postMessage({
    type: 'build_start',
    totalVectors: vectors.length,
    threadId,
    timestamp: Date.now()
  });

  try {
    // 创建索引
    const index = new HNSWIndex({
      dimension: options.dimension || 128,
      M: options.M || 16,
      efConstruction: options.efConstruction || 200
    });

    // 批量插入向量
    const batchSize = 1000;
    for (let i = 0; i < vectors.length; i += batchSize) {
      // 检查内存
      if (!checkMemory()) {
        throw new Error(`Worker memory limit exceeded: ${state.memoryLimitMB}MB`);
      }

      const batch = vectors.slice(i, Math.min(i + batchSize, vectors.length));
      
      for (const { id, vector } of batch) {
        index.insert(id, vector);
      }

      state.vectorsProcessed += batch.length;

      // 每批次报告进度
      if (i % (batchSize * 10) === 0) {
        parentPort.postMessage({
          type: 'progress',
          processed: state.vectorsProcessed,
          total: state.totalVectors,
          percent: ((state.vectorsProcessed / state.totalVectors) * 100).toFixed(1),
          threadId
        });
      }

      // 让出时间片，避免阻塞
      await new Promise(resolve => setImmediate(resolve));
    }

    const duration = Date.now() - state.buildStartTime;

    // 序列化索引数据返回
    const indexData = {
      nodes: Array.from(index.nodes.entries()),
      entryPoint: index.entryPoint,
      maxLevel: index.maxLevel,
      elementCount: index.elementCount,
      dimension: index.dimension
    };

    parentPort.postMessage({
      type: 'build_complete',
      duration,
      vectorsProcessed: state.vectorsProcessed,
      threadId,
      indexData
    });

    state.isBuilding = false;

  } catch (err) {
    state.isBuilding = false;
    parentPort.postMessage({
      type: 'build_error',
      error: err.message,
      stack: err.stack,
      threadId
    });
  }
}

// 处理搜索请求
async function search(data) {
  const { indexData, query, k = 10 } = data;
  
  try {
    // 重建索引（简化版）
    const index = new HNSWIndex({ dimension: indexData.dimension });
    index.nodes = new Map(indexData.nodes);
    index.entryPoint = indexData.entryPoint;
    index.maxLevel = indexData.maxLevel;
    index.elementCount = indexData.elementCount;

    const startTime = Date.now();
    const results = index.search(query, k);
    const duration = Date.now() - startTime;

    parentPort.postMessage({
      type: 'search_complete',
      results,
      duration,
      threadId
    });

  } catch (err) {
    parentPort.postMessage({
      type: 'search_error',
      error: err.message,
      threadId
    });
  }
}

// 获取统计信息
function getStats() {
  const memUsage = process.memoryUsage();
  
  parentPort.postMessage({
    type: 'stats',
    isBuilding: state.isBuilding,
    vectorsProcessed: state.vectorsProcessed,
    totalVectors: state.totalVectors,
    memory: {
      rss: memUsage.rss,
      rssMB: (memUsage.rss / 1024 / 1024).toFixed(2),
      heapUsed: memUsage.heapUsed,
      heapUsedMB: (memUsage.heapUsed / 1024 / 1024).toFixed(2)
    },
    threadId
  });
}

// 监听主线程消息
parentPort.on('message', async (message) => {
  const { type, data } = message;

  switch (type) {
    case 'build':
      await buildIndex(data);
      break;
    case 'search':
      await search(data);
      break;
    case 'stats':
      getStats();
      break;
    case 'ping':
      parentPort.postMessage({ type: 'pong', threadId, timestamp: Date.now() });
      break;
    default:
      parentPort.postMessage({
        type: 'error',
        error: `Unknown message type: ${type}`,
        threadId
      });
  }
});

// 报告Worker已就绪
parentPort.postMessage({
  type: 'ready',
  threadId,
  pid: process.pid,
  timestamp: Date.now()
});
