/**
 * P2PQueueDB - LevelDB持久化队列 (DEBT-P2P-004清偿)
 * 约束: ≤120行 | level@8.x | 自动重建
 */
import { Level } from 'level';
import * as crypto from 'crypto';
import * as path from 'path';
import * as os from 'os';

interface QueueEntry { id: string; op: any; seq: number; }

export class P2PQueueDB {
  private db: Level<string, string> | null = null;
  private dbPath: string;
  private seqKey = '__seq__';
  private isCorrupted = false;
  private encryptionKey?: Buffer;

  constructor(options: { path?: string; encryptionKey?: string } = {}) {
    this.dbPath = options.path || path.join(os.homedir(), '.hajimi', 'p2p-queue');
    if (options.encryptionKey) this.encryptionKey = crypto.scryptSync(options.encryptionKey, 'salt', 32);
  }

  async open(): Promise<void> {
    try {
      this.db = new Level(this.dbPath, { valueEncoding: 'utf8' });
      await this.db.open(); this.isCorrupted = false;
    } catch (err: any) {
      if (err.code === 'LEVEL_DATABASE_NOT_OPEN' || err.message?.includes('Corruption')) await this.rebuild();
      else throw err;
    }
  }

  async close(): Promise<void> { if (this.db) { await this.db.close(); this.db = null; } }

  private async rebuild(): Promise<void> {
    const fs = await import('fs/promises');
    await fs.rm(this.dbPath, { recursive: true, force: true });
    this.db = new Level(this.dbPath, { valueEncoding: 'utf8' });
    await this.db.open(); this.isCorrupted = true;
  }

  private encrypt(data: string): string {
    if (!this.encryptionKey) return data;
    const iv = crypto.randomBytes(16);
    const cipher = crypto.createCipheriv('aes-256-gcm', this.encryptionKey, iv);
    const enc = cipher.update(data, 'utf8', 'hex') + cipher.final('hex');
    return iv.toString('hex') + ':' + cipher.getAuthTag().toString('hex') + ':' + enc;
  }

  private decrypt(data: string): string {
    if (!this.encryptionKey) return data;
    const [ivHex, authHex, enc] = data.split(':');
    const decipher = crypto.createDecipheriv('aes-256-gcm', this.encryptionKey, Buffer.from(ivHex, 'hex'));
    decipher.setAuthTag(Buffer.from(authHex, 'hex'));
    return decipher.update(enc, 'hex', 'utf8') + decipher.final('utf8');
  }

  async push(op: any): Promise<void> {
    if (!this.db) throw new Error('DB not open');
    const seq = await this.incSeq();
    const entry: QueueEntry = { id: crypto.randomUUID(), op, seq };
    await this.db.put(String(seq), this.encrypt(JSON.stringify(entry)));
  }

  private async incSeq(): Promise<number> {
    if (!this.db) throw new Error('DB not open');
    try { const v = parseInt(await this.db.get(this.seqKey), 10) + 1; await this.db.put(this.seqKey, String(v)); return v; }
    catch { await this.db.put(this.seqKey, '1'); return 1; }
  }

  async pop(): Promise<any | null> {
    if (!this.db) throw new Error('DB not open');
    const keys: string[] = [];
    for await (const k of this.db.keys({ gte: '0', lt: '~' })) if (k !== this.seqKey) keys.push(k);
    if (keys.length === 0) return null;
    keys.sort((a, b) => parseInt(a) - parseInt(b));
    const raw = await this.db.get(keys[0]); await this.db.del(keys[0]);
    return JSON.parse(this.decrypt(raw)).op;
  }

  async peek(): Promise<any | null> {
    if (!this.db) throw new Error('DB not open');
    const keys: string[] = [];
    for await (const k of this.db.keys({ gte: '0', lt: '~' })) if (k !== this.seqKey) keys.push(k);
    if (keys.length === 0) return null;
    keys.sort((a, b) => parseInt(a) - parseInt(b));
    return JSON.parse(this.decrypt(await this.db.get(keys[0]))).op;
  }

  async clear(): Promise<void> {
    if (!this.db) throw new Error('DB not open');
    const batch = this.db.batch();
    for await (const k of this.db.keys()) batch.del(k);
    await batch.write(); await this.db.put(this.seqKey, '0');
  }

  async getAll(): Promise<any[]> {
    if (!this.db) throw new Error('DB not open');
    const ops: any[] = [];
    for await (const [k, v] of this.db.iterator()) { if (k === this.seqKey) continue; ops.push(JSON.parse(this.decrypt(v)).op); }
    return ops.sort((a, b) => a.timestamp - b.timestamp);
  }

  async size(): Promise<number> {
    let c = 0;
    for await (const k of this.db!.keys()) if (k !== this.seqKey) c++;
    return c;
  }

  wasCorrupted(): boolean { return this.isCorrupted; }
}
