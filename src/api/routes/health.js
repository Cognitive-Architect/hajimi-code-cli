/**
 * 健康检查路由
 * Health Check Routes
 * 
 * GET /health - 基础健康检查
 * GET /health/ready - 就绪检查
 * GET /health/live - 存活检查
 * GET /health/metrics - 指标数据
 */

const { HajimiError } = require('../middleware/error-handler');

class HealthRoutes {
  constructor(options = {}) {
    this.vectorAPI = options.vectorAPI;
    this.storage = options.storage;
    this.startTime = Date.now();
    this.version = options.version || '3.0.0';
  }

  /**
   * 注册路由到服务器
   */
  register(app) {
    // 基础健康检查
    app.get('/health', (req, res) => {
      res.end(JSON.stringify({
        status: 'ok',
        timestamp: new Date().toISOString(),
        version: this.version
      }));
    });

    // 就绪检查 (检查依赖服务)
    app.get('/health/ready', async (req, res) => {
      const checks = await this._runReadinessChecks();
      const allReady = checks.every(c => c.status === 'ok');
      
      res.writeHead(allReady ? 200 : 503, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({
        status: allReady ? 'ready' : 'not_ready',
        checks,
        timestamp: new Date().toISOString()
      }));
    });

    // 存活检查
    app.get('/health/live', (req, res) => {
      const uptime = Date.now() - this.startTime;
      
      res.end(JSON.stringify({
        status: 'alive',
        uptime: this._formatUptime(uptime),
        uptimeMs: uptime,
        timestamp: new Date().toISOString()
      }));
    });

    // 指标数据
    app.get('/health/metrics', async (req, res) => {
      const metrics = await this._collectMetrics();
      res.end(JSON.stringify(metrics));
    });
  }

  /**
   * 运行就绪检查
   */
  async _runReadinessChecks() {
    const checks = [];

    // 检查存储
    checks.push({
      name: 'storage',
      status: this.storage ? 'ok' : 'unknown',
      message: this.storage ? 'Storage connected' : 'Storage not initialized'
    });

    // 检查向量API
    checks.push({
      name: 'vector_api',
      status: this.vectorAPI ? 'ok' : 'unknown',
      message: this.vectorAPI ? 'Vector API ready' : 'Vector API not initialized'
    });

    // 检查内存
    const memUsage = process.memoryUsage();
    const rssMB = memUsage.rss / 1024 / 1024;
    const memOk = rssMB < 400; // < 400MB threshold
    
    checks.push({
      name: 'memory',
      status: memOk ? 'ok' : 'warning',
      message: `RSS: ${rssMB.toFixed(2)}MB`,
      details: {
        rss: memUsage.rss,
        heapTotal: memUsage.heapTotal,
        heapUsed: memUsage.heapUsed,
        external: memUsage.external
      }
    });

    return checks;
  }

  /**
   * 收集指标数据
   */
  async _collectMetrics() {
    const memUsage = process.memoryUsage();
    const uptime = Date.now() - this.startTime;

    const metrics = {
      timestamp: new Date().toISOString(),
      version: this.version,
      uptime: {
        seconds: Math.floor(uptime / 1000),
        formatted: this._formatUptime(uptime)
      },
      memory: {
        rss: memUsage.rss,
        rssMB: (memUsage.rss / 1024 / 1024).toFixed(2),
        heapTotal: memUsage.heapTotal,
        heapTotalMB: (memUsage.heapTotal / 1024 / 1024).toFixed(2),
        heapUsed: memUsage.heapUsed,
        heapUsedMB: (memUsage.heapUsed / 1024 / 1024).toFixed(2),
        external: memUsage.external,
        externalMB: (memUsage.external / 1024 / 1024).toFixed(2)
      },
      process: {
        pid: process.pid,
        nodeVersion: process.version,
        platform: process.platform,
        arch: process.arch
      }
    };

    // 添加向量API指标（如果有）
    if (this.vectorAPI && this.vectorAPI.getStats) {
      try {
        metrics.vector = await this.vectorAPI.getStats();
      } catch (err) {
        metrics.vector = { error: err.message };
      }
    }

    // 添加存储指标（如果有）
    if (this.storage && this.storage.getStats) {
      try {
        metrics.storage = await this.storage.getStats();
      } catch (err) {
        metrics.storage = { error: err.message };
      }
    }

    return metrics;
  }

  /**
   * 格式化运行时间
   */
  _formatUptime(ms) {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ${hours % 24}h ${minutes % 60}m`;
    if (hours > 0) return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
    if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
    return `${seconds}s`;
  }
}

module.exports = { HealthRoutes };
