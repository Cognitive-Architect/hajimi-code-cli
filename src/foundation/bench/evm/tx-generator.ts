/**
 * TX Generator - Phase 2 EVM-03
 * Converts exploit templates to transaction sequences
 */

import { VulnerabilityTest, Transaction, TransactionSequence } from './types';

const TEMPLATE_REGEX = /\{\{([a-z_]+)\}\}/g;

class InvalidTemplateError extends Error { constructor(m: string) { super(`Invalid: ${m}`); } }

export class TxGenerator {
  private impersonatedAddr = '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266';

  generateSequence(test: VulnerabilityTest, contractAddr: string): TransactionSequence {
    console.log(`[TxGen] ${test.id}`);
    const txs = this.parseTemplate(test.exploitTemplate, contractAddr);
    return { txs, expectedOutcome: this.inferOutcome(test.difficulty) };
  }

  private parseTemplate(template: string, contractAddr: string): Transaction[] {
    const txs: Transaction[] = [];
    for (const line of template.split(';')) {
      const t = line.trim();
      if (!t) continue;
      const m = t.match(/^(\w+)\s*\(([^)]*)\)(?:\s*with\s+(\d+)\s*eth)?$/);
      if (!m) throw new InvalidTemplateError(t);
      const [, func, argsStr, eth] = m;
      const args = argsStr.split(',').map(a => a.trim().replace(TEMPLATE_REGEX, contractAddr));
      txs.push({ from: this.impersonatedAddr, to: args[0] || contractAddr, data: this.encodeCalldata(func, args.slice(1)), value: eth ? BigInt(eth) * BigInt(10 ** 18) : BigInt(0), gasLimit: BigInt(100000) });
    }
    return txs;
  }

  private encodeCalldata(func: string, args: string[]): string {
    const sig = `${func}(${args.map(() => 'address').join(',')})`;
    const hash = this.keccak256(sig).slice(0, 10);
    const encoded = args.map(a => a.slice(2).padStart(64, '0')).join('');
    return hash + encoded;
  }

  private keccak256(str: string): string {
    const { createHash } = require('crypto');
    return '0x' + createHash('sha3-256').update(str).digest('hex');
  }

  private inferOutcome(d: string): 'success' | 'revert' | 'overflow' {
    return d === 'critical' ? 'overflow' : d === 'hard' ? 'revert' : 'success';
  }
}

/** Generate sequence convenience function */
export function generateSequence(test: VulnerabilityTest, contractAddr: string): TransactionSequence {
  return new TxGenerator().generateSequence(test, contractAddr);
}

/** Estimate gas for transaction sequence */
export function estimateGas(sequence: TransactionSequence): bigint {
  return sequence.txs.reduce((sum, tx) => sum + tx.gasLimit, BigInt(0));
}
