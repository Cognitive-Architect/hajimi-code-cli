/**
 * Worker Thread 池管理器
 * 管理多个HNSW Worker，支持负载均衡和故障恢复
 */

const { Worker } = require('worker_threads');
const path = require('path');
const EventEmitter = require('events');

class WorkerPool extends EventEmitter {
  constructor(options = {}) {
    super();
    
    this.poolSize = options.poolSize || Math.max(1, require('os').cpus().length - 1);
    // 使用相对于cwd的路径
    this.workerScript = options.workerScript || './src/worker/hnsw-worker.js';
    this.taskTimeout = options.taskTimeout || 300000; // 5分钟
    this.maxRestarts = options.maxRestarts || 3;
    
    this.workers = []; // { worker, id, isBusy, restartCount }
    this.taskQueue = [];
    this.isShutdown = false;
    
    // 统计
    this.stats = {
      tasksSubmitted: 0,
      tasksCompleted: 0,
      tasksFailed: 0,
      workerRestarts: 0
    };
  }

  /**
   * 初始化Worker池
   */
  async init() {
    console.log(`🚀 Initializing Worker Pool (size=${this.poolSize})...`);
    
    for (let i = 0; i < this.poolSize; i++) {
      await this._createWorker(i);
    }
    
    console.log(`✅ Worker Pool ready with ${this.workers.length} workers`);
  }

  /**
   * 创建单个Worker
   */
  async _createWorker(id) {
    return new Promise((resolve, reject) => {
      try {
        const worker = new Worker(this.workerScript, {
          workerData: {
            memoryLimitMB: 300,
            workerId: id
          }
        });

        const workerInfo = {
          worker,
          id,
          isBusy: false,
          restartCount: 0,
          currentTask: null,
          ready: false
        };

        worker.on('message', (message) => {
          this._handleWorkerMessage(id, message);
        });

        worker.on('error', (err) => {
          console.error(`Worker ${id} error:`, err);
          this._handleWorkerError(id, err);
        });

        worker.on('exit', (code) => {
          if (code !== 0) {
            console.error(`Worker ${id} exited with code ${code}`);
            this._handleWorkerExit(id, code);
          }
        });

        // 等待Worker就绪
        const readyTimeout = setTimeout(() => {
          if (!workerInfo.ready) {
            reject(new Error(`Worker ${id} startup timeout`));
          }
        }, 10000);
        
        const checkReady = () => {
          if (workerInfo.ready) {
            clearTimeout(readyTimeout);
            this.workers[id] = workerInfo;
            resolve(workerInfo);
          } else {
            setTimeout(checkReady, 50);
          }
        };
        
        checkReady();

      } catch (err) {
        reject(err);
      }
    });
  }

  /**
   * 处理Worker消息
   */
  _handleWorkerMessage(workerId, message) {
    const workerInfo = this.workers[workerId];
    if (!workerInfo) return;

    switch (message.type) {
      case 'ready':
        workerInfo.ready = true;
        this.emit('worker:ready', { workerId, threadId: message.threadId });
        break;
        
      case 'build_complete':
      case 'search_complete':
        this.stats.tasksCompleted++;
        workerInfo.isBusy = false;
        
        if (workerInfo.currentTask) {
          workerInfo.currentTask.resolve(message);
          workerInfo.currentTask = null;
        }
        
        this._processQueue();
        break;
        
      case 'build_error':
      case 'search_error':
        this.stats.tasksFailed++;
        workerInfo.isBusy = false;
        
        if (workerInfo.currentTask) {
          workerInfo.currentTask.reject(new Error(message.error));
          workerInfo.currentTask = null;
        }
        
        this._processQueue();
        break;
        
      case 'progress':
        this.emit('progress', { workerId, ...message });
        break;
        
      case 'memory_warning':
        console.warn(`Worker ${workerId} memory warning: ${message.rssMB}MB`);
        this.emit('memory:warning', { workerId, ...message });
        break;
        
      case 'stats':
        this.emit('stats', { workerId, ...message });
        break;
    }
  }

  /**
   * 处理Worker错误
   */
  _handleWorkerError(workerId, err) {
    const workerInfo = this.workers[workerId];
    if (!workerInfo) return;

    // 拒绝当前任务
    if (workerInfo.currentTask) {
      workerInfo.currentTask.reject(err);
      workerInfo.currentTask = null;
    }

    workerInfo.isBusy = false;
    this._restartWorker(workerId);
  }

  /**
   * 处理Worker退出
   */
  _handleWorkerExit(workerId, code) {
    const workerInfo = this.workers[workerId];
    if (!workerInfo) return;

    // 拒绝当前任务
    if (workerInfo.currentTask) {
      workerInfo.currentTask.reject(new Error(`Worker exited with code ${code}`));
      workerInfo.currentTask = null;
    }

    this._restartWorker(workerId);
  }

