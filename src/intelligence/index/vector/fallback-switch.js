/**
 * 降级决策模块 - Fallback Switch
 * 
 * 职责：
 * 1. 监控 HNSW 健康状态（内存、索引完整性、性能）
 * 2. 自动触发降级到 LSH
 * 3. 熔断机制：连续失败N次后强制降级
 * 4. 恢复探测：定期尝试恢复 HNSW
 */

// 熔断器状态
const CircuitState = {
  CLOSED: 'CLOSED',      // 正常，使用 HNSW
  OPEN: 'OPEN',          // 熔断，使用 LSH
  HALF_OPEN: 'HALF_OPEN' // 半开，试探性恢复
};

// 默认配置
const DEFAULT_FALLBACK_CONFIG = {
  // 内存阈值
  memoryThresholdMB: 400,       // 超过此值触发降级
  
  // 熔断配置
  failureThreshold: 5,          // 连续失败N次后熔断
  recoveryTimeout: 30000,       // 熔断后30秒尝试恢复
  halfOpenMaxCalls: 3,          // 半开状态最多允许3次试探
  
  // 性能阈值
  latencyThresholdMs: 100,      // 单次查询超过100ms算失败
  accuracyThreshold: 0.95,      // 准确率低于95%算失败
  
  // 探测配置
  probeInterval: 60000,         // 每分钟探测一次
  
  // 日志
  enableLogging: true
};

/**
 * 熔断器类
 */
class CircuitBreaker {
  constructor(config = {}) {
    this.config = { ...DEFAULT_FALLBACK_CONFIG, ...config };
    this.state = CircuitState.CLOSED;
    this.failureCount = 0;
    this.successCount = 0;
    this.lastFailureTime = 0;
    this.halfOpenCalls = 0;
    this.stats = {
      totalCalls: 0,
      hnswCalls: 0,
      lshCalls: 0,
      failures: 0,
      fallbackTriggers: 0
    };
  }
  
  /**
   * 检查是否允许使用 HNSW
   */
  canUseHNSW() {
    const memUsage = this._getMemoryUsageMB();
    
    // P0: 内存超过硬限制，直接拒绝
    if (memUsage > this.config.memoryThresholdMB) {
      this._log(`Memory exceeded: ${memUsage.toFixed(1)}MB > ${this.config.memoryThresholdMB}MB`);
      return false;
    }
    
    switch (this.state) {
      case CircuitState.CLOSED:
        return true;
        
      case CircuitState.OPEN:
        // 检查是否应该进入半开状态
        if (Date.now() - this.lastFailureTime > this.config.recoveryTimeout) {
          this._transitionTo(CircuitState.HALF_OPEN);
          this.halfOpenCalls = 0;
          return true;
        }
        return false;
        
      case CircuitState.HALF_OPEN:
        // 半开状态限制试探次数
        if (this.halfOpenCalls < this.config.halfOpenMaxCalls) {
          this.halfOpenCalls++;
          return true;
        }
        return false;
        
      default:
        return false;
    }
  }
  
  /**
   * 记录成功
   */
  recordSuccess(latency) {
    this.stats.totalCalls++;
    this.stats.hnswCalls++;
    
    if (this.state === CircuitState.HALF_OPEN) {
      this.successCount++;
      // 连续成功则关闭熔断
      if (this.successCount >= 2) {
        this._transitionTo(CircuitState.CLOSED);
        this.failureCount = 0;
        this.successCount = 0;
      }
    } else {
      this.failureCount = 0;
    }
  }
  
