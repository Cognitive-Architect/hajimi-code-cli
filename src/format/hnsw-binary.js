/**
 * HNSW 二进制格式规范 - Binary Format
 * 
 * 文件结构：
 * 
 * [Header] - 256 bytes
 *   - magic (4 bytes): "HNSW"
 *   - version (2 bytes): 1
 *   - flags (2 bytes): 保留
 *   - timestamp (8 bytes): 创建时间
 *   - vectorCount (4 bytes): 向量数量
 *   - dimension (2 bytes): 向量维度
 *   - maxLevel (2 bytes): 最大层数
 *   - entryPoint (4 bytes): 入口点ID
 *   - checksum (32 bytes): SHA256
 *   - reserved (196 bytes)
 * 
 * [Vector Table] - N × 16 bytes
 *   - id (4 bytes)
 *   - level (2 bytes)
 *   - deleted (1 byte)
 *   - reserved (1 byte)
 *   - offset (8 bytes): 数据在文件中的偏移
 * 
 * [Vector Data] - 连续存储
 *   - float32[]: 向量数据 (dimension × 4 bytes)
 *   - connections: 每层连接
 *     - count (4 bytes)
 *     - ids[] (4 bytes each)
 * 
 * DEBT-PHASE2-005 清偿方案
 */

const MAGIC = Buffer.from('HNSW');
const VERSION = 1;
const HEADER_SIZE = 256;
const VECTOR_TABLE_ENTRY_SIZE = 16;

/**
 * 二进制序列化器
 */
class BinarySerializer {
  constructor() {
    this.buffers = [];
    this.totalSize = 0;
  }
  
  /**
   * 写入 Header
   */
  writeHeader(metadata) {
    const header = Buffer.alloc(HEADER_SIZE);
    let offset = 0;
    
    // Magic (4 bytes)
    MAGIC.copy(header, offset);
    offset += 4;
    
    // Version (2 bytes)
    header.writeUInt16BE(VERSION, offset);
    offset += 2;
    
    // Flags (2 bytes)
    header.writeUInt16BE(0, offset);
    offset += 2;
    
    // Timestamp (8 bytes)
    header.writeBigUInt64BE(BigInt(metadata.timestamp || Date.now()), offset);
    offset += 8;
    
    // Vector count (4 bytes)
    header.writeUInt32BE(metadata.vectorCount || 0, offset);
    offset += 4;
    
    // Dimension (2 bytes)
    header.writeUInt16BE(metadata.dimension || 128, offset);
    offset += 2;
    
    // Max level (2 bytes)
    header.writeUInt16BE(metadata.maxLevel || 0, offset);
    offset += 2;
    
    // Entry point (4 bytes)
    header.writeInt32BE(metadata.entryPoint || -1, offset);
    offset += 4;
    
    // Checksum placeholder (32 bytes)
    offset += 32;
    
    // Reserved (剩余空间)
    
    this.buffers.push(header);
    this.totalSize += header.length;
    
    return header;
  }
  
  /**
   * 写入向量表（占位，后面回填）
   */
  writeVectorTablePlaceholder(count) {
    const size = count * VECTOR_TABLE_ENTRY_SIZE;
    const table = Buffer.alloc(size);
    this.buffers.push(table);
    this.totalSize += table.length;
    return table;
  }
  
  /**
   * 序列化单个向量
   */
  serializeVector(node, dimension) {
    const parts = [];
    
    // 向量数据
    if (node.vector instanceof Float32Array) {
      const vecBuffer = Buffer.from(node.vector.buffer);
      parts.push(vecBuffer);
    } else {
      // bigint (SimHash)
      const buf = Buffer.alloc(8);
      buf.writeBigUInt64BE(node.vector, 0);
      parts.push(buf);
    }
    
    // 连接数据
    for (let level = 0; level <= node.level; level++) {
      const connections = node.connections[level] || [];
      const connBuffer = Buffer.alloc(4 + connections.length * 4);
      connBuffer.writeUInt32BE(connections.length, 0);
      
      for (let i = 0; i < connections.length; i++) {
        connBuffer.writeInt32BE(connections[i], 4 + i * 4);
      }
      
      parts.push(connBuffer);
    }
    
    return Buffer.concat(parts);
  }
  
  /**
   * 计算 checksum
   */
  computeChecksum(data) {
    const crypto = require('crypto');
    return crypto.createHash('sha256').update(data).digest();
  }
  
  /**
   * 完成序列化（回填 checksum）
   */
  finalize() {
    let data = Buffer.concat(this.buffers);
    
    // 先把 checksum 位置清零（24-56 字节）
    data.fill(0, 24, 56);
    
    // 计算 checksum
    const checksum = this.computeChecksum(data);
    
    // 回填 checksum 到 header
    checksum.copy(data, 24);
    
    return data;
  }
}

/**
 * 二进制反序列化器
 */
class BinaryDeserializer {
  constructor(buffer) {
    this.buffer = buffer;
    this.offset = 0;
  }
  
  /**
   * 验证 magic
   */
  validateMagic() {
    const magic = this.buffer.slice(0, 4);
    return magic.equals(MAGIC);
  }
  
