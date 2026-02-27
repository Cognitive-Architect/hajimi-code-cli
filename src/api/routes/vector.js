/**
 * 向量操作路由
 * Vector API Routes
 * 
 * POST   /vector/add      - 添加向量
 * POST   /vector/batch    - 批量添加
 * POST   /vector/search   - 搜索最近邻
 * GET    /vector/:id      - 获取向量
 * DELETE /vector/:id      - 删除向量
 * GET    /vector          - 列表/统计
 */

const { HajimiError } = require('../middleware/error-handler');

class VectorRoutes {
  constructor(options = {}) {
    this.vectorAPI = options.vectorAPI;
    this.maxVectorDim = options.maxVectorDim || 1024;
    this.maxBatchSize = options.maxBatchSize || 100;
    this.rateLimiter = options.rateLimiter || null;
  }

  /**
   * 注册路由到服务器
   */
  register(app) {
    app.post('/vector/add', this._add.bind(this));
    app.post('/vector/batch', this._batchAdd.bind(this));
    app.post('/vector/search', this._search.bind(this));
    app.get('/vector/:id', this._get.bind(this));
    app.delete('/vector/:id', this._delete.bind(this));
    app.get('/vector', this._list.bind(this));
    app.post('/vector/build', this._buildIndex.bind(this));
    app.get('/vector/stats', this._stats.bind(this));
  }

  /**
   * 添加向量
   */
  async _add(req, res) {
    this._checkVectorAPI();
    
    const { id, vector, metadata = {} } = req.body;
    
    if (!id) {
      throw new HajimiError('BAD_REQUEST', 'Missing required field: id');
    }
    
    if (!vector || !Array.isArray(vector)) {
      throw new HajimiError('BAD_REQUEST', 'Missing or invalid field: vector (must be array)');
    }
    
    if (vector.length > this.maxVectorDim) {
      throw new HajimiError('PAYLOAD_TOO_LARGE', 
        `Vector dimension ${vector.length} exceeds maximum ${this.maxVectorDim}`);
    }
    
    if (!vector.every(v => typeof v === 'number' && !isNaN(v))) {
      throw new HajimiError('UNPROCESSABLE_ENTITY', 'Vector must contain only valid numbers');
    }

    try {
      const result = await this.vectorAPI.add(id, vector, metadata);
      res.writeHead(201, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({
        success: true,
        id,
        message: 'Vector added successfully',
        details: result
      }));
    } catch (err) {
      if (err.message?.includes('exists')) {
        throw new HajimiError('CONFLICT', `Vector with id '${id}' already exists`);
      }
      throw err;
    }
  }

  /**
   * 批量添加
   */
  async _batchAdd(req, res) {
    this._checkVectorAPI();
    
    const { vectors } = req.body;
    
    if (!vectors || !Array.isArray(vectors)) {
      throw new HajimiError('BAD_REQUEST', 'Missing or invalid field: vectors (must be array)');
    }
    
    if (vectors.length > this.maxBatchSize) {
      throw new HajimiError('PAYLOAD_TOO_LARGE', 
        `Batch size ${vectors.length} exceeds maximum ${this.maxBatchSize}`);
    }

    const results = { success: [], failed: [], total: vectors.length };

    for (const item of vectors) {
      try {
        if (!item.id || !item.vector) {
          results.failed.push({ item, error: 'Missing id or vector' });
          continue;
        }
        await this.vectorAPI.add(item.id, item.vector, item.metadata || {});
        results.success.push(item.id);
      } catch (err) {
        results.failed.push({ id: item.id, error: err.message });
      }
    }

    res.writeHead(201, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({
      success: results.failed.length === 0,
      message: `Added ${results.success.length}/${results.total} vectors`,
      results
    }));
  }

  /**
   * 搜索向量
   */
  async _search(req, res) {
    this._checkVectorAPI();
    
    const { vector, k = 10, options = {} } = req.body;
    
    if (!vector || !Array.isArray(vector)) {
      throw new HajimiError('BAD_REQUEST', 'Missing or invalid field: vector (must be array)');
    }
    
    if (!vector.every(v => typeof v === 'number' && !isNaN(v))) {
      throw new HajimiError('UNPROCESSABLE_ENTITY', 'Vector must contain only valid numbers');
    }

    try {
      const startTime = Date.now();
      const results = await this.vectorAPI.search(vector, k, options);
      const latency = Date.now() - startTime;

      res.end(JSON.stringify({
        success: true,
        results,
        meta: {
          queryVectorDim: vector.length,
          k,
          returned: results.length,
          latencyMs: latency
        }
      }));
    } catch (err) {
      throw new HajimiError('INTERNAL_ERROR', `Search failed: ${err.message}`);
    }
  }

  /**
   * 获取向量
   */
  async _get(req, res) {
    this._checkVectorAPI();
    
    const { id } = req.params;
    
    try {
      const vector = await this.vectorAPI.get(id);
      
      if (!vector) {
        throw new HajimiError('VECTOR_NOT_FOUND', `Vector with id '${id}' not found`);
      }

      res.end(JSON.stringify({
        success: true,
        id,
        vector: vector.vector || vector,
        metadata: vector.metadata || {}
      }));
    } catch (err) {
      if (err instanceof HajimiError) throw err;
      throw new HajimiError('INTERNAL_ERROR', `Failed to get vector: ${err.message}`);
    }
  }

  /**
   * 删除向量
   */
  async _delete(req, res) {
    this._checkVectorAPI();
    
    const { id } = req.params;
    
    try {
      const result = await this.vectorAPI.delete(id);
      
      res.end(JSON.stringify({
        success: true,
        id,
        message: 'Vector deleted successfully',
        deleted: result
      }));
    } catch (err) {
      if (err.message?.includes('not found')) {
        throw new HajimiError('VECTOR_NOT_FOUND', `Vector with id '${id}' not found`);
      }
      throw new HajimiError('INTERNAL_ERROR', `Failed to delete vector: ${err.message}`);
    }
  }

  /**
   * 列表和统计
   */
  async _list(req, res) {
    this._checkVectorAPI();
    
    try {
      const stats = await this.vectorAPI.getStats();
      
      res.end(JSON.stringify({
        success: true,
        stats: {
          totalVectors: stats.totalVectors || stats.elementCount || 0,
          dimension: stats.dimension || 0,
          indexBuilt: stats.indexBuilt || false
        }
      }));
    } catch (err) {
      res.end(JSON.stringify({
        success: true,
        stats: {
          totalVectors: 0,
          note: 'Statistics not available'
        }
      }));
    }
  }

  /**
   * 构建索引
   */
  async _buildIndex(req, res) {
    this._checkVectorAPI();
    
    try {
      const result = await this.vectorAPI.buildIndex();
      
      res.end(JSON.stringify({
        success: true,
        message: 'Index built successfully',
        result
      }));
    } catch (err) {
      throw new HajimiError('INTERNAL_ERROR', `Failed to build index: ${err.message}`);
    }
  }

  /**
   * 索引统计
   */
  async _stats(req, res) {
    this._checkVectorAPI();
    
    try {
      const stats = await this.vectorAPI.getStats();
      res.end(JSON.stringify({ success: true, stats }));
    } catch (err) {
      throw new HajimiError('INTERNAL_ERROR', `Failed to get stats: ${err.message}`);
    }
  }

  /**
   * 检查VectorAPI是否可用
   */
  _checkVectorAPI() {
    if (!this.vectorAPI) {
      throw new HajimiError('SERVICE_UNAVAILABLE', 'Vector API not initialized');
    }
  }
}

module.exports = { VectorRoutes };
