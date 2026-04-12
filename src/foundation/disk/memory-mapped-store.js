/**
 * Memory-Mapped Store - 模拟mmap文件管理器
 * Simulated memory-mapped file store for Node.js
 * 
 * 注意: Node.js没有原生mmap支持，使用fs.read/write模拟
 * 在Termux环境下，这是更实际的选择
 */

const fs = require('fs').promises;
const fsSync = require('fs');
const path = require('path');
const { BlockCache } = require('./block-cache');

class MemoryMappedStore {
  constructor(options = {}) {
    this.basePath = options.basePath || './data/disk';
    this.blockSize = options.blockSize || 4096;
    this.cache = new BlockCache({
      blockSize: this.blockSize,
      maxBlocks: options.maxCacheBlocks || 500
    });
    this.openFiles = new Map(); // fileId -> { fd, size, path }
    this.options = {
      syncOnWrite: options.syncOnWrite !== false, // 默认开启同步写入
      readOnly: options.readOnly || false
    };
  }

  /**
   * 初始化存储目录
   */
  async init() {
    try {
      await fs.mkdir(this.basePath, { recursive: true });
    } catch (err) {
      if (err.code !== 'EEXIST') throw err;
    }
  }

  /**
   * 打开或创建内存映射文件
   */
  async open(fileId, options = {}) {
    const filePath = path.join(this.basePath, `${fileId}.hnsw.disk`);
    
    if (this.openFiles.has(fileId)) {
      return this.openFiles.get(fileId);
    }

    const flags = this.options.readOnly ? 'r' : 'r+';
    let fd;
    let size = 0;

    try {
      // Try to open existing file
      fd = await fs.open(filePath, flags);
      const stat = await fd.stat();
      size = stat.size;
    } catch (err) {
      if (err.code === 'ENOENT' && !this.options.readOnly) {
        // Create new file
        fd = await fs.open(filePath, 'w+');
      } else {
        throw err;
      }
    }

    const fileInfo = {
      fd,
      path: filePath,
      size,
      id: fileId
    };

    this.openFiles.set(fileId, fileInfo);
    return fileInfo;
  }

  /**
   * 关闭文件
   */
  async close(fileId) {
    const file = this.openFiles.get(fileId);
    if (!file) return;

    await file.fd.close();
    this.openFiles.delete(fileId);
    this.cache.invalidateFile(fileId);
  }

  /**
   * 读取数据块
   */
  async read(fileId, offset, length) {
    const file = this.openFiles.get(fileId) || await this.open(fileId);
    
    // Calculate block range
    const startBlock = Math.floor(offset / this.blockSize);
    const endBlock = Math.floor((offset + length - 1) / this.blockSize);
    
    const buffers = [];
    let currentOffset = offset;
    let remaining = length;

    for (let blockIdx = startBlock; blockIdx <= endBlock; blockIdx++) {
      const blockOffset = blockIdx * this.blockSize;
      const inBlockOffset = currentOffset - blockOffset;
      const toRead = Math.min(remaining, this.blockSize - inBlockOffset);

      // Try cache first
      let blockData = this.cache.get(fileId, blockIdx);
      
      if (!blockData) {
        // Read from disk
        blockData = Buffer.alloc(this.blockSize);
        try {
          await file.fd.read(blockData, 0, this.blockSize, blockOffset);
          this.cache.set(fileId, blockIdx, blockData);
        } catch (err) {
          // If reading past EOF, return zeros
          if (err.code === 'EOF' || blockOffset >= file.size) {
            blockData.fill(0);
          } else {
            throw err;
          }
        }
      }

      buffers.push(blockData.subarray(inBlockOffset, inBlockOffset + toRead));
      currentOffset += toRead;
      remaining -= toRead;
    }

    return Buffer.concat(buffers);
  }

  /**
   * 写入数据
   */
  async write(fileId, offset, data) {
    if (this.options.readOnly) {
      throw new Error('Cannot write to read-only store');
    }

    const file = this.openFiles.get(fileId) || await this.open(fileId);
    const buffer = Buffer.isBuffer(data) ? data : Buffer.from(data);
    
    // Calculate block range
    const startBlock = Math.floor(offset / this.blockSize);
    const endBlock = Math.floor((offset + buffer.length - 1) / this.blockSize);
    
    let currentOffset = offset;
    let sourceOffset = 0;
    let remaining = buffer.length;

    for (let blockIdx = startBlock; blockIdx <= endBlock; blockIdx++) {
      const blockOffset = blockIdx * this.blockSize;
      const inBlockOffset = currentOffset - blockOffset;
      const toWrite = Math.min(remaining, this.blockSize - inBlockOffset);

      // Read existing block or create new
      let blockData = this.cache.get(fileId, blockIdx);
      
      if (!blockData) {
        blockData = Buffer.alloc(this.blockSize);
        if (blockOffset < file.size) {
          try {
            await file.fd.read(blockData, 0, this.blockSize, blockOffset);
          } catch (err) {
            // Ignore EOF errors
          }
        }
      }

      // Modify block
      buffer.copy(blockData, inBlockOffset, sourceOffset, sourceOffset + toWrite);
      
      // Write to disk
      await file.fd.write(blockData, 0, this.blockSize, blockOffset);
      
      // Update cache
      this.cache.set(fileId, blockIdx, blockData);
      
      currentOffset += toWrite;
      sourceOffset += toWrite;
      remaining -= toWrite;
    }

    // Update file size if needed
    const newSize = Math.max(file.size, offset + buffer.length);
    if (newSize > file.size) {
      file.size = newSize;
    }

    // Sync if required
    if (this.options.syncOnWrite) {
      await file.fd.sync();
    }

    return buffer.length;
  }

  /**
   * 截断/扩展文件
   */
  async truncate(fileId, size) {
    if (this.options.readOnly) {
      throw new Error('Cannot truncate read-only store');
    }

    const file = this.openFiles.get(fileId) || await this.open(fileId);
    await file.fd.truncate(size);
    file.size = size;
    
    // Invalidate cache entries beyond new size
    const maxBlock = Math.floor(size / this.blockSize);
    for (const key of this.cache.cache.keys()) {
      if (key.startsWith(`${fileId}:`)) {
        const blockIdx = parseInt(key.split(':')[1]);
        if (blockIdx > maxBlock) {
          this.cache.invalidate(fileId, blockIdx);
        }
      }
    }
  }

  /**
   * 获取文件大小
   */
  async getSize(fileId) {
    const file = this.openFiles.get(fileId);
    if (file) return file.size;

    const filePath = path.join(this.basePath, `${fileId}.hnsw.disk`);
    try {
      const stat = await fs.stat(filePath);
      return stat.size;
    } catch (err) {
      if (err.code === 'ENOENT') return 0;
      throw err;
    }
  }

  /**
   * 同步所有文件到磁盘
   */
  async sync() {
    for (const file of this.openFiles.values()) {
      await file.fd.sync();
    }
  }

  /**
   * 关闭所有文件
   */
  async closeAll() {
    await this.sync();
    for (const fileId of Array.from(this.openFiles.keys())) {
      await this.close(fileId);
    }
  }

  /**
   * 获取统计信息
   */
  getStats() {
    return {
      openFiles: this.openFiles.size,
      cache: this.cache.getStats()
    };
  }

  /**
   * 删除文件
   */
  async delete(fileId) {
    await this.close(fileId);
    const filePath = path.join(this.basePath, `${fileId}.hnsw.disk`);
    try {
      await fs.unlink(filePath);
    } catch (err) {
      if (err.code !== 'ENOENT') throw err;
    }
  }
}

module.exports = { MemoryMappedStore };
