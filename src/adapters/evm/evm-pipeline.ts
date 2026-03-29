/** EVM Pipeline - Phase 3.3 */
import { IPipelineResult, ISlitherResult } from './types';
import { PIPELINE_TIMEOUTS, LOG_PREFIX } from './constants';
import { SlitherDetector } from './slither-detector';
import { PatchGenerator } from './patch-generator';
import { VerifyRunner } from './verify-runner';
import { ParsedVulnerability, Severity } from './slither-parser';

export interface IPipelineConfig {
  contractCode: string;
  contractName: string;
  enablePatch?: boolean;
  enableVerify?: boolean;
}

export class EVMPipeline {
  private slither = new SlitherDetector();
  private patcher = new PatchGenerator();
  private verifier = new VerifyRunner();

  async runPipeline(config: IPipelineConfig): Promise<IPipelineResult> {
    const start = Date.now();
    console.log(`${LOG_PREFIX} [Pipeline] Starting: ${config.contractName}`);

    try {
      const vulns = await this.detectPhase(config);
      if (vulns.length === 0) {
        console.log(`${LOG_PREFIX} [Pipeline] No vulnerabilities found`);
        return { success: true, phase: 'detect', vulnerabilities: [], duration: Date.now() - start };
      }

      const patched = config.enablePatch !== false ? await this.patchPhase(config, vulns) : undefined;
      const testPass = config.enableVerify !== false && patched ? await this.verifyPhase(config, patched) : false;

      console.log(`${LOG_PREFIX} [Pipeline] Complete: ${vulns.length} vulns, test=${testPass}`);
      return {
        success: true,
        phase: 'verify',
        vulnerabilities: vulns,
        patchedCode: patched,
        testPass,
        duration: Date.now() - start
      };
    } catch (error: unknown) {
      const msg = error instanceof Error ? error.message : String(error);
      console.error(`${LOG_PREFIX} [Pipeline] Failed:`, msg);
      return { success: false, phase: 'detect', vulnerabilities: [], error: msg, duration: Date.now() - start };
    }
  }

  private async detectPhase(config: IPipelineConfig): Promise<ISlitherResult[]> {
    console.log(`${LOG_PREFIX} [Phase 1/3] Detecting vulnerabilities...`);
    const results = await this.slither.runSlither({
      contractCode: config.contractCode,
      contractName: config.contractName,
      timeout: PIPELINE_TIMEOUTS.DETECT
    });
    console.log(`${LOG_PREFIX} [Phase 1/3] Found ${results.length} vulnerabilities`);
    return results;
  }

  private async patchPhase(config: IPipelineConfig, vulns: ISlitherResult[]): Promise<string> {
    console.log(`${LOG_PREFIX} [Phase 2/3] Generating patches...`);
    const parsed: ParsedVulnerability[] = vulns.map(v => ({
      contract: v.contractName,
      vulnerability: v.detector,
      line: 1,
      severity: v.impact === 'High' ? Severity.High : Severity.Medium,
      description: v.description
    }));
    const patches = this.patcher.generatePatch(config.contractCode, parsed);
    let code = config.contractCode;
    for (const p of patches) {
      code = code.replace(p.original, p.patched);
    }
    console.log(`${LOG_PREFIX} [Phase 2/3] Applied ${patches.length} patches`);
    return code;
  }

  private async verifyPhase(config: IPipelineConfig, code: string): Promise<boolean> {
    console.log(`${LOG_PREFIX} [Phase 3/3] Verifying patched contract...`);
    const result = await this.verifier.verifyContract(code, config.contractName);
    console.log(`${LOG_PREFIX} [Phase 3/3] Verification ${result.passed ? 'PASSED' : 'FAILED'}`);
    return result.passed;
  }
}

/**
 * Execute full Detect-Patch-Verify pipeline
 * @param config Pipeline configuration
 * @returns Pipeline execution result
 */
export async function runPipeline(config: IPipelineConfig): Promise<IPipelineResult> {
  const pipeline = new EVMPipeline();
  return pipeline.runPipeline(config);
}
