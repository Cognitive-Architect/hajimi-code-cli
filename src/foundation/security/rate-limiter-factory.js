/**
 * Rate Limiter Factory - 限流器工厂
 * 
 * 自动选择最优限流器实现：
 * 1. Redis优先（分布式场景）
 * 2. SQLite降级（单机/Redis故障）
 * 3. 内存兜底（极端情况）
 */

const { LuxurySQLiteRateLimiter } = require('./rate-limiter-sqlite-luxury.js');
const { RedisRateLimiter } = require('./rate-limiter-redis.js');

class RateLimiterFactory {
  constructor(options = {}) {
    this.config = {
      // Redis配置
      redis: {
        host: options.redisHost || process.env.REDIS_HOST || 'localhost',
        port: options.redisPort || process.env.REDIS_PORT || 6379,
        password: options.redisPassword || process.env.REDIS_PASSWORD,
        db: options.redisDb || 0,
        enabled: options.redisEnabled !== false // 默认启用Redis尝试
      },
      // SQLite配置
      sqlite: {
        dbPath: options.sqliteDbPath || './data/rate-limiter.db',
        enabled: true // SQLite始终启用作为降级
      },
      // Token Bucket配置
      capacity: options.capacity || 100,
      refillRate: options.refillRate || (100 / 60)
    };

    this.primaryLimiter = null;
    this.fallbackLimiter = null;
    this.currentMode = 'unknown';
  }

  /**
   * 初始化限流器
   * @returns {Promise<Object>} - 限流器实例
   */
  async init() {
    // 先初始化SQLite作为降级备份
    if (this.config.sqlite.enabled) {
      this.fallbackLimiter = new LuxurySQLiteRateLimiter({
        dbPath: this.config.sqlite.dbPath,
        capacity: this.config.capacity,
        refillRate: this.config.refillRate
      });
      await this.fallbackLimiter.init();
      console.log('[RateLimiterFactory] SQLite fallback initialized');
    }

    // 尝试连接Redis
    if (this.config.redis.enabled) {
      const redisLimiter = new RedisRateLimiter({
        host: this.config.redis.host,
        port: this.config.redis.port,
        password: this.config.redis.password,
        db: this.config.redis.db,
        capacity: this.config.capacity,
        refillRate: this.config.refillRate
      });

      const connected = await redisLimiter.init();
      
      if (connected) {
        this.primaryLimiter = redisLimiter;
        this.currentMode = 'redis';
        console.log('[RateLimiterFactory] Using Redis rate limiter');
        return this._createProxy();
      } else {
        console.log('[RateLimiterFactory] Redis unavailable, using SQLite');
        await redisLimiter.close(); // 清理失败的连接
      }
    }

    // 降级到SQLite
    if (this.fallbackLimiter) {
      this.primaryLimiter = this.fallbackLimiter;
      this.fallbackLimiter = null; // 已经是主限流器
      this.currentMode = 'sqlite';
      console.log('[RateLimiterFactory] Using SQLite rate limiter');
      return this._createProxy();
    }

    throw new Error('No rate limiter available');
  }

  /**
   * 创建代理对象
   * 包装主限流器，添加故障检测和自动降级
   */
  _createProxy() {
    const factory = this;
    
    return {
      async checkLimit(ip, tokens = 1) {
        try {
          // 尝试主限流器
          const result = await factory.primaryLimiter.checkLimit(ip, tokens);
          
          // 如果Redis故障（后续检测），尝试降级
          if (factory.currentMode === 'redis' && factory.fallbackLimiter) {
            // 简单健康检查：如果Redis连续失败，切换到SQLite
            // 这里简化处理，实际应该有健康检查机制
          }
          
          return result;
        } catch (err) {
          // 主限流器失败，尝试降级
          if (factory.fallbackLimiter && factory.currentMode !== 'sqlite') {
            console.warn('[RateLimiter] Primary failed, falling back to SQLite:', err.message);
            factory.currentMode = 'sqlite';
            factory.primaryLimiter = factory.fallbackLimiter;
            factory.fallbackLimiter = null;
            
            // 使用降级后的限流器
            return factory.primaryLimiter.checkLimit(ip, tokens);
          }
          
          throw err;
        }
      },

      async getStats(ip) {
        try {
          return await factory.primaryLimiter.getStats(ip);
        } catch (err) {
          if (factory.fallbackLimiter) {
            return factory.fallbackLimiter.getStats(ip);
          }
          throw err;
        }
      },

      async close() {
        await factory.primaryLimiter?.close();
        await factory.fallbackLimiter?.close();
      },

      getMode() {
        return factory.currentMode;
      },

      isDistributed() {
        return factory.currentMode === 'redis';
      }
    };
  }

  /**
   * 获取当前模式
   */
  getMode() {
    return this.currentMode;
  }

  /**
   * 获取统计信息
   */
  getStats() {
    return {
      mode: this.currentMode,
      redisConfigured: this.config.redis.enabled,
      sqliteConfigured: this.config.sqlite.enabled,
      hasFallback: this.fallbackLimiter !== null
    };
  }
}

module.exports = { RateLimiterFactory };