  /**
   * 重启Worker
   */
  async _restartWorker(id) {
    const workerInfo = this.workers[id];
    if (!workerInfo) return;

    if (workerInfo.restartCount >= this.maxRestarts) {
      console.error(`Worker ${id} exceeded max restarts (${this.maxRestarts})`);
      this.emit('worker:failed', { workerId: id });
      return;
    }

    console.log(`🔄 Restarting Worker ${id} (attempt ${workerInfo.restartCount + 1})...`);
    
    try {
      workerInfo.restartCount++;
      this.stats.workerRestarts++;
      
      // 终止旧Worker
      await workerInfo.worker.terminate();
      
      // 创建新Worker
      await this._createWorker(id);
      
      console.log(`✅ Worker ${id} restarted successfully`);
      this.emit('worker:restarted', { workerId: id });
      
    } catch (err) {
      console.error(`Failed to restart Worker ${id}:`, err);
      this.emit('worker:restart_failed', { workerId: id, error: err });
    }
  }

  /**
   * 提交任务
   */
  async execute(taskType, data, options = {}) {
    if (this.isShutdown) {
      throw new Error('Worker pool is shutdown');
    }

    this.stats.tasksSubmitted++;

    return new Promise((resolve, reject) => {
      const task = {
        type: taskType,
        data,
        options,
        resolve,
        reject,
        timestamp: Date.now()
      };

      // 尝试立即分配
      const worker = this._getAvailableWorker();
      if (worker) {
        this._assignTask(worker, task);
      } else {
        // 加入队列
        this.taskQueue.push(task);
        
        // 设置超时
        if (this.taskTimeout) {
          setTimeout(() => {
            const index = this.taskQueue.indexOf(task);
            if (index !== -1) {
              this.taskQueue.splice(index, 1);
              reject(new Error('Task timeout'));
            }
          }, this.taskTimeout);
        }
      }
    });
  }

  /**
   * 获取可用Worker
   */
  _getAvailableWorker() {
    return this.workers.find(w => w.ready && !w.isBusy);
  }

  /**
   * 分配任务给Worker
   */
  _assignTask(workerInfo, task) {
    workerInfo.isBusy = true;
    workerInfo.currentTask = task;
    
    workerInfo.worker.postMessage({
      type: task.type,
      data: task.data
    });
  }

  /**
   * 处理任务队列
   */
  _processQueue() {
    while (this.taskQueue.length > 0) {
      const worker = this._getAvailableWorker();
      if (!worker) break;
      
      const task = this.taskQueue.shift();
      this._assignTask(worker, task);
    }
  }

  /**
   * 构建索引（便捷方法）
   */
  async buildIndex(vectors, options = {}) {
    return this.execute('build', { vectors, options });
  }

  /**
   * 搜索（便捷方法）
   */
  async search(indexData, query, k = 10) {
    return this.execute('search', { indexData, query, k });
  }

  /**
   * 获取Worker统计
   */
  async getWorkerStats(workerId) {
    const workerInfo = this.workers[workerId];
    if (!workerInfo) return null;

    return new Promise((resolve) => {
      const timeout = setTimeout(() => resolve(null), 1000);
      
      const onMessage = (message) => {
        if (message.type === 'stats' && message.threadId === workerInfo.id) {
          clearTimeout(timeout);
          workerInfo.worker.off('message', onMessage);
          resolve(message);
        }
      };
      
      workerInfo.worker.on('message', onMessage);
      workerInfo.worker.postMessage({ type: 'stats' });
    });
  }

  /**
   * 获取所有Worker统计
   */
  async getAllStats() {
    const stats = await Promise.all(
      this.workers.map((_, id) => this.getWorkerStats(id))
    );
    return stats.filter(Boolean);
  }

  /**
   * 获取池统计
   */
  getStats() {
    return {
      ...this.stats,
      poolSize: this.poolSize,
      queueLength: this.taskQueue.length,
      activeWorkers: this.workers.filter(w => w.isBusy).length,
      readyWorkers: this.workers.filter(w => w.ready).length
    };
  }

  /**
   * 关闭Worker池
   */
  async shutdown() {
    console.log('👋 Shutting down Worker Pool...');
    this.isShutdown = true;

    // 清空队列
    for (const task of this.taskQueue) {
      task.reject(new Error('Worker pool shutdown'));
    }
    this.taskQueue = [];

    // 终止所有Worker
    await Promise.all(
      this.workers.map(w => w.worker.terminate().catch(() => {}))
    );

    console.log('✅ Worker Pool shutdown complete');
  }
}

module.exports = { WorkerPool };
