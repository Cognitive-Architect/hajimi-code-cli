/**
 * Shared types for EVM bench modules
 */

export interface ExploitConfig {
  anvilPath?: string;
  port?: number;
  forkUrl?: string;
  timeoutMs?: number;
}

export interface VulnerabilityTest {
  id: string;
  contractCode: string;
  exploitTemplate: string;
  difficulty: 'easy' | 'medium' | 'hard' | 'critical';
}

export interface Transaction {
  from: string;
  to: string;
  data: string;
  value: bigint;
  gasLimit: bigint;
}

export interface TransactionSequence {
  txs: Transaction[];
  expectedOutcome: 'success' | 'revert' | 'overflow';
}

export interface ExploitResult {
  testId: string;
  success: boolean;
  gasUsed?: bigint;
  balanceChange?: bigint;
  errorMessage?: string;
}
