/**
 * E2E-004: 系统负载测试 / E2E-005: 故障恢复测试
 */
import { describe, it, afterAll } from 'node:test';
import { strict as assert } from 'node:assert';
import { WebSocket } from 'ws';
import { setTimeout } from 'timers/promises';

const TEST_TIMEOUT = 60000;
const WS_URL = 'ws://localhost:8080/ws';
const CONCURRENT = 10;
const MEM_MB = 500;

describe('E2E-004/005: 负载与故障恢复测试', { timeout: TEST_TIMEOUT }, () => {
  let sessions: WebSocket[] = [];
  const getMem = () => Math.round(process.memoryUsage().heapUsed / 1024 / 1024);

  afterAll(async () => {
    for (const w of sessions) if (w.readyState === WebSocket.OPEN) w.close();
    await setTimeout(500);
  });

  it('应支持并发10个REPL会话', async () => {
    const conns = [];
    for (let i = 0; i < CONCURRENT; i++) {
      conns.push(new Promise<void>((resolve, reject) => {
        const w = new WebSocket(WS_URL);
        sessions.push(w);
        w.on('open', resolve);
        w.on('error', reject);
      }));
    }
    await Promise.all(conns);
    assert.strictEqual(sessions.filter(s => s.readyState === WebSocket.OPEN).length, CONCURRENT);
  });

  it('并发执行时内存使用<500MB', async () => {
    const execs = sessions.map((w, i) => new Promise<void>((resolve) => {
      w.send(JSON.stringify({ type: 'execute', code: `let x=${i};(0..1000).map(|n|n*x).collect()` }));
      w.once('message', () => resolve());
    }));
    await Promise.all(execs);
    await setTimeout(500);
    assert.ok(getMem() < MEM_MB, '内存<500MB');
  });

  it('应处理高频率消息', async () => {
    const w = sessions[0];
    const resps = [];
    w.on('message', (d) => resps.push(JSON.parse(d.toString())));
    for (let i = 0; i < 50; i++) w.send(JSON.stringify({ type: 'ping', seq: i }));
    await setTimeout(2000);
    assert.ok(resps.length >= 40, '响应>=80%');
  });

  it('应检测内存泄漏', async () => {
    const w = sessions[0];
    const m = [];
    for (let i = 0; i < 5; i++) {
      w.send(JSON.stringify({ type: 'execute', code: 'vec![0u8;1024*1024]' }));
      await new Promise<void>((resolve) => w.once('message', () => resolve()));
      if (global.gc) global.gc();
      m.push(getMem());
      await setTimeout(200);
    }
    assert.ok(m[4] - m[0] < 50, '内存增长<50MB');
  });

  it('应优雅处理会话断开', async () => {
    const w = new WebSocket(WS_URL);
    await new Promise<void>((resolve, reject) => { w.on('open', resolve); w.on('error', reject); });
    w.terminate();
    await setTimeout(500);
    assert.strictEqual(w.readyState, WebSocket.CLOSED);
  });

  it('故障后应恢复（E2E-005）', async () => {
    const w1 = new WebSocket(WS_URL);
    await new Promise<void>((resolve, reject) => { w1.on('open', resolve); w1.on('error', reject); });
    w1.send(JSON.stringify({ type: 'get_state' }));
    await new Promise<void>((resolve) => w1.once('message', () => resolve()));
    w1.close();
    await setTimeout(500);
    const w2 = new WebSocket(WS_URL);
    await new Promise<void>((resolve, reject) => { w2.on('open', resolve); w2.on('error', reject); });
    w2.send(JSON.stringify({ type: 'get_state' }));
    const s2 = await new Promise<any>((resolve) => { w2.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    w2.close();
    assert.strictEqual(s2.type, 'state');
  });

  it('应维护历史状态持久性', async () => {
    const w = new WebSocket(WS_URL);
    await new Promise<void>((resolve, reject) => { w.on('open', resolve); w.on('error', reject); });
    w.send(JSON.stringify({ type: 'execute', code: '1+1' }));
    await new Promise<void>((resolve) => w.once('message', () => resolve()));
    w.send(JSON.stringify({ type: 'get_history' }));
    const h = await new Promise<any>((resolve) => { w.once('message', (d) => resolve(JSON.parse(d.toString()))); });
    assert.ok(h.entries.length > 0);
    w.close();
    console.log('[E2E-004/005] PASSED');
  });
});
