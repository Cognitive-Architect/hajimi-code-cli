#!/usr/bin/env node
/**
 * Data Honesty Validator — ensures corrected reports use realistic values.
 */

const fs = require("fs");
const path = require("path");

class DataHonestyError extends Error {
  constructor(message) { super(message); this.name = "DataHonestyError"; }
}

function loadArtifact(p) {
  if (!fs.existsSync(p)) throw new DataHonestyError(`Artifact missing: ${p}`);
  return JSON.parse(fs.readFileSync(p, "utf8"));
}

function validateLatency(artifactPath) {
  const measured = loadArtifact(artifactPath).p95LatencyMs;
  if (measured <= 0 || measured >= 1000) {
    throw new DataHonestyError(
      `Latency unrealistic: ${measured} ms (must be >0 and <1000)`
    );
  }
  console.log(`✓ Latency realistic: ${measured} ms`);
}

function validateMemory(artifactPath) {
  const measured = loadArtifact(artifactPath).memoryRssMB;
  if (measured < 50 || measured > 2048) {
    throw new DataHonestyError(
      `Memory unrealistic: ${measured} MB (must be between 50 and 2048)`
    );
  }
  console.log(`✓ Memory realistic: ${measured} MB`);
}

function validateCorrectedReport(reportPath) {
  const report = fs.readFileSync(reportPath, "utf8");
  const p95Match = report.match(/- \*\*P95 Latency:\*\*\s*(\d+)\s*ms/);
  const rssMatch = report.match(/- \*\*End RSS:\*\*\s*~?(\d+)\s*MB/);
  if (!p95Match || parseInt(p95Match[1], 10) <= 0 || parseInt(p95Match[1], 10) >= 1000) {
    throw new DataHonestyError(`Corrected report latency value missing or unrealistic`);
  }
  if (!rssMatch || parseInt(rssMatch[1], 10) < 50 || parseInt(rssMatch[1], 10) > 2048) {
    throw new DataHonestyError(`Corrected report memory value missing or unrealistic`);
  }
  console.log("✓ Corrected report contains realistic values");
}

function validateOriginalReportPunished(originalPath) {
  const report = fs.readFileSync(originalPath, "utf8");
  if (report.includes("Overall Grade: A+") || report.includes("**Rating:** A+")) {
    throw new DataHonestyError(`Original report still claims A+ grade without correction note`);
  }
  console.log("✓ Original report no longer claims unqualified A+");
}

function main() {
  const artifact = path.join(__dirname, "..", "tests", "stress", "output", "end_to_end_10k_summary.json");
  const corrected = path.join(__dirname, "..", "docs", "perf", "END-TO-END-10K-BENCHMARK-001-CORRECTED.md");
  const original = path.join(__dirname, "..", "docs", "perf", "END-TO-END-10K-BENCHMARK-001.md");
  try {
    validateLatency(artifact);
    validateMemory(artifact);
    validateCorrectedReport(corrected);
    validateOriginalReportPunished(original);
    console.log("\n✅ All data-honesty checks passed.");
  } catch (err) {
    console.error(`\n❌ ${err.name}: ${err.message}`);
    process.exitCode = 1;
  }
}

main();
