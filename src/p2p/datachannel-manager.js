/**
 * DataChannel Manager - Sprint4全量实现
 * 文件传输 | 文本加密 | 断点续传 | 拥塞控制
 * 约束: ≤300行
 */
const crypto = require('crypto'), { EventEmitter } = require('events');
const wrtc = require('@koush/wrtc');

const CHUNK_SIZE = 64 * 1024; // 64KB分片
const DEFAULT_WINDOW = 8; // 初始滑动窗口
const MAX_WINDOW = 32;
const MIN_WINDOW = 1;
const RTT_ALPHA = 0.125;
const TIMEOUT_BASE = 1000;

class DataChannelManager extends EventEmitter {
  constructor() {
    super();
    this.channels = new Map(); // peerId -> {channel, pc, stats}
    this.transfers = new Map(); // transferId -> {chunks, sent, ack, progress}
    this.cryptoKey = crypto.randomBytes(32);
    this.seqCounter = 0;
  }

  createDataChannel(peerId, pc, options = { ordered: true, maxRetransmits: 3 }) {
    const channel = pc.createDataChannel('data', options);
    const ctx = { channel, pc, peerId, state: 'connecting', rtt: 50, window: DEFAULT_WINDOW };
    
    channel.onopen = () => { ctx.state = 'open'; this.emit('open', peerId); };
    channel.onclose = () => this.cleanup(peerId);
    channel.onerror = (e) => { this.emit('error', peerId, e); this.cleanup(peerId); };
    channel.onmessage = (e) => this.handleMessage(peerId, e.data);
    
    this.channels.set(peerId, ctx);
    return channel;
  }

  async sendFile(file, peerId, onProgress) {
    const ctx = this.channels.get(peerId);
    if (!ctx || ctx.state !== 'open') throw new Error('E_DC_001: Channel not open');
    
    const transferId = crypto.randomUUID();
    const buffer = Buffer.from(await file.arrayBuffer?.() || file);
    const totalChunks = Math.ceil(buffer.length / CHUNK_SIZE);
    const chunks = [];
    
    for (let i = 0; i < totalChunks; i++) {
      const start = i * CHUNK_SIZE;
      const chunkData = buffer.slice(start, start + CHUNK_SIZE);
      const hash = crypto.createHash('sha256').update(chunkData).digest('hex');
      chunks.push({ index: i, data: chunkData.toString('base64'), hash });
    }
    
    this.transfers.set(transferId, { 
      type: 'file', peerId, chunks, sent: 0, ack: 0, total: totalChunks, 
      onProgress, window: DEFAULT_WINDOW, inflight: new Set(), startTime: Date.now() 
    });
    
    this.emit('transfer-start', { transferId, total: totalChunks });
    await this.sendChunks(transferId);
    return transferId;
  }

  async sendChunks(transferId) {
    const tx = this.transfers.get(transferId);
    if (!tx) return;
    
    while (tx.sent < tx.total && tx.inflight.size < tx.window) {
      const chunk = tx.chunks[tx.sent];
      tx.inflight.add(chunk.index);
      
      const msg = {
        type: 'file-chunk', transferId, chunkIndex: chunk.index,
        totalChunks: tx.total, data: chunk.data, checksum: chunk.hash,
        timestamp: Date.now()
      };
      
      this.send(tx.peerId, msg);
      tx.sent++;
      
      // 模拟丢包检测重传
      setTimeout(() => {
        if (tx.inflight.has(chunk.index)) {
          tx.inflight.delete(chunk.index);
          this.adjustWindow(transferId, 'loss');
        }
      }, TIMEOUT_BASE + tx.window * 50);
    }
  }

  sendText(text, peerId) {
    const ctx = this.channels.get(peerId);
    if (!ctx || ctx.state !== 'open') throw new Error('E_DC_001: Channel not open');
    
    this.seqCounter++;
    const iv = crypto.randomBytes(16);
    const cipher = crypto.createCipheriv('aes-256-gcm', this.cryptoKey, iv);
    const encrypted = Buffer.concat([cipher.update(text, 'utf8'), cipher.final()]);
    const authTag = cipher.getAuthTag();
    
    const msg = {
      type: 'text-message', seq: this.seqCounter,
      data: encrypted.toString('base64'), iv: iv.toString('base64'),
      authTag: authTag.toString('base64'), timestamp: Date.now()
    };
    
    this.send(peerId, msg);
    return this.seqCounter;
  }

  resumeTransfer(transferId, receivedChunks = []) {
    const tx = this.transfers.get(transferId);
    if (!tx) throw new Error('E_DC_004: Transfer not found');
    
    // 断点检测 - HTTP Range风格
    const missing = [];
    for (let i = 0; i < tx.total; i++) {
      if (!receivedChunks.includes(i)) missing.push(i);
    }
    
    if (missing.length === 0) {
      this.emit('transfer-complete', { transferId });
      return { completed: true };
    }
    
    // 请求缺失分片
    const rangeStart = missing[0];
    const rangeEnd = missing[Math.min(missing.length - 1, tx.window - 1)];
    
    const resumeMsg = {
      type: 'resume-request', transferId,
      receivedChunks, requestedRange: { start: rangeStart, end: rangeEnd },
      timestamp: Date.now()
    };
    
    this.send(tx.peerId, resumeMsg);
    tx.sent = rangeStart; // 重置发送指针
    tx.inflight.clear();
    
    // 重新发送缺失分片
    setTimeout(() => this.sendChunks(transferId), 100);
    return { resumed: true, range: { start: rangeStart, end: rangeEnd } };
  }

