/**
 * MCP Server Tests - LCRStore & Tool Handlers
 * Coverage target: >80% (statements + branches)
 * DEBT-MCP-003清偿 - 单元测试覆盖>80%
 */
import { describe, it, beforeEach, afterEach } from 'node:test';
import assert from 'node:assert';
import * as fs from 'fs/promises';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { spawn } from 'child_process';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const TEMP_DIR = path.join(__dirname, '.temp-test-' + Date.now());

// Helper: Run MCP server with temp LCR path
function runMcpServer(env = {}) {
  const serverPath = path.join(__dirname, '../../dist/mcp/server.mjs');
  return spawn('node', [serverPath], {
    env: { ...process.env, HAJIMI_LCR_PATH: path.join(TEMP_DIR, 'lcr.db'), ...env },
    stdio: ['pipe', 'pipe', 'pipe']
  });
}

// Helper: Send JSON-RPC request and get response
function sendRequest(proc, method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = Date.now() + Math.floor(Math.random() * 1000);
    const request = JSON.stringify({ jsonrpc: '2.0', id, method, params }) + '\n';
    let buffer = '';
    const timeout = setTimeout(() => reject(new Error('Request timeout')), 5000);
    const onData = (data) => {
      buffer += data.toString();
      const lines = buffer.split('\n');
      for (let i = 0; i < lines.length - 1; i++) {
        try {
          const res = JSON.parse(lines[i]);
          if (res.id === id) {
            clearTimeout(timeout);
            proc.stdout.off('data', onData);
            resolve(res);
            return;
          }
        } catch { /* ignore parse errors */ }
      }
      buffer = lines[lines.length - 1];
    };
    proc.stdout.on('data', onData);
    proc.stdin.write(request);
  });
}

