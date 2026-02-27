/**
 * 索引构建桥接层
 * 主线程与Worker之间的桥接，提供统一的API
 */

const { WorkerPool } = require('./worker-pool');
const EventEmitter = require('events');

class IndexBuilderBridge extends EventEmitter {
  constructor(options = {}) {
    super();
    
    this.useWorker = options.useWorker !== false; // 默认启用Worker
    this.fallbackToMain = options.fallbackToMain !== false; // Worker失败时回退主线程
    
    this.workerPool = null;
    this.isInitialized = false;
    
    // 构建状态
    this.buildState = {
      isBuilding: false,
      progress: 0,
      lastBuildTime: null,
      buildCount: 0
    };
  }

  /**
   * 初始化
   */
  async init() {
    if (!this.useWorker) {
      console.log('ℹ️ Worker mode disabled, using main thread');
      this.isInitialized = true;
      return;
    }

    try {
      this.workerPool = new WorkerPool({
        poolSize: 1, // 索引构建单Worker即可
        taskTimeout: 600000 // 10分钟超时
      });

      await this.workerPool.init();

      // 监听Worker事件
      this.workerPool.on('progress', (data) => {
        this.buildState.progress = parseFloat(data.percent);
        this.emit('progress', data);
      });

      this.workerPool.on('memory:warning', (data) => {
        console.warn('Worker memory warning:', data);
        this.emit('memory:warning', data);
      });

      this.workerPool.on('worker:failed', () => {
        if (this.fallbackToMain) {
          console.warn('Worker failed, will fallback to main thread');
        }
      });

      this.isInitialized = true;
      console.log('✅ Index Builder Bridge initialized (Worker mode)');

    } catch (err) {
      console.error('Failed to initialize Worker Pool:', err);
      
      if (this.fallbackToMain) {
        console.log('⚠️ Falling back to main thread mode');
        this.useWorker = false;
        this.isInitialized = true;
      } else {
        throw err;
      }
    }
  }

  /**
   * 构建索引
   */
  async buildIndex(vectors, options = {}) {
    if (this.buildState.isBuilding) {
      throw new Error('Another build is in progress');
    }

    this.buildState.isBuilding = true;
    this.buildState.progress = 0;

    const startTime = Date.now();

    try {
      let result;

      if (this.useWorker && this.workerPool) {
        // 使用Worker构建
        console.log(`🚀 Building index in Worker (${vectors.length} vectors)...`);
        
        result = await this.workerPool.buildIndex(vectors, options);
        
        console.log(`✅ Worker build completed in ${result.duration}ms`);
        
      } else {
        // 主线程构建
        console.log(`🚀 Building index in main thread (${vectors.length} vectors)...`);
        
        result = await this._buildInMainThread(vectors, options);
        
        console.log(`✅ Main thread build completed in ${result.duration}ms`);
      }

      this.buildState.lastBuildTime = Date.now() - startTime;
      this.buildState.buildCount++;
      
      return result;

    } catch (err) {
      console.error('Build failed:', err);
      
      // 如果Worker失败且允许回退，尝试主线程
      if (this.useWorker && this.fallbackToMain && this.workerPool) {
        console.log('⚠️ Worker build failed, trying main thread...');
        this.useWorker = false;
        return this.buildIndex(vectors, options);
      }
      
      throw err;
      
    } finally {
      this.buildState.isBuilding = false;
      this.buildState.progress = 100;
    }
  }

  /**
   * 在主线程构建（回退方案）
   */
  async _buildInMainThread(vectors, options) {
    const { HNSWIndex } = require('../vector/hnsw-core');
    
    const startTime = Date.now();
    
    const index = new HNSWIndex({
      dimension: options.dimension || 128,
      M: options.M || 16,
      efConstruction: options.efConstruction || 200
    });

    // 分批插入，避免阻塞
    const batchSize = 100;
    for (let i = 0; i < vectors.length; i += batchSize) {
      const batch = vectors.slice(i, Math.min(i + batchSize, vectors.length));
      
      for (const { id, vector } of batch) {
        index.insert(id, vector);
      }

      // 更新进度
      this.buildState.progress = ((i + batch.length) / vectors.length) * 100;
      
      // 报告进度
      this.emit('progress', {
        processed: i + batch.length,
        total: vectors.length,
        percent: this.buildState.progress.toFixed(1)
      });

      // 让出时间片
      await new Promise(resolve => setImmediate(resolve));
    }

    const duration = Date.now() - startTime;

    return {
      type: 'build_complete',
      duration,
      vectorsProcessed: vectors.length,
      threadId: 'main',
      indexData: {
        nodes: Array.from(index.nodes.entries()),
        entryPoint: index.entryPoint,
        maxLevel: index.maxLevel,
        elementCount: index.elementCount,
        dimension: index.dimension
      }
    };
  }

  /**
   * 检查是否正在构建
   */
  isBuilding() {
    return this.buildState.isBuilding;
  }

  /**
   * 获取构建进度
   */
  getProgress() {
    return this.buildState.progress;
  }

  /**
   * 获取统计
   */
  getStats() {
    return {
      ...this.buildState,
      useWorker: this.useWorker,
      isInitialized: this.isInitialized,
      workerStats: this.workerPool ? this.workerPool.getStats() : null
    };
  }

  /**
   * 关闭
   */
  async shutdown() {
    if (this.workerPool) {
      await this.workerPool.shutdown();
      this.workerPool = null;
    }
    this.isInitialized = false;
  }
}

module.exports = { IndexBuilderBridge };
