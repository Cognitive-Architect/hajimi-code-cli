/**
 * WebRTC Signaling Server - WebSocket SDP/ICE exchange
 * HARDENED: CSPRNG via crypto.randomUUID(), PSK authentication from env (HAJIMI_SIGNALING_PSK)
 * MITM protection with constant-time comparison. DEBT-P0-001 for key rotation.
 * Reuses crypto patterns from datachannel-manager.js (scrypt, randomUUID, timingSafeEqual)
 */
const crypto = require('crypto');
const url = require('url');
const WebSocket = require('ws'), http = require('http'), CONFIG = require('./config');

const E_SIGNALING = {
  INVALID_MESSAGE: 'E_SIGNALING_INVALID_MESSAGE',
  INVALID_JSONRPC: 'E_SIGNALING_INVALID_JSONRPC',
  PEER_NOT_FOUND: 'E_SIGNALING_PEER_NOT_FOUND',
  TIMEOUT: 'E_SIGNALING_TIMEOUT',
  CONNECTION_ERROR: 'E_SIGNALING_CONNECTION_ERROR',
  AUTH_FAILED: 'E_SIGNALING_AUTH_FAILED'
};

class SignalingServer {
  constructor(port = CONFIG.SIGNALING.PORT) {
    this.port = port;
    this.clients = new Map();
    this.timeouts = new Map();
    this.server = null;
    this.wss = null;
    this.psk = process.env.HAJIMI_SIGNALING_PSK;
    if (!this.psk || this.psk.length < 16) {
      console.error('❌ PSK must be set via HAJIMI_SIGNALING_PSK and length >=16 chars for security (B-06 P0 optimization). process.exit(1)');
      process.exit(1);
    } else {
      console.log('✅ PSK authentication enabled for signaling server (length validated)');
    }
  }
  start() {
    this.server = http.createServer();
    this.wss = new WebSocket.Server({ server: this.server });
    this.wss.on('connection', (ws, req) => this.handleConnection(ws, req));
    this.wss.on('error', (err) => console.error('WebSocket error:', err));
    this.server.listen(this.port, () => {
      console.log(`Signaling server started on ws://localhost:${this.port}`);
      console.log(`Active clients: ${this.clients.size}`);
      if (this.psk) console.log(`PSK auth enabled (env: HAJIMI_SIGNALING_PSK)`);
    });
    return this;
  }

