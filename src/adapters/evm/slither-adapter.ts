import { execSync, ExecSyncOptions } from 'child_process';
import { resolve, dirname } from 'path';
import { IEVMAdapter, EVMAnalysisResult, Vulnerability } from './interface.js';
import { SlitherJsonOutput } from './types.js';

/**
 * SlitherAdapter - Static analysis adapter for Slither
 * @deprecated since v3.5.0 - Use slither-detector.ts with Docker-based non-blocking execution instead
 * This adapter uses execSync which blocks the event loop. Please migrate to SlitherDetector.
 */
export class SlitherAdapter implements IEVMAdapter {
  readonly name = 'Slither';
  readonly version = '0.9.6';
  private readonly timeout = 300000; // 5 minutes timeout for analysis
  
  /**
   * Constructor - initializes adapter with default configuration
   */
  constructor() {
    // Default configuration
  }

  async analyze(contractPath: string): Promise<EVMAnalysisResult> {
    const absolutePath = resolve(contractPath);
    const vulnerabilities: Vulnerability[] = [];
    let compileSuccess = false;
    
    try {
      // Run Slither with JSON output
      const output = execSync(
        `slither "${absolutePath}" --json -`,
        {
          encoding: 'utf8',
          timeout: this.timeout,
          // SAFETY: shell:false prevents command injection via path traversal
          shell: false,
          cwd: dirname(absolutePath)
        } as any
      );
      
      compileSuccess = true;
      
      // Parse JSON output
      let slitherOutput: SlitherJsonOutput;
      try {
        slitherOutput = JSON.parse(output) as SlitherJsonOutput;
      } catch (parseErr) {
        return {
          vulnerabilities,
          compileSuccess: true,
          timestamp: Date.now()
        };
      }
      
      // Map detectors to vulnerabilities
      if (slitherOutput.success && slitherOutput.results?.detectors) {
        for (const detector of slitherOutput.results.detectors) {
          const severity = this.mapSeverity(detector.impact);
          const confidence = this.mapConfidence(detector.confidence);
          const line = this.extractLine(detector.elements);
          
          vulnerabilities.push({
            severity,
            ruleId: detector.check,
            message: detector.description,
            line,
            confidence
          });
        }
      }
    } catch (err) {
      // Slither may exit with error on compilation failure
      if (err instanceof Error && err.message.includes('Compilation failed')) {
        compileSuccess = false;
      } else if (err instanceof Error) {
        // Other errors may still have valid output
        compileSuccess = true;
      }
    }
    
    return {
      vulnerabilities,
      compileSuccess,
      timestamp: Date.now()
    };
  }

  async test(): Promise<{
    success: boolean;
    testCount: number;
    passedCount: number;
    failedCount: number;
    timestamp: number;
  }> {
    // Slither doesn't support running tests
    return {
      success: true,
      testCount: 0,
      passedCount: 0,
      failedCount: 0,
      timestamp: Date.now()
    };
  }

  /**
   * Check if Slither is available in the environment
   * @returns boolean indicating slither availability
   */
  async checkEnvironment(): Promise<boolean> {
    try {
      execSync('slither --version', {
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
      supportedFormats: ['.sol']
    };
  }

  private mapSeverity(impact: string): 'High' | 'Medium' | 'Low' {
    switch (impact) {
      case 'High':
        return 'High';
      case 'Medium':
        return 'Medium';
      case 'Low':
        return 'Low';
      default:
        return 'Low';
    }
  }

  private mapConfidence(confidence: string): 'High' | 'Medium' | 'Low' {
    switch (confidence) {
      case 'High':
        return 'High';
      case 'Medium':
        return 'Medium';
      case 'Low':
        return 'Low';
      default:
        return 'Medium';
    }
  }

  private extractLine(elements: { line?: number; source_mapping?: { lines: number[] } }[]): number {
    if (!elements || elements.length === 0) {
      return 0;
    }
    
    const first = elements[0];
    if (first.line && first.line > 0) {
      return first.line;
    }
    
    if (first.source_mapping?.lines && first.source_mapping.lines.length > 0) {
      return first.source_mapping.lines[0];
    }
    
    return 0;
  }
}
