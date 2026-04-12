/**
 * FoundryAdapter Unit Tests
 * Tests the Foundry testing and compilation adapter functionality
 */

import { describe, it, beforeAll } from 'node:test';
import assert from 'node:assert';
import { FoundryAdapter } from '../../src/adapters/evm/foundry-adapter.js';
import { resolve } from 'path';

describe('FoundryAdapter', () => {
  let adapter: FoundryAdapter;
  const testContract = resolve(process.cwd(), 'test-data/evm/vulnerable-token.sol');

  /**
   * Setup - Initialize adapter before tests
   */
  beforeAll(async () => {
    adapter = new FoundryAdapter();
    const envReady = await adapter.checkEnvironment();
    if (!envReady) {
      console.warn('Warning: Foundry not installed, some tests may fail');
    }
  });

  describe('checkEnvironment', () => {
    it('should return boolean indicating forge availability', async () => {
      const result = await adapter.checkEnvironment();
      assert.strictEqual(typeof result, 'boolean');
    });
  });

  describe('analyze', () => {
    it('should return compile result for contract', async () => {
      const result = await adapter.analyze(testContract);
      
      assert.strictEqual(typeof result.compileSuccess, 'boolean');
      assert.ok(Array.isArray(result.vulnerabilities));
      assert.strictEqual(typeof result.timestamp, 'number');
    });

    it('should handle non-existent contract', async () => {
      const result = await adapter.analyze('/non/existent/contract.sol');
      
      assert.strictEqual(result.compileSuccess, false);
    });
  });

  describe('test', () => {
    it('should attempt to run tests and return result structure', async () => {
      const result = await adapter.test(testContract);
      
      assert.strictEqual(typeof result.success, 'boolean');
      assert.strictEqual(typeof result.testCount, 'number');
      assert.strictEqual(typeof result.passedCount, 'number');
      assert.strictEqual(typeof result.failedCount, 'number');
      assert.strictEqual(typeof result.timestamp, 'number');
      
      // Verify counts are consistent
      assert.ok(result.passedCount + result.failedCount <= result.testCount);
    });

    it('should include gas report when available', async () => {
      const result = await adapter.test(testContract);
      
      if (result.gasReport) {
        for (const [name, gas] of Object.entries(result.gasReport)) {
          assert.strictEqual(typeof name, 'string');
          assert.strictEqual(typeof gas.gasUsed, 'number');
          assert.ok(gas.gasUsed >= 0);
        }
      }
    });
  });

  describe('getMetadata', () => {
    it('should return adapter metadata', () => {
      const metadata = adapter.getMetadata();
      
      assert.strictEqual(metadata.name, 'Foundry');
      assert.strictEqual(metadata.version, '0.2.0');
      assert.ok(Array.isArray(metadata.supportedFormats));
      assert.ok(metadata.supportedFormats.includes('.sol'));
      assert.ok(metadata.supportedFormats.includes('.t.sol'));
    });
  });

  describe('error handling', () => {
    it('should handle empty test path', async () => {
      const result = await adapter.test('');
      
      assert.strictEqual(typeof result.success, 'boolean');
      assert.strictEqual(typeof result.testCount, 'number');
    });

    it('should handle relative paths', async () => {
      const result = await adapter.analyze('test-data/evm/vulnerable-token.sol');
      
      assert.strictEqual(typeof result.compileSuccess, 'boolean');
    });

    it('should handle directory paths gracefully', async () => {
      const result = await adapter.analyze('test-data/evm/');
      
      assert.strictEqual(typeof result.compileSuccess, 'boolean');
    });

    it('should handle non-existent test files', async () => {
      const result = await adapter.test('/non/existent/test.t.sol');
      
      assert.strictEqual(typeof result.success, 'boolean');
      assert.strictEqual(typeof result.testCount, 'number');
    });
  });

  describe('test result consistency', () => {
    it('should have consistent test counts', async () => {
      const result = await adapter.test(testContract);
      
      // passed + failed should not exceed total
      assert.ok(result.passedCount + result.failedCount <= result.testCount);
      
      // All counts should be non-negative
      assert.ok(result.testCount >= 0);
      assert.ok(result.passedCount >= 0);
      assert.ok(result.failedCount >= 0);
    });

    it('should have valid timestamp', async () => {
      const result = await adapter.test(testContract);
      
      assert.strictEqual(typeof result.timestamp, 'number');
      assert.ok(result.timestamp > 0);
      assert.ok(result.timestamp <= Date.now());
    });
  });
});
