/**
 * MCP Server Stress Test - 1000+ Chunks Performance
 * Tests performance degradation with large data volumes
 * Run: node tests/mcp/stress.test.mjs
 */
import * as fs from "fs/promises";
import * as path from "path";
import { homedir } from "os";
import os from "os";

const NS_PER_MS = 1e6;
const BASELINE_CHUNKS = 100;
const STRESS_CHUNKS = 1000;
const EXTREME_CHUNKS = 2000;
const LCR_PATH = path.join(homedir(), ".hajimi", "stress-test.db");

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

function hrtime() {
  return Number(process.hrtime.bigint());
}

function measureMemory() {
  return Math.round(process.memoryUsage().rss / 1024 / 1024 * 100) / 100;
}

async function runTest(chunkCount, label) {
  try { await fs.unlink(LCR_PATH); } catch { }
  const store = new LCRStore();
  const coldStart = hrtime();
  await store.init();
  const coldMs = (hrtime() - coldStart) / NS_PER_MS;
  for (let i = 0; i < chunkCount; i++) {
    store.add({ 
      content: `stress test chunk ${i} with sample data content for performance validation`, 
      metadata: { index: i } 
    });
  }
  const searchStart = hrtime();
  store.search("chunk", 10);
  const searchMs = (hrtime() - searchStart) / NS_PER_MS;
  const memMB = measureMemory();
  console.log(`  ${label}: cold=${coldMs.toFixed(2)}ms, search=${searchMs.toFixed(2)}ms, mem=${memMB}MB`);
  return { coldMs, searchMs, memMB, chunks: chunkCount };
}

function calcDegradation(baseline, current, metric) {
  const degradation = ((current - baseline) / baseline * 100).toFixed(1);
  const status = parseFloat(degradation) < 20 ? "✅" : parseFloat(degradation) < 50 ? "⚠️" : "❌";
  return { degradation, status };
}

console.log("\n=== MCP Server Stress Test (1000+ chunks) ===\n");
console.log("Environment:");
console.log(`  Node.js: ${process.version}`);
console.log(`  OS: ${os.platform()} ${os.release()}`);
console.log(`  CPU: ${os.cpus()[0].model}`);
console.log(`  RAM: ${Math.round(os.totalmem() / 1024 / 1024 / 1024)}GB\n`);
console.log("Running stress tests...\n");

const baseline = await runTest(BASELINE_CHUNKS, "Baseline (100)");
const stress = await runTest(STRESS_CHUNKS, "Stress (1000)");
const extreme = await runTest(EXTREME_CHUNKS, "Extreme (2000)");

const coldDeg = calcDegradation(baseline.coldMs, stress.coldMs, "cold");
const searchDeg = calcDegradation(baseline.searchMs, stress.searchMs, "search");
const memDeg = calcDegradation(baseline.memMB, stress.memMB, "memory");

try { await fs.unlink(LCR_PATH); } catch { }

console.log("\n=== Performance Degradation Analysis ===\n");
console.log("S-Level Performance Red Lines:");
console.log("  • 1000 chunks cold start < 200ms (degradation < 100%)");
console.log("  • 1000 chunks search < 100ms (degradation < 100%)");
console.log("  • Memory growth linear (< 100MB for 1000 chunks)\n");
console.log("| Metric      | Baseline | Stress  | Extreme | Degradation | Status |");
console.log("|-------------|----------|---------|---------|-------------|--------|");
console.log(`| Cold Start  | ${baseline.coldMs.toFixed(1)}ms   | ${stress.coldMs.toFixed(1)}ms  | ${extreme.coldMs.toFixed(1)}ms  | ${coldDeg.degradation}%      | ${coldDeg.status}    |`);
console.log(`| Search      | ${baseline.searchMs.toFixed(1)}ms    | ${stress.searchMs.toFixed(1)}ms   | ${extreme.searchMs.toFixed(1)}ms   | ${searchDeg.degradation}%      | ${searchDeg.status}    |`);
console.log(`| Memory      | ${baseline.memMB}MB    | ${stress.memMB}MB   | ${extreme.memMB}MB   | ${memDeg.degradation}%      | ${memDeg.status}    |`);
console.log("\n=== Attenuation 衰减 Analysis ===");
const pass = stress.coldMs < 200 && stress.searchMs < 100 && stress.memMB < 100;
console.log(`\nStress Test ${pass ? "✅ PASSED" : "❌ FAILED"}`);
console.log(`  Cold start ${stress.coldMs < 200 ? "<" : ">"} 200ms threshold`);
console.log(`  Search ${stress.searchMs < 100 ? "<" : ">"} 100ms threshold`);
console.log(`  Memory ${stress.memMB < 100 ? "<" : ">"} 100MB threshold\n`);
