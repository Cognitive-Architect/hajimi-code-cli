/**
 * Token Bucket 限流器
 * 基于内存的限流算法实现
 * 
 * 配置：
 * - capacity: 桶容量（突发请求数）
 * - refillRate: 每秒补充token数（ sustained rate）
 * 
 * 默认：100 req/min = 100/60 per second, burst=20
 * 
 * 债务声明 DEBT-SEC-001:
 * - 使用内存Map存储，进程重启后数据清零
 * - 后续可替换为Redis实现分布式限流
 */

class TokenBucketRateLimiter {
  constructor(options = {}) {
    // 默认：100 req/min，burst 20
    this.capacity = options.capacity || 20;
    this.refillRate = options.refillRate || (100 / 60); // 每秒100/60个token
    this.buckets = new Map(); // IP -> {tokens, lastRefill}
  }

  /**
   * 消费token
   * @param {string} ip - 客户端IP
   * @param {number} tokens - 消费token数（默认1）
   * @returns {object} - { allowed, remaining, resetTime }
   */
  consume(ip, tokens = 1) {
    const now = Date.now();
    
    // 获取或创建bucket
    let bucket = this.buckets.get(ip);
    if (!bucket) {
      bucket = {
        tokens: this.capacity,
        lastRefill: now
      };
      this.buckets.set(ip, bucket);
    }

    // 补充token
    this._refill(bucket, now);

    // 检查是否足够
    if (bucket.tokens >= tokens) {
      bucket.tokens -= tokens;
      return {
        allowed: true,
        remaining: Math.floor(bucket.tokens),
        resetTime: this._calculateResetTime(bucket)
      };
    }

    // 限流触发
    return {
      allowed: false,
      remaining: 0,
      resetTime: this._calculateResetTime(bucket),
      retryAfter: Math.ceil((tokens - bucket.tokens) / this.refillRate)
    };
  }

  /**
   * 补充token
   * @private
   */
  _refill(bucket, now) {
    const elapsedMs = now - bucket.lastRefill;
    const tokensToAdd = (elapsedMs / 1000) * this.refillRate;
    
    bucket.tokens = Math.min(this.capacity, bucket.tokens + tokensToAdd);
    bucket.lastRefill = now;
  }

  /**
   * 计算reset时间
   * @private
   */
  _calculateResetTime(bucket) {
    const tokensNeeded = this.capacity - bucket.tokens;
    const secondsToRefill = tokensNeeded / this.refillRate;
    return new Date(Date.now() + secondsToRefill * 1000);
  }

  /**
   * 获取统计信息（调试用）
   */
  getStats(ip) {
    const bucket = this.buckets.get(ip);
    if (!bucket) {
      return { tokens: this.capacity, remaining: this.capacity };
    }
    this._refill(bucket, Date.now());
    return {
      tokens: bucket.tokens,
      remaining: Math.floor(bucket.tokens),
      capacity: this.capacity,
      refillRate: this.refillRate
    };
  }

  /**
   * 清理过期bucket（防止内存泄漏）
   * @param {number} maxAgeMs - 最大存活时间（默认1小时）
   */
  cleanup(maxAgeMs = 60 * 60 * 1000) {
    const now = Date.now();
    for (const [ip, bucket] of this.buckets.entries()) {
      if (now - bucket.lastRefill > maxAgeMs) {
        this.buckets.delete(ip);
      }
    }
  }
}

module.exports = { TokenBucketRateLimiter };
