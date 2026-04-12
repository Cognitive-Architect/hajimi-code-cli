/** EVM Pipeline E2E Tests - Phase 3.3 */
import { describe, it, afterAll } from 'node:test';
import { strict as assert } from 'node:assert';
import { EVMPipeline, runPipeline } from '../../src/adapters/evm/evm-pipeline';
import { loadVulnSamples, getSampleByName, filterBySeverity } from '../../src/adapters/evm/vuln-loader';
import { SlitherDetector } from '../../src/adapters/evm/slither-detector';
import { DockerFoundryProvider } from '../../src/adapters/evm/docker-foundry-adapter';
import { checkDockerDaemon } from '../../src/adapters/evm/health-check';
import { parseSlitherJSON, Severity } from '../../src/adapters/evm/slither-parser';
import { PatchGenerator, generatePatch } from '../../src/adapters/evm/patch-generator';
import { VerifyRunner } from '../../src/adapters/evm/verify-runner';
import { ContainerManager } from '../../src/adapters/evm/container-manager';

const cleanupInstances: Array<{ stop?: () => Promise<void> }> = [];

describe('EVM Pipeline E2E Tests', () => {
  afterAll(async () => {
    console.log('[Test] Cleanup: Stopping instances...');
    for (const instance of cleanupInstances) {
      try { await instance.stop?.(); } catch (e) { console.error('[Test] Cleanup error:', e); }
    }
    console.log('[Test] Cleanup complete');
  });

  it('should load vulnerability samples', () => {
    const samples = loadVulnSamples();
    assert.ok(samples.length >= 5, `Expected >=5, got ${samples.length}`);
    console.log(`[TEST] Loaded ${samples.length} samples`);
  });

  it('should filter samples by severity', () => {
    const samples = loadVulnSamples();
    const high = filterBySeverity(samples, 'High');
    const critical = filterBySeverity(samples, 'Critical');
    assert.ok(high.length + critical.length > 0, 'Should have High/Critical');
  });

  it('should find ReentrancyVulnerable sample', () => {
    const sample = getSampleByName('ReentrancyVulnerable');
    assert.ok(sample, 'ReentrancyVulnerable should exist');
    assert.equal(sample?.vulnerability, 'Reentrancy');
  });

  it('should detect Docker daemon status', async () => {
    const isRunning = await checkDockerDaemon();
    console.log(`[TEST] Docker: ${isRunning ? 'running' : 'not running'}`);
    assert.strictEqual(typeof isRunning, 'boolean');
  });

  it('should instantiate all core components', () => {
    assert.ok(new DockerFoundryProvider(), 'DockerFoundryProvider');
    assert.ok(new SlitherDetector(), 'SlitherDetector');
    assert.ok(new EVMPipeline(), 'EVMPipeline');
    assert.ok(new PatchGenerator(), 'PatchGenerator');
    assert.ok(new VerifyRunner(), 'VerifyRunner');
    assert.ok(new ContainerManager(), 'ContainerManager');
  });

  it('should parse Slither JSON output', () => {
    const mock = JSON.stringify({
      results: {
        detectors: [{
          check: 'reentrancy-eth', impact: 'High', confidence: 'High',
          description: 'Reentrancy detected',
          elements: [{ source_mapping: { lines: [10] } }]
        }]
      }
    });
    const parsed = parseSlitherJSON(mock, 'Test');
    assert.ok(parsed.length > 0, 'Should parse');
    assert.equal(parsed[0].vulnerability, 'reentrancy-eth');
    assert.equal(parsed[0].severity, Severity.High);
  });

  it('should generate patches for vulnerabilities', () => {
    const code = 'function withdraw() { (bool s,) = msg.sender.call{value:1}(""); require(s); }';
    const vuln = { contract: 'Test', vulnerability: 'Reentrancy', line: 1, severity: Severity.High, description: 'Reentrancy issue' };
    const patch = generatePatch(code, vuln);
    assert.ok(patch, 'Should generate');
    assert.ok(patch.patched.includes('checks-effects-interactions'));
  });

  it('should run full pipeline', async () => {
    const sample = getSampleByName('ReentrancyVulnerable');
    if (!sample) { console.log('[TEST] SKIP: Sample not found'); return; }
    const dockerRunning = await checkDockerDaemon();
    if (!dockerRunning) { console.log('[TEST] SKIP: Docker not running'); return; }

    const pipeline = new EVMPipeline();
    const result = await pipeline.runPipeline({
      contractCode: sample.code, contractName: sample.contractName, enablePatch: false, enableVerify: false
    });

    assert.ok(result.duration > 0, 'Should have duration');
    assert.ok(Array.isArray(result.vulnerabilities));
    console.log(`[TEST] Pipeline: ${result.duration}ms, ${result.vulnerabilities.length} issues`);
  });

  it('should start and stop Anvil instance', async () => {
    const dockerRunning = await checkDockerDaemon();
    if (!dockerRunning) { console.log('[TEST] SKIP: Docker not running'); return; }

    const provider = new DockerFoundryProvider();
    const instance = await provider.startAnvil({ port: 18555 });
    cleanupInstances.push(instance);
    
    assert.ok(instance.isRunning(), 'Running');
    assert.ok(instance.rpcUrl.includes('18555'), 'Port in URL');
    
    await instance.stop();
    assert.ok(!instance.isRunning(), 'Stopped');
    console.log('[TEST] Anvil start/stop: PASS');
  });

  it('should run pipeline via exported function', async () => {
    const sample = getSampleByName('TimestampVulnerable');
    if (!sample) { console.log('[TEST] SKIP'); return; }
    const dockerRunning = await checkDockerDaemon();
    if (!dockerRunning) { console.log('[TEST] SKIP'); return; }

    const result = await runPipeline({
      contractCode: sample.code, contractName: sample.contractName, enablePatch: false, enableVerify: false
    });

    assert.strictEqual(typeof result.success, 'boolean');
    assert.ok(['detect', 'patch', 'verify'].includes(result.phase));
  });
});
