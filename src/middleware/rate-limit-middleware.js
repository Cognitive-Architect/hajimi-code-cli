/**
 * Rate Limit Middleware - API层限流中间件
 * 
 * 功能：
 * 1. API请求限流
 * 2. 429响应码
 * 3. 限流头信息
 * 4. WebSocket限流
 * 5. 熔断降级
 */

const { LuxurySQLiteRateLimiter } = require('../security/rate-limiter-sqlite-luxury');
const { logger } = require('../utils/logger');

class RateLimitMiddleware {
  constructor(options = {}) {
    this.limiter = options.limiter || new LuxurySQLiteRateLimiter({
      capacity: options.capacity || 100,
      refillRate: options.refillRate || (100 / 60),
      dbPath: options.dbPath || './data/rate-limiter.db'
    });
    
    this.circuitBreaker = {
      enabled: options.circuitBreaker !== false,
      failureThreshold: options.failureThreshold || 5,
      recoveryTimeout: options.recoveryTimeout || 30000,
      state: 'CLOSED', // CLOSED, OPEN, HALF_OPEN
      failures: 0,
      lastFailureTime: null
    };

    this.isInitialized = false;
  }

  /**
   * 初始化
   */
  async init() {
    if (!this.isInitialized) {
      await this.limiter.init();
      this.isInitialized = true;
      logger.info('RateLimitMiddleware initialized');
    }
  }

  /**
   * Express中间件
   */
  middleware() {
    return async (req, res, next) => {
      try {
        await this.init();

        // 获取客户端IP
        const ip = this._getClientIP(req);

        // 检查熔断器
        if (this._isCircuitOpen()) {
          return this._sendCircuitOpenResponse(res);
        }

        // 检查限流
        const result = await this.limiter.checkLimit(ip);

        // 设置限流响应头
        res.setHeader('X-RateLimit-Limit', '100');
        res.setHeader('X-RateLimit-Remaining', result.remaining.toString());
        res.setHeader('X-RateLimit-Reset', Math.floor(result.resetTime.getTime() / 1000).toString());

        if (!result.allowed) {
          // 熔断器计数
          this._recordFailure();

          // 记录限流日志
          logger.warn('Rate limit exceeded', {
            ip,
            path: req.path,
            requestId: req.requestId,
            retryAfter: result.retryAfter
          });

          // 返回429响应
          return res.status(429).json({
            error: 'Too Many Requests',
            message: '请求过于频繁，请稍后再试',
            retryAfter: result.retryAfter,
            requestId: req.requestId
          });
        }

        // 成功，重置熔断器
        this._recordSuccess();

        next();
      } catch (err) {
        logger.error('Rate limit middleware error', { error: err.message, requestId: req.requestId });
        // 出错时放行（降级策略）
        next();
      }
    };
  }

  /**
   * WebSocket限流
   */
  wsMiddleware() {
    return async (ws, req) => {
      try {
        await this.init();

        const ip = this._getClientIP(req);
        const result = await this.limiter.checkLimit(ip);

        if (!result.allowed) {
          ws.close(1013, 'Too Many Requests'); // 1013 = Try Again Later
          return false;
        }

        return true;
      } catch (err) {
        logger.error('WebSocket rate limit error', { error: err.message });
        return true; // 出错时放行
      }
    };
  }

  /**
   * 获取客户端IP
   */
  _getClientIP(req) {
    return req.headers['x-forwarded-for']?.split(',')[0].trim() 
      || req.headers['x-real-ip']
      || req.ip
      || req.connection?.remoteAddress
      || 'unknown';
  }

  /**
   * 检查熔断器是否打开
   */
  _isCircuitOpen() {
    if (!this.circuitBreaker.enabled) return false;

    if (this.circuitBreaker.state === 'OPEN') {
      // 检查是否到达恢复时间
      const now = Date.now();
      if (now - this.circuitBreaker.lastFailureTime > this.circuitBreaker.recoveryTimeout) {
        this.circuitBreaker.state = 'HALF_OPEN';
        logger.info('Circuit breaker entering HALF_OPEN state');
        return false;
      }
      return true;
    }

    return false;
  }

  /**
   * 记录失败
   */
  _recordFailure() {
    if (!this.circuitBreaker.enabled) return;

    this.circuitBreaker.failures++;
    this.circuitBreaker.lastFailureTime = Date.now();

    if (this.circuitBreaker.failures >= this.circuitBreaker.failureThreshold) {
      this.circuitBreaker.state = 'OPEN';
      logger.warn('Circuit breaker opened due to excessive failures');
    }
  }

  /**
   * 记录成功
   */
  _recordSuccess() {
    if (!this.circuitBreaker.enabled) return;

    this.circuitBreaker.failures = 0;
    if (this.circuitBreaker.state === 'HALF_OPEN') {
      this.circuitBreaker.state = 'CLOSED';
      logger.info('Circuit breaker closed');
    }
  }

  /**
   * 发送熔断响应
   */
  _sendCircuitOpenResponse(res) {
    return res.status(503).json({
      error: 'Service Unavailable',
      message: '服务暂时不可用，请稍后重试',
      retryAfter: Math.ceil(this.circuitBreaker.recoveryTimeout / 1000)
    });
  }

  /**
   * 关闭
   */
  async close() {
    if (this.limiter) {
      await this.limiter.close();
    }
  }
}

module.exports = { RateLimitMiddleware };
