/**
 * Anvil Client - Phase 2 EVM-02
 * JSON-RPC 2.0 client with connection pool
 */

import { spawn, ChildProcess } from 'child_process';
import { ExploitConfig } from './types';

interface AnvilInstance { process: ChildProcess; port: number; rpcUrl: string; pid: number; }
interface RPCRequest { jsonrpc: '2.0'; method: string; params: unknown[]; id: number; }

class AnvilNotFoundError extends Error { constructor() { super('Anvil not found. Install: cargo install foundry'); } }
class ConnectionTimeoutError extends Error { constructor() { super('Connection timeout'); } }
class RPCError extends Error { constructor(code: number, msg: string) { super(msg); } }

export class AnvilClient {
  private pool: AnvilInstance[] = [];
  private isConnecting = false;
  private anvilPath: string;

  constructor(config?: { anvilPath?: string }) {
    this.anvilPath = config?.anvilPath || process.env.ANVIL_PATH || 'anvil';
  }

  async startInstance(config?: { port?: number; forkUrl?: string }): Promise<AnvilInstance> {
    if (this.isConnecting) throw new Error('Connection in progress');
    this.isConnecting = true;

    try {
      const port = config?.port || 8545 + Math.floor(Math.random() * 1000);
      const args = ['--port', String(port), '--host', '0.0.0.0'];
      if (config?.forkUrl) args.push('--fork-url', config.forkUrl);

      console.log(`[Anvil] Starting on port ${port}...`);
      const proc = spawn(this.anvilPath, args, { stdio: ['ignore', 'pipe', 'pipe'] });
      if (!proc.pid) throw new AnvilNotFoundError();

      const inst: AnvilInstance = { process: proc, port, rpcUrl: `http://127.0.0.1:${port}`, pid: proc.pid };
      await this.waitForReady(inst);
      this.pool.push(inst);
      console.log(`[Anvil] Ready at ${inst.rpcUrl}`);
      return inst;
    } finally { this.isConnecting = false; }
  }

  async anvil_impersonateAccount(addr: string, inst?: AnvilInstance): Promise<void> {
    await this.rpcCall('anvil_impersonateAccount', [addr], inst);
  }

  async eth_sendTransaction(tx: Record<string, unknown>, inst?: AnvilInstance): Promise<string> {
    return this.rpcCall('eth_sendTransaction', [tx], inst) as Promise<string>;
  }

  async cleanup(): Promise<void> {
    console.log(`[Anvil] Cleanup ${this.pool.length} instances...`);
    for (const inst of this.pool) {
      try { inst.process.kill(); await new Promise(r => setTimeout(r, 500)); } catch {}
    }
    this.pool = [];
  }

  private async rpcCall(method: string, params: unknown[], inst?: AnvilInstance): Promise<unknown> {
    const i = inst || this.pool[0];
    if (!i) throw new Error('No instance');

    const req: RPCRequest = { jsonrpc: '2.0', method, params, id: Date.now() };
    const res = await fetch(i.rpcUrl, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(req) });

    if (!res.ok) throw new RPCError(res.status, `HTTP ${res.status}`);
    const data = await res.json() as { error?: { code: number; message: string }; result?: unknown };
    if (data.error) throw new RPCError(data.error.code, data.error.message);
    return data.result;
  }

  private async waitForReady(inst: AnvilInstance, retries = 30): Promise<void> {
    for (let i = 0; i < retries; i++) {
      try {
        await fetch(inst.rpcUrl, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ jsonrpc: '2.0', method: 'eth_chainId', params: [], id: 1 }) });
        return;
      } catch { await new Promise(r => setTimeout(r, 300)); }
    }
    throw new ConnectionTimeoutError();
  }
}

export async function createAnvilClient(config?: ExploitConfig): Promise<AnvilClient> {
  const c = new AnvilClient(config);
  await c.startInstance({ port: config?.port, forkUrl: config?.forkUrl });
  return c;
}
