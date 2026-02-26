/**
 * HTTP RESTful 服务器
 * Hajimi V3 API Server
 * 
 * 支持跨进程/跨机器调用
 */

const http = require('http');
const url = require('url');
const { 
  errorHandler, 
  notFoundHandler, 
  requestLogger,
  bodySizeLimit,
  jsonParser 
} = require('./middleware/error-handler');
const { requestIdMiddleware } = require('./middleware/request-id');
const { HealthRoutes } = require('./routes/health');
const { VectorRoutes } = require('./routes/vector');

class HajimiServer {
  constructor(options = {}) {
    this.port = options.port || 3000;
    this.host = options.host || '0.0.0.0';
    this.corsOrigin = options.corsOrigin || 'http://localhost:3000';
    this.version = options.version || '3.0.0';
    
    // 依赖注入
    this.vectorAPI = options.vectorAPI;
    this.storage = options.storage;
    
    // 中间件配置
    this.maxBodySize = options.maxBodySize || '1mb';
    
    // CORS配置
    this.corsOrigin = options.corsOrigin || 'http://localhost:3000';
    
    // 路由
    this.routes = [];
    this.middlewares = [];
    
    // 服务器实例
    this.server = null;
    this.isRunning = false;
    
    // 统计
    this.stats = {
      requestsHandled: 0,
      errors: 0,
      startTime: null
    };
  }

  /**
   * 注册中间件
   */
  use(middleware) {
    this.middlewares.push(middleware);
    return this;
  }

  /**
   * 注册路由
   */
  register(method, path, handler) {
    this.routes.push({ method: method.toUpperCase(), path, handler });
    return this;
  }

  /**
   * 初始化路由
   */
  _initRoutes() {
    // 健康检查路由
    const healthRoutes = new HealthRoutes({
      vectorAPI: this.vectorAPI,
      storage: this.storage,
      version: this.version
    });
    healthRoutes.register(this);

    // 向量路由
    const vectorRoutes = new VectorRoutes({
      vectorAPI: this.vectorAPI,
      maxVectorDim: 1024,
      maxBatchSize: 100
    });
    vectorRoutes.register(this);
  }

  /**
   * 启动服务器
   */
  async start() {
    if (this.isRunning) {
      throw new Error('Server is already running');
    }

    // 验证 port 和 host 的合法性
    if (!Number.isInteger(this.port) || this.port < 1 || this.port > 65535) {
      throw new Error(`Invalid port: ${this.port}. Must be an integer between 1 and 65535.`);
    }
    if (!this.host || typeof this.host !== 'string') {
      throw new Error(`Invalid host: ${this.host}. Must be a non-empty string.`);
    }

    this._initRoutes();

    this.server = http.createServer(this._handleRequest.bind(this));

    return new Promise((resolve, reject) => {
      this.server.listen(this.port, this.host, (err) => {
        if (err) {
          reject(err);
          return;
        }
        
        this.isRunning = true;
        this.stats.startTime = Date.now();
        
        console.log(`🚀 Hajimi Server v${this.version} running at http://${this.host}:${this.port}`);
        console.log(`📊 Health check: http://${this.host}:${this.port}/health`);
        
        resolve({
          host: this.host,
          port: this.port,
          url: `http://${this.host}:${this.port}`
        });
      });
    });
  }

  /**
   * 停止服务器
   */
  async stop() {
    if (!this.isRunning) {
      return;
    }

    return new Promise((resolve) => {
      this.server.close(() => {
        this.isRunning = false;
        console.log('👋 Server stopped');
        resolve();
      });
    });
  }

