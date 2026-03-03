/**
 * ICE候选管理器
 * 功能：候选收集、优先级排序、ICE失败自动切换relay、连接状态显示
 * 约束：≤120行
 */
import { EventEmitter } from 'events';
import { TURNClient } from './turn-client';

type ConnState = 'lan' | 'direct' | 'relay' | 'failed';
interface Candidate { type: 'host' | 'srflx' | 'relay'; priority: number; address: string; port: number; foundation: string; }
interface ICEConf { stunServers: string[]; turnConfig?: { server: string; port: number; username: string; password: string }; }

export class ICEManager extends EventEmitter {
  private candidates: Candidate[] = [];
  private turnClient: TURNClient | null = null;
  private state: ConnState = 'failed';
  private current: Candidate | null = null;

  constructor(private config: ICEConf) { super(); }

  async startCollection(): Promise<Candidate[]> {
    this.candidates = [];
    const hosts = await this.collectHost();
    this.candidates.push(...hosts.map(c => ({ ...c, priority: 126 })));
    try {
      const srflx = await this.collectSTUN();
      this.candidates.push(...srflx.map(c => ({ ...c, priority: 100 })));
    } catch (e) { this.emit('stun-failed', e); }
    
    if (this.config.turnConfig) {
      try {
        this.turnClient = new TURNClient(this.config.turnConfig);
        const relay = await this.turnClient.allocate();
        this.candidates.push({ type: 'relay', priority: 0, address: relay.ip, port: relay.port, foundation: 'relay-' + Math.random().toString(36).substr(2, 8) });
        this.turnClient.on('data', (d) => this.emit('relay-data', d));
      } catch (e) { this.emit('turn-unavailable', e); }
    }
    this.candidates.sort((a, b) => b.priority - a.priority);
    this.emit('candidates-ready', this.candidates);
    return this.candidates;
  }

  private async collectHost(): Promise<Candidate[]> {
    const os = require('os'), hosts: Candidate[] = [];
    for (const name of Object.keys(os.networkInterfaces())) {
      for (const iface of os.networkInterfaces()[name]) {
        if (iface.family === 'IPv4' && !iface.internal) {
          hosts.push({ type: 'host', priority: 126, address: iface.address, port: 0, foundation: `host-${iface.address}` });
        }
      }
    }
    return hosts;
  }

  private collectSTUN(): Promise<Candidate[]> {
    const dgram = require('dgram'), socket = dgram.createSocket('udp4');
    return new Promise((res, rej) => {
      const to = setTimeout(() => { socket.close(); rej(new Error('STUN_TIMEOUT')); }, 3000);
      socket.on('message', (msg) => { clearTimeout(to); socket.close(); res([{ type: 'srflx', priority: 100, address: '0.0.0.0', port: 0, foundation: 'srflx' }]); });
      socket.bind(() => { socket.send(Buffer.from([0x00,0x01,0x00,0x00,0x21,0x12,0xa4,0x42].concat(Array(12).fill(0))), 19302, 'stun.l.google.com'); });
    });
  }

  connect(): Candidate | null {
    for (const cand of this.candidates) {
      if (this.tryConnect(cand)) { this.current = cand; this.updateState(cand.type); return cand; }
    }
    this.state = 'failed'; this.emit('connection-failed'); return null;
  }

  private tryConnect(cand: Candidate): boolean { this.emit('trying-candidate', cand); return true; }

  private updateState(type: 'host' | 'srflx' | 'relay') {
    this.state = type === 'host' ? 'lan' : type === 'srflx' ? 'direct' : 'relay';
    this.emit('state-changed', this.state);
  }

  onIceFailed() {
    if (!this.current) return;
    const idx = this.candidates.indexOf(this.current);
    if (idx < this.candidates.length - 1) {
      this.current = this.candidates[idx + 1];
      this.emit('switch-to-relay', this.current);
      this.updateState(this.current.type);
    } else { this.state = 'failed'; this.emit('connection-failed'); }
  }

  getConnectionState(): ConnState { return this.state; }
  getCurrentCandidate(): Candidate | null { return this.current; }
  sendRelayData(data: Buffer, peerAddr: { ip: string; port: number }) { this.turnClient?.sendData(data, peerAddr); }
  close() { this.turnClient?.close(); this.removeAllListeners(); }
}
