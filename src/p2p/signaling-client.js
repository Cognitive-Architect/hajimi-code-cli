/**
 * WebRTC Signaling Client - RTCPeerConnection wrapper
 */
const WebSocket = require('ws'), CONFIG = require('./config');

class SignalingClient {
  constructor(peerId, signalingUrl = `ws://${CONFIG.SIGNALING.HOST}:${CONFIG.SIGNALING.PORT}`) {
    this.peerId = peerId; this.signalingUrl = signalingUrl; this.ws = null;
    this.pc = null; this.reconnectAttempts = 0; this.onDataChannel = null; this.iceQueue = [];
  }
  async connect() {
    return new Promise((resolve, reject) => {
      this.ws = new WebSocket(this.signalingUrl);
      this.ws.onopen = () => console.log('Signaling connected');
      this.ws.onmessage = (e) => this.handleMessage(JSON.parse(e.data), resolve);
      this.ws.onclose = () => this.handleReconnect();
      this.ws.onerror = (err) => reject(err);
    });
  }
  createPeerConnection() {
    this.pc = new RTCPeerConnection({ iceServers: CONFIG.STUN_SERVERS, ...CONFIG.ICE_POLICY });
    this.pc.onicecandidate = (e) => { if (e.candidate) this.send({ type: 'icecandidate', candidate: e.candidate }); };
    this.pc.onicegatheringstatechange = () => console.log('ICE state:', this.pc.iceGatheringState);
    this.pc.onconnectionstatechange = () => {
      console.log('Connection state:', this.pc.connectionState);
      if (this.pc.connectionState === 'failed') this.handleReconnect();
    };
    this.pc.ondatachannel = (e) => { if (this.onDataChannel) this.onDataChannel(e.channel); };
    return this.pc;
  }
  async createOffer(targetId) {
    if (!this.pc) this.createPeerConnection();
    const dc = this.pc.createDataChannel('data');
    dc.onopen = () => console.log('Data channel open');
    const offer = await this.pc.createOffer();
    await this.pc.setLocalDescription(offer);
    this.send({ type: 'offer', sdp: offer, targetId }); return offer;
  }
  async handleOffer(msg) {
    if (!this.pc) this.createPeerConnection();
    await this.pc.setRemoteDescription(new RTCSessionDescription(msg.sdp));
    const answer = await this.pc.createAnswer();
    await this.pc.setLocalDescription(answer);
    this.send({ type: 'answer', sdp: answer, targetId: msg.from });
    this.drainIceQueue();
  }
  async handleAnswer(msg) {
    await this.pc.setRemoteDescription(new RTCSessionDescription(msg.sdp));
    this.drainIceQueue();
  }
  async handleIceCandidate(msg) {
    if (this.pc?.remoteDescription) await this.pc.addIceCandidate(new RTCIceCandidate(msg.candidate));
    else this.iceQueue.push(msg.candidate);
  }
  drainIceQueue() {
    while (this.iceQueue.length) {
      const c = this.iceQueue.shift();
      this.pc.addIceCandidate(new RTCIceCandidate(c)).catch(console.error);
    }
  }
  handleMessage(msg, resolve) {
    switch (msg.type) {
      case 'connected': this.send({ type: 'register', peerId: this.peerId }); if (resolve) resolve(msg); break;
      case 'offer': this.handleOffer(msg); break; case 'answer': this.handleAnswer(msg); break;
      case 'icecandidate': this.handleIceCandidate(msg); break; case 'error': console.error('Server error:', msg.code); break;
    }
  }
  send(msg) { if (this.ws?.readyState === WebSocket.OPEN) this.ws.send(JSON.stringify(msg)); }
  handleReconnect() {
    if (this.reconnectAttempts < CONFIG.RECONNECT.MAX_RETRIES) {
      this.reconnectAttempts++;
      console.log(`Reconnecting... (${this.reconnectAttempts}/${CONFIG.RECONNECT.MAX_RETRIES})`);
      setTimeout(() => this.connect().catch(console.error), CONFIG.RECONNECT.DELAY);
    }
  }
  close() { this.pc?.close(); this.ws?.close(); }
}
module.exports = SignalingClient;
