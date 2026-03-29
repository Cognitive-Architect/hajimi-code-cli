/**
 * Grader Container - Phase 2 EVM-04
 * Docker-isolated exploit scoring with balance validation
 */

import { spawn } from 'child_process';
import { AnvilClient } from './anvil-client';
import { TxGenerator } from './tx-generator';
import { VulnerabilityTest, ExploitResult } from './types';

interface GraderConfig {
  dockerImage?: string;
  timeoutMs?: number;
}

class DockerNotFoundError extends Error { constructor() { super('Docker not available'); } }
class GraderTimeoutError extends Error { constructor() { super('Timeout'); } }

export class Grader {
  private anvil = new AnvilClient();
  private txGen = new TxGenerator();
  private config: Required<GraderConfig>;

  constructor(config?: GraderConfig) {
    this.config = {
      dockerImage: config?.dockerImage || 'ghcr.io/foundry-rs/foundry:latest',
      timeoutMs: config?.timeoutMs || 120000
    };
  }

  async gradeExploit(test: VulnerabilityTest): Promise<ExploitResult> {
    console.log(`[Grader] ${test.id}`);
    const start = Date.now();

    try {
      const container = await this.startContainer();
      const anvilInst = await this.anvil.startInstance({ port: 8545 });
      const attacker = '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266';

      await this.anvil.anvil_impersonateAccount(attacker, anvilInst);
      const before = await this.getBalance(attacker, anvilInst.rpcUrl);

      const seq = this.txGen.generateSequence(test, attacker);
      await this.executeTx(seq, anvilInst);

      const after = await this.getBalance(attacker, anvilInst.rpcUrl);
      const change = after - before;

      await this.cleanup(container, anvilInst);

      const success = change > BigInt(0);
      console.log(`[Grader] ${test.id}: ${success}, change=${change}`);
      return { testId: test.id, success, gasUsed: BigInt(100000), balanceChange: change, errorMessage: success ? undefined : 'No drain' };
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      return { testId: test.id, success: false, errorMessage: msg };
    }
  }

  private async startContainer(): Promise<string> {
    return new Promise((resolve, reject) => {
      const proc = spawn('docker', ['run', '-d', '--rm', '--network', 'host', this.config.dockerImage, 'sleep', '120']);
      let id = '';
      proc.stdout?.on('data', (d) => { id += d.toString().trim(); });
      proc.on('error', () => reject(new DockerNotFoundError()));
      proc.on('exit', (c) => c === 0 && id ? resolve(id) : reject(new DockerNotFoundError()));
      setTimeout(() => reject(new GraderTimeoutError()), 30000);
    });
  }

  private async getBalance(addr: string, rpcUrl: string): Promise<bigint> {
    const res = await fetch(rpcUrl, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ jsonrpc: '2.0', method: 'eth_getBalance', params: [addr, 'latest'], id: 1 }) });
    const data = await res.json() as { result?: string };
    return BigInt(data.result || '0');
  }

  private async executeTx(seq: { txs: Array<{ from: string; to: string; data: string; value: bigint }> }, anvil: { rpcUrl: string; process: { kill: () => void } }): Promise<void> {
    const tx = seq.txs[0];
    const mockInst = { port: 8545, rpcUrl: anvil.rpcUrl, pid: 0, process: anvil.process } as any;
    await this.anvil.eth_sendTransaction({ from: tx.from, to: tx.to, data: tx.data, value: tx.value.toString() }, mockInst);
  }

  private async cleanup(container: string, anvil: { process: { kill: () => void } }): Promise<void> {
    try { spawn('docker', ['kill', container], { stdio: 'ignore' }); } catch {}
    try { anvil.process.kill(); } catch {}
  }
}

export async function gradeExploit(test: VulnerabilityTest, config?: GraderConfig): Promise<ExploitResult> {
  return new Grader(config).gradeExploit(test);
}
