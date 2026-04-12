/**
 * MCP-FFI Tools Bridge - 15 Tools直连四级内存
 */
import { z } from 'zod';
import type { Tool, TextContent } from '@modelcontextprotocol/sdk/types.js';

enum MemoryLevel { FOCUS = 'focus', WORKING = 'working', ARCHIVE = 'archive', ANY = 'any' }
enum McpErrorCode { InvalidParams = -32602, ResourceNotFound = -32002 }

class McpError extends Error { constructor(public code: number, message: string) { super(message); } }

const PutSchema = z.object({ handleId: z.string().optional(), key: z.string().min(1), value: z.string().max(1_000_000) });
const GetSchema = z.object({ handleId: z.string().optional(), key: z.string().min(1) });
const ClearSchema = z.object({ handleId: z.string().optional() });
const OptimizeSchema = z.object({ handleId: z.string().optional(), targetLevel: z.enum(['focus', 'working', 'archive']) });

// Tool定义工厂
const basePut = { handleId: { type: 'string' }, key: { type: 'string' }, value: { type: 'string' } };
const baseGet = { handleId: { type: 'string' }, key: { type: 'string' } };
const putTool = (level: string, desc: string): Tool => ({ name: `hajimi:memory_put_${level}`, description: desc, inputSchema: { type: 'object', properties: basePut, required: ['key', 'value'] } });
const getTool = (level: string, desc: string): Tool => ({ name: `hajimi:memory_get_${level}`, description: desc, inputSchema: { type: 'object', properties: baseGet, required: ['key'] } });

export const HAJIMI_TOOLS: Tool[] = [
  putTool('focus', '写入Focus内存层（4000 tokens，LRU淘汰）'),
  putTool('working', '写入Working内存层（16000 tokens）'),
  putTool('archive', '写入Archive内存层（100万tokens，mmap懒加载）'),
  getTool('focus', '读取Focus内存层'),
  getTool('working', '读取Working内存层'),
  getTool('archive', '读取Archive内存层（mmap懒加载，首次慢）'),
  getTool('any', '四级降级查询（Focus→Working→Archive）'),
  { name: 'hajimi:memory_stats', description: '获取四级内存统计信息', inputSchema: { type: 'object', properties: { handleId: { type: 'string' } } } },
  { name: 'hajimi:budget_get_default', description: '获取默认Token预算配置', inputSchema: { type: 'object', properties: {} } },
  { name: 'hajimi:gateway_create', description: '创建新的MemoryGateway实例', inputSchema: { type: 'object', properties: { focusLimit: { type: 'number' }, workingLimit: { type: 'number' }, archiveLimit: { type: 'number' } } } },
  { name: 'hajimi:gateway_drop', description: '释放MemoryGateway实例', inputSchema: { type: 'object', properties: { handleId: { type: 'string' } }, required: ['handleId'] } },
  { name: 'hajimi:memory_clear_focus', description: '清空Focus内存层', inputSchema: { type: 'object', properties: { handleId: { type: 'string' } } } },
  { name: 'hajimi:memory_clear_working', description: '清空Working内存层', inputSchema: { type: 'object', properties: { handleId: { type: 'string' } } } },
  { name: 'hajimi:memory_optimize', description: '优化内存布局', inputSchema: { type: 'object', properties: { handleId: { type: 'string' }, targetLevel: { type: 'string' } } } },
  { name: 'hajimi:health_check', description: '健康检查', inputSchema: { type: 'object', properties: {} } },
];

type ToolHandler = (args: unknown) => Promise<TextContent[]>;
const handlers: Map<string, ToolHandler> = new Map();

// FFI类型定义
interface FFI {
  createMemoryGateway: () => { id: string };
  memoryPut: (id: string, key: string, value: string, level: string) => Promise<void>;
  memoryGet: (id: string, key: string) => Promise<string | null>;
  memoryStats: (id: string) => Promise<{ focus_entries: number; focus_tokens: number; working_entries: number; working_tokens: number; archive_entries: number; archive_tokens: number; }>;
  getDefaultMemoryBudget: () => { focus: number; working: number; archive: number; rag: number };
  memoryClear: (id: string, level: MemoryLevel) => Promise<void>;
  memoryOptimize: (id: string, level: string) => Promise<unknown>;
}

