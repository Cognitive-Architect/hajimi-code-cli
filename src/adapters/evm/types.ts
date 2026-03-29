/** EVM Adapter Type Definitions - Phase 3.0 */
import { ChildProcess } from 'child_process';

// Phase 3.0 Core Types
export interface IAnvilInstance {
  readonly containerId: string;
  readonly rpcUrl: string;
  readonly chainId: number;
  readonly port: number;
  process: ChildProcess | null;
  stop(): Promise<void>;
  isRunning(): boolean;
}

export interface IAnvilConfig {
  readonly port?: number;
  readonly chainId?: number;
  readonly forkUrl?: string;
  readonly blockTime?: number;
  readonly accounts?: number;
  readonly balance?: string;
}

export interface ISlitherResult {
  readonly contractName: string;
  readonly detector: string;
  readonly impact: 'High' | 'Medium' | 'Low' | 'Informational';
  readonly confidence: 'High' | 'Medium' | 'Low';
  readonly description: string;
  readonly check: string;
}

export interface IVulnerabilitySample {
  readonly contractName: string;
  readonly vulnerability: string;
  readonly severity: 'Critical' | 'High' | 'Medium' | 'Low';
  readonly code: string;
}

export interface IPipelineResult {
  readonly success: boolean;
  readonly phase: 'detect' | 'patch' | 'verify';
  readonly vulnerabilities: ISlitherResult[];
  readonly patchedCode?: string;
  readonly testPass?: boolean;
  readonly error?: string;
  readonly duration: number;
}

export interface IDockerProvider {
  startAnvil(config?: IAnvilConfig): Promise<IAnvilInstance>;
  executeForge(args: string[], cwd?: string): Promise<string>;
  executeSlither(path: string): Promise<ISlitherResult[]>;
}

export interface IHealthStatus {
  readonly docker: boolean;
  readonly port: boolean;
  readonly timestamp: number;
}

// Legacy Types (for backward compatibility with existing adapters)
export interface FoundryTestOutput {
  test_results?: Record<string, Record<string, { status: string; gas_used?: number }>>;
  gas_report?: Record<string, Record<string, { gas_used: number; calls: number }>>;
}

export interface SlitherJsonOutput {
  success: boolean;
  results?: {
    detectors: Array<{
      check: string;
      description: string;
      impact: string;
      confidence: string;
      elements: Array<{ line?: number; source_mapping?: { lines: number[] } }>;
    }>;
  };
}
