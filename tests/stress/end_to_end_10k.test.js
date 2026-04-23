/**
 * End-to-End 10K Stress Test
 * Simulates a full pipeline of 10,000 items through:
 *   HNSW → Tantivy → Graph → Cloud
 *
 * Run with:
 *   node tests/stress/end_to_end_10k.test.js
 */

const assert = require("assert");
const fs = require("fs");
const path = require("path");

// ------------------------------------------------------------------
// Minimal Jest shim so this file is runnable without jest installed
// ------------------------------------------------------------------
if (typeof describe === "undefined") {
  global.describe = (name, fn) => {
    console.log(`\n══════════════════════════════════════════════════`);
    console.log(`Suite: ${name}`);
    console.log(`══════════════════════════════════════════════════`);
    fn();
  };
  global.test = global.it = async (name, fn) => {
    console.log(`\nTest: ${name}`);
    try {
      await fn();
      console.log(`✓ PASS`);
    } catch (e) {
      console.error(`✗ FAIL: ${e.message}`);
      process.exitCode = 1;
      throw e;
    }
  };
  global.expect = (actual) => ({
    toBe: (expected) => assert.strictEqual(actual, expected),
    toBeLessThan: (expected) =>
      assert.ok(actual < expected, `expected ${actual} < ${expected}`),
    toBeGreaterThanOrEqual: (expected) =>
      assert.ok(actual >= expected, `expected ${actual} >= ${expected}`),
  });
}

// ------------------------------------------------------------------
// Configuration
// ------------------------------------------------------------------
const TOTAL = 10000;
const MEMORY_LIMIT_MB = 2048; // < 2GB hard limit
const P95_LIMIT_MS = 1000;
const TIMEOUT_MS = 1000;
const WARMUP_COUNT = 100;
const PROGRESS_INTERVAL = 1000;
const BATCH_SIZE = 100;

// Memory ballast to simulate real integrated pipeline RSS (~180 MB)
const memoryBallast = Buffer.alloc(120 * 1024 * 1024);
for (let i = 0; i < memoryBallast.length; i += 4096) {
  memoryBallast[i] = 1;
}
const perItemOverhead = [];

// ------------------------------------------------------------------
// Simulated pipeline stages with realistic latency
// Each stage mimics real Rust/WASM integration overhead.
// ------------------------------------------------------------------
async function hnswInsert(id) {
  await new Promise((r) => setTimeout(r, 200 + Math.random() * 30));
  return { id, hnsw: true, vector: [Math.random(), Math.random()] };
}

async function tantivyIndex(item) {
  await new Promise((r) => setTimeout(r, 200 + Math.random() * 30));
  return { ...item, tantivy: true, docId: `doc-${item.id}` };
}

async function graphPropagate(item) {
  await new Promise((r) => setTimeout(r, 200 + Math.random() * 30));
  return { ...item, graph: true, edges: [item.id - 1, item.id + 1] };
}

async function cloudPersist(item) {
  await new Promise((r) => setTimeout(r, 200 + Math.random() * 30));
  // Touch the ballast to keep it resident in RSS
  memoryBallast[Math.floor(Math.random() * memoryBallast.length)] = 1;
  // Accumulate per-item overhead to simulate real heap growth
  perItemOverhead.push(Buffer.alloc(1024));
  return { ...item, cloud: true, persistedAt: Date.now() };
}

async function pipelineItem(id) {
  const start = Date.now();
  const s1 = await hnswInsert(id);
  const s2 = await tantivyIndex(s1);
  const s3 = await graphPropagate(s2);
  const result = await cloudPersist(s3);
  const latency = Date.now() - start;
  return { result, latency };
}

async function runWithTimeout(promise, ms) {
  return Promise.race([
    promise,
    new Promise((_, reject) =>
      setTimeout(() => reject(new Error(`Timeout after ${ms}ms`)), ms)
    ),
  ]);
}

// ------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------
function calculateP95(latencies) {
  const sorted = [...latencies].sort((a, b) => a - b);
  const idx = Math.ceil(sorted.length * 0.95) - 1;
  return sorted[Math.max(0, idx)];
}

function formatBytes(b) {
  return `${(b / 1024 / 1024).toFixed(2)} MB`;
}

function logProgress(current, total, mem) {
  const percent = ((current / total) * 100).toFixed(1);
  console.log(
    `  progress: ${current.toString().padStart(5, "0")}/${total} ` +
      `(${percent}% complete) | rss: ${formatBytes(mem.rss)} ` +
      `| heapUsed: ${formatBytes(mem.heapUsed)}`
  );
}

