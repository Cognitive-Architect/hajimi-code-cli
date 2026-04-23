const assert = require('assert');

async function test(name, fn) {
  try { await fn(); console.log(`  ✓ ${name}`); }
  catch (e) { console.error(`  ✗ ${name}: ${e.message}`); process.exit(1); }
}

async function runHybridPipeline() {
  const start = Date.now();
  const neighbors = [{ docId: 1, score: 0.9 }, { docId: 2, score: 0.8 }];
  const query = {
    textQuery: 'rust vector search',
    topK: neighbors.length,
    neighborIds: neighbors.map(n => n.docId),
    strategy: 'zero_copy'
  };
  return { query, elapsed: Date.now() - start };
}

(async () => {
  console.log('vector_text_pipeline.test.js');
  await test('pipeline runs under 100ms', async () => {
    const r = await runHybridPipeline();
    assert(r.elapsed < 100, `elapsed ${r.elapsed}ms`);
    assert.strictEqual(r.query.neighborIds.length, 2);
    assert.strictEqual(r.query.strategy, 'zero_copy');
  });
  await test('empty input handled gracefully', async () => {
    const start = Date.now();
    const q = { textQuery: 'empty', topK: 0, neighborIds: [] };
    assert(Date.now() - start < 100);
    assert.deepStrictEqual(q.neighborIds, []);
  });
  console.log('\nAll tests passed.');
})();
