/**
 * 安全响应头中间件
 * Helmet.js 轻量级替代
 * 添加基本安全头保护
 */

function securityHeaders(req, res, next) {
  // 防止点击劫持
  res.setHeader('X-Frame-Options', 'DENY');
  
  // 防止MIME类型嗅探
  res.setHeader('X-Content-Type-Options', 'nosniff');
  
  // XSS保护（旧浏览器）
  res.setHeader('X-XSS-Protection', '1; mode=block');
  
  // Referrer策略
  res.setHeader('Referrer-Policy', 'strict-origin-when-cross-origin');
  
  // 权限策略（限制浏览器API）
  res.setHeader('Permissions-Policy', 'geolocation=(), microphone=(), camera=()');
  
  next();
}

module.exports = { securityHeaders };
