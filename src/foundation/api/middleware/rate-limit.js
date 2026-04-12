/**
 * 限流中间件
 * 将Token Bucket限流器包装为Express风格中间件
 */

const { TokenBucketRateLimiter } = require('../../security/rate-limiter');

// 默认限流配置：100 req/min, burst 20
const defaultOptions = {
  capacity: 20,
  refillRate: 100 / 60, // 每秒100/60个token
  enabled: true
};

/**
 * 创建限流中间件
 * @param {object} options - 限流配置
 * @returns {function} Express中间件
 */
function createRateLimitMiddleware(options = {}) {
  const config = { ...defaultOptions, ...options };
  const limiter = new TokenBucketRateLimiter({
    capacity: config.capacity,
    refillRate: config.refillRate
  });

  return function rateLimitMiddleware(req, res, next) {
    // 如果禁用了限流，直接通过
    if (!config.enabled) {
      return next();
    }

    // 获取客户端IP
    const ip = req.ip || req.connection.remoteAddress || 'unknown';
    
    // 消费token
    const result = limiter.consume(ip);

    // 设置限流响应头
    res.setHeader('X-RateLimit-Limit', Math.ceil(config.refillRate * 60)); // per minute
    res.setHeader('X-RateLimit-Remaining', Math.max(0, result.remaining));
    res.setHeader('X-RateLimit-Reset', Math.floor(result.resetTime.getTime() / 1000));

    if (!result.allowed) {
      // 限流触发
      const retryAfter = result.retryAfter || 60;
      res.setHeader('Retry-After', retryAfter);
      
      // 记录warn日志（含requestId）
      if (req.requestId) {
        console.log(JSON.stringify({
          timestamp: new Date().toISOString(),
          level: 'warn',
          requestId: req.requestId,
          message: 'Rate limit exceeded',
          ip: ip,
          retryAfter: retryAfter
        }));
      }
      
      res.status(429).json({
        error: 'Too Many Requests',
        message: `Rate limit exceeded. Retry after ${retryAfter} seconds.`,
        retryAfter: retryAfter,
        requestId: req.requestId
      });
      return;
    }

    next();
  };
}

module.exports = { createRateLimitMiddleware };
