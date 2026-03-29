/**
 * EVM Adapter Interface
 * Unified interface for EVM security analysis tools
 */

export interface IEVMAdapter {
  readonly name: string;
  readonly version: string;
  
  analyze(contractPath: string): Promise<EVMAnalysisResult>;
  test(testPath: string): Promise<EVMTestResult>;
  checkEnvironment(): Promise<boolean>;
}

export interface EVMAnalysisResult {
  vulnerabilities: Vulnerability[];
  gasEstimate?: number;
  compileSuccess: boolean;
  timestamp: number;
}

export interface Vulnerability {
  severity: 'High' | 'Medium' | 'Low';
  ruleId: string;
  message: string;
  line: number;
  confidence: 'High' | 'Medium' | 'Low';
}

export interface EVMTestResult {
  success: boolean;
  testCount: number;
  passedCount: number;
  failedCount: number;
  gasReport?: GasReport;
  timestamp: number;
}

export interface GasReport {
  [testName: string]: {
    gasUsed: number;
    gasLimit: number;
  };
}
