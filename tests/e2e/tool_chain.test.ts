/**
 * E2E-003: 工具调用链测试 - Tools+WebUI端到端
 * 通过标准: 命令输入→工具选择→执行→结果展示
 */
import { describe, it, beforeAll, afterAll } from 'node:test';
import { strict as assert } from 'node:assert';
import { WebSocket } from 'ws';

const TEST_TIMEOUT = 30000;
const WS_URL = 'ws://localhost:8080/ws';

describe('E2E-003: 工具调用链测试', { timeout: TEST_TIMEOUT }, () => {
  let ws: WebSocket;

  beforeAll(async () => {
    ws = new WebSocket(WS_URL);
    await new Promise<void>((resolve, reject) => { ws.on('open', resolve); ws.on('error', reject); });
    await new Promise<void>((resolve) => ws.once('message', () => resolve()));
  });

  afterAll(() => { if (ws.readyState === WebSocket.OPEN) ws.close(); });

  it('应初始化工具注册表', async () => {
    ws.send(JSON.stringify({ type: 'tools_init' }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.strictEqual(r.type, 'tools_ready');
    assert.ok(Array.isArray(r.tools) && r.tools.length > 0);
  });

  it('应返回可用工具列表', async () => {
    ws.send(JSON.stringify({ type: 'list_tools' }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.strictEqual(r.type, 'tool_list');
    const n = r.tools.map((t: any) => t.name);
    ['read_file', 'write_file', 'grep', 'find', 'ls'].forEach(t => assert.ok(n.includes(t)));
  });

  it('应解析自然语言命令', async () => {
    ws.send(JSON.stringify({ type: 'parse_command', command: '查找TODO的Rust文件' }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.strictEqual(r.type, 'tool_selection');
    assert.ok(r.tool && r.confidence > 0);
  });

  it('应执行ReadFile工具', async () => {
    ws.send(JSON.stringify({ type: 'execute_tool', tool: 'read_file', args: { path: 'Cargo.toml' } }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.strictEqual(r.type, 'tool_result');
    assert.ok(r.success && (r.stdout || r.data));
  });

  it('应执行Ls工具', async () => {
    ws.send(JSON.stringify({ type: 'execute_tool', tool: 'ls', args: { path: '.' } }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.strictEqual(r.type, 'tool_result');
    assert.ok(r.success);
  });

  it('应执行Grep工具', async () => {
    ws.send(JSON.stringify({ type: 'execute_tool', tool: 'grep', args: { pattern: 'fn main', path: 'src' } }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.strictEqual(r.type, 'tool_result');
    assert.ok(r.success !== undefined);
  });

  it('应处理工具执行错误', async () => {
    ws.send(JSON.stringify({ type: 'execute_tool', tool: 'read_file', args: { path: '/nonexistent/file.txt' } }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.ok(!r.success || r.error);
  });

  it('应执行工具链', async () => {
    ws.send(JSON.stringify({ type: 'execute_tool_chain', steps: [{ tool: 'find', args: { pattern: '*.rs' } }, { tool: 'grep', args: { pattern: 'TODO' } }] }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.strictEqual(r.type, 'tool_chain_result');
    assert.ok(Array.isArray(r.results));
  });

  it('应验证工具权限', async () => {
    ws.send(JSON.stringify({ type: 'check_permission', tool: 'write_file', args: { path: 'src/main.rs' } }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.strictEqual(r.type, 'permission_check');
    assert.ok(['allow', 'ask', 'deny'].includes(r.level));
    console.log('[E2E-003] PASSED');
  });
});
