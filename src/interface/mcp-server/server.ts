/**
 * Hajimi MCP Server — Security Hardened (B-03/04 Audit)
 * Input validation, path traversal protection, injection prevention
 */
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { CallToolRequestSchema, ListToolsRequestSchema, ListResourcesRequestSchema, ReadResourceRequestSchema } from "@modelcontextprotocol/sdk/types.js";
import * as fs from "fs/promises";
import * as path from "path";
import { homedir } from "os";

// ============================================================
// Security: Input Validation & Path Traversal Protection
// ============================================================
const MAX_INPUT_LEN = 10 * 1024;
const MAX_PATH_LEN = 260;
const CTRL_CHARS = /[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/;
const TRAVERSAL = /\.\.[\/\\]|~\//;
const EVIL_PATHS = [/\/etc\/passwd/, /\/etc\/shadow/, /\\Windows\\System32/, /C:\\Windows/, /\.ssh\/id_rsa/];

class SecurityError extends Error {}

export function validate(input: unknown, field: string): string {
  if (typeof input !== "string") throw new SecurityError(`${field} must be a string`);
  if (!input) throw new SecurityError(`${field} cannot be empty`);
  if (input.length > MAX_INPUT_LEN) throw new SecurityError(`${field} exceeds maximum length of ${MAX_INPUT_LEN} bytes`);
  if (CTRL_CHARS.test(input)) throw new SecurityError(`${field} contains invalid characters`);
  if (input.includes("\x00")) throw new SecurityError(`${field} contains null bytes`);
  return input;
}

export function validatePath(fp: string): string {
  if (fp.length > MAX_PATH_LEN) throw new SecurityError("Path too long");
  if (TRAVERSAL.test(fp)) throw new SecurityError("Path traversal detected: directory escape blocked");
  for (const p of EVIL_PATHS) if (p.test(fp)) throw new SecurityError("System path forbidden");
  const norm = path.normalize(fp);
  if (norm.startsWith("..") || norm.includes("../") || norm.includes("..\\")) throw new SecurityError("Path traversal in normalized path");
  return norm;
}

export function sanitizeMeta(meta: unknown): Record<string, unknown> {
  if (typeof meta !== "object" || !meta) return {};
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(meta)) {
    if (k === "__proto__" || k === "constructor") continue;
    out[k] = typeof v === "object" && v !== null ? sanitizeMeta(v) : typeof v === "string" ? (CTRL_CHARS.test(v) ? v.replace(CTRL_CHARS, "") : v) : v;
  }
  return out;
}

// ============================================================
// Server Configuration
// ============================================================
export const SERVER_INFO = { name: "hajimi-mcp", version: "1.0.0" };
let LCR_PATH: string;
try {
  LCR_PATH = process.env.HAJIMI_LCR_PATH ? path.resolve(validatePath(process.env.HAJIMI_LCR_PATH)) : path.join(homedir(), ".hajimi", "lcr.db");
} catch (e) {
  console.error("Security: Invalid HAJIMI_LCR_PATH, using default");
  LCR_PATH = path.join(homedir(), ".hajimi", "lcr.db");
}

// ============================================================
// LCR Core (In-Memory with File Persistence)
// ============================================================
interface Chunk { id: string; content: string; metadata: Record<string, any>; timestamp: number; }

class LCRStore {
  chunks: Chunk[] = [];
  initialized = false;
  async init(): Promise<void> {
    if (this.initialized) return;
    try { validatePath(LCR_PATH); this.chunks = JSON.parse(await fs.readFile(LCR_PATH, "utf-8").catch(() => "[]")); } catch { this.chunks = []; }
    this.initialized = true;
  }
  async search(query: string, limit = 10): Promise<Chunk[]> {
    await this.init(); const q = query.toLowerCase();
    return this.chunks.filter(c => c.content.toLowerCase().includes(q)).sort((a, b) => b.timestamp - a.timestamp).slice(0, limit);
  }
  async add(chunk: Omit<Chunk, "id" | "timestamp">): Promise<Chunk> {
    await this.init();
    const nc: Chunk = { ...chunk, id: `chunk_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`, timestamp: Date.now() };
    this.chunks.push(nc); await this.persist(); return nc;
  }
  async persist(): Promise<void> { validatePath(LCR_PATH); await fs.mkdir(path.dirname(LCR_PATH), { recursive: true }); await fs.writeFile(LCR_PATH, JSON.stringify(this.chunks, null, 2)); }
  getStats() { return { total: this.chunks.length, path: LCR_PATH }; }
}
export const lcrStore = new LCRStore();

// ============================================================
// Handlers (extracted to keep server.ts within line budget)
// ============================================================
import {
  handleSearch, handleAdd, handleStats, handleReadFile, handleGrep,
  handleGitStatus, handleRunTests, handleSecurityAudit, handleAdrSearch, handleAgentStart,
  handleBuild, handleListDir, handleGitLog, handleClippy,
} from "./handlers/index.js";
import { handleChatWithTrace, handleAgentRun } from "./handlers/trace_handler.js";
import { registerToolMeta, formatToolTable } from "./capabilities/help.js";

// ============================================================
// MCP Server
// ============================================================
const server = new Server({ name: SERVER_INFO.name, version: SERVER_INFO.version }, { capabilities: { tools: { listChanged: false }, resources: { subscribe: false, listChanged: false } } });

