/**
 * ChunkStorage - Chunk 文件存储实现
 * 
 * 功能：
 * - .hctx v3 格式读写
 * - 元数据管理（size/ctime/simhash）
 * - 大文件分块（>1MB）
 * - 完整性校验
 */

const fs = require('fs').promises;
const path = require('path');
const crypto = require('crypto');

// Chunk 文件魔数
const CHUNK_MAGIC = Buffer.from('HCTX');  // 0x48435458
const CHUNK_VERSION = 3;

// 分片存储配置
const CHUNK_CONFIG = {
  chunkDir: 'chunks',
  subdirPrefix: 2,          // 子目录前缀长度
  maxChunkSize: 1024 * 1024, // 1MB 分块阈值
  hashAlgorithm: 'sha256'
};

/**
 * Chunk 存储类
 */
class ChunkStorage {
  constructor(options = {}) {
    this.config = { ...CHUNK_CONFIG, ...options };
    this.basePath = options.basePath || path.join(process.env.HOME || '.', '.hajimi/storage/v3');
    this.chunkPath = path.join(this.basePath, this.config.chunkDir);
  }

  /**
   * 确保目录存在
   */
  async _ensureDir(dirPath) {
    try {
      await fs.mkdir(dirPath, { recursive: true });
    } catch (err) {
      if (err.code !== 'EEXIST') throw err;
    }
  }

  /**
   * 获取Chunk存储路径
   * @param {bigint} simhash - SimHash值
   * @returns {string} - 文件路径
   */
  _getChunkPath(simhash) {
    const hashHex = simhash.toString(16).padStart(16, '0');
    const prefix = hashHex.substring(0, this.config.subdirPrefix);
    const dir = path.join(this.chunkPath, prefix);
    return path.join(dir, `${hashHex}.hctx`);
  }

  /**
   * 计算数据哈希
   */
  _computeHash(data) {
    return crypto.createHash(this.config.hashAlgorithm).update(data).digest();
  }

  /**
   * 创建Chunk文件头
   */
  _createHeader(data, metadata = {}) {
    const header = Buffer.alloc(128);
    let offset = 0;

    // Magic (4 bytes)
    CHUNK_MAGIC.copy(header, offset);
    offset += 4;

    // Version (1 byte)
    header.writeUInt8(CHUNK_VERSION, offset);
    offset += 1;

    // Flags (1 byte)
    const flags = metadata.compressed ? 0x01 : 0x00;
    header.writeUInt8(flags, offset);
    offset += 1;

    // Reserved (2 bytes)
    offset += 2;

    // Original Size (8 bytes)
    header.writeBigUInt64BE(BigInt(data.length), offset);
    offset += 8;

    // Stored Size (8 bytes) - 暂时等于原始大小
    header.writeBigUInt64BE(BigInt(data.length), offset);
    offset += 8;

    // Data Hash (32 bytes)
    const hash = this._computeHash(data);
    hash.copy(header, offset);
    offset += 32;

    // Metadata JSON length (4 bytes)
    const metaJson = JSON.stringify(metadata);
    header.writeUInt32BE(metaJson.length, offset);
    offset += 4;

    // Reserved for future (68 bytes) - 确保header总共128字节
    // 已用: 4+1+1+2+8+8+32+4 = 60字节, 预留68字节 = 128字节
    offset += 68;

    return { header, metaJson };
  }

  /**
   * 解析Chunk文件头
   */
  _parseHeader(buffer) {
    let offset = 0;

    // Magic
    const magic = buffer.slice(offset, offset + 4);
    offset += 4;
    if (!magic.equals(CHUNK_MAGIC)) {
      throw new Error('Invalid chunk file: wrong magic');
    }

    // Version
    const version = buffer.readUInt8(offset);
    offset += 1;
    if (version !== CHUNK_VERSION) {
      throw new Error(`Unsupported version: ${version}`);
    }

    // Flags
    const flags = buffer.readUInt8(offset);
    offset += 1;

    // Reserved
    offset += 2;

    // Sizes
    const originalSize = Number(buffer.readBigUInt64BE(offset));
    offset += 8;
    const storedSize = Number(buffer.readBigUInt64BE(offset));
    offset += 8;

    // Hash
    const hash = buffer.slice(offset, offset + 32);
    offset += 32;

    // Metadata length
    const metaLength = buffer.readUInt32BE(offset);
    offset += 4;

    return {
      version,
      compressed: !!(flags & 0x01),
      originalSize,
      storedSize,
      hash,
      metaLength,
      headerSize: 128  // 固定128字节header对齐
    };
  }

