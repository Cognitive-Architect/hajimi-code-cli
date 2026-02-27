/**
 * WASM优化的向量路由
 * 提供WASM模式检测和性能指标
 */

const { HajimiError } = require('../middleware/error-handler');
const { HybridHNSWIndex } = require('../../vector/hnsw-index-hybrid');

class VectorWASMRoutes {
  constructor(options = {}) {
    this.hybridIndex = options.hybridIndex;
    this.indexBuilder = options.indexBuilder;
  }

  /**
   * 注册路由
   */
  register(app) {
    // 覆盖标准向量路由，增加WASM支持
    app.post('/vector/add', this._add.bind(this));
    app.post('/vector/search', this._search.bind(this));
    
    // WASM特定端点
    app.get('/vector/mode', this._getMode.bind(this));
    app.post('/vector/mode/force-js', this._forceJSMode.bind(this));
    app.get('/vector/performance', this._getPerformance.bind(this));
    
    // Worker状态
    app.get('/vector/worker/status', this._getWorkerStatus.bind(this));
    app.post('/vector/build-async', this._buildAsync.bind(this));
  }

  /**
   * 添加向量（WASM优化）
   */
  async _add(req, res) {
    if (!this.hybridIndex) {
      throw new HajimiError('SERVICE_UNAVAILABLE', 'Hybrid index not initialized');
    }

    const { id, vector } = req.body;
    
    if (!id || !vector) {
      throw new HajimiError('BAD_REQUEST', 'Missing id or vector');
    }

    try {
      const result = this.hybridIndex.insert(id, vector);
      
      res.writeHead(201, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({
        success: true,
        id,
        mode: result.mode,
        message: `Vector added using ${result.mode} mode`
      }));
    } catch (err) {
      throw new HajimiError('INTERNAL_ERROR', `Insert failed: ${err.message}`);
    }
  }

  /**
   * 搜索向量（WASM优化）
   */
  async _search(req, res) {
    if (!this.hybridIndex) {
      throw new HajimiError('SERVICE_UNAVAILABLE', 'Hybrid index not initialized');
    }

    const { vector, k = 10 } = req.body;
    
    if (!vector) {
      throw new HajimiError('BAD_REQUEST', 'Missing vector');
    }

    try {
      const result = this.hybridIndex.search(vector, k);
      
      res.end(JSON.stringify({
        success: true,
        results: result.results,
        mode: result.mode,
        latencyMs: result.latency,
        meta: {
          k,
          returned: result.results.length,
          wasmAccelerated: result.mode === 'wasm'
        }
      }));
    } catch (err) {
      throw new HajimiError('INTERNAL_ERROR', `Search failed: ${err.message}`);
    }
  }

  /**
   * 获取当前运行模式
   */
  async _getMode(req, res) {
    const mode = this.hybridIndex ? this.hybridIndex.getMode() : 'unknown';
    const stats = this.hybridIndex ? this.hybridIndex.getStats() : {};
    
    res.end(JSON.stringify({
      mode,
      wasmAvailable: mode === 'wasm',
      stats,
      hint: mode === 'wasm' 
        ? 'Running with WASM acceleration (5x faster)'
        : 'Running in JavaScript mode (WASM not available)'
    }));
  }

  /**
   * 强制降级到JS模式
   */
  async _forceJSMode(req, res) {
    if (!this.hybridIndex) {
      throw new HajimiError('SERVICE_UNAVAILABLE', 'Hybrid index not initialized');
    }

    const previousMode = this.hybridIndex.getMode();
    this.hybridIndex.forceDowngrade();
    
    res.end(JSON.stringify({
      success: true,
      previousMode,
      currentMode: 'javascript',
      message: 'Force downgraded to JavaScript mode'
    }));
  }

  /**
   * 获取性能指标
   */
  async _getPerformance(req, res) {
    const indexStats = this.hybridIndex ? this.hybridIndex.getStats() : {};
    const workerStats = this.indexBuilder ? this.indexBuilder.getStats() : {};
    
    const memUsage = process.memoryUsage();
    
    res.end(JSON.stringify({
      mode: this.hybridIndex ? this.hybridIndex.getMode() : 'unknown',
      index: indexStats,
      worker: workerStats,
      memory: {
        rss: memUsage.rss,
        rssMB: (memUsage.rss / 1024 / 1024).toFixed(2),
        heapUsed: memUsage.heapUsed,
        heapUsedMB: (memUsage.heapUsed / 1024 / 1024).toFixed(2)
      },
      timestamp: new Date().toISOString()
    }));
  }

  /**
   * 获取Worker状态
   */
  async _getWorkerStatus(req, res) {
    if (!this.indexBuilder) {
      res.end(JSON.stringify({
        available: false,
        message: 'Worker builder not configured'
      }));
      return;
    }

    const stats = this.indexBuilder.getStats();
    
    res.end(JSON.stringify({
      available: true,
      isBuilding: stats.isBuilding,
      progress: stats.progress,
      useWorker: stats.useWorker,
      stats
    }));
  }

  /**
   * 异步构建索引
   */
  async _buildAsync(req, res) {
    if (!this.indexBuilder) {
      throw new HajimiError('SERVICE_UNAVAILABLE', 'Worker builder not configured');
    }

    if (this.indexBuilder.isBuilding()) {
      throw new HajimiError('CONFLICT', 'Another build is in progress');
    }

    const { vectors } = req.body;
    
    if (!vectors || !Array.isArray(vectors)) {
      throw new HajimiError('BAD_REQUEST', 'Missing vectors array');
    }

    // 启动异步构建
    this.indexBuilder.buildIndex(vectors, req.body.options || {})
      .then(result => {
        console.log('Async build completed:', result);
      })
      .catch(err => {
        console.error('Async build failed:', err);
      });

    // 立即返回202 Accepted
    res.writeHead(202, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({
      success: true,
      message: 'Build started asynchronously',
      status: 'building',
      checkStatus: 'GET /vector/worker/status'
    }));
  }
}

module.exports = { VectorWASMRoutes };
