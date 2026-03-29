/**
 * 统一错误处理中间件
 * Unified Error Handler Middleware
 */

const ERROR_CODES = {
  // 客户端错误 (4xx)
  BAD_REQUEST: { code: 'E400', status: 400, message: 'Bad Request' },
  UNAUTHORIZED: { code: 'E401', status: 401, message: 'Unauthorized' },
  FORBIDDEN: { code: 'E403', status: 403, message: 'Forbidden' },
  NOT_FOUND: { code: 'E404', status: 404, message: 'Not Found' },
  CONFLICT: { code: 'E409', status: 409, message: 'Conflict' },
  PAYLOAD_TOO_LARGE: { code: 'E413', status: 413, message: 'Payload Too Large' },
  UNSUPPORTED_MEDIA_TYPE: { code: 'E415', status: 415, message: 'Unsupported Media Type' },
  UNPROCESSABLE_ENTITY: { code: 'E422', status: 422, message: 'Unprocessable Entity' },
  TOO_MANY_REQUESTS: { code: 'E429', status: 429, message: 'Too Many Requests' },
  
  // 服务器错误 (5xx)
  INTERNAL_ERROR: { code: 'E500', status: 500, message: 'Internal Server Error' },
  NOT_IMPLEMENTED: { code: 'E501', status: 501, message: 'Not Implemented' },
  SERVICE_UNAVAILABLE: { code: 'E503', status: 503, message: 'Service Unavailable' },
  
  // 业务错误 (自定义)
  VECTOR_NOT_FOUND: { code: 'E1001', status: 404, message: 'Vector Not Found' },
  INVALID_VECTOR_DIMENSION: { code: 'E1002', status: 400, message: 'Invalid Vector Dimension' },
  SHARD_UNAVAILABLE: { code: 'E1003', status: 503, message: 'Shard Unavailable' },
  INDEX_BUILD_IN_PROGRESS: { code: 'E1004', status: 409, message: 'Index Build In Progress' }
};

class HajimiError extends Error {
  constructor(errorType, customMessage, details = {}) {
    const errorDef = ERROR_CODES[errorType] || ERROR_CODES.INTERNAL_ERROR;
    super(customMessage || errorDef.message);
    this.code = errorDef.code;
    this.status = errorDef.status;
    this.details = details;
    this.timestamp = new Date().toISOString();
  }

  toJSON() {
    return {
      error: {
        code: this.code,
        message: this.message,
        details: this.details,
        timestamp: this.timestamp
      }
    };
  }
}

/**
 * Express/Connect 风格的错误处理中间件
 */
function errorHandler(err, req, res, next) {
  // 如果是 HajimiError，使用其定义的状态码
  if (err instanceof HajimiError) {
    res.writeHead(err.status, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify(err.toJSON()));
    return;
  }

  // 处理特定的错误类型
  let status = 500;
  let code = 'E500';
  let message = 'Internal Server Error';
  let details = {};

  // JSON 解析错误
  if (err instanceof SyntaxError && err.status === 400 && 'body' in err) {
    status = 400;
    code = 'E400';
    message = 'Invalid JSON';
    details = { body: err.message };
  }
  
  // 超时错误
  else if (err.code === 'ETIMEDOUT' || err.code === 'ETIMEOUT') {
    status = 504;
    code = 'E504';
    message = 'Gateway Timeout';
  }
  
  // 其他错误
  else {
    // 在生产环境中不暴露详细的错误信息
    if (process.env.NODE_ENV === 'production') {
      details = {};
    } else {
      details = { 
        stack: err.stack,
        originalError: err.message 
      };
    }
  }

  // 记录错误日志
  console.error(`[Error ${code}] ${req.method} ${req.path}:`, err.message);

  res.writeHead(status, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({
    error: {
      code,
      message,
      details,
      timestamp: new Date().toISOString()
    }
  }));
}

/**
 * 404 处理中间件
 */
function notFoundHandler(req, res) {
  const error = new HajimiError('NOT_FOUND', `Route ${req.method} ${req.path} not found`);
  res.writeHead(404, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify(error.toJSON()));
}

/**
 * 请求日志中间件
 */
function requestLogger(req, res, next) {
  const start = Date.now();
  
  res.on('finish', () => {
    const duration = Date.now() - start;
    console.log(
      `[${new Date().toISOString()}] ${req.method} ${req.path} ` +
      `${res.statusCode} ${duration}ms`
    );
  });
  
  next();
}

/**
 * 请求体大小限制中间件
 */
function bodySizeLimit(maxSize = '1mb') {
  const bytes = parseSize(maxSize);
  
  return (req, res, next) => {
    let body = '';
    let size = 0;
    
    req.on('data', chunk => {
      size += chunk.length;
      if (size > bytes) {
        const error = new HajimiError('PAYLOAD_TOO_LARGE', `Request body exceeds ${maxSize}`);
        res.writeHead(413, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(error.toJSON()));
        req.destroy();
        return;
      }
      body += chunk;
    });
    
    req.on('end', () => {
      req.body = body;
      next();
    });
    
    req.on('error', next);
  };
}

/**
 * JSON 解析中间件
 */
function jsonParser(req, res, next) {
  if (req.headers['content-type']?.includes('application/json')) {
    try {
      if (req.body) {
        req.body = JSON.parse(req.body);
      }
      next();
    } catch (err) {
      const error = new HajimiError('BAD_REQUEST', 'Invalid JSON in request body');
      res.writeHead(400, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(error.toJSON()));
    }
  } else {
    next();
  }
}

/**
 * 解析大小字符串 (如 '1mb', '512kb')
 */
function parseSize(size) {
  const units = {
    b: 1,
    kb: 1024,
    mb: 1024 * 1024,
    gb: 1024 * 1024 * 1024
  };
  
  const match = size.toString().toLowerCase().match(/^(\d+(?:\.\d+)?)\s*(b|kb|mb|gb)?$/);
  if (!match) return 1024 * 1024; // default 1MB
  
  const value = parseFloat(match[1]);
  const unit = match[2] || 'b';
  return Math.floor(value * units[unit]);
}

module.exports = {
  HajimiError,
  ERROR_CODES,
  errorHandler,
  notFoundHandler,
  requestLogger,
  bodySizeLimit,
  jsonParser
};