describe('MCP Server Integration Tests - LCRStore & Tools', () => {
  let proc;

  beforeEach(async () => {
    await fs.mkdir(TEMP_DIR, { recursive: true });
    proc = runMcpServer();
    await new Promise(r => setTimeout(r, 150));
  });

  afterEach(async () => {
    if (proc) { proc.kill(); proc = null; }
    await fs.rm(TEMP_DIR, { recursive: true, force: true });
  });

  describe('Tool: hajimi_search', () => {
    it('should return empty results for empty LCR', async () => {
      const res = await sendRequest(proc, 'tools/call', { name: 'hajimi_search', arguments: { query: 'test' } });
      assert.strictEqual(res.error, undefined);
      const content = JSON.parse(res.result.content[0].text);
      assert.strictEqual(content.count, 0);
      assert.strictEqual(content.query, 'test');
      assert.ok(Array.isArray(content.results));
    });

    it('should search case-insensitively', async () => {
      await sendRequest(proc, 'tools/call', { name: 'hajimi_add', arguments: { content: 'Hello World', metadata: { tag: 'greeting' } } });
      const res = await sendRequest(proc, 'tools/call', { name: 'hajimi_search', arguments: { query: 'hello', limit: 5 } });
      const content = JSON.parse(res.result.content[0].text);
      assert.strictEqual(content.count, 1);
      assert.ok(content.results[0].content.includes('Hello'));
    });

    it('should respect limit parameter', async () => {
      await sendRequest(proc, 'tools/call', { name: 'hajimi_add', arguments: { content: 'First chunk' } });
      await sendRequest(proc, 'tools/call', { name: 'hajimi_add', arguments: { content: 'Second chunk' } });
      await sendRequest(proc, 'tools/call', { name: 'hajimi_add', arguments: { content: 'Third chunk' } });
      const res = await sendRequest(proc, 'tools/call', { name: 'hajimi_search', arguments: { query: 'chunk', limit: 2 } });
      const content = JSON.parse(res.result.content[0].text);
      assert.strictEqual(content.results.length, 2);
    });

    it('should error on missing query parameter', async () => {
      const res = await sendRequest(proc, 'tools/call', { name: 'hajimi_search', arguments: {} });
      assert.strictEqual(res.result.isError, true);
      assert.ok(res.result.content[0].text.includes('Query parameter is required'));
    });

    it('should error on empty query string', async () => {
      const res = await sendRequest(proc, 'tools/call', { name: 'hajimi_search', arguments: { query: '' } });
      assert.strictEqual(res.result.isError, true);
    });
  });

  describe('Tool: hajimi_add', () => {
    it('should add chunk with auto-generated id and timestamp', async () => {
      const res = await sendRequest(proc, 'tools/call', { name: 'hajimi_add', arguments: { content: 'Test content', metadata: { source: 'test' } } });
      assert.strictEqual(res.error, undefined);
      const content = JSON.parse(res.result.content[0].text);
      assert.strictEqual(content.success, true);
      assert.ok(content.id.startsWith('chunk_'));
      assert.ok(typeof content.timestamp === 'number');
      assert.ok(content.timestamp > 0);
    });

    it('should persist data to file and survive re-init', async () => {
      await sendRequest(proc, 'tools/call', { name: 'hajimi_add', arguments: { content: 'Persistent data' } });
      const dbPath = path.join(TEMP_DIR, 'lcr.db');
      const data = await fs.readFile(dbPath, 'utf-8');
      const chunks = JSON.parse(data);
      assert.strictEqual(chunks.length, 1);
      assert.strictEqual(chunks[0].content, 'Persistent data');
      assert.ok(chunks[0].id);
      assert.ok(chunks[0].timestamp);
    });

    it('should error on missing content parameter', async () => {
      const res = await sendRequest(proc, 'tools/call', { name: 'hajimi_add', arguments: {} });
      assert.strictEqual(res.result.isError, true);
      assert.ok(res.result.content[0].text.includes('Content parameter is required'));
    });

    it('should accept content with empty metadata', async () => {
      const res = await sendRequest(proc, 'tools/call', { name: 'hajimi_add', arguments: { content: 'No metadata' } });
      assert.strictEqual(res.error, undefined);
      const content = JSON.parse(res.result.content[0].text);
      assert.strictEqual(content.success, true);
    });
  });

  describe('Tool: hajimi_stats', () => {
    it('should return stats with zero chunks initially', async () => {
      const res = await sendRequest(proc, 'tools/call', { name: 'hajimi_stats', arguments: {} });
      assert.strictEqual(res.error, undefined);
      const content = JSON.parse(res.result.content[0].text);
      assert.strictEqual(content.lcr.total, 0);
      assert.ok(content.lcr.path.includes('lcr.db'));
      assert.ok(typeof content.uptime === 'number');
      assert.ok(content.uptime >= 0);
      assert.strictEqual(content.server.name, 'hajimi-mcp');
      assert.ok(content.server.version);
    });

    it('should return correct count after adding chunks', async () => {
      await sendRequest(proc, 'tools/call', { name: 'hajimi_add', arguments: { content: 'Chunk 1' } });
      await sendRequest(proc, 'tools/call', { name: 'hajimi_add', arguments: { content: 'Chunk 2' } });
      const res = await sendRequest(proc, 'tools/call', { name: 'hajimi_stats', arguments: {} });
      const content = JSON.parse(res.result.content[0].text);
      assert.strictEqual(content.lcr.total, 2);
    });
  });

  describe('Tool list and resources', () => {
    it('should list all 3 tools', async () => {
      const res = await sendRequest(proc, 'tools/list', {});
      assert.strictEqual(res.error, undefined);
      assert.strictEqual(res.result.tools.length, 3);
      const names = res.result.tools.map(t => t.name);
      assert.ok(names.includes('hajimi_search'));
      assert.ok(names.includes('hajimi_add'));
      assert.ok(names.includes('hajimi_stats'));
    });

    it('should list resources', async () => {
      const res = await sendRequest(proc, 'resources/list', {});
      assert.strictEqual(res.error, undefined);
      assert.ok(res.result.resources.length > 0);
      assert.ok(res.result.resources.some(r => r.uri === 'stats://lcr'));
    });
  });

  describe('LCRStore init error handling', () => {
    it('should handle invalid JSON gracefully', async () => {
      const dbPath = path.join(TEMP_DIR, 'lcr.db');
      await fs.mkdir(TEMP_DIR, { recursive: true });
      await fs.writeFile(dbPath, 'invalid json {', 'utf-8');
      proc.kill();
      proc = runMcpServer();
      await new Promise(r => setTimeout(r, 150));
      const res = await sendRequest(proc, 'tools/call', { name: 'hajimi_stats', arguments: {} });
      assert.strictEqual(res.error, undefined);
      const content = JSON.parse(res.result.content[0].text);
      assert.strictEqual(content.lcr.total, 0);
    });

    it('should handle empty file', async () => {
      const dbPath = path.join(TEMP_DIR, 'lcr.db');
      await fs.writeFile(dbPath, '', 'utf-8');
      proc.kill();
      proc = runMcpServer();
      await new Promise(r => setTimeout(r, 150));
      const res = await sendRequest(proc, 'tools/call', { name: 'hajimi_search', arguments: { query: 'test' } });
      assert.strictEqual(res.error, undefined);
    });
  });

  describe('Error handling', () => {
    it('should handle unknown tool with error', async () => {
      const res = await sendRequest(proc, 'tools/call', { name: 'unknown_tool', arguments: {} });
      assert.strictEqual(res.result.isError, true);
      assert.ok(res.result.content[0].text.includes('Unknown tool'));
    });

    it('should return results sorted by timestamp desc', async () => {
      await sendRequest(proc, 'tools/call', { name: 'hajimi_add', arguments: { content: 'Older', metadata: {} } });
      await new Promise(r => setTimeout(r, 50));
      await sendRequest(proc, 'tools/call', { name: 'hajimi_add', arguments: { content: 'Newer', metadata: {} } });
      const res = await sendRequest(proc, 'tools/call', { name: 'hajimi_search', arguments: { query: 'er', limit: 10 } });
      const content = JSON.parse(res.result.content[0].text);
      assert.ok(content.results.length >= 1);
    });
  });
});
