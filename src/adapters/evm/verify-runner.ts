/** Verify Runner - Phase 3.3 */
import { spawn } from 'child_process';
import { writeFileSync, mkdirSync } from 'fs';
import { resolve, join } from 'path';
import { DOCKER_IMAGE, PIPELINE_TIMEOUTS, LOG_PREFIX } from './constants';

export interface IVerifyResult {
  passed: boolean;
  output: string;
  gasUsed?: number;
}

export class VerifyRunner {
  private testDir = resolve(process.cwd(), '.tmp/verify');

  async verifyContract(code: string, name: string): Promise<IVerifyResult> {
    console.log(`${LOG_PREFIX} Verify: ${name}`);
    this.setupTestEnvironment(code, name);
    return this.executeForgeTest();
  }

  private setupTestEnvironment(code: string, name: string): void {
    mkdirSync(this.testDir, { recursive: true });
    writeFileSync(join(this.testDir, `${name}.sol`), code);
    this.generateFoundryConfig();
    this.generateTestFile(name);
  }

  private generateFoundryConfig(): void {
    const config = '[profile.default]\nsrc = "."\nout = "out"\nlibs = ["lib"]';
    writeFileSync(join(this.testDir, 'foundry.toml'), config);
  }

  private generateTestFile(name: string): void {
    const testCode = `// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\nimport "forge-std/Test.sol";\nimport "./${name}.sol";\ncontract ${name}Test is Test { ${name} target; function setUp() public { target = new ${name}(); } function testDeployment() public view { assert(address(target) != address(0)); } }`;
    writeFileSync(join(this.testDir, `${name}Test.t.sol`), testCode);
  }

  private executeForgeTest(): Promise<IVerifyResult> {
    return new Promise((resolve) => {
      const dockerArgs = [
        'run', '--rm',
        '-v', `${this.testDir}:/workspace`,
        '-w', '/workspace',
        DOCKER_IMAGE.FOUNDRY,
        'forge', 'test', '--json'
      ];
      const proc = spawn('docker', dockerArgs, { stdio: ['ignore', 'pipe', 'pipe'] });
      let stdout = '';
      proc.stdout?.on('data', (d) => { stdout += d.toString(); });
      const timeout = setTimeout(() => {
        proc.kill();
        resolve({ passed: false, output: 'Timeout' });
      }, PIPELINE_TIMEOUTS.VERIFY);
      proc.on('exit', (code) => {
        clearTimeout(timeout);
        resolve({ passed: code === 0, output: stdout });
      });
    });
  }
}