  handleConnection(ws, req) {
    // PSK Authentication - query param or first message (MITM protection)
    const parsed = url.parse(req.url || '', true);
    let providedPsk = parsed.query.psk || '';

    // Fallback: check for auth in upgrade headers or initial message (but query preferred)
    if (!providedPsk && req.headers['x-psk']) {
      providedPsk = req.headers['x-psk'];
    }

    if (this.psk) {
      const pskBuffer = Buffer.from(this.psk);
      const providedBuffer = Buffer.from(providedPsk);
      if (pskBuffer.length !== providedBuffer.length || !crypto.timingSafeEqual(pskBuffer, providedBuffer)) {
        console.error(`PSK validation failed for connection from ${req.socket.remoteAddress}`);
        ws.close(1008, 'Invalid or missing PSK - authentication failed');
        return;
      }
      console.log(`✅ PSK authentication successful for new client`);
    } else {
      console.warn('⚠️ No PSK configured - proceeding unauthenticated (DEBT-P0-001)');
    }

    const clientId = crypto.randomUUID();  // CSPRNG replacement
    ws.isAlive = true;
    ws.heartbeatInterval = setInterval(() => {
      if (!ws.isAlive) {
        console.log(`Heartbeat timeout for client ${clientId}, terminating connection`);
        ws.terminate();
        return;
      }
      ws.isAlive = false;
      ws.ping();
    }, CONFIG.HEARTBEAT.INTERVAL);
    ws.on('pong', () => {
      ws.isAlive = true;
      console.log(`Received pong from client ${clientId}`);
    });
    this.clients.set(clientId, { ws, peerId: null, authenticated: !!this.psk });
    this.setTimeout(clientId);
    ws.on('message', (data) => this.handleMessage(clientId, data));
    ws.on('close', () => this.handleDisconnect(clientId));
    ws.on('error', (err) => console.error(`Client ${clientId} error:`, err.message));
    this.send(ws, { type: 'connected', clientId, authenticated: !!this.psk });
    console.log(`Client ${clientId} connected (PSK: ${!!this.psk}). Total: ${this.clients.size}`);
  }
  setTimeout(clientId) {
    const timer = setTimeout(() => {
      const client = this.clients.get(clientId);
      if (client && !client.peerId) {
        this.send(client.ws, { type: 'error', code: E_SIGNALING.TIMEOUT });
        client.ws.close();
      }
    }, CONFIG.TIMEOUT);
    this.timeouts.set(clientId, timer);
  }
  handleMessage(clientId, data) {
    try {
      const msg = JSON.parse(data), client = this.clients.get(clientId);
      if (!client) return;
      // JSON-RPC 2.0 version check
      if (msg.jsonrpc !== '2.0') {
        this.send(client.ws, { type: 'error', code: E_SIGNALING.INVALID_JSONRPC });
        console.error(`Invalid jsonrpc version from client ${clientId}: ${msg.jsonrpc}`);
        return;
      }
      switch (msg.type) {
        case 'register':
          client.peerId = msg.peerId; clearTimeout(this.timeouts.get(clientId));
          this.broadcast(clientId, { type: 'peer-joined', peerId: msg.peerId }); break;
        case 'offer': this.forward(clientId, msg.targetId, { type: 'offer', sdp: msg.sdp, from: clientId }); break;
        case 'answer': this.forward(clientId, msg.targetId, { type: 'answer', sdp: msg.sdp, from: clientId }); break;
        case 'icecandidate': this.forward(clientId, msg.targetId, { type: 'icecandidate', candidate: msg.candidate, from: clientId }); break;
        case 'datachannel': this.forward(clientId, msg.targetId, { type: 'datachannel', data: msg.data, from: clientId }); break;
        default: this.send(client.ws, { type: 'error', code: E_SIGNALING.INVALID_MESSAGE });
      }
    } catch (err) { console.error('Message parse error:', err); }
  }
  forward(fromId, targetId, msg) {
    let sent = false;
    for (const [cid, client] of this.clients) {
      if (client.peerId === targetId || cid === targetId) { this.send(client.ws, msg); sent = true; break; }
    }
    if (!sent) {
      const from = this.clients.get(fromId);
      if (from) this.send(from.ws, { type: 'error', code: E_SIGNALING.PEER_NOT_FOUND });
    }
  }
  broadcast(excludeId, msg) {
    for (const [cid, client] of this.clients) if (cid !== excludeId) this.send(client.ws, msg);
  }
  send(ws, msg) { if (ws.readyState === WebSocket.OPEN) ws.send(JSON.stringify(msg)); }
  handleDisconnect(clientId) {
    const client = this.clients.get(clientId);
    if (client?.ws?.heartbeatInterval) {
      clearInterval(client.ws.heartbeatInterval);
    }
    if (this.timeouts.has(clientId)) clearTimeout(this.timeouts.get(clientId));
    this.clients.delete(clientId); this.timeouts.delete(clientId);
    if (client?.peerId) this.broadcast(clientId, { type: 'peer-left', peerId: client.peerId });
    console.log(`Client ${clientId} disconnected. Total: ${this.clients.size}`);
  }
  stop() {
    this.timeouts.forEach(t => clearTimeout(t));
    this.clients.forEach(c => {
      if (c.ws.heartbeatInterval) clearInterval(c.ws.heartbeatInterval);
      c.ws.close();
    });
    this.wss?.close(); this.server?.close();
    console.log('Signaling server stopped');
  }
}

if (require.main === module) {
  const server = new SignalingServer().start();
  process.on('SIGINT', () => server.stop());
}
module.exports = { SignalingServer, E_SIGNALING };
