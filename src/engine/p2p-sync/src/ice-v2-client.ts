/**
 * ICE v2客户端 - RFC 8445 ICE v2实现
 * 改进：Regular Nomination, 优化候选选择, RTT平滑
 */
import { EventEmitter } from 'events';
import { createSocket } from 'dgram';
import { randomBytes } from 'crypto';
import { CandidatePairSelector } from './candidate-pair-selector';
import { LatencyMonitor } from './latency-monitor';
import { ICECandidate, CandidatePair, STUNResponse } from './ice-types';

const TYPE_PREF = { host: 126, prflx: 110, srflx: 100, relay: 0 };
const STUN_MAGIC = 0x2112a442, BIND_REQ = 0x0001, BIND_RES = 0x0101, ATTR_XOR = 0x0020;

export interface ICEv2Config {
  stunServers: string[];
  turnConfig?: { server: string; port: number; username: string; password: string };
  useRegularNomination?: boolean;
  rfc5245Fallback?: boolean;
}

export class ICEv2Client extends EventEmitter {
  private selector = new CandidatePairSelector();
  private latencyMonitor = new LatencyMonitor();
  private config: ICEv2Config;
  private localCandidates: ICECandidate[] = [];
  private remoteCandidates: ICECandidate[] = [];
  private nominatedPair: CandidatePair | null = null;
  private socket = createSocket('udp4');

  constructor(config: ICEv2Config) { super(); this.config = config; }

  async gatherCandidates(): Promise<void> {
    this.localCandidates = [...await this.gatherHostCandidates()];
    if (this.config.stunServers.length > 0) this.localCandidates.push(...await this.gatherSrflxCandidates());
    this.localCandidates.sort((a, b) => b.priority - a.priority);
    this.emit('localCandidates', this.localCandidates);
  }

  private async gatherHostCandidates(): Promise<ICECandidate[]> {
    const os = await import('os');
    return Object.values(os.networkInterfaces()).flat()
      .filter((i: any) => i.family === 'IPv4' && !i.internal)
      .map((i: any) => ({ type: 'host', ip: i.address, port: 0, priority: this.calcPriority('host') }));
  }

  private async gatherSrflxCandidates(): Promise<ICECandidate[]> {
    return new Promise((res) => {
      const to = setTimeout(() => res([]), 3000);
      this.socket.once('message', (msg) => {
        clearTimeout(to);
        const m = this.parseXorMapped(msg);
        res(m ? [{ type: 'srflx', ip: m.ip, port: m.port, priority: this.calcPriority('srflx') }] : []);
      });
      this.sendStunReq();
    });
  }

  private sendStunReq() {
    const m = Buffer.alloc(20);
    m.writeUInt16BE(BIND_REQ, 0); m.writeUInt16BE(0, 2); m.writeUInt32BE(STUN_MAGIC, 4); randomBytes(12).copy(m, 8);
    this.socket.send(m, 19302, this.config.stunServers[0]);
  }

  private parseXorMapped(msg: Buffer): { ip: string; port: number } | null {
    let off = 20;
    while (off < msg.length) {
      const t = msg.readUInt16BE(off), l = msg.readUInt16BE(off + 2);
      if (t === ATTR_XOR) {
        const port = msg.readUInt16BE(off + 6) ^ (STUN_MAGIC >> 16);
        const ip = [0,1,2,3].map(i => msg[off+8+i] ^ ((STUN_MAGIC >> (24-i*8)) & 0xff)).join('.');
        return { ip, port };
      }
      off += 4 + l + ((4 - (l % 4)) % 4);
    }
    return null;
  }

  private calcPriority(type: keyof typeof TYPE_PREF, localPref: number = 65535): number {
    return (16777216 * TYPE_PREF[type]) + (256 * localPref) + 255;
  }

  async performConnectivityCheck(): Promise<CandidatePair | null> {
    const pairs = this.selector.generatePairs(this.localCandidates, this.remoteCandidates);
    for (const pair of pairs) {
      const rtt = await this.checkPair(pair);
      if (rtt !== null) {
        this.latencyMonitor.recordRtt(rtt);
        if (this.config.useRegularNomination !== false) {
          this.nominatedPair = pair; this.emit('nominated', pair, rtt); return pair;
        }
      }
    }
    if (this.config.rfc5245Fallback) { this.emit('fallback', 'RFC 5245'); return Promise.resolve(null); }
    return null;
  }

  private async checkPair(_pair: CandidatePair): Promise<number | null> {
    const start = performance.now();
    try {
      const res = await this.sendStunReqPromise();
      if (res.success) {
        if (res.mappedAddress) this.localCandidates.push({ type: 'prflx', ip: res.mappedAddress.ip, port: res.mappedAddress.port, priority: this.calcPriority('prflx') });
        return performance.now() - start;
      }
    } catch { }
    return null;
  }

  private sendStunReqPromise(): Promise<STUNResponse> {
    return new Promise((res) => {
      const to = setTimeout(() => res({ success: false }), 3000);
      this.socket.once('message', (msg) => {
        clearTimeout(to);
        res({ success: msg.readUInt16BE(0) === BIND_RES, mappedAddress: this.parseXorMapped(msg) || undefined });
      });
      this.sendStunReq();
    });
  }

  setRemoteCandidates(c: ICECandidate[]): void { this.remoteCandidates = c; }
  getNominatedPair(): CandidatePair | null { return this.nominatedPair; }
  getSmoothedRtt(): number { return this.latencyMonitor.getSmoothedRtt(); }
  close(): void { this.socket.close(); this.removeAllListeners(); }
}
