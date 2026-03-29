/**
 * TURN客户端 - RFC 5766实现
 * 功能：分配relay地址、转发数据、401挑战响应处理
 * 约束：≤130行
 */
import { createSocket, RemoteInfo } from 'dgram';
import { createHmac, randomBytes } from 'crypto';
import { EventEmitter } from 'events';

const STUN_MAGIC = 0x2112a442;
const MSG_ALLOCATE = 0x003, MSG_DATA = 0x007;
const ATTR_USER = 0x006, ATTR_REALM = 0x014, ATTR_NONCE = 0x015;
const ATTR_INTEGRITY = 0x008, ATTR_XOR_RELAY = 0x016;

interface TURNConfig { server: string; port: number; username: string; password: string; }
interface RelayAddr { ip: string; port: number; }

export class TURNClient extends EventEmitter {
  private socket = createSocket('udp4');
  private realm = ''; private nonce = '';
  private relayAddr: RelayAddr | null = null;
  private retryCount = 0; private maxRetries = 3;

  constructor(private config: TURNConfig) {
    super();
    this.socket.on('message', (msg, rinfo) => this.handleMessage(msg, rinfo));
    this.socket.on('error', (err) => this.emit('error', err));
  }

  async allocate(): Promise<RelayAddr> {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('TURN_TIMEOUT')), 5000);
      this.once('allocated', (addr) => { clearTimeout(timeout); resolve(addr); });
      this.once('error', (err) => { clearTimeout(timeout); reject(err); });
      this.sendAllocate();
    });
  }

  private sendAllocate() {
    const attrs = this.buildAttributes();
    const msg = this.buildSTUN(MSG_ALLOCATE, attrs);
    this.socket.send(msg, this.config.port, this.config.server);
  }

  private buildAttributes(): Buffer[] {
    const attrs: Buffer[] = [];
    attrs.push(this.strAttr(ATTR_USER, this.config.username));
    if (this.realm) attrs.push(this.strAttr(ATTR_REALM, this.realm));
    if (this.nonce) attrs.push(this.strAttr(ATTR_NONCE, this.nonce));
    if (this.nonce) attrs.push(this.integrityAttr());
    return attrs;
  }

  private strAttr(type: number, val: string): Buffer {
    const buf = Buffer.from(val, 'utf8');
    const pad = (4 - (buf.length % 4)) % 4;
    const attr = Buffer.allocUnsafe(4 + buf.length + pad);
    attr.writeUInt16BE(type, 0); attr.writeUInt16BE(buf.length, 2);
    buf.copy(attr, 4); return attr;
  }

  private computeKey(): string { return `${this.config.username}:${this.realm}:${this.config.password}`; }

  private integrityAttr(): Buffer {
    const key = createHmac('sha1', 'hajimi-turn').update(this.computeKey()).digest();
    const result = Buffer.allocUnsafe(24);
    result.writeUInt16BE(ATTR_INTEGRITY, 0); result.writeUInt16BE(20, 2);
    createHmac('sha1', key).update(result.slice(0, 4)).digest().copy(result, 4);
    return result;
  }

  private buildSTUN(type: number, attrs: Buffer[]): Buffer {
    const attrLen = attrs.reduce((s, a) => s + a.length, 0);
    const msg = Buffer.allocUnsafe(20 + attrLen);
    const tid = randomBytes(12);
    msg.writeUInt16BE(type, 0); msg.writeUInt16BE(attrLen, 2);
    msg.writeUInt32BE(STUN_MAGIC, 4); tid.copy(msg, 8);
    let off = 20; for (const a of attrs) { a.copy(msg, off); off += a.length; }
    return msg;
  }

  private handleMessage(msg: Buffer, rinfo: RemoteInfo) {
    const type = msg.readUInt16BE(0);
    if (type === 0x111) this.handle401(msg);
    else if (type === 0x113) this.emit('error', new Error('TURN_403_FORBIDDEN'));
    else if (type === 0x103) this.parseAllocate(msg);
    else if (type === MSG_DATA) this.emit('data', msg.slice(24));
  }

  private handle401(msg: Buffer) {
    let off = 20;
    while (off < msg.length) {
      const t = msg.readUInt16BE(off), l = msg.readUInt16BE(off + 2);
      if (t === ATTR_REALM) this.realm = msg.slice(off + 4, off + 4 + l).toString();
      if (t === ATTR_NONCE) this.nonce = msg.slice(off + 4, off + 4 + l).toString();
      off += 4 + l + ((4 - (l % 4)) % 4);
    }
    if (this.retryCount < this.maxRetries) {
      setTimeout(() => { this.retryCount++; this.sendAllocate(); }, 1000 * Math.pow(2, this.retryCount));
    } else this.emit('error', new Error('TURN_AUTH_FAILED'));
  }

  private parseAllocate(msg: Buffer) {
    let off = 20;
    while (off < msg.length) {
      const t = msg.readUInt16BE(off), l = msg.readUInt16BE(off + 2);
      if (t === ATTR_XOR_RELAY) {
        const port = msg.readUInt16BE(off + 6) ^ (STUN_MAGIC >> 16);
        const ip = [0,1,2,3].map(i => msg[off+8+i] ^ ((STUN_MAGIC >> (24-i*8)) & 0xff)).join('.');
        this.relayAddr = { ip, port }; this.emit('allocated', this.relayAddr);
      }
      off += 4 + l + ((4 - (l % 4)) % 4);
    }
  }

  sendData(data: Buffer, peerAddr: { ip: string; port: number }) {
    if (!this.relayAddr) throw new Error('TURN_NOT_ALLOCATED');
    const peerAttr = Buffer.allocUnsafe(8);
    peerAttr.writeUInt16BE(0x012, 0); peerAttr.writeUInt16BE(8, 2);
    peerAttr.writeUInt16BE(peerAddr.port ^ (STUN_MAGIC >> 16), 4);
    const ipParts = peerAddr.ip.split('.').map(Number);
    for (let i = 0; i < 4; i++) peerAttr[6 + i] = ipParts[i] ^ ((STUN_MAGIC >> (24 - i * 8)) & 0xff);
    const msg = this.buildSTUN(MSG_DATA, [peerAttr, Buffer.concat([Buffer.alloc(4), data])]);
    this.socket.send(msg, this.config.port, this.config.server);
  }

  close() { this.socket.close(); this.removeAllListeners(); }
}
