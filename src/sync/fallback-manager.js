/**
 * WebRTC 降级管理器
 * 
 * 功能：
 * - 状态机管理：IDLE → DISCOVERING → CONNECTING → (CONNECTED | ICE_FAILED | TIMEOUT) → FILE_EXPORT
 * - ICE失败自动降级到文件导出
 * - 事件通知机制
 * 
 * 输入基线：src/sync/fallback-strategy.md
 */

const EventEmitter = require('events');

// 状态常量
const STATES = {
  IDLE: 'IDLE',
  DISCOVERING: 'DISCOVERING',
  CONNECTING: 'CONNECTING',
  CONNECTED: 'CONNECTED',
  ICE_FAILED: 'ICE_FAILED',
  TIMEOUT: 'TIMEOUT',
  FILE_EXPORT: 'FILE_EXPORT',
  IMPORTING: 'IMPORTING'
};

// 默认配置
const DEFAULT_CONFIG = {
  // ICE 超时配置
  gatheringTimeout: 5000,      // 候选收集超时 (5s)
  connectionTimeout: 10000,    // 连接建立超时 (10s)
  failedStateDelay: 2000,      // failed 状态确认延迟 (2s)
  
  // 降级触发
  enableAutoFallback: true,    // 自动降级开关
  webrtcTimeout: 10000,        // WebRTC总超时
  
  // 重连配置
  maxReconnectAttempts: 3,
  reconnectDelay: 3000
};

/**
 * WebRTC降级管理器类
 */
class SyncFallbackManager extends EventEmitter {
  constructor(options = {}) {
    super();
    
    this.config = {
      ...DEFAULT_CONFIG,
      ...options
    };
    
    // 状态
    this.state = STATES.IDLE;
    this.currentStrategy = null;
    this.strategies = new Map();
    
    // 运行时数据
    this.iceStartTime = null;
    this.fallbackTimer = null;
    this.reconnectAttempts = 0;
    this.lastError = null;
    
    // 注册默认策略
    this._registerDefaultStrategies();
  }
  
  /**
   * 注册默认策略（模拟实现）
   */
  _registerDefaultStrategies() {
    // WebRTC策略（模拟）
    this.registerStrategy('webrtc', {
      name: 'webrtc',
      connect: async (peerId) => {
        // 模拟连接建立
        return new Promise((resolve, reject) => {
          // 实际实现中这里会创建RTCPeerConnection
          setTimeout(() => {
            if (Math.random() > 0.5) {
              resolve({ peerId, connection: 'mock-rtc-connection' });
            } else {
              reject(new Error('ICE_FAILED'));
            }
          }, 100);
        });
      },
      sync: async (connection, manifest) => {
        // 模拟同步
        return { success: true, transferred: manifest.size || 0 };
      }
    });
    
    // 文件导出策略（模拟）
    this.registerStrategy('file_export', {
      name: 'file_export',
      export: async (manifest) => {
        // 模拟导出
        const filePath = `/tmp/sync_${Date.now()}.hajimi`;
        return { 
          filePath, 
          size: manifest.size || 0,
          timestamp: new Date().toISOString()
        };
      },
      import: async (filePath) => {
        // 模拟导入
        return { success: true, imported: filePath };
      }
    });
  }
  
  /**
   * 注册策略
   */
  registerStrategy(name, strategy) {
    this.strategies.set(name, strategy);
  }
  
  /**
   * 获取当前状态
   */
  getState() {
    return this.state;
  }
  
  /**
   * 执行同步（带自动降级）
   */
  async sync(peerId, manifest) {
    this.iceStartTime = Date.now();
    this.lastError = null;
    
    // 1. 尝试 WebRTC
    this.state = STATES.DISCOVERING;
    this.currentStrategy = 'webrtc';
    
    this.emit('sync:start', { strategy: 'webrtc', peerId, timestamp: this.iceStartTime });
    
    try {
      const result = await this._tryWebRTC(peerId, manifest);
      if (result.success) {
        this.state = STATES.CONNECTED;
        this.emit('sync:complete', { 
          strategy: 'webrtc', 
          result,
          duration: Date.now() - this.iceStartTime
        });
        return result;
      }
    } catch (error) {
      this.lastError = { code: error.message, details: error.stack };
      this.emit('sync:webrtc:failed', { error: error.message, peerId });
    }
    
    // 2. 自动降级到文件导出
    if (this.config.enableAutoFallback) {
      return this._triggerFallback('ICE_FAILED');
    }
    
    throw new Error('WebRTC failed and fallback disabled');
  }
  
  /**
   * 尝试WebRTC连接
   */
  async _tryWebRTC(peerId, manifest) {
    const strategy = this.strategies.get('webrtc');
    
    return new Promise((resolve, reject) => {
      // 设置连接超时
      const timeout = setTimeout(() => {
        this.state = STATES.TIMEOUT;
        reject(new Error('ICE_TIMEOUT'));
      }, this.config.webrtcTimeout);
      
      // 尝试连接
      strategy.connect(peerId)
        .then(connection => {
          clearTimeout(timeout);
          this.state = STATES.CONNECTED;
          return strategy.sync(connection, manifest);
        })
        .then(resolve)
        .catch(err => {
          clearTimeout(timeout);
          this.state = STATES.ICE_FAILED;
          reject(err);
        });
    });
  }
  
  /**
   * 触发降级
   */
  async _triggerFallback(reason) {
    clearTimeout(this.fallbackTimer);
    
    const fallbackInfo = {
      from: 'webrtc',
      to: 'file_export',
      reason,
      timestamp: new Date().toISOString(),
      duration: Date.now() - this.iceStartTime
    };
    
    this.emit('sync:fallback', fallbackInfo);
    
    this.state = STATES.FILE_EXPORT;
    this.currentStrategy = 'file_export';
    
    const strategy = this.strategies.get('file_export');
    
    try {
      const result = await strategy.export({ size: 0 }); // 实际应从manifest获取
      
      this.emit('sync:export:ready', {
        filePath: result.filePath,
        size: result.size,
        instructions: '通过邮件/IM/蓝牙传输此文件到目标设备',
        duration: fallbackInfo.duration
      });
      
      return { 
        success: true, 
        strategy: 'file_export',
        ...result
      };
    } catch (error) {
      this.emit('sync:error', { phase: 'file_export', error: error.message });
      throw error;
    }
  }
  
  /**
   * 手动切换到文件导出
   */
  async forceFileExport(manifest) {
    return this._triggerFallback('MANUAL');
  }
  
  /**
   * 重置状态
   */
  reset() {
    this.state = STATES.IDLE;
    this.currentStrategy = null;
    this.iceStartTime = null;
    this.reconnectAttempts = 0;
    this.lastError = null;
    clearTimeout(this.fallbackTimer);
  }
  
  /**
   * 销毁实例
   */
  destroy() {
    this.reset();
    this.removeAllListeners();
  }
}

// 导出
module.exports = {
  SyncFallbackManager,
  STATES
};