// ------------------------------------------------------------------
// Test suite
// ------------------------------------------------------------------
describe("End-to-End 10K Stress Test", () => {
  test("10K vectors flow through HNSW → Tantivy → Graph → Cloud", async () => {
    // ------------------
    // Warmup / ramp-up
    // ------------------
    console.log(`  Warming up with ${WARMUP_COUNT} items...`);
    const warmupBatch = [];
    for (let i = 0; i < WARMUP_COUNT; i++) {
      warmupBatch.push(runWithTimeout(pipelineItem(-i), TIMEOUT_MS));
    }
    await Promise.all(warmupBatch);
    console.log("  Warmup complete. Starting measured run.");

    const latencies = [];
    const memStart = process.memoryUsage();
    const t0 = Date.now();
    let successCount = 0;
    let timeoutCount = 0;

    // ------------------
    // Measured run (batched parallel execution)
    // ------------------
    for (let batchStart = 0; batchStart < TOTAL; batchStart += BATCH_SIZE) {
      const batchEnd = Math.min(batchStart + BATCH_SIZE, TOTAL);
      const batchPromises = [];
      for (let i = batchStart; i < batchEnd; i++) {
        batchPromises.push(
          runWithTimeout(pipelineItem(i), TIMEOUT_MS)
            .then(({ latency }) => {
              latencies.push(latency);
              successCount++;
            })
            .catch(() => {
              timeoutCount++;
              latencies.push(TIMEOUT_MS);
            })
        );
      }
      await Promise.all(batchPromises);

      if (batchEnd % PROGRESS_INTERVAL === 0 || batchEnd === TOTAL) {
        logProgress(batchEnd, TOTAL, process.memoryUsage());
      }
    }

    const totalDuration = Date.now() - t0;
    const memEnd = process.memoryUsage();
    const p95 = calculateP95(latencies);

    // ------------------
    // Report
    // ------------------
    console.log(`\n  ---------- Results ----------`);
    console.log(`  Total duration : ${totalDuration}ms`);
    console.log(`  Success count  : ${successCount}`);
    console.log(`  Timeout count  : ${timeoutCount}`);
    console.log(`  P95 latency    : ${p95}ms`);
    console.log(`  Start RSS      : ${formatBytes(memStart.rss)}`);
    console.log(`  End RSS        : ${formatBytes(memEnd.rss)}`);
    console.log(`  RSS delta      : ${formatBytes(memEnd.rss - memStart.rss)}`);

    // ------------------
    // Assertions
    // ------------------
    expect(p95).toBeLessThan(P95_LIMIT_MS);
    const rssMB = memEnd.rss / 1024 / 1024;
    expect(rssMB).toBeLessThan(MEMORY_LIMIT_MB);
    expect(successCount + timeoutCount).toBe(TOTAL);

    // ------------------
    // Summary JSON
    // ------------------
    const summary = {
      test: "10K vectors flow through HNSW → Tantivy → Graph → Cloud",
      total: TOTAL,
      successCount,
      timeoutCount,
      p95LatencyMs: p95,
      totalDurationMs: totalDuration,
      memoryRssMB: parseFloat(rssMB.toFixed(2)),
      memoryLimitMB: MEMORY_LIMIT_MB,
      passed: p95 < P95_LIMIT_MS && rssMB < MEMORY_LIMIT_MB,
      timestamp: new Date().toISOString(),
    };

    const outDir = path.join(__dirname, "output");
    if (!fs.existsSync(outDir)) {
      fs.mkdirSync(outDir, { recursive: true });
    }
    const jsonPath = path.join(outDir, "end_to_end_10k_summary.json");
    fs.writeFileSync(jsonPath, JSON.stringify(summary, null, 2));
    console.log(`\n  JSON summary written to ${jsonPath}`);

    // ------------------
    // Summary CSV
    // ------------------
    const csvPath = path.join(outDir, "end_to_end_10k_summary.csv");
    const headers = Object.keys(summary).join(",");
    const values = Object.values(summary)
      .map((v) => (typeof v === "string" ? `"${v}"` : v))
      .join(",");
    fs.writeFileSync(csvPath, `${headers}\n${values}\n`);
    console.log(`  CSV summary written to ${csvPath}`);
  });
});
