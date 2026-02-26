/**
 * JSON结构化日志工具
 * 统一日志格式，便于分析和追踪
 */

/**
 * 敏感字段掩码
 * @param {object} obj - 原始对象
 * @returns {object} - 掩码后的对象
 */
function maskSensitiveData(obj) {
  if (!obj || typeof obj !== 'object') return obj;
  
  const sensitiveFields = ['password', 'secret', 'api_key', 'apikey', 'token', 'authorization', 'auth'];
  const masked = { ...obj };
  
  for (const key of Object.keys(masked)) {
    const lowerKey = key.toLowerCase();
    if (sensitiveFields.some(sf => lowerKey.includes(sf))) {
      masked[key] = '***MASKED***';
    } else if (typeof masked[key] === 'object') {
      masked[key] = maskSensitiveData(masked[key]);
    }
  }
  
  return masked;
}

/**
 * JSON日志记录
 * @param {string} level - 日志级别 (debug, info, warn, error)
 * @param {string} message - 日志消息
 * @param {object} metadata - 元数据
 */
function log(level, message, metadata = {}) {
  const logEntry = {
    timestamp: new Date().toISOString(),
    level: level.toLowerCase(),
    message,
    ...maskSensitiveData(metadata)
  };
  
  console.log(JSON.stringify(logEntry));
}

/**
 * 便捷方法
 */
const logger = {
  debug: (message, metadata) => log('debug', message, metadata),
  info: (message, metadata) => log('info', message, metadata),
  warn: (message, metadata) => log('warn', message, metadata),
  error: (message, metadata) => log('error', message, metadata),
  
  // 请求日志
  request: (req, res, duration) => {
    log('info', 'Request completed', {
      requestId: req.requestId,
      method: req.method,
      path: req.path,
      statusCode: res.statusCode,
      duration: `${duration}ms`,
      ip: req.ip || req.connection?.remoteAddress
    });
  }
};

module.exports = { logger, log };
