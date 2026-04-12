/**
 * Hajimi MCP Server - Security Hardened (B-03/04 Audit)
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
const MAX_INPUT_LEN = 10 * 1024; // 10KB limit
const MAX_PATH_LEN = 260;
const CTRL_CHARS = /[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/;
const TRAVERSAL = /\.\.[\/\\]|~\//; // path traversal detection: ../, ..\, ~/
const EVIL_PATHS = [/\/etc\/passwd/, /\/etc\/shadow/, /\\Windows\\System32/, /C:\\Windows/, /\.ssh\/id_rsa/];

class SecurityError extends Error {}

/** Validates input - length, type, control chars */
function validate(input: unknown, field: string): string {
  if (typeof input !== "string") throw new SecurityError(`${field} must be a string`);
  if (!input) throw new SecurityError(`${field} cannot be empty`);
  if (input.length > MAX_INPUT_LEN) throw new SecurityError(`${field} exceeds maximum length of ${MAX_INPUT_LEN} bytes`);
  if (CTRL_CHARS.test(input)) throw new SecurityError(`${field} contains invalid characters`);
  if (input.includes("\x00")) throw new SecurityError(`${field} contains null bytes`);
  return input;
}

/** Validates path - prevents path traversal attacks */
function validatePath(fp: string): string {
  if (fp.length > MAX_PATH_LEN) throw new SecurityError("Path too long");
  if (TRAVERSAL.test(fp)) throw new SecurityError("Path traversal detected: directory escape blocked");
  for (const p of EVIL_PATHS) if (p.test(fp)) throw new SecurityError("System path forbidden");
  const norm = path.normalize(fp);
  if (norm.startsWith("..") || norm.includes("../") || norm.includes("..\\")) throw new SecurityError("Path traversal in normalized path");
  return norm;
}

/** Sanitizes metadata - prevents prototype pollution */
function sanitizeMeta(meta: unknown): Record<string, unknown> {
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
const SERVER_INFO = { name: "hajimi-mcp", version: "1.0.0" };
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
    try {
      validatePath(LCR_PATH);
      this.chunks = JSON.parse(await fs.readFile(LCR_PATH, "utf-8").catch(() => "[]"));
    } catch { this.chunks = []; }
    this.initialized = true;
  }
  async search(query: string, limit = 10): Promise<Chunk[]> {
    await this.init();
    const q = query.toLowerCase();
    return this.chunks.filter(c => c.content.toLowerCase().includes(q)).sort((a, b) => b.timestamp - a.timestamp).slice(0, limit);
  }
  async add(chunk: Omit<Chunk, "id" | "timestamp">): Promise<Chunk> {
    await this.init();
    const nc: Chunk = { ...chunk, id: `chunk_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`, timestamp: Date.now() };
    this.chunks.push(nc);
    await this.persist();
    return nc;
  }
  async persist(): Promise<void> {
    validatePath(LCR_PATH);
    await fs.mkdir(path.dirname(LCR_PATH), { recursive: true });
    await fs.writeFile(LCR_PATH, JSON.stringify(this.chunks, null, 2));
  }
  getStats() { return { total: this.chunks.length, path: LCR_PATH }; }
}
const lcrStore = new LCRStore();

// ============================================================
// MCP Server
// ============================================================
const server = new Server({ name: SERVER_INFO.name, version: SERVER_INFO.version }, { capabilities: { tools: { listChanged: false }, resources: { subscribe: false, listChanged: false } } });

const TOOLS = [
  { name: "hajimi_search", description: "Search LCR for context chunks", inputSchema: { type: "object" as const, properties: { query: { type: "string" }, limit: { type: "number", default: 10 } }, required: ["query"] } },
  { name: "hajimi_add", description: "Add a context chunk to LCR", inputSchema: { type: "object" as const, properties: { content: { type: "string" }, metadata: { type: "object", default: {} } }, required: ["content"] } },
  { name: "hajimi_stats", description: "Get LCR statistics", inputSchema: { type: "object" as const, properties: {} } },
];
const RESOURCES = [{ uri: "stats://lcr", name: "LCR Statistics", description: "LCR stats and health", mimeType: "application/json" }];

// ============================================================
// Request Handlers
// ============================================================
server.setRequestHandler(ListToolsRequestSchema, async () => ({ tools: TOOLS }));
server.setRequestHandler(CallToolRequestSchema, async (req) => {
  const { name, arguments: args } = req.params;
  try {
    switch (name) {
      case "hajimi_search": {
        if (args?.query === undefined) throw new Error("Query parameter is required");
        const query = validate(args?.query, "query");
        let limit = args?.limit !== undefined ? Number(args.limit) : 10;
        if (isNaN(limit) || limit < 1 || limit > 100) throw new SecurityError("Limit must be a number between 1 and 100");
        const results = await lcrStore.search(query, limit);
        return { content: [{ type: "text" as const, text: JSON.stringify({ query, count: results.length, results: results.map(r => ({ id: r.id, content: r.content.slice(0, 500), metadata: r.metadata })) }, null, 2) }] };
      }
      case "hajimi_add": {
        if (args?.content === undefined) throw new Error("Content parameter is required");
        const content = validate(args?.content, "content");
        const metadata = sanitizeMeta(args?.metadata);
        const chunk = await lcrStore.add({ content, metadata });
        return { content: [{ type: "text" as const, text: JSON.stringify({ success: true, id: chunk.id, timestamp: chunk.timestamp }, null, 2) }] };
      }
      case "hajimi_stats": {
        const stats = lcrStore.getStats();
        return { content: [{ type: "text" as const, text: JSON.stringify({ server: SERVER_INFO, lcr: stats, uptime: process.uptime() }, null, 2) }] };
      }
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
  console.error(`Hajimi MCP v${SERVER_INFO.version} started | LCR: ${LCR_PATH} | Security: enabled`);
}
main().catch(e => { console.error("Failed to start:", e); process.exit(1); });
