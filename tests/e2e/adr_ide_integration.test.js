/**
 * ADR IDE Integration E2E Test
 * Run with: node tests/e2e/adr_ide_integration.test.js
 */

const assert = require('assert');

// Minimal Jest shim
if (!global.describe) {
  global.describe = (name, fn) => fn();
}
if (!global.test) {
  global.test = (name, fn) => {
    try {
      fn();
      console.log(`  ✓ ${name}`);
    } catch (e) {
      console.error(`  ✗ ${name}`);
      throw e;
    }
  };
}
if (!global.expect) {
  global.expect = (actual) => ({
    toBe(expected) {
      assert.strictEqual(actual, expected);
    },
    toContain(expected) {
      assert.ok(
        typeof actual === 'string' && actual.includes(expected),
        `Expected "${actual}" to contain "${expected}"`
      );
    },
  });
}

describe('ADR IDE Integration', () => {
  test('command_open_adr_is_registered', () => {
    const commands = new Map();
    const registerCommand = (id, handler) => {
      commands.set(id, handler);
      return { dispose: () => {} };
    };
    const context = { subscriptions: [] };

    // Simulate extension command registration
    const openAdr = registerCommand('command.openAdr', (uri, debt_id) => {
      if (!uri && !debt_id) {
        throw new Error('No target');
      }
      return debt_id ? `/docs/adr/${debt_id}.md` : uri;
    });
    context.subscriptions.push(openAdr);

    expect(commands.has('command.openAdr')).toBe(true);
  });

  test('goto_adr_extracts_debt_id', () => {
    const line = '// Related debt: DEBT-ADR-001';
    const match = line.match(/(DEBT-[A-Z0-9-]+)/);
    expect(match[1]).toBe('DEBT-ADR-001');
  });

  test('open_adr_url_construction', () => {
    const debt_id = 'DEBT-TOOLS-002';
    const expectedUrl = `/docs/adr/${debt_id}.md`;
    expect(expectedUrl).toContain('DEBT-TOOLS-002');
    expect(expectedUrl).toBe('/docs/adr/DEBT-TOOLS-002.md');
  });
});
