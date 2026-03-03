/**
 * Redis Rate Limiter v2 - 生产加固版
 * 
 * 改进：
 * 1. 连接池管理
 * 2. 自动重试与指数退避
 * 3. 健康检查与故障检测
 * 4. 更完善的降级策略
 */

const Redis = require('ioredis');

class RedisRateLimiterV2 {
  constructor(options = {}) {
    this.config = {
      host: options.host || 'localhost',
      port: options.port || 6379,
      password: options.password,
      db: options.db || 0,
      
      // Token Bucket配置
      capacity: options.capacity || 100,
      refillRate: options.refillRate || (100 / 60),
      keyPrefix: options.keyPrefix || 'rate_limit:',
      keyTTL: options.keyTTL || 3600,
      
      // 连接池配置
      maxRetries: options.maxRetries || 3,
      retryDelay: options.retryDelay || 100,
      connectTimeout: options.connectTimeout || 5000,
      
      // 健康检查
      healthCheckInterval: options.healthCheckInterval || 30000,
      
      // 降级配置
      fallbackEnabled: options.fallbackEnabled !== false
    };

    this.redis = null;
    this.state = {
      isConnected: false,
      isHealthy: false,
      lastError: null,
      connectAttempts: 0,
      consecutiveFailures: 0
    };
    
    this.healthCheckTimer = null;
    this.stats = {
      totalRequests: 0,
      successfulRequests: 0,
      failedRequests: 0,
      fallbackTriggers: 0
    };
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
        
        // 连接池与重试
        retryStrategy: (times) => {
          this.state.connectAttempts = times;
          if (times > this.config.maxRetries) {
            return null; // 停止重试
          }
          // 指数退避
          return Math.min(times * this.config.retryDelay, 3000);
        },
        maxRetriesPerRequest: this.config.maxRetries,
        
        // 超时
        connectTimeout: this.config.connectTimeout,
        commandTimeout: 5000,
        
        // 连接池
        lazyConnect: true,
        keepAlive: 30000
      });

      // 事件监听
      this.redis.on('connect', () => {
        this.state.isConnected = true;
        this.state.consecutiveFailures = 0;
        console.log('[RedisV2] Connected');
      });

      this.redis.on('error', (err) => {
        this.state.lastError = err;
        this.state.consecutiveFailures++;
        console.error('[RedisV2] Error:', err.message);
      });

      this.redis.on('close', () => {
        this.state.isConnected = false;
        console.log('[RedisV2] Connection closed');
      });

      // 主动连接测试
      await this.redis.connect();
      await this.redis.ping();
      
      this.state.isHealthy = true;
      
      // 启动健康检查
      this._startHealthCheck();
      
      return true;
    } catch (err) {
      this.state.lastError = err;
      this.state.isConnected = false;
      this.state.isHealthy = false;
      console.error('[RedisV2] Init failed:', err.message);
      return false;
    }
  }

  /**
   * 健康检查（修复FIND-025-02：添加3秒超时保护）
   */
  async healthCheck() {
    if (!this.redis) return false;
    
    try {
      // Promise.race超时模式：3秒超时
      const result = await Promise.race([
        this.redis.ping(),
        new Promise((_, reject) => 
          setTimeout(() => reject(new Error('healthCheck timeout')), 3000)
        )
      ]);
      
      const healthy = result === 'PONG';
      this.state.isHealthy = healthy;
      this.state.lastError = null;
      return healthy;
    } catch (err) {
      this.state.isHealthy = false;
      this.state.lastError = err;
      return false;
    }
  }

  /**
   * 启动定时健康检查
   */
  _startHealthCheck() {
    if (this.healthCheckTimer) {
      clearInterval(this.healthCheckTimer);
    }
    
    this.healthCheckTimer = setInterval(async () => {
      const healthy = await this.healthCheck();
      if (!healthy && this.state.consecutiveFailures > 3) {
        console.warn('[RedisV2] Health check failed, marking unhealthy');
      }
    }, this.config.healthCheckInterval);
  }

  /**
   * 检查限流（带自动重试）
   */
  async checkLimit(ip, tokens = 1) {
    this.stats.totalRequests++;
    
    // RISK-03 FIX: 如果Redis不健康，先尝试主动重连
    if (!this.state.isHealthy) {
      console.info('[RedisV2] Redis unhealthy, attempting proactive reconnection...');
      const reconnected = await this.healthCheck();
      
      if (reconnected) {
        console.info('[RedisV2] Redis recovered');
        this.state.consecutiveFailures = 0;
      } else if (this.config.fallbackEnabled) {
        // 重连失败且启用了降级，触发降级
        this.stats.fallbackTriggers++;
        console.warn('[RedisV2] Reconnection failed, triggering fallback');
        throw new Error('Redis unhealthy and reconnection failed, fallback required');
      }
    }

    const key = `${this.config.keyPrefix}${ip}`;
    const now = Date.now();

    // Lua脚本（原子操作）
    const luaScript = `
      local key = KEYS[1]
      local capacity = tonumber(ARGV[1])
      local refillRate = tonumber(ARGV[2])
      local tokens = tonumber(ARGV[3])
      local now = tonumber(ARGV[4])
      local ttl = tonumber(ARGV[5])
      
      local data = redis.call('HMGET', key, 'tokens', 'lastRefill')
      local currentTokens = tonumber(data[1]) or capacity
      local lastRefill = tonumber(data[2]) or now
      
      local elapsed = now - lastRefill
      local tokensToAdd = (elapsed / 1000) * refillRate
      currentTokens = math.min(capacity, currentTokens + tokensToAdd)
      
      local allowed = 0
      if currentTokens >= tokens then
        currentTokens = currentTokens - tokens
        allowed = 1
      end
      
      redis.call('HMSET', key, 'tokens', currentTokens, 'lastRefill', now)
      redis.call('EXPIRE', key, ttl)
      
      return {allowed, currentTokens}
    `;

    try {
      const result = await this.redis.eval(
        luaScript,
        1,
        key,
        this.config.capacity,
        this.config.refillRate,
        tokens,
        now,
        this.config.keyTTL
      );

      this.stats.successfulRequests++;

      const allowed = result[0] === 1;
      const remaining = Math.floor(result[1]);
      
      const tokensNeeded = this.config.capacity - remaining;
      const secondsToRefill = tokensNeeded / this.config.refillRate;
      const resetTime = new Date(now + secondsToRefill * 1000);

      const response = { allowed, remaining, resetTime };
      if (!allowed) {
        response.retryAfter = Math.ceil((tokens - remaining) / this.config.refillRate);
      }

      return response;
    } catch (err) {
      this.stats.failedRequests++;
      this.state.consecutiveFailures++;
      
      // 连续失败超过阈值，标记不健康
      if (this.state.consecutiveFailures > 3) {
        this.state.isHealthy = false;
      }
      
      throw err;
    }
  }

  /**
   * 获取统计信息
   */
  getStats() {
    return {
      state: { ...this.state },
      stats: { ...this.stats },
      config: {
        capacity: this.config.capacity,
        refillRate: this.config.refillRate,
        fallbackEnabled: this.config.fallbackEnabled
      }
    };
  }

  /**
   * 关闭连接
   */
  async close() {
    if (this.healthCheckTimer) {
      clearInterval(this.healthCheckTimer);
      this.healthCheckTimer = null;
    }
    
    if (this.redis) {
      await this.redis.quit();
      this.state.isConnected = false;
      this.state.isHealthy = false;
    }
  }
}

module.exports = { RedisRateLimiterV2 };