  /**
   * 记录失败
   * @param {string} reason - 'memory' | 'latency' | 'error' | 'accuracy'
   */
  recordFailure(reason) {
    this.stats.totalCalls++;
    this.stats.failures++;
    this.failureCount++;
    this.lastFailureTime = Date.now();
    
    this._log(`HNSW failure: ${reason}, count=${this.failureCount}`);
    
    // 半开状态失败，重新熔断
    if (this.state === CircuitState.HALF_OPEN) {
      this._transitionTo(CircuitState.OPEN);
      return;
    }
    
    // 达到阈值，触发熔断
    if (this.failureCount >= this.config.failureThreshold) {
      this._transitionTo(CircuitState.OPEN);
      this.stats.fallbackTriggers++;
      this._log(`Circuit OPENED due to ${this.failureCount} failures`);
    }
  }
  
  /**
   * 记录 LSH 调用
   */
  recordLSHCall() {
    this.stats.totalCalls++;
    this.stats.lshCalls++;
  }
  
  /**
   * 状态转移
   */
  _transitionTo(newState) {
    const oldState = this.state;
    this.state = newState;
    this._log(`State: ${oldState} -> ${newState}`);
    
    if (newState === CircuitState.CLOSED) {
      this.halfOpenCalls = 0;
    }
  }
  
  /**
   * 获取内存使用（MB）
   */
  _getMemoryUsageMB() {
    if (typeof process !== 'undefined' && process.memoryUsage) {
      return process.memoryUsage().rss / 1024 / 1024;
    }
    return 0;
  }
  
  /**
   * 日志输出
   */
  _log(message) {
    if (this.config.enableLogging) {
      console.log(`[Fallback] ${new Date().toISOString()} ${message}`);
    }
  }
  
  /**
   * 获取当前状态
   */
  getStatus() {
    return {
      state: this.state,
      failureCount: this.failureCount,
      successCount: this.successCount,
      memoryUsageMB: this._getMemoryUsageMB(),
      stats: { ...this.stats }
    };
  }
  
  /**
   * 手动重置熔断器
   */
  reset() {
    this.state = CircuitState.CLOSED;
    this.failureCount = 0;
    this.successCount = 0;
    this.halfOpenCalls = 0;
    this._log('Manual reset to CLOSED');
  }
  
  /**
   * 强制熔断（用于紧急降级）
   */
  forceOpen() {
    this._transitionTo(CircuitState.OPEN);
    this.lastFailureTime = Date.now();
  }
}

/**
 * 健康检查器
 */
class HealthChecker {
  constructor(config = {}) {
    this.config = {
      maxLatencyMs: 100,
      minAccuracy: 0.95,
      ...config
    };
  }
  
  /**
   * 检查 HNSW 健康状态
   * @param {Object} hnswIndex - HNSW索引实例
   * @returns {Object} - { healthy: boolean, issues: string[] }
   */
  check(hnswIndex) {
    const issues = [];
    
    if (!hnswIndex) {
      return { healthy: false, issues: ['HNSW index not initialized'] };
    }
    
    const stats = hnswIndex.getStats();
    
    // 检查内存
    const memUsage = (process.memoryUsage?.().rss || 0) / 1024 / 1024;
    if (memUsage > 400) {
      issues.push(`Memory usage too high: ${memUsage.toFixed(1)}MB`);
    }
    
    // 检查索引状态
    if (stats.activeCount === 0) {
      issues.push('No active vectors in index');
    }
    
    return {
      healthy: issues.length === 0,
      issues
    };
  }
  
  /**
   * 验证搜索结果质量
   * @param {Array} results - HNSW搜索结果
   * @param {Array} groundTruth - 暴力搜索结果
   * @returns {number} - 召回率 [0, 1]
   */
  validateAccuracy(results, groundTruth) {
    if (!results.length || !groundTruth.length) return 0;
    
    const resultIds = new Set(results.map(r => r.id));
    const truthIds = new Set(groundTruth.map(t => t.id));
    
    let matchCount = 0;
    for (const id of resultIds) {
      if (truthIds.has(id)) matchCount++;
    }
    
    return matchCount / Math.min(results.length, groundTruth.length);
  }
}

module.exports = {
  CircuitBreaker,
  HealthChecker,
  CircuitState,
  DEFAULT_FALLBACK_CONFIG
};