const TOOLS = [
  { name: "hajimi_search", description: "Search LCR for context chunks", inputSchema: { type: "object" as const, properties: { query: { type: "string" }, limit: { type: "number", default: 10 } }, required: ["query"] } },
  { name: "hajimi_add", description: "Add a context chunk to LCR", inputSchema: { type: "object" as const, properties: { content: { type: "string" }, metadata: { type: "object", default: {} } }, required: ["content"] } },
  { name: "hajimi_stats", description: "Get LCR statistics", inputSchema: { type: "object" as const, properties: {} } },
  { name: "hajimi_read_file", description: "Read a file from the workspace", inputSchema: { type: "object" as const, properties: { path: { type: "string" } }, required: ["path"] } },
  { name: "hajimi_grep", description: "Search for a pattern in files", inputSchema: { type: "object" as const, properties: { pattern: { type: "string" }, paths: { type: "array", items: { type: "string" } } }, required: ["pattern"] } },
  { name: "hajimi_git_status", description: "Show git repository status", inputSchema: { type: "object" as const, properties: {} } },
  { name: "hajimi_run_tests", description: "Run Rust test suite via cargo test", inputSchema: { type: "object" as const, properties: { crate: { type: "string" } } } },
  { name: "hajimi_security_audit", description: "Run cargo audit for security vulnerabilities", inputSchema: { type: "object" as const, properties: {} } },
  { name: "hajimi_adr_search", description: "Search Architecture Decision Records", inputSchema: { type: "object" as const, properties: { keyword: { type: "string" }, limit: { type: "number", default: 10 } }, required: ["keyword"] } },
  { name: "hajimi_agent_start", description: "Start the Hajimi agent loop", inputSchema: { type: "object" as const, properties: { config: { type: "string" } } } },
  { name: "hajimi_build", description: "Build the workspace via cargo build", inputSchema: { type: "object" as const, properties: { crate: { type: "string" } } } },
  { name: "hajimi_list_dir", description: "List directory contents", inputSchema: { type: "object" as const, properties: { path: { type: "string" } }, required: ["path"] } },
  { name: "hajimi_git_log", description: "Show recent git commit history", inputSchema: { type: "object" as const, properties: {} } },
  { name: "hajimi_clippy", description: "Run cargo clippy for lint checks", inputSchema: { type: "object" as const, properties: { crate: { type: "string" } } } },
  { name: "hajimi_help", description: "List all available tools and their descriptions", inputSchema: { type: "object" as const, properties: { tool: { type: "string" } } } },
  { name: "hajimi_chat_with_trace", description: "Chat with Hajimi agent and receive streaming thinking trace", inputSchema: { type: "object" as const, properties: { query: { type: "string" }, thinking_trace: { type: "boolean", default: true } }, required: ["query"] } },
  { name: "hajimi_agent_run", description: "Run Hajimi agent with thinking trace support", inputSchema: { type: "object" as const, properties: { query: { type: "string" }, thinking_trace: { type: "boolean", default: true } }, required: ["query"] } },
];

const RESOURCES = [{ uri: "stats://lcr", name: "LCR Statistics", description: "LCR stats and health", mimeType: "application/json" }];

TOOLS.forEach((t) => registerToolMeta({ name: t.name, description: t.description, schema: t.inputSchema }));

// ============================================================
// Request Handlers
// ============================================================
server.setRequestHandler(ListToolsRequestSchema, async () => ({ tools: TOOLS }));
server.setRequestHandler(CallToolRequestSchema, async (req) => {
  const { name, arguments: args } = req.params;
  try {
    switch (name) {
      case "hajimi_search": return await handleSearch(args as Record<string, unknown>);
      case "hajimi_add": return await handleAdd(args as Record<string, unknown>);
      case "hajimi_stats": return await handleStats();
      case "hajimi_read_file": return await handleReadFile(args as Record<string, unknown>);
      case "hajimi_grep": return await handleGrep(args as Record<string, unknown>);
      case "hajimi_git_status": return await handleGitStatus();
      case "hajimi_run_tests": return await handleRunTests(args as Record<string, unknown>);
      case "hajimi_security_audit": return await handleSecurityAudit();
      case "hajimi_adr_search": return await handleAdrSearch(args as Record<string, unknown>);
      case "hajimi_agent_start": return await handleAgentStart(args as Record<string, unknown>);
      case "hajimi_build": return await handleBuild(args as Record<string, unknown>);
      case "hajimi_list_dir": return await handleListDir(args as Record<string, unknown>);
      case "hajimi_git_log": return await handleGitLog();
      case "hajimi_clippy": return await handleClippy(args as Record<string, unknown>);
      case "hajimi_help": return { content: [{ type: "text" as const, text: formatToolTable() }] };
      case "hajimi_chat_with_trace": return await handleChatWithTrace(args as Record<string, unknown>);
      case "hajimi_agent_run": return await handleAgentRun(args as Record<string, unknown>);
      default: throw new Error(`Unknown tool: ${name}`);
    }
  } catch (err) {
    return { content: [{ type: "text" as const, text: `Error: ${err instanceof Error ? err.message : "Unknown"}` }], isError: true };
  }
});
server.setRequestHandler(ListResourcesRequestSchema, async () => ({ resources: RESOURCES }));
server.setRequestHandler(ReadResourceRequestSchema, async (req) => {
  if (req.params.uri === "stats://lcr") {
    const stats = lcrStore.getStats();
    return { contents: [{ uri: req.params.uri, mimeType: "application/json", text: JSON.stringify(stats, null, 2) }] };
  }
  throw new Error(`Resource not found: ${req.params.uri}`);
});

// ============================================================
// Startup
// ============================================================
async function main() {
  await lcrStore.init();
  await server.connect(new StdioServerTransport());
  console.error(`Hajimi MCP v${SERVER_INFO.version} started | LCR: ${LCR_PATH} | Security: enabled | Tools: ${TOOLS.length}`);
}
main().catch((e) => { console.error("Failed to start:", e); process.exit(1); });
