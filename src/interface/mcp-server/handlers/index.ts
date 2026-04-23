/**
 * MCP Tool Handlers
 * Extracted from server.ts to keep the server file within line limits.
 */

import * as fs from "fs/promises";
import { spawn } from "child_process";
import { lcrStore, SERVER_INFO, validate, validatePath, sanitizeMeta } from "../server.js";

export async function handleSearch(args: Record<string, unknown>) {
  if (args?.query === undefined) throw new Error("Query parameter is required");
  const query = validate(args?.query, "query");
  let limit = args?.limit !== undefined ? Number(args.limit) : 10;
  if (isNaN(limit) || limit < 1 || limit > 100) throw new Error("Limit must be a number between 1 and 100");
  const results = await lcrStore.search(query, limit);
  return { content: [{ type: "text" as const, text: JSON.stringify({ query, count: results.length, results: results.map(r => ({ id: r.id, content: r.content.slice(0, 500), metadata: r.metadata })) }, null, 2) }] };
}

export async function handleAdd(args: Record<string, unknown>) {
  if (args?.content === undefined) throw new Error("Content parameter is required");
  const content = validate(args?.content, "content");
  const metadata = sanitizeMeta(args?.metadata);
  const chunk = await lcrStore.add({ content, metadata });
  return { content: [{ type: "text" as const, text: JSON.stringify({ success: true, id: chunk.id, timestamp: chunk.timestamp }, null, 2) }] };
}

export async function handleStats() {
  const stats = lcrStore.getStats();
  return { content: [{ type: "text" as const, text: JSON.stringify({ server: SERVER_INFO, lcr: stats, uptime: process.uptime() }, null, 2) }] };
}

export async function handleReadFile(args: Record<string, unknown>) {
  const fp = validatePath(validate(args?.path, "path"));
  const content = await fs.readFile(fp, "utf-8");
  return { content: [{ type: "text" as const, text: content.slice(0, 5000) }] };
}

export async function handleGrep(args: Record<string, unknown>) {
  const pattern = validate(args?.pattern, "pattern");
  const paths = Array.isArray(args?.paths) ? args.paths.map((p: unknown) => validatePath(validate(p, "path"))) : ["."];
  return new Promise((resolve, reject) => {
    const proc = spawn("grep", ["-rn", "--color=never", pattern, ...paths]);
    let stdout = "";
    proc.stdout?.on("data", (d) => { stdout += d; });
    proc.on("close", (code) => {
      resolve({ content: [{ type: "text" as const, text: stdout || `Pattern '${pattern}' not found (exit ${code})` }] });
    });
    proc.on("error", reject);
  });
}

export async function handleGitStatus() {
  return new Promise((resolve, reject) => {
    const proc = spawn("git", ["status", "--short"]);
    let stdout = "";
    proc.stdout?.on("data", (d) => { stdout += d; });
    proc.on("close", () => {
      resolve({ content: [{ type: "text" as const, text: stdout || "clean" }] });
    });
    proc.on("error", reject);
  });
}

export async function handleRunTests(args: Record<string, unknown>) {
  const crate = args?.crate ? String(args.crate) : undefined;
  const cargoArgs = crate ? ["test", "-p", crate] : ["test"];
  return new Promise((resolve, reject) => {
    const proc = spawn("cargo", cargoArgs);
    let stdout = "";
    proc.stdout?.on("data", (d) => { stdout += d; });
    proc.on("close", () => {
      const summary = stdout.match(/test result:.*?\n/)?.[0]?.trim() || "Tests completed";
      resolve({ content: [{ type: "text" as const, text: summary }] });
    });
    proc.on("error", reject);
  });
}

export async function handleSecurityAudit() {
  return new Promise((resolve, reject) => {
    const proc = spawn("cargo", ["audit"]);
    let stdout = "";
    let stderr = "";
    proc.stdout?.on("data", (d) => { stdout += d; });
    proc.stderr?.on("data", (d) => { stderr += d; });
    proc.on("close", () => {
      resolve({ content: [{ type: "text" as const, text: stdout || stderr || "Audit completed" }] });
    });
    proc.on("error", reject);
  });
}

export async function handleAdrSearch(args: Record<string, unknown>) {
  const keyword = validate(args?.keyword, "keyword");
  const limit = args?.limit !== undefined ? Number(args.limit) : 10;
  // Stub: in production this would query the knowledge graph
  return { content: [{ type: "text" as const, text: `ADR search for '${keyword}' (limit ${limit}) — stub: integrate with knowledge graph` }] };
}

export async function handleAgentStart(args: Record<string, unknown>) {
  const config = args?.config ? String(args.config) : "default";
  return { content: [{ type: "text" as const, text: `Agent start requested with config: ${config}` }] };
}

export async function handleBuild(args: Record<string, unknown>) {
  const crate = args?.crate ? String(args.crate) : undefined;
  const cargoArgs = crate ? ["build", "-p", crate] : ["build"];
  return new Promise((resolve, reject) => {
    const proc = spawn("cargo", cargoArgs);
    let stdout = "";
    let stderr = "";
    proc.stdout?.on("data", (d) => { stdout += d; });
    proc.stderr?.on("data", (d) => { stderr += d; });
    proc.on("close", () => {
      resolve({ content: [{ type: "text" as const, text: stdout || stderr || "Build completed" }] });
    });
    proc.on("error", reject);
  });
}

export async function handleListDir(args: Record<string, unknown>) {
  const dirPath = validatePath(validate(args?.path, "path"));
  const entries = await fs.readdir(dirPath, { withFileTypes: true });
  const lines = entries.map((e) => `${e.isDirectory() ? "D" : "F"} ${e.name}`);
  return { content: [{ type: "text" as const, text: lines.join("\n") }] };
}

export async function handleGitLog() {
  return new Promise((resolve, reject) => {
    const proc = spawn("git", ["log", "--oneline", "-n", "20"]);
    let stdout = "";
    proc.stdout?.on("data", (d) => { stdout += d; });
    proc.on("close", () => {
      resolve({ content: [{ type: "text" as const, text: stdout || "No commits" }] });
    });
    proc.on("error", reject);
  });
}

export async function handleClippy(args: Record<string, unknown>) {
  const crate = args?.crate ? String(args.crate) : undefined;
  const cargoArgs = crate ? ["clippy", "-p", crate, "--", "-D", "warnings"] : ["clippy", "--", "-D", "warnings"];
  return new Promise((resolve, reject) => {
    const proc = spawn("cargo", cargoArgs);
    let stdout = "";
    let stderr = "";
    proc.stdout?.on("data", (d) => { stdout += d; });
    proc.stderr?.on("data", (d) => { stderr += d; });
    proc.on("close", () => {
      resolve({ content: [{ type: "text" as const, text: stdout || stderr || "Clippy completed" }] });
    });
    proc.on("error", reject);
  });
}
