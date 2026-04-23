/**
 * MCP Help Capability
 * Global help system: list all tools and retrieve per-tool documentation.
 */

export interface ToolMeta {
  name: string;
  description: string;
  schema: object;
}

const TOOL_REGISTRY: ToolMeta[] = [];

/** Register a tool's metadata for the help system */
export function registerToolMeta(meta: ToolMeta): void {
  const idx = TOOL_REGISTRY.findIndex((t) => t.name === meta.name);
  if (idx >= 0) TOOL_REGISTRY[idx] = meta;
  else TOOL_REGISTRY.push(meta);
}

/** List all registered tools */
export function listTools(): ToolMeta[] {
  return [...TOOL_REGISTRY];
}

/** Get detailed help for a single tool */
export function getToolHelp(name: string): ToolMeta | undefined {
  return TOOL_REGISTRY.find((t) => t.name === name);
}

/** Format tool list as a readable markdown table */
export function formatToolTable(): string {
  const header = "| Tool | Description |\n|:---|:---|";
  const rows = TOOL_REGISTRY.map((t) => `| ${t.name} | ${t.description} |`).join("\n");
  return `${header}\n${rows}`;
}
