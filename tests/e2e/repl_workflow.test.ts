/**
 * E2E-001: REPL全流程测试 - EventLoop+WebUI端到端
 * 验证: 输入→执行→流式输出→历史，<2s
 */
import { describe, it, beforeAll, afterAll } from 'node:test';
import { strict as assert } from 'node:assert';
import { spawn } from 'child_process';
import { WebSocket } from 'ws';
import { setTimeout } from 'timers/promises';

const TEST_TIMEOUT = 30000;
const WS_URL = 'ws://localhost:8080/ws';
const PERF_THRESHOLD_MS = 2000;

describe('E2E-001: REPL全流程测试', { timeout: TEST_TIMEOUT }, () => {
  let server: any;
  let ws: WebSocket;

  beforeAll(async () => {
    server = spawn('cargo', ['run', '--package', 'chimera-repl'], {
      cwd: process.cwd(), env: { ...process.env, RUST_LOG: 'info' }
    });
    await setTimeout(3000);
  });

  afterAll(async () => {
    if (ws?.readyState === WebSocket.OPEN) ws.close();
    if (server) { server.kill('SIGTERM'); await setTimeout(500); if (!server.killed) server.kill('SIGKILL'); }
  });

  it('应建立WebSocket连接', async () => {
    ws = new WebSocket(WS_URL);
    await new Promise<void>((resolve, reject) => { ws.on('open', resolve); ws.on('error', reject); });
    assert.strictEqual(ws.readyState, WebSocket.OPEN);
  });

  it('应接收会话ID', async () => {
    const msg = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.ok(msg.sessionId, '应接收会话ID');
  });

  it('应执行命令并返回结果<2s', async () => {
    const start = Date.now();
    ws.send(JSON.stringify({ type: 'execute', code: '1 + 1' }));
    const res = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.strictEqual(res.type, 'result');
    assert.ok(Date.now() - start < PERF_THRESHOLD_MS, '执行<2s');
  });

  it('应支持流式输出', async () => {
    const chunks: any[] = [];
    ws.send(JSON.stringify({ type: 'execute', code: '"test".chars()' }));
    await new Promise<void>((resolve) => {
      const to = setTimeout(() => resolve(), 1000);
      const handler = (d: Buffer) => {
        const m = JSON.parse(d.toString()); chunks.push(m);
        if (m.type === 'stream_end') { clearTimeout(to); ws.off('message', handler); resolve(); }
      };
      ws.on('message', handler);
    });
    assert.ok(chunks.length >= 2 && chunks.some(c => c.type === 'stream_start') && chunks.some(c => c.type === 'stream_end'));
  });

  it('应保存命令历史', async () => {
    ws.send(JSON.stringify({ type: 'get_history' }));
    const h = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.strictEqual(h.type, 'history');
    assert.ok(Array.isArray(h.entries) && h.entries.length >= 1);
  });

  it('应处理多行输入', async () => {
    const code = `fn f() -> i32 { 42 } f()`;
    ws.send(JSON.stringify({ type: 'execute', code }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.strictEqual(r.type, 'result');
  });

  it('应优雅处理错误', async () => {
    ws.send(JSON.stringify({ type: 'execute', code: 'bad_var' }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.ok(r.type === 'error' || r.error);
  });

  it('完整REPL流程性能<2s', async () => {
    const w = new WebSocket(WS_URL);
    const s = Date.now();
    await new Promise<void>((resolve, reject) => { w.on('open', resolve); w.on('error', reject); });
    w.send(JSON.stringify({ type: 'execute', code: '2+2' }));
    await new Promise<void>((resolve) => w.once('message', () => resolve()));
    const d = Date.now() - s;
    w.close();
    assert.ok(d < PERF_THRESHOLD_MS, `流程<2s,实际${d}ms`);
    console.log(`[E2E-001] PASSED: ${d}ms`);
  });
});
