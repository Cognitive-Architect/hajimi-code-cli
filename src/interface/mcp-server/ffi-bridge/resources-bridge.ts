/**
 * MCP-FFI Resources Bridge - 3 Resources暴露四级内存状态
 */
import type { Resource, TextResourceContents } from '@modelcontextprotocol/sdk/types.js';
import { memoryStats, getDefaultMemoryBudget, createMemoryGateway, type MemoryGatewayHandle } from '../../../crates/codex-twist/index.js';

enum ResourceUri { MEMORY_STATS = 'hajimi://memory/stats', MEMORY_BUDGET = 'hajimi://memory/budget', HEALTH_CHECK = 'hajimi://health' }
enum MimeType { JSON = 'application/json', TEXT = 'text/plain' }

let defaultGateway: MemoryGatewayHandle | null = null;
const getDefaultGateway = (): MemoryGatewayHandle => defaultGateway ??= createMemoryGateway();

const fmtJson = (uri: string, data: unknown): TextResourceContents => ({ uri, mimeType: MimeType.JSON, text: JSON.stringify(data, null, 2) });
const fmtText = (uri: string, text: string): TextResourceContents => ({ uri, mimeType: MimeType.TEXT, text });
const pick = (obj: Record<string, number>, p: string) => ({ entries: obj[`${p}_entries`], tokens: obj[`${p}_tokens`] });

export const HAJIMI_RESOURCES: Resource[] = [
  { uri: ResourceUri.MEMORY_STATS, name: 'Memory Stats', description: '四级内存统计信息', mimeType: MimeType.JSON },
  { uri: ResourceUri.MEMORY_BUDGET, name: 'Memory Budget', description: '默认Token预算配置', mimeType: MimeType.JSON },
  { uri: ResourceUri.HEALTH_CHECK, name: 'Health Check', description: 'FFI层健康检查', mimeType: MimeType.TEXT },
];

export async function readResource(uri: string): Promise<TextResourceContents> {
  const gateway = getDefaultGateway();
  switch (uri) {
    case ResourceUri.MEMORY_STATS: {
      const s = await memoryStats(gateway.id);
      return fmtJson(uri, { focus: pick(s, 'focus'), working: pick(s, 'working'), archive: pick(s, 'archive'), total: { entries: s.focus_entries + s.working_entries + s.archive_entries, tokens: s.focus_tokens + s.working_tokens + s.archive_tokens } });
    }
    case ResourceUri.MEMORY_BUDGET: {
      const b = getDefaultMemoryBudget();
      return fmtJson(uri, { focus_limit: b.focus, working_limit: b.working, archive_limit: b.archive, rag_limit: b.rag, total_limit: b.focus + b.working + b.archive + b.rag });
    }
    case ResourceUri.HEALTH_CHECK: {
      const b = getDefaultMemoryBudget();
      return fmtText(uri, b.focus > 0 && b.working > 0 && b.archive > 0 ? `OK: FFI层连通，Focus=${b.focus}, Working=${b.working}, Archive=${b.archive}` : 'ERROR: FFI层异常');
    }
    default: throw new Error(`Resource not found: ${uri}`);
  }
}

const subscribers = new Set<string>();
export const subscribeResource = (uri: string): void => { if (HAJIMI_RESOURCES.some(r => r.uri === uri)) subscribers.add(uri); };
export const unsubscribeResource = (uri: string): void => { subscribers.delete(uri); };
export function notifyResourceChanged(uri: string): void { if (subscribers.has(uri)) console.log(`[Resources-Bridge] Resource changed: ${uri}`); }
export function cleanupResources(): void { subscribers.clear(); defaultGateway = null; }