let ffiRef: FFI;
const gateways = new Map<string, { id: string }>();
const getOrCreateGateway = (handleId?: string) => {
  const g = gateways.get(handleId || 'default') || ffiRef.createMemoryGateway();
  if (!handleId) gateways.set(g.id, g);
  return g;
};

// Handler工厂
const createPutHandler = (level: MemoryLevel): ToolHandler => async (args) => {
  const p = PutSchema.parse(args);
  const g = getOrCreateGateway(p.handleId);
  await ffiRef.memoryPut(g.id, p.key, p.value, level);
  return [{ type: 'text', text: JSON.stringify({ success: true, level }) }];
};

const createGetHandler = (): ToolHandler => async (args) => {
  const p = GetSchema.parse(args);
  const g = getOrCreateGateway(p.handleId);
  const v = await ffiRef.memoryGet(g.id, p.key);
  return [{ type: 'text', text: JSON.stringify({ key: p.key, value: v, found: v !== null }) }];
};

const createClearHandler = (level: MemoryLevel): ToolHandler => async (args) => {
  const p = ClearSchema.parse(args);
  const g = getOrCreateGateway(p.handleId);
  await ffiRef.memoryClear(g.id, level);
  return [{ type: 'text', text: JSON.stringify({ cleared: true, level, handleId: g.id }) }];
};

export function registerHandlers(ffi: FFI): void {
  ffiRef = ffi;

  // Put handlers (3)
  handlers.set('hajimi:memory_put_focus', createPutHandler(MemoryLevel.FOCUS));
  handlers.set('hajimi:memory_put_working', createPutHandler(MemoryLevel.WORKING));
  handlers.set('hajimi:memory_put_archive', createPutHandler(MemoryLevel.ARCHIVE));

  // Get handlers (4)
  ['focus', 'working', 'archive', 'any'].forEach(l => handlers.set(`hajimi:memory_get_${l}`, createGetHandler()));

  // Stats & Budget (2)
  handlers.set('hajimi:memory_stats', async (args) => {
    const g = getOrCreateGateway(GetSchema.partial().parse(args).handleId);
    return [{ type: 'text', text: JSON.stringify(await ffiRef.memoryStats(g.id)) }];
  });
  handlers.set('hajimi:budget_get_default', async () => [{ type: 'text', text: JSON.stringify(ffiRef.getDefaultMemoryBudget()) }]);

  // Gateway管理 (2)
  handlers.set('hajimi:gateway_create', async () => {
    const g = ffiRef.createMemoryGateway();
    gateways.set(g.id, g);
    return [{ type: 'text', text: JSON.stringify({ handleId: g.id }) }];
  });
  handlers.set('hajimi:gateway_drop', async (args) => {
    const dropped = gateways.delete(z.object({ handleId: z.string() }).parse(args).handleId);
    if (!dropped) throw new McpError(McpErrorCode.ResourceNotFound, 'Gateway not found');
    return [{ type: 'text', text: JSON.stringify({ dropped }) }];
  });

  // Clear handlers (2)
  handlers.set('hajimi:memory_clear_focus', createClearHandler(MemoryLevel.FOCUS));
  handlers.set('hajimi:memory_clear_working', createClearHandler(MemoryLevel.WORKING));

  // Optimize (1)
  handlers.set('hajimi:memory_optimize', async (args) => {
    const p = OptimizeSchema.parse(args);
    const g = getOrCreateGateway(p.handleId);
    const result = await ffiRef.memoryOptimize(g.id, p.targetLevel);
    return [{ type: 'text', text: JSON.stringify({ optimized: true, target: p.targetLevel, result, handleId: g.id }) }];
  });

  // Health check (1)
  handlers.set('hajimi:health_check', async () => [{ type: 'text', text: JSON.stringify({ status: 'ok', budget: ffiRef.getDefaultMemoryBudget() }) }]);
}

export async function handleToolCall(toolName: string, args: unknown): Promise<TextContent[]> {
  const handler = handlers.get(toolName);
  if (!handler) throw new McpError(McpErrorCode.InvalidParams, `Unknown tool: ${toolName}`);
  return handler(args);
}

export function cleanupToolHandlers(): void { handlers.clear(); gateways.clear(); }
