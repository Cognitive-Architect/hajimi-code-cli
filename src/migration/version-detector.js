/**
 * 版本检测器
 * Version Detector - 自动识别存储格式版本
 */

const fs = require('fs').promises;
const path = require('path');

// 魔数定义
const MAGIC = {
  HNSW: Buffer.from('HNSW'),      // 二进制格式 v1+
  JSON: Buffer.from('    ')       // JSON格式 (以 { 或 [ 开头)
};

const VERSIONS = {
  V0_JSON: 0,      // Phase 2 JSON格式
  V1_BINARY: 1,    // Phase 2.1 二进制格式
  V2_WASM: 2       // Phase 3 WASM格式 (预留)
};

class VersionDetector {
  constructor(options = {}) {
    this.basePath = options.basePath || './data';
  }

  /**
   * 检测文件版本
   */
  async detect(filePath) {
    try {
      // 读取文件头部
      const fd = await fs.open(filePath, 'r');
      const header = Buffer.alloc(16);
      
      try {
        await fd.read(header, 0, 16, 0);
      } finally {
        await fd.close();
      }

      // 检查魔数
      if (this._isBinaryFormat(header)) {
        return this._parseBinaryVersion(header);
      }
      
      // 检查JSON格式
      if (this._isJsonFormat(header)) {
        return {
          version: VERSIONS.V0_JSON,
          format: 'json',
          path: filePath
        };
      }

      return {
        version: null,
        format: 'unknown',
        path: filePath,
        error: 'Unrecognized file format'
      };

    } catch (err) {
      return {
        version: null,
        format: 'error',
        path: filePath,
        error: err.message
      };
    }
  }

  /**
   * 批量检测目录
   */
  async detectDirectory(dirPath, options = {}) {
    const pattern = options.pattern || /\.hnsw$/;
    const results = [];

    try {
      const files = await fs.readdir(dirPath);
      
      for (const file of files) {
        if (pattern.test(file)) {
          const fullPath = path.join(dirPath, file);
          const stat = await fs.stat(fullPath);
          
          if (stat.isFile()) {
            const info = await this.detect(fullPath);
            results.push({
              ...info,
              size: stat.size,
              mtime: stat.mtime
            });
          }
        }
      }
    } catch (err) {
      if (err.code !== 'ENOENT') {
        throw err;
      }
    }

    return results;
  }

  /**
   * 检查是否为二进制格式
   */
  _isBinaryFormat(header) {
    return header.slice(0, 4).toString() === 'HNSW';
  }

  /**
   * 检查是否为JSON格式
   */
  _isJsonFormat(header) {
    const firstChar = header[0];
    return firstChar === 0x7B || firstChar === 0x5B; // { or [
  }

  /**
   * 解析二进制版本
   */
  _parseBinaryVersion(header) {
    const version = header.readUInt16BE(4);
    const flags = header.readUInt16BE(6);
    
    return {
      version,
      format: 'binary',
      flags,
      header: {
        magic: header.slice(0, 4).toString(),
        version,
        flags
      }
    };
  }

  /**
   * 获取版本信息
   */
  getVersionInfo(versionCode) {
    const info = {
      [VERSIONS.V0_JSON]: {
        name: 'V0_JSON',
        description: 'Phase 2 JSON Format',
        features: ['Basic HNSW', 'JSON serialization'],
        deprecated: true
      },
      [VERSIONS.V1_BINARY]: {
        name: 'V1_BINARY',
        description: 'Phase 2.1 Binary Format',
        features: ['Binary serialization', 'Checksum', 'Compression'],
        deprecated: false
      },
      [VERSIONS.V2_WASM]: {
        name: 'V2_WASM',
        description: 'Phase 3 WASM Format',
        features: ['WASM acceleration', 'Memory mapping'],
        deprecated: false
      }
    };

    return info[versionCode] || { name: 'UNKNOWN', description: 'Unknown version' };
  }
}

module.exports = { 
  VersionDetector, 
  VERSIONS,
  MAGIC 
};
