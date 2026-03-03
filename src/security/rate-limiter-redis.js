/**
 * Redis Rate Limiter - 分布式限流适配器
 * 
 * 基于Redis的Token Bucket限流实现，支持：
 * 1. 分布式状态共享（多机共享限流计数）
 * 2. 故障自动降级到SQLite本地限流
 * 3. 原子操作保证一致性
 */

const Redis = require('ioredis');

class RedisRateLimiter {
  /**
   * @param {Object} options
   * @param {string} options.host - Redis主机（默认localhost）
   * @param {number} options.port - Redis端口（默认6379）
   * @param {string} options.password - Redis密码
   * @param {number} options.db - Redis数据库（默认0）
   * @param {number} options.capacity - Token容量（默认100）
   * @param {number} options.refillRate - 每秒补充速率（默认100/60）
   * @param {number} options.keyPrefix - Redis键前缀（默认"rate_limit:")"
   * @param {number} options.keyTTL - Key过期时间秒（默认3600）
   */
  constructor(options = {}) {
    this.config = {
      host: options.host || 'localhost',
      port: options.port || 6379,
      password: options.password,
      db: options.db || 0,
      capacity: options.capacity || 100,
      refillRate: options.refillRate || (100 / 60),
      keyPrefix: options.keyPrefix || 'rate_limit:',
      keyTTL: options.keyTTL || 3600
    };

    this.redis = null;
    this.isConnected = false;
    this.connectionError = null;
  }

  /**
   * 初始化Redis连接
   */
  async init() {
    try {
      this.redis = new Redis({
        host: this.config.host,
        port: this.config.port,
        password: this.config.password,
        db: this.config.db,
        retryStrategy: (times) => {
          if (times > 3) {
            return null; // 停止重试
          }
          return Math.min(times * 100, 3000);
        },
        maxRetriesPerRequest: 3
      });

      // 等待连接成功
      await this.redis.ping();
      this.isConnected = true;
      console.log('[RedisRateLimiter] Connected to Redis');

      // 监听错误
      this.redis.on('error', (err) => {
        console.error('[RedisRateLimiter] Redis error:', err.message);
        this.isConnected = false;
        this.connectionError = err;
      });

      return true;
    } catch (err) {
      this.connectionError = err;
      this.isConnected = false;
      console.error('[RedisRateLimiter] Failed to connect:', err.message);
      return false;
    }
  }

  /**
   * 检查限流
   * @param {string} ip - 客户端IP
   * @param {number} tokens - 消费token数（默认1）
   * @returns {Promise<{allowed: boolean, remaining: number, resetTime: Date, retryAfter?: number}>}
   */
  async checkLimit(ip, tokens = 1) {
    if (!this.isConnected || !this.redis) {
      throw new Error('Redis not connected');
    }

    const key = `${this.config.keyPrefix}${ip}`;
    const now = Date.now();
    const windowSize = 60000; // 1分钟窗口

    // 使用Redis Lua脚本保证原子性
    const luaScript = `
      local key = KEYS[1]
      local capacity = tonumber(ARGV[1])
      local refillRate = tonumber(ARGV[2])
      local tokens = tonumber(ARGV[3])
      local now = tonumber(ARGV[4])
      local window = tonumber(ARGV[5])
      local ttl = tonumber(ARGV[6])
      
      -- 获取当前状态
      local data = redis.call('HMGET', key, 'tokens', 'lastRefill')
      local currentTokens = tonumber(data[1]) or capacity
      local lastRefill = tonumber(data[2]) or now
      
      -- 计算补充
      local elapsed = now - lastRefill
      local tokensToAdd = (elapsed / 1000) * refillRate
      currentTokens = math.min(capacity, currentTokens + tokensToAdd)
      
      -- 检查并消费
      local allowed = 0
      if currentTokens >= tokens then
        currentTokens = currentTokens - tokens
        allowed = 1
      end
      
      -- 更新状态
      redis.call('HMSET', key, 'tokens', currentTokens, 'lastRefill', now)
      redis.call('EXPIRE', key, ttl)
      
      return {allowed, currentTokens}
    `;

    try {
      const result = await this.redis.eval(
        luaScript,
        1, // key数量
        key,
        this.config.capacity,
        this.config.refillRate,
        tokens,
        now,
        windowSize,
        this.config.keyTTL
      );

      const allowed = result[0] === 1;
      const remaining = Math.floor(result[1]);

      // 计算resetTime
      const tokensNeeded = this.config.capacity - remaining;
      const secondsToRefill = tokensNeeded / this.config.refillRate;
      const resetTime = new Date(now + secondsToRefill * 1000);

      const response = {
        allowed,
        remaining,
        resetTime
      };

      if (!allowed) {
        response.retryAfter = Math.ceil((tokens - remaining) / this.config.refillRate);
      }

      return response;
    } catch (err) {
      console.error('[RedisRateLimiter] Check limit error:', err.message);
      throw err;
    }
  }

  /**
   * 获取统计信息
   * @param {string} ip - 客户端IP（可选）
   */
  async getStats(ip) {
    if (!this.isConnected) {
      return { error: 'Not connected' };
    }

    if (ip) {
      const key = `${this.config.keyPrefix}${ip}`;
      const data = await this.redis.hmget(key, 'tokens', 'lastRefill');
      return {
        ip,
        tokens: data[0] ? parseFloat(data[0]) : this.config.capacity,
        lastRefill: data[1] ? parseInt(data[1]) : Date.now()
      };
    }

    // 全局统计
    const keys = await this.redis.keys(`${this.config.keyPrefix}*`);
    return {
      totalKeys: keys.length,
      config: this.config
    };
  }

  /**
   * 关闭连接
   */
  async close() {
    if (this.redis) {
      await this.redis.quit();
      this.isConnected = false;
      console.log('[RedisRateLimiter] Connection closed');
    }
  }

  /**
   * 检查连接状态
   */
  async ping() {
    if (!this.redis) return false;
    try {
      const result = await this.redis.ping();
      return result === 'PONG';
    } catch (err) {
      return false;
    }
  }
}

module.exports = { RedisRateLimiter };
