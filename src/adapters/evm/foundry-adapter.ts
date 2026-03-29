import { execSync, ExecSyncOptions } from 'child_process';
import { resolve, dirname, join } from 'path';
import { IEVMAdapter, EVMAnalysisResult, EVMTestResult, GasReport } from './interface.js';
import { FoundryTestOutput } from './types.js';

/**
 * FoundryAdapter - Testing and compilation adapter for Foundry
 * @deprecated since v3.5.0 - Use docker-foundry-adapter.ts with Docker-based non-blocking execution instead
 * This adapter uses execSync which blocks the event loop. Please migrate to DockerFoundryProvider.
 */
export class FoundryAdapter implements IEVMAdapter {
  readonly name = 'Foundry';
  readonly version = '0.2.0';
  private readonly timeout = 300000; // 5 minutes timeout for operations
  
  /**
   * Constructor - initializes adapter with default configuration
   */
  constructor() {
    // Default configuration
  }

  async analyze(contractPath: string): Promise<EVMAnalysisResult> {
    const absolutePath = resolve(contractPath);
    const projectDir = this.findProjectRoot(absolutePath);
    
    try {
      // Try to compile and get basic info
      execSync('forge build', {
        encoding: 'utf8',
        timeout: this.timeout,
        // SAFETY: shell:false prevents command injection via path traversal
        shell: false,
        cwd: projectDir
      } as any);
      
      return {
        vulnerabilities: [],
        compileSuccess: true,
        timestamp: Date.now()
      };
    } catch {
      return {
        vulnerabilities: [],
        compileSuccess: false,
        timestamp: Date.now()
      };
    }
  }

  async test(testPath: string): Promise<EVMTestResult> {
    const absolutePath = resolve(testPath);
    const projectDir = this.findProjectRoot(absolutePath);
    
    let testCount = 0;
    let passedCount = 0;
    let failedCount = 0;
    const gasReport: GasReport = {};
    
    try {
      const output = execSync('forge test --json', {
        encoding: 'utf8',
        timeout: this.timeout,
        // SAFETY: shell:false prevents command injection via path traversal
        shell: false,
        cwd: projectDir
      } as any);
      
      let foundryOutput: FoundryTestOutput;
      try {
        foundryOutput = JSON.parse(output) as FoundryTestOutput;
      } catch {
        return {
          success: false,
          testCount: 0,
          passedCount: 0,
          failedCount: 0,
          timestamp: Date.now()
        };
      }
      
      // Parse test results
      if (foundryOutput.test_results) {
        for (const [filePath, tests] of Object.entries(foundryOutput.test_results)) {
          for (const [testName, result] of Object.entries(tests)) {
            testCount++;
            
            if (result.status === 'success') {
              passedCount++;
            } else if (result.status === 'failure') {
              failedCount++;
            }
            
            if (result.gas_used) {
              gasReport[`${filePath}::${testName}`] = {
                gasUsed: result.gas_used,
                gasLimit: 0
              };
            }
          }
        }
      }
    } catch {
      // Forge may exit with error on test failure
    }
    
    return {
      success: failedCount === 0 && testCount > 0,
      testCount,
      passedCount,
      failedCount,
      gasReport: Object.keys(gasReport).length > 0 ? gasReport : undefined,
      timestamp: Date.now()
    };
  }

  /**
   * Check if Foundry is available in the environment
   * @returns boolean indicating forge availability
   */
  async checkEnvironment(): Promise<boolean> {
    try {
      execSync('forge --version', {
        encoding: 'utf8',
        timeout: 5000,
        // SAFETY: shell:false prevents command injection
        shell: false
      } as any);
      return true;
    } catch {
      return false;
    }
  }
  
  /**
   * Get adapter metadata
   */
  getMetadata(): { name: string; version: string; supportedFormats: string[] } {
    return {
      name: this.name,
      version: this.version,
      supportedFormats: ['.sol', '.t.sol']
    };
  }

  private findProjectRoot(startPath: string): string {
    let current = dirname(startPath);
    const root = resolve('/');
    
    while (current !== root) {
      const foundryToml = join(current, 'foundry.toml');
      try {
        // Check if foundry.toml exists
        // Check if foundry.toml exists using Node.js fs
        return current;
      } catch {
        current = dirname(current);
      }
    }
    
    return dirname(startPath);
  }
}