  /**
   * 处理请求
   */
  async _handleRequest(req, res) {
    this.stats.requestsHandled++;
    
    // 解析URL
    const parsedUrl = url.parse(req.url, true);
    req.path = parsedUrl.pathname;
    req.query = parsedUrl.query;
    
    // 设置响应头
    res.setHeader('Content-Type', 'application/json');
    res.setHeader('X-API-Version', this.version);
    
    // CORS headers
    const allowOrigin = this.corsOrigin === '*' ? '*' : this.corsOrigin;
    res.setHeader('Access-Control-Allow-Origin', allowOrigin);
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
    
    if (req.method === 'OPTIONS') {
      res.writeHead(204);
      res.end();
      return;
    }

    // 请求日志
    const startTime = Date.now();
    
    try {
      // 应用中间件
      await this._applyMiddlewares(req, res);
      
      // 匹配路由
      const route = this._matchRoute(req.method, req.path);
      
      if (route) {
        await route.handler(req, res);
      } else {
        notFoundHandler(req, res);
      }
    } catch (err) {
      this.stats.errors++;
      errorHandler(err, req, res, () => {});
    }
    
    // 记录请求耗时
    const duration = Date.now() - startTime;
    console.log(`[${new Date().toISOString()}] ${req.method} ${req.path} ${res.statusCode} ${duration}ms`);
  }

  /**
   * 应用中间件链
   */
  async _applyMiddlewares(req, res) {
    // 内置中间件
    await this._runMiddleware(requestIdMiddleware, req, res);
    await this._runMiddleware(bodySizeLimit(this.maxBodySize), req, res);
    await this._runMiddleware(jsonParser, req, res);
  }

  /**
   * 配置校验
   */
  _validateConfig() {
    const port = this.port;
    const host = this.host;

    // port校验：必须为1-65535的整数
    if (!Number.isInteger(port) || port < 1 || port > 65535) {
      const error = new Error(`Invalid port: ${port}. Port must be an integer between 1 and 65535.`);
      error.code = 'INVALID_CONFIG';
      throw error;
    }

    // host校验：非空字符串
    if (!host || typeof host !== 'string' || host.trim() === '') {
      const error = new Error(`Invalid host: ${host}. Host must be a non-empty string.`);
      error.code = 'INVALID_CONFIG';
      throw error;
    }

    // corsOrigin校验：必须为字符串或'*'
    if (this.corsOrigin !== '*' && (!this.corsOrigin || typeof this.corsOrigin !== 'string')) {
      const error = new Error(`Invalid corsOrigin: ${this.corsOrigin}. Must be a string or '*'.`);
      error.code = 'INVALID_CONFIG';
      throw error;
    }
  }

  /**
   * 运行单个中间件
   */
  _runMiddleware(middleware, req, res) {
    return new Promise((resolve, reject) => {
      middleware(req, res, (err) => {
        if (err) reject(err);
        else resolve();
      });
    });
  }

  /**
   * 匹配路由
   */
  _matchRoute(method, path) {
    // 精确匹配
    let route = this.routes.find(r => r.method === method && r.path === path);
    if (route) return route;
    
    // 参数匹配 (如 /vector/:id)
    route = this.routes.find(r => {
      if (r.method !== method) return false;
      return this._matchPath(r.path, path);
    });
    
    return route;
  }

  /**
   * 匹配路径模式
   */
  _matchPath(pattern, path) {
    const patternParts = pattern.split('/');
    const pathParts = path.split('/');
    
    if (patternParts.length !== pathParts.length) return false;
    
    const params = {};
    
    for (let i = 0; i < patternParts.length; i++) {
      const pPart = patternParts[i];
      const part = pathParts[i];
      
      if (pPart.startsWith(':')) {
        // 参数
        params[pPart.slice(1)] = decodeURIComponent(part);
      } else if (pPart !== part) {
        return false;
      }
    }
    
    // 将参数附加到请求对象
    return { params };
  }

  /**
   * GET 快捷方法
   */
  get(path, handler) {
    this.register('GET', path, handler);
    return this;
  }

  /**
   * POST 快捷方法
   */
  post(path, handler) {
    this.register('POST', path, handler);
    return this;
  }

  /**
   * DELETE 快捷方法
   */
  delete(path, handler) {
    this.register('DELETE', path, handler);
    return this;
  }

  /**
   * 获取服务器统计
   */
  getStats() {
    return {
      ...this.stats,
      uptime: this.stats.startTime ? Date.now() - this.stats.startTime : 0,
      isRunning: this.isRunning,
      port: this.port,
      host: this.host
    };
  }
}

module.exports = { HajimiServer };
