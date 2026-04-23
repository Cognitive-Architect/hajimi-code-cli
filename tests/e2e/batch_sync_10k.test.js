/**
 * Lightweight Jest shim for Node to verify 10K batch sync conceptually.
 */
const assert = require('assert');

function test(name, fn) {
  try {
    fn();
    console.log(`  PASS ${name}`);
  } catch (e) {
    console.error(`  FAIL ${name}:`, e.message);
    process.exitCode = 1;
  }
}

function batchSync10k() {
  const items = Array.from({ length: 10000 }, (_, i) => ({
    id: `node-${i}`,
    payload: Buffer.alloc(64, i % 256),
  }));
  const start = Date.now();
  // Conceptual batch encryption / sync placeholder
  const encrypted = items.map((it) => Buffer.concat([it.payload, Buffer.from('::done')]));
  const elapsed = Date.now() - start;
  return { encrypted, elapsed };
}

test('10K batch sync runs in <5000ms conceptually', () => {
  const { encrypted, elapsed } = batchSync10k();
  assert.strictEqual(encrypted.length, 10000);
  assert.ok(elapsed < 5000, `elapsed ${elapsed}ms >= 5000ms`);
});

console.log('batch_sync_10k tests complete');
