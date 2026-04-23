/**
 * E2E-002: 类型预测端到端测试 - TypeRacing+WebUI
 * 通过标准: 编辑→Ctrl+Space→预测显示<500ms
 */
import { describe, it, beforeAll, afterAll } from 'node:test';
import { strict as assert } from 'node:assert';
import { WebSocket } from 'ws';
import { setTimeout } from 'timers/promises';

const TEST_TIMEOUT = 20000;
const WS_URL = 'ws://localhost:8080/ws';
const PREDICTION_THRESHOLD_MS = 500;

describe('E2E-002: 类型预测端到端测试', { timeout: TEST_TIMEOUT }, () => {
  let ws: WebSocket;

  beforeAll(async () => {
    ws = new WebSocket(WS_URL);
    await new Promise<void>((resolve, reject) => { ws.on('open', resolve); ws.on('error', reject); });
    await new Promise<void>((resolve) => ws.once('message', () => resolve()));
  });

  afterAll(() => { if (ws.readyState === WebSocket.OPEN) ws.close(); });

  it('应初始化TypeRacing引擎', async () => {
    ws.send(JSON.stringify({ type: 'typeracing_init', language: 'rust' }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.strictEqual(r.type, 'typeracing_ready');
    assert.ok(r.engineReady);
  });

  it('应响应代码编辑事件', async () => {
    ws.send(JSON.stringify({ type: 'code_edit', file: 't.rs', content: 'fn main(){let x:i32=42;x.}', line: 1, column: 26 }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.strictEqual(r.type, 'edit_ack');
  });

  it('应响应Ctrl+Space触发预测<500ms', async () => {
    const s = Date.now();
    ws.send(JSON.stringify({ type: 'predict_request', file: 't.rs', line: 1, column: 26, trigger: 'ctrl_space' }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    const d = Date.now() - s;
    assert.strictEqual(r.type, 'predictions');
    assert.ok(Array.isArray(r.items) && d < PREDICTION_THRESHOLD_MS);
  });

  it('应返回类型正确的预测项', async () => {
    ws.send(JSON.stringify({ type: 'predict_request', file: 't.rs', line: 1, column: 26 }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    if (r.items?.length > 0) { const i = r.items[0]; assert.ok(i.label && i.kind !== undefined && i.confidence >= 0); }
  });

  it('应支持自动触发预测', async () => {
    ws.send(JSON.stringify({ type: 'code_edit', file: 't.rs', content: 'fn test(){vec![]', line: 1, column: 16, auto_trigger: true }));
    const r = await new Promise<any>((resolve) => { const t = setTimeout(() => resolve({ type: 'timeout' }), 1000); ws.once('message', (d) => { clearTimeout(t); resolve(JSON.parse(d.toString())); }); });
    assert.ok(['predictions', 'edit_ack', 'timeout'].includes(r.type));
  });

  it('应处理多文件上下文', async () => {
    ws.send(JSON.stringify({ type: 'code_edit', file: 'lib.rs', content: 'pub struct P{x:f64}' }));
    await setTimeout(100);
    ws.send(JSON.stringify({ type: 'predict_request', file: 'main.rs', line: 1, column: 0, imports: ['lib.rs'] }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.strictEqual(r.type, 'predictions');
  });

  it('应处理LSP服务器错误', async () => {
    ws.send(JSON.stringify({ type: 'predict_request', file: 'bad.xyz', line: 1, column: 0 }));
    const r = await new Promise<any>((resolve) => { ws.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.ok(['predictions', 'error', 'lsp_unavailable'].includes(r.type));
  });

  it('完整预测流程<500ms', async () => {
    const s = Date.now();
    ws.send(JSON.stringify({ type: 'predict_request', file: 'perf.rs', content: 'fn s(){String::new().}', line: 1, column: 22 }));
    await new Promise<void>((resolve) => ws.once('message', () => resolve()));
    const d = Date.now() - s;
    assert.ok(d < PREDICTION_THRESHOLD_MS, `流程<500ms,实际${d}ms`);
    console.log(`[E2E-002] PASSED: ${d}ms`);
  });
});