  /**
   * 读取 Header
   */
  readHeader() {
    const header = {};
    let offset = 0;
    
    // Magic
    header.magic = this.buffer.slice(offset, offset + 4).toString();
    offset += 4;
    
    // Version
    header.version = this.buffer.readUInt16BE(offset);
    offset += 2;
    
    // Flags
    header.flags = this.buffer.readUInt16BE(offset);
    offset += 2;
    
    // Timestamp
    header.timestamp = Number(this.buffer.readBigUInt64BE(offset));
    offset += 8;
    
    // Vector count
    header.vectorCount = this.buffer.readUInt32BE(offset);
    offset += 4;
    
    // Dimension
    header.dimension = this.buffer.readUInt16BE(offset);
    offset += 2;
    
    // Max level
    header.maxLevel = this.buffer.readUInt16BE(offset);
    offset += 2;
    
    // Entry point
    header.entryPoint = this.buffer.readInt32BE(offset);
    offset += 4;
    
    // Checksum
    header.checksum = this.buffer.slice(offset, offset + 32);
    offset += 32;
    
    return header;
  }
  
  /**
   * 验证 checksum
   */
  validateChecksum() {
    const storedChecksum = this.buffer.slice(24, 56);
    
    // 创建临时 buffer，checksum 位置置零
    const tempBuffer = Buffer.alloc(this.buffer.length);
    this.buffer.copy(tempBuffer);
    tempBuffer.fill(0, 24, 56);
    
    const crypto = require('crypto');
    const computedChecksum = crypto.createHash('sha256').update(tempBuffer).digest();
    
    return storedChecksum.equals(computedChecksum);
  }
  
  /**
   * 读取向量表
   */
  readVectorTable(count) {
    const table = [];
    const startOffset = HEADER_SIZE;
    
    for (let i = 0; i < count; i++) {
      const offset = startOffset + i * VECTOR_TABLE_ENTRY_SIZE;
      
      table.push({
        id: this.buffer.readInt32BE(offset),
        level: this.buffer.readUInt16BE(offset + 4),
        deleted: this.buffer.readUInt8(offset + 6) !== 0,
        reserved: this.buffer.readUInt8(offset + 7),
        dataOffset: Number(this.buffer.readBigUInt64BE(offset + 8))
      });
    }
    
    return table;
  }
  
  /**
   * 读取向量数据
   */
  readVectorData(entry, dimension) {
    let offset = entry.dataOffset;
    
    // 读取向量
    const vector = new Float32Array(dimension);
    for (let i = 0; i < dimension; i++) {
      vector[i] = this.buffer.readFloatBE(offset);
      offset += 4;
    }
    
    // 读取连接
    const connections = [];
    for (let level = 0; level <= entry.level; level++) {
      const count = this.buffer.readUInt32BE(offset);
      offset += 4;
      
      const levelConnections = [];
      for (let i = 0; i < count; i++) {
        levelConnections.push(this.buffer.readInt32BE(offset));
        offset += 4;
      }
      
      connections.push(levelConnections);
    }
    
    return {
      id: entry.id,
      vector,
      level: entry.level,
      deleted: entry.deleted,
      connections
    };
  }
}

/**
 * HNSW 索引二进制序列化
 */
function serializeHNSW(index, metadata = {}) {
  const serializer = new BinarySerializer();
  const nodes = Array.from(index.nodes.values());
  
  // Header
  const headerMeta = {
    timestamp: Date.now(),
    vectorCount: nodes.length,
    dimension: metadata.dimension || 128,
    maxLevel: index.maxLevel,
    entryPoint: index.entryPoint,
    ...metadata
  };
  
  serializer.writeHeader(headerMeta);
  
  // 向量表占位
  const tableBuffer = serializer.writeVectorTablePlaceholder(nodes.length);
  
  // 序列化向量数据并记录偏移
  const dataBuffers = [];
  let dataOffset = HEADER_SIZE + nodes.length * VECTOR_TABLE_ENTRY_SIZE;
  
  for (let i = 0; i < nodes.length; i++) {
    const node = nodes[i];
    const nodeData = serializer.serializeVector(node, headerMeta.dimension);
    
    // 回填向量表 (tableBuffer 内部偏移从 0 开始)
    const tableOffset = i * VECTOR_TABLE_ENTRY_SIZE;
    tableBuffer.writeInt32BE(node.id, tableOffset);
    tableBuffer.writeUInt16BE(node.level, tableOffset + 4);
    tableBuffer.writeUInt8(node.deleted ? 1 : 0, tableOffset + 6);
    tableBuffer.writeBigUInt64BE(BigInt(dataOffset), tableOffset + 8);
    
    dataBuffers.push(nodeData);
    dataOffset += nodeData.length;
  }
  
  // 添加数据缓冲区
  serializer.buffers.push(...dataBuffers);
  serializer.totalSize = dataOffset;
  
  return serializer.finalize();
}

/**
 * HNSW 索引二进制反序列化
 */
function deserializeHNSW(buffer, options = {}) {
  const deserializer = new BinaryDeserializer(buffer);
  
  // 验证 magic
  if (!deserializer.validateMagic()) {
    throw new Error('Invalid binary format: wrong magic');
  }
  
  // 读取 header
  const header = deserializer.readHeader();
  
  // 验证 checksum
  if (!options.skipChecksum && !deserializer.validateChecksum()) {
    throw new Error('Binary file corrupted: checksum mismatch');
  }
  
  // 读取向量表
  const table = deserializer.readVectorTable(header.vectorCount);
  
  // 读取向量数据
  const nodes = [];
  for (const entry of table) {
    const nodeData = deserializer.readVectorData(entry, header.dimension);
    nodes.push(nodeData);
  }
  
  return {
    header,
    nodes
  };
}

module.exports = {
  BinarySerializer,
  BinaryDeserializer,
  serializeHNSW,
  deserializeHNSW,
  HEADER_SIZE,
  VECTOR_TABLE_ENTRY_SIZE,
  MAGIC,
  VERSION
};
