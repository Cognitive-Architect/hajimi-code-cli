/**
 * ENOSPC (磁盘空间不足) 错误处理器
 * 优雅降级策略
 */

const EventEmitter = require('events');

class ENOSPCError extends Error {
  constructor(message, path) {
    super(message);
    this.name = 'ENOSPCError';
    this.code = 'ENOSPC';
    this.path = path;
  }
}

class ENOSPCHandler extends EventEmitter {
  constructor(options = {}) {
    super();
    
    this.checkInterval = options.checkInterval || 30000; // 30秒检查一次
    this.emergencyThreshold = options.emergencyThreshold || 100; // 100MB紧急阈值
    this.warningThreshold = options.warningThreshold || 500; // 500MB警告阈值
    
    this.state = {
      isEmergencyMode: false,
      lastCheck: null,
      freeSpace: null,
      diskFullCount: 0
    };
    
    this.monitorTimer = null;
  }

  /**
   * 检查是否为ENOSPC错误
   */
  static isENOSPCError(error) {
    return error.code === 'ENOSPC' || 
           (error.message && error.message.includes('no space left')) ||
           (error.message && error.message.includes('ENOSPC'));
  }

  /**
   * 包装函数，自动处理ENOSPC
   */
  async withFallback(fn, fallbackFn, context = {}) {
    try {
      return await fn();
    } catch (err) {
      if (ENOSPCHandler.isENOSPCError(err)) {
        this.state.diskFullCount++;
        this.emit('enospc', { error: err, context });
        
        console.warn(`⚠️ ENOSPC detected, executing fallback for: ${context.operation || 'unknown'}`);
        
        // 进入紧急模式
        await this.enterEmergencyMode();
        
        // 执行降级方案
        if (fallbackFn) {
          return await fallbackFn();
        }
        
        throw new ENOSPCError('Disk full and no fallback provided', context.path);
      }
      throw err;
    }
  }

  /**
   * 进入紧急模式
   */
  async enterEmergencyMode() {
    if (this.state.isEmergencyMode) return;
    
    console.error('🚨 ENTERING EMERGENCY MODE - Disk Full');
    this.state.isEmergencyMode = true;
    this.emit('emergency:enter', { timestamp: Date.now() });
  }

  /**
   * 退出紧急模式
   */
  async exitEmergencyMode() {
    if (!this.state.isEmergencyMode) return;
    
    console.log('✅ EXITING EMERGENCY MODE - Disk space recovered');
    this.state.isEmergencyMode = false;
    this.emit('emergency:exit', { timestamp: Date.now() });
  }

  /**
   * 检查是否在紧急模式
   */
  isEmergencyMode() {
    return this.state.isEmergencyMode;
  }

  /**
   * 获取可用磁盘空间（简化版）
   */
  async getFreeSpace(path) {
    // 在Termux中无法直接使用df命令获取准确值
    // 使用估算方法
    try {
      const fs = require('fs').promises;
      const stat = await fs.statfs(path || '.');
      
      // statfs返回的块大小和可用块数
      const freeBytes = stat.bavail * stat.bsize;
      const freeMB = freeBytes / 1024 / 1024;
      
      return {
        freeBytes,
        freeMB: Math.floor(freeMB),
        totalBytes: stat.blocks * stat.bsize,
        totalMB: Math.floor((stat.blocks * stat.bsize) / 1024 / 1024)
      };
    } catch (err) {
      // 无法获取时返回一个安全估计值
      return { freeBytes: Infinity, freeMB: Infinity, estimated: true };
    }
  }

  /**
   * 检查磁盘空间
   */
  async checkDiskSpace(path) {
    const space = await this.getFreeSpace(path);
    this.state.lastCheck = Date.now();
    this.state.freeSpace = space.freeMB;
    
    if (space.freeMB < this.emergencyThreshold) {
      if (!this.state.isEmergencyMode) {
        await this.enterEmergencyMode();
      }
      this.emit('space:critical', space);
    } else if (space.freeMB < this.warningThreshold) {
      this.emit('space:warning', space);
    } else {
      if (this.state.isEmergencyMode) {
        await this.exitEmergencyMode();
      }
      this.emit('space:ok', space);
    }
    
    return space;
  }

  /**
   * 启动监控
   */
  startMonitoring(path) {
    if (this.monitorTimer) return;
    
    console.log(`🔍 Starting disk space monitoring (interval: ${this.checkInterval}ms)`);
    
    // 立即检查一次
    this.checkDiskSpace(path);
    
    // 定时检查
    this.monitorTimer = setInterval(() => {
      this.checkDiskSpace(path);
    }, this.checkInterval);
  }

  /**
   * 停止监控
   */
  stopMonitoring() {
    if (this.monitorTimer) {
      clearInterval(this.monitorTimer);
      this.monitorTimer = null;
      console.log('👋 Disk space monitoring stopped');
    }
  }

  /**
   * 获取状态
   */
  getState() {
    return { ...this.state };
  }

  /**
   * 创建写入保护包装
   */
  createWriteProtector(writerFn) {
    return async (data, options = {}) => {
      // 如果在紧急模式，直接拒绝写入
      if (this.state.isEmergencyMode && !options.force) {
        throw new ENOSPCError('Write rejected: system in emergency mode', options.path);
      }
      
      // 预估写入大小
      const estimatedSize = Buffer.isBuffer(data) ? data.length : JSON.stringify(data).length;
      const estimatedMB = estimatedSize / 1024 / 1024;
      
      // 如果预估大小超过剩余空间，提前拒绝
      if (this.state.freeSpace !== null && estimatedMB > this.state.freeSpace * 0.8) {
        this.emit('write:rejected', { estimatedMB, freeMB: this.state.freeSpace });
        throw new ENOSPCError(`Write rejected: insufficient space (need ${estimatedMB.toFixed(2)}MB, have ${this.state.freeSpace}MB)`, options.path);
      }
      
      return await writerFn(data, options);
    };
  }
}

module.exports = { 
  ENOSPCHandler, 
  ENOSPCError,
  isENOSPCError: ENOSPCHandler.isENOSPCError 
};