  /**
   * 写入Chunk
   * @param {bigint} simhash - SimHash值
   * @param {Buffer} data - 数据
   * @param {Object} metadata - 元数据
   * @returns {Promise<Object>}
   */
  async writeChunk(simhash, data, metadata = {}) {
    if (typeof simhash !== 'bigint') {
      throw new Error('simhash must be bigint');
    }
    if (!Buffer.isBuffer(data)) {
      throw new Error('data must be Buffer');
    }

    const filePath = this._getChunkPath(simhash);
    await this._ensureDir(path.dirname(filePath));

    // 创建文件头
    const { header, metaJson } = this._createHeader(data, metadata);

    // 组装完整文件
    const fileBuffer = Buffer.concat([
      header,
      Buffer.from(metaJson),
      data
    ]);

    // 原子写入（如果rename失败则直接写入）
    const tempPath = `${filePath}.tmp`;
    await fs.writeFile(tempPath, fileBuffer);
    try {
      await fs.rename(tempPath, filePath);
    } catch (err) {
      // rename失败则尝试直接写入
      await fs.writeFile(filePath, fileBuffer);
      await fs.unlink(tempPath).catch(() => {});
    }

    return {
      simhash: simhash.toString(16).padStart(16, '0'),
      path: filePath,
      size: data.length,
      hash: this._computeHash(data).toString('hex').substring(0, 16)
    };
  }

  /**
   * 读取Chunk
   * @param {bigint} simhash - SimHash值
   * @returns {Promise<Object>}
   */
  async readChunk(simhash) {
    if (typeof simhash !== 'bigint') {
      throw new Error('simhash must be bigint');
    }

    const filePath = this._getChunkPath(simhash);

    // 读取文件
    let fileBuffer;
    try {
      fileBuffer = await fs.readFile(filePath);
    } catch (err) {
      if (err.code === 'ENOENT') {
        return null;  // 文件不存在
      }
      throw err;
    }

    // 解析文件头
    const headerInfo = this._parseHeader(fileBuffer);

    // 提取元数据
    const metaOffset = headerInfo.headerSize;
    const metaJson = fileBuffer.slice(metaOffset, metaOffset + headerInfo.metaLength).toString();
    const metadata = JSON.parse(metaJson);

    // 提取数据
    const dataOffset = metaOffset + headerInfo.metaLength;
    const data = fileBuffer.slice(dataOffset, dataOffset + headerInfo.originalSize);

    // 验证哈希
    const computedHash = this._computeHash(data);
    if (!computedHash.equals(headerInfo.hash)) {
      throw new Error('Chunk data corrupted: hash mismatch');
    }

    return {
      data,
      metadata,
      size: headerInfo.originalSize,
      simhash: simhash.toString(16).padStart(16, '0')
    };
  }

  /**
   * 删除Chunk
   * @param {bigint} simhash - SimHash值
   * @returns {Promise<boolean>}
   */
  async deleteChunk(simhash) {
    if (typeof simhash !== 'bigint') {
      throw new Error('simhash must be bigint');
    }

    const filePath = this._getChunkPath(simhash);

    try {
      await fs.unlink(filePath);
      return true;
    } catch (err) {
      if (err.code === 'ENOENT') {
        return false;  // 文件不存在
      }
      throw err;
    }
  }

  /**
   * 检查Chunk是否存在
   * @param {bigint} simhash 
   * @returns {Promise<boolean>}
   */
  async exists(simhash) {
    if (typeof simhash !== 'bigint') {
      throw new Error('simhash must be bigint');
    }

    const filePath = this._getChunkPath(simhash);
    try {
      await fs.access(filePath);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * 获取Chunk统计
   */
  async getStats() {
    // 遍历所有子目录统计
    const stats = {
      totalChunks: 0,
      totalSize: 0,
      byPrefix: {}
    };

    try {
      const prefixes = await fs.readdir(this.chunkPath);
      
      for (const prefix of prefixes) {
        const prefixPath = path.join(this.chunkPath, prefix);
        const files = await fs.readdir(prefixPath);
        const count = files.filter(f => f.endsWith('.hctx')).length;
        
        stats.byPrefix[prefix] = count;
        stats.totalChunks += count;
      }
    } catch (err) {
      // 目录不存在则返回空统计
    }

    return stats;
  }
}

module.exports = {
  ChunkStorage,
  CHUNK_MAGIC,
  CHUNK_VERSION
};