  handleMessage(peerId, data) {
    try {
      const msg = JSON.parse(data);
      const ctx = this.channels.get(peerId);
      
      switch (msg.type) {
        case 'file-chunk':
          this.handleFileChunk(peerId, msg);
          this.sendAck(peerId, msg.transferId, msg.chunkIndex);
          break;
        case 'text-message':
          this.handleTextMessage(peerId, msg);
          break;
        case 'chunk-ack':
          this.handleAck(msg.transferId, msg.chunkIndex);
          break;
        case 'resume-request':
          this.handleResumeRequest(peerId, msg);
          break;
        case 'resume-ack':
          break; // handled by sender
        case 'congestion-control':
          this.handleCongestion(msg);
          break;
      }
      
      // RTT测量
      if (msg.timestamp && ctx) {
        const rtt = Date.now() - msg.timestamp;
        ctx.rtt = (1 - RTT_ALPHA) * ctx.rtt + RTT_ALPHA * rtt;
      }
    } catch (e) { /* silent fail */ }
  }

  handleFileChunk(peerId, msg) {
    const { transferId, chunkIndex, totalChunks, data, checksum } = msg;
    const chunkData = Buffer.from(data, 'base64');
    const hash = crypto.createHash('sha256').update(chunkData).digest('hex');
    
    if (hash !== checksum) {
      this.emit('chunk-error', { transferId, chunkIndex, error: 'E_DC_002: Checksum mismatch' });
      return;
    }
    
    this.emit('chunk-received', { transferId, chunkIndex, total: totalChunks, checksum });
  }

  handleTextMessage(peerId, msg) {
    try {
      const { data, iv, authTag } = msg;
      const decipher = crypto.createDecipheriv('aes-256-gcm', this.cryptoKey, Buffer.from(iv, 'base64'));
      decipher.setAuthTag(Buffer.from(authTag, 'base64'));
      const decrypted = Buffer.concat([decipher.update(Buffer.from(data, 'base64')), decipher.final()]);
      this.emit('text-message', { peerId, seq: msg.seq, text: decrypted.toString('utf8'), timestamp: msg.timestamp });
    } catch (e) { this.emit('error', peerId, new Error('E_DC_003: Decryption failed')); }
  }

  handleAck(transferId, chunkIndex) {
    const tx = this.transfers.get(transferId);
    if (!tx) return;
    
    tx.inflight.delete(chunkIndex);
    tx.ack++;
    
    // 进度回调
    const percent = Math.round((tx.ack / tx.total) * 100);
    tx.onProgress?.(percent);
    
    // 拥塞控制 - 成功发送时增大窗口
    if (tx.ack % tx.window === 0) this.adjustWindow(transferId, 'success');
    
    // 完成检测
    if (tx.ack >= tx.total) {
      this.emit('transfer-complete', { transferId, duration: Date.now() - tx.startTime });
      this.transfers.delete(transferId);
      return;
    }
    
    // 继续发送 (异步避免栈溢出)
    if (tx.sent < tx.total) setImmediate(() => this.sendChunks(transferId));
  }

  adjustWindow(transferId, event) {
    const tx = this.transfers.get(transferId);
    if (!tx) return;
    
    if (event === 'success' && tx.window < MAX_WINDOW) {
      tx.window = Math.min(tx.window + 1, MAX_WINDOW);
    } else if (event === 'loss' && tx.window > MIN_WINDOW) {
      tx.window = Math.max(Math.floor(tx.window / 2), MIN_WINDOW);
    }
    
    // 发送拥塞控制消息
    const ctx = this.channels.get(tx.peerId);
    if (ctx) {
      this.send(tx.peerId, {
        type: 'congestion-control', action: 'window-adjust',
        windowSize: tx.window, rtt: ctx.rtt, timestamp: Date.now()
      });
    }
  }

  sendAck(peerId, transferId, chunkIndex) {
    this.send(peerId, { type: 'chunk-ack', transferId, chunkIndex, timestamp: Date.now() });
  }

  handleResumeRequest(peerId, msg) {
    const { transferId, requestedRange } = msg;
    this.send(peerId, { type: 'resume-ack', transferId, range: requestedRange, timestamp: Date.now() });
    // 触发重新发送
    const tx = this.transfers.get(transferId);
    if (tx) {
      tx.sent = requestedRange.start;
      tx.inflight.clear();
      this.sendChunks(transferId);
    }
  }

  handleCongestion(msg) {
    this.emit('congestion-update', { windowSize: msg.windowSize, rtt: msg.rtt });
  }

  send(peerId, msg) {
    const ctx = this.channels.get(peerId);
    if (ctx?.channel?.readyState === 'open') {
      ctx.channel.send(JSON.stringify(msg));
    }
  }

  cleanup(peerId) {
    const ctx = this.channels.get(peerId);
    if (ctx) {
      ctx.channel?.close?.();
      ctx.pc?.close?.();
      this.channels.delete(peerId);
    }
    // 清理该peer的所有传输
    for (const [tid, tx] of this.transfers) {
      if (tx.peerId === peerId) this.transfers.delete(tid);
    }
    this.removeAllListeners();
    this.emit('close', peerId);
  }

  closeAll() {
    for (const peerId of this.channels.keys()) this.cleanup(peerId);
  }
}

module.exports = { DataChannelManager, CHUNK_SIZE };
