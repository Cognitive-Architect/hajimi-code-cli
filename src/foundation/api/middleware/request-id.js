/**
 * Request ID Middleware
 * 为每个请求生成唯一的追踪ID
 */

const crypto = require('crypto');

/**
 * 请求ID中间件
 * 生成UUID v4作为请求ID，并设置到请求对象和响应头中
 */
function requestIdMiddleware(req, res, next) {
  // 生成UUID v4
  const requestId = crypto.randomUUID();
  req.requestId = requestId;
  res.setHeader('X-Request-Id', requestId);
  next();
}

module.exports = { requestIdMiddleware };
