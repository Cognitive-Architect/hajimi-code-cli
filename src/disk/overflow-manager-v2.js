/**
 * 增强版磁盘溢出管理器 (v2)
 * 增加磁盘满检测、优雅降级、紧急模式
 */

const { OverflowManager } = require('./overflow-manager');
const { ENOSPCHandler, ENOSPCError } = require('./enospc-handler');
const EventEmitter = require('events');

class OverflowManagerV2 extends EventEmitter {
  constructor(options = {}) {
    super();
    
    this.basePath = options.basePath || './data/overflow';
    
    // 嵌入基础OverflowManager
    this.baseManager = new OverflowManager(options);
    
    // ENOSPC处理器
    this.enospcHandler = new ENOSPCHandler({
      checkInterval: options.diskCheckInterval || 30000,
      emergencyThreshold: options.emergencyThreshold || 100,
      warningThreshold: options.warningThreshold || 500
    });
    
    // 状态
    this.state = {
      mode: 'normal', // normal, warning, emergency
      diskWritesPaused: false,
      queuePaused: false
    };
    
    // 写入队列
    this.writeQueue = [];
    this.maxQueueSize = options.maxQueueSize || 1000;
    
    // 设置事件监听
    this._setupEventListeners();
  }

  /**
   * 设置事件监听
   */
  _setupEventListeners() {
    // ENOSPC事件
    this.enospcHandler.on('enospc', (data) => {
      this.emit('enospc', data);
      this._enterEmergencyMode();
    });
    
    this.enospcHandler.on('emergency:enter', () => {
      this.state.mode = 'emergency';
      this.emit('mode:change', { mode: 'emergency' });
    });
    
    this.enospcHandler.on('emergency:exit', () => {
      this.state.mode = 'normal';
      this.state.diskWritesPaused = false;
      this.emit('mode:change', { mode: 'normal' });
      this._resumeWrites();
    });
    
    this.enospcHandler.on('space:warning', (space) => {
      if (this.state.mode !== 'emergency') {
        this.state.mode = 'warning';
        this.emit('space:warning', space);
      }
    });
  }

  /**
   * 初始化
   */
  async init() {
    await this.baseManager.init();
    this.enospcHandler.startMonitoring(this.basePath);
    
    console.log(`✅ OverflowManagerV2 initialized (mode: ${this.state.mode})`);
  }

  /**
   * 进入紧急模式
   */
  _enterEmergencyMode() {
    console.error('🚨 EMERGENCY MODE ACTIVATED');
    
    this.state.mode = 'emergency';
    this.state.diskWritesPaused = true;
    
    // 暂停写入队列
    this._pauseQueue();
    
    this.emit('emergency', {
      timestamp: Date.now(),
      queueSize: this.writeQueue.length
    });
  }

  /**
   * 暂停队列
   */
  _pauseQueue() {
    if (!this.state.queuePaused) {
      this.state.queuePaused = true;
      console.log('⏸️ Write queue paused');
    }
  }

  /**
   * 恢复写入
   */
  async _resumeWrites() {
    console.log('▶️ Resuming writes...');
    
    this.state.diskWritesPaused = false;
    this.state.queuePaused = false;
    
    // 处理队列中的积压
    await this._flushQueue();
  }

  /**
   * 刷新队列
   */
  async _flushQueue() {
    while (this.writeQueue.length > 0 && !this.state.diskWritesPaused) {
      const item = this.writeQueue.shift();
      try {
        await this._doWrite(item);
      } catch (err) {
        console.error('Failed to flush queue item:', err);
        // 如果再次失败，放回队列
        if (ENOSPCHandler.isENOSPCError(err)) {
          this.writeQueue.unshift(item);
          break;
        }
      }
    }
    
    if (this.writeQueue.length > 0) {
      console.log(`⏳ Queue still has ${this.writeQueue.length} items pending`);
    }
  }

  /**
   * 执行实际写入
   */
  async _doWrite(item) {
    return await this.enospcHandler.withFallback(
      async () => {
        return await this.baseManager.store.write(
          item.fileId,
          item.offset,
          item.data
        );
      },
      async () => {
        // 降级方案：只保留在内存中
        console.warn('⚠️ Disk write failed, keeping in memory only');
        this.emit('write:memory_only', item);
        return { memoryOnly: true, size: item.data.length };
      },
      { operation: 'write', path: `${this.basePath}/${item.fileId}` }
    );
  }

  /**
   * 添加数据（增强版）
   */
  async add(id, data) {
    // 如果在紧急模式，直接内存存储
    if (this.state.mode === 'emergency') {
      this.baseManager.touch(id);
      this.baseManager.state.totalCount++;
      this.baseManager.state.inMemoryCount++;
      
      this.emit('add:memory_only', { id, mode: 'emergency' });
      return { id, inMemory: true, emergency: true };
    }
    
    // 否则使用基础管理器
    return await this.baseManager.add(id, data);
  }

  /**
   * 溢出到磁盘（带保护）
   */
  async overflowToDisk(id, serializer) {
    // 如果磁盘写入暂停，直接返回
    if (this.state.diskWritesPaused) {
      console.log(`⏸️ Disk writes paused, skipping overflow for ${id}`);
      return { skipped: true, reason: 'disk_writes_paused' };
    }
    
    const data = await serializer(id);
    if (!data) return null;
    
    const fileId = `overflow-${Math.floor(id / 1000)}`;
    const offset = await this.baseManager.store.getSize(fileId);
    
    const item = { fileId, offset, data, id, timestamp: Date.now() };
    
    // 如果队列已暂停，直接尝试写入
    if (this.state.queuePaused) {
      try {
        return await this._doWrite(item);
      } catch (err) {
        if (ENOSPCHandler.isENOSPCError(err)) {
          // 加入队列稍后重试
          this._enqueue(item);
          return { queued: true, id };
        }
        throw err;
      }
    }
    
    // 正常写入
    return await this._doWrite(item);
  }

  /**
   * 加入队列
   */
  _enqueue(item) {
    if (this.writeQueue.length >= this.maxQueueSize) {
      // 队列已满，移除最旧的
      const dropped = this.writeQueue.shift();
      this.emit('queue:drop', dropped);
    }
    
    this.writeQueue.push(item);
    this.emit('queue:add', { queueSize: this.writeQueue.length });
  }

  /**
   * 获取状态
   */
  getState() {
    return {
      ...this.state,
      queueSize: this.writeQueue.length,
      enospc: this.enospcHandler.getState(),
      base: this.baseManager.getStats()
    };
  }

  /**
   * 关闭
   */
  async close() {
    this.enospcHandler.stopMonitoring();
    
    // 尝试刷新队列
    if (this.writeQueue.length > 0) {
      console.log(`Flushing ${this.writeQueue.length} queued items...`);
      await this._flushQueue();
    }
    
    await this.baseManager.close();
  }

  /**
   * 手动触发磁盘检查
   */
  async checkDiskSpace() {
    return await this.enospcHandler.checkDiskSpace(this.basePath);
  }

  /**
   * 强制进入紧急模式（测试用）
   */
  async forceEmergencyMode() {
    await this.enospcHandler.enterEmergencyMode();
  }

  /**
   * 强制退出紧急模式（测试用）
   */
  async forceExitEmergencyMode() {
    await this.enospcHandler.exitEmergencyMode();
  }
}

module.exports = { OverflowManagerV2 };
