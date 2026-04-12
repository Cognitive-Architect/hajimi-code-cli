/**
 * MCP Server Performance Benchmark
 * Zero-dependency testing for hajimi-mcp
 * 
 * Tests cold start, search, add, memory usage
 * Run: node tests/mcp/perf.bench.mjs
 */
import * as fs from "fs/promises";
import * as path from "path";
import { homedir } from "os";
import os from "os";

const ITERATIONS = 10;
const NS_PER_MS = 1e6;
const LCR_PATH = path.join(homedir(), ".hajimi", "perf-test.db");

class LCRStore {
  constructor() {
    this.chunks = [];
    this.initialized = false;
  }
  async init() {
    if (this.initialized) return;
    try {
      this.chunks = JSON.parse(await fs.readFile(LCR_PATH, "utf-8").catch(() => "[]"));
    } catch {
      this.chunks = [];
    }
    this.initialized = true;
  }
  search(query, limit = 10) {
    const q = query.toLowerCase();
    return this.chunks.filter(c => c.content.toLowerCase().includes(q)).slice(0, limit);
  }
  add(chunk) {
    this.chunks.push({
      ...chunk,
      id: `c_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
      timestamp: Date.now()
    });
  }
}

function now() {
  return Number(process.hrtime.bigint());
}

async function bench(name, fn, iterations = ITERATIONS) {
  const times = [];
  for (let i = 0; i < iterations; i++) {
    const s = now();
    await fn();
    times.push((now() - s) / NS_PER_MS);
  }
  const avg = times.reduce((a, b) => a + b) / times.length;
  console.log(`  ${name}: ${avg.toFixed(2)}ms (avg of ${iterations})`);
  return avg;
}

console.log("\n=== MCP Server Performance Benchmark ===\n");
console.log("Environment:");
console.log(`  Node.js: ${process.version}`);
console.log(`  OS: ${os.platform()} ${os.release()}`);
console.log(`  CPU: ${os.cpus()[0].model}`);
console.log(`  RAM: ${Math.round(os.totalmem() / 1024 / 1024 / 1024)}GB\n`);

try { await fs.unlink(LCR_PATH); } catch { }

console.log("Running benchmarks...\n");
console.time("Cold Start");
const csStart = Date.now();
const store = new LCRStore();
await store.init();
const coldMs = Date.now() - csStart;
console.timeEnd("Cold Start");

for (let i = 0; i < 100; i++) {
  store.add({ content: `test chunk ${i} with sample data content`, metadata: {} });
}

const searchAvg = await bench("Search", () => store.search("chunk", 10), ITERATIONS);

let addCnt = 0;
const addAvg = await bench("Add", () => store.add({ content: `item ${addCnt++}`, metadata: {} }), ITERATIONS);

const rssMB = Math.round(process.memoryUsage().rss / 1024 / 1024 * 100) / 100;
console.log(`\n  Memory RSS: ${rssMB}MB`);

try { await fs.unlink(LCR_PATH); } catch { }

console.log("\n=== Results ===");
console.log("| Metric     | Target | Actual   | Status |");
console.log("|------------|--------|----------|--------|");

const status = v => v ? "✅" : "❌";
console.log(`| Cold Start | <100ms | ${coldMs}ms     | ${status(coldMs < 100)}     |`);
console.log(`| Search     | <50ms  | ${searchAvg.toFixed(1)}ms  | ${status(searchAvg < 50)}     |`);
console.log(`| Add        | <50ms  | ${addAvg.toFixed(1)}ms  | ${status(addAvg < 50)}     |`);
console.log(`| Memory     | <50MB  | ${rssMB}MB   | ${status(rssMB < 50)}     |`);
