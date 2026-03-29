/** Slither Detector - Phase 3.2 */
import { spawn } from 'child_process';
import { writeFileSync, mkdirSync, rmSync } from 'fs';
import { resolve, join } from 'path';
import { ISlitherResult } from './types';
import { DOCKER_IMAGE, PIPELINE_TIMEOUTS, LOG_PREFIX } from './constants';
import { EVMErrorCode, getErrorMessage } from './errors';

export interface ISlitherOptions {
  contractCode: string;
  contractName: string;
  timeout?: number;
}

export class SlitherDetector {
  private tempDir: string;

  constructor() {
    this.tempDir = resolve(process.cwd(), '.tmp/slither');
  }

  async runSlither(options: ISlitherOptions): Promise<ISlitherResult[]> {
    console.log(`${LOG_PREFIX} Running Slither on ${options.contractName}...`);
    
    mkdirSync(this.tempDir, { recursive: true });
    const contractPath = join(this.tempDir, `${options.contractName}.sol`);
    writeFileSync(contractPath, options.contractCode);

    try {
      const results = await this.executeDocker(contractPath, options.contractName, options.timeout);
      console.log(`${LOG_PREFIX} Slither found ${results.length} issues`);
      return results;
    } finally {
      try { rmSync(contractPath); } catch {}
    }
  }

  private executeDocker(contractPath: string, contractName: string, timeout = PIPELINE_TIMEOUTS.DETECT): Promise<ISlitherResult[]> {
    return new Promise((resolve, reject) => {
      const workDir = process.cwd();
      const relPath = contractPath.replace(workDir, '/workspace');
      const args = [
        'run', '--rm', 
        '-v', `${workDir}:/workspace`,
        '-w', '/workspace',
        DOCKER_IMAGE.SLITHER,
        'slither', relPath, '--json', '-', '--filter-paths', 'node_modules'
      ];

      const proc = spawn('docker', args, { stdio: ['ignore', 'pipe', 'pipe'] });
      let stdout = '', stderr = '';
      proc.stdout?.on('data', (d) => { stdout += d.toString(); });
      proc.stderr?.on('data', (d) => { stderr += d.toString(); });

      const to = setTimeout(() => {
        proc.kill();
        reject(new Error(getErrorMessage(EVMErrorCode.SlitherError)));
      }, timeout);

      proc.on('exit', (code) => {
        clearTimeout(to);
        if (stderr?.includes('Compilation failed') || stderr?.includes('Invalid solc')) {
          reject(new Error(`${getErrorMessage(EVMErrorCode.SlitherError)}: Compilation failed`));
          return;
        }
        try {
          const parsed = JSON.parse(stdout);
          const detectors = parsed.results?.detectors || [];
          const results: ISlitherResult[] = detectors.map((d: Record<string, unknown>) => ({
            contractName,
            detector: String(d.check || ''),
            impact: this.mapImpact(String(d.impact)),
            confidence: this.mapConfidence(String(d.confidence)),
            description: String(d.description || ''),
            check: String(d.check || '')
          }));
          resolve(results);
        } catch {
          resolve([]);
        }
      });
    });
  }

  private mapImpact(impact: string): ISlitherResult['impact'] {
    if (impact === 'High') return 'High';
    if (impact === 'Medium') return 'Medium';
    if (impact === 'Low') return 'Low';
    return 'Informational';
  }

  private mapConfidence(conf: string): ISlitherResult['confidence'] {
    if (conf === 'High') return 'High';
    if (conf === 'Medium') return 'Medium';
    return 'Low';
  }
}

export async function runSlither(contractCode: string, contractName: string): Promise<ISlitherResult[]> {
  const detector = new SlitherDetector();
  return detector.runSlither({ contractCode, contractName });
}
