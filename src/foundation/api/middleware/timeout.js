/**
 * 请求超时中间件
 * 防止慢查询阻塞服务器
 */

/**
 * 创建超时中间件
 * @param {object} options - 超时配置
 * @param {number} options.timeout - 超时时间（毫秒），默认30000
 * @returns {function} Express中间件
 */
function createTimeoutMiddleware(options = {}) {
  const timeoutMs = options.timeout || 30000;

  return function timeoutMiddleware(req, res, next) {
    // 设置超时定时器
    const timer = setTimeout(() => {
      // 超时触发
      if (!res.headersSent) {
        res.status(504).json({
          error: 'Gateway Timeout',
          message: `Request timeout after ${timeoutMs}ms`,
          requestId: req.requestId
        });
      }
    }, timeoutMs);

    // 响应完成时清除定时器
    res.on('finish', () => {
      clearTimeout(timer);
    });

    // 响应关闭时清除定时器（防止内存泄漏）
    res.on('close', () => {
      clearTimeout(timer);
    });

    next();
  };
}

module.exports = { createTimeoutMiddleware };
