/**
 * MCP-10: Integration Test Suite - initialize → tool call → resource read → prompt get
 * Performance: 1000 concurrent requests
 */

import { describe, test, expect, beforeAll, afterAll } from '@jest/globals';
import { MCPServer, SUPPORTED_PROTOCOL_VERSION } from '../lifecycle';
import { ToolsCapability, TOOLS_LIST_ENDPOINT, TOOLS_CALL_ENDPOINT } from '../capabilities/tools';
import { ResourcesCapability, RESOURCES_LIST_ENDPOINT, RESOURCES_READ_ENDPOINT } from '../capabilities/resources';
import { PromptsCapability, PROMPTS_LIST_ENDPOINT, PROMPTS_GET_ENDPOINT } from '../capabilities/prompts';
import { JSONRPCMessage, JSONRPCRequest, JSONRPCResponse, isResponse } from '../protocol/jsonrpc';
import * as z from 'zod';
import { MessageTransport } from '../transport/message-adapter';

/** In-memory transport for testing */
class InMemoryTransport implements MessageTransport {
  onMessage?: (msg: JSONRPCMessage) => void; onError?: (error: Error) => void; onClose?: () => void;
  private connected = true; private peer?: InMemoryTransport;
  connect(peer: InMemoryTransport): void { this.peer = peer; peer.peer = this; }
  send(msg: JSONRPCMessage): void { if (!this.connected || !this.peer) throw new Error('Not connected'); setImmediate(() => this.peer!.onMessage?.(msg)); }
  close(): void { this.connected = false; this.peer?.onClose?.(); }
  isConnected(): boolean { return this.connected; }
}

type ToolListResult = { tools: unknown[] };
type ResourceListResult = { resources: unknown[] };
type PromptListResult = { prompts: unknown[] };
type PromptResult = { messages: { content: { text: string } }[] };

/** Test suite: MCP Integration - initialize → tool call → resource read → prompt get */
describe('MCP Integration', () => {
  let serverTransport: InMemoryTransport, clientTransport: InMemoryTransport;
  let server: MCPServer, tools: ToolsCapability, resources: ResourcesCapability, prompts: PromptsCapability;
  let requestId = 0;
  const sendRequest = (method: string, params?: unknown): Promise<JSONRPCResponse> => new Promise((resolve) => {
    const id = ++requestId, handler = (msg: JSONRPCMessage) => { if (isResponse(msg) && msg.id === id) { clientTransport.onMessage = undefined; resolve(msg as JSONRPCResponse); } };
    clientTransport.onMessage = handler; clientTransport.send({ jsonrpc: '2.0', id, method, params } as JSONRPCRequest);
  });

  beforeAll(async () => {
    serverTransport = new InMemoryTransport(); clientTransport = new InMemoryTransport(); serverTransport.connect(clientTransport);
    tools = new ToolsCapability(); tools.register({ name: 'echo', description: 'Echo tool', inputSchema: z.object({ message: z.string() }), handler: (p) => ({ result: p.message }) });
    resources = new ResourcesCapability(); resources.register({ uri: 'file:///test.txt', name: 'test', mimeType: 'text/plain' }, () => ({ uri: 'file:///test.txt', text: 'Hello Resource' }));
    prompts = new PromptsCapability(); prompts.register({ name: 'greeting', description: 'Greeting prompt', arguments: [{ name: 'name', required: true, type: 'string' }], render: (a) => [{ role: 'user', content: { type: 'text', text: `Hello ${a.name}` } }] });
    server = new MCPServer({ name: 'test-server', version: '1.0.0' }, tools, resources, prompts); server.attach(serverTransport);
  });

  afterAll(async () => { server.close(); clientTransport.close(); });

  describe('Lifecycle', () => {
    test('initialize handshake returns correct protocol version', async () => {
      const res = await sendRequest('initialize', { protocolVersion: SUPPORTED_PROTOCOL_VERSION, capabilities: {}, clientInfo: { name: 'test', version: '1.0.0' } });
      expect(res.result).toMatchObject({ protocolVersion: SUPPORTED_PROTOCOL_VERSION, serverInfo: { name: 'test-server', version: '1.0.0' } });
    });
    test('initialized notification sent after handshake', async () => {
      let notified = false; clientTransport.onMessage = (msg) => { if ('method' in msg && msg.method === 'initialized') notified = true; };
      await sendRequest('initialize', { protocolVersion: SUPPORTED_PROTOCOL_VERSION, capabilities: {}, clientInfo: { name: 'test', version: '1.0.0' } }); await new Promise(r => setTimeout(r, 50)); expect(notified).toBe(true);
    });
  });

  describe('Tools Capability', () => {
    test('tools/list returns registered tools', async () => { const res = await sendRequest(TOOLS_LIST_ENDPOINT); expect((res.result as ToolListResult).tools).toHaveLength(1); });
    test('tools/call executes handler and returns result', async () => { const res = await sendRequest(TOOLS_CALL_ENDPOINT, { name: 'echo', arguments: { message: 'hello' } }); expect(res.result).toEqual({ result: 'hello' }); });
  });

  describe('Resources Capability', () => {
    test('resources/list returns registered resources', async () => { const res = await sendRequest(RESOURCES_LIST_ENDPOINT); expect((res.result as ResourceListResult).resources).toHaveLength(1); });
    test('resources/read returns resource content', async () => { const res = await sendRequest(RESOURCES_READ_ENDPOINT, { uri: 'file:///test.txt' }); expect(res.result).toMatchObject({ uri: 'file:///test.txt', text: 'Hello Resource' }); });
  });

  describe('Prompts Capability', () => {
    test('prompts/list returns registered prompts', async () => { const res = await sendRequest(PROMPTS_LIST_ENDPOINT); expect((res.result as PromptListResult).prompts).toHaveLength(1); });
    test('prompts/get renders prompt with arguments', async () => { const res = await sendRequest(PROMPTS_GET_ENDPOINT, { name: 'greeting', arguments: { name: 'World' } }); expect((res.result as PromptResult).messages[0].content.text).toBe('Hello World'); });
  });

  describe('End-to-End Flow', () => {
    test('initialize with tool call resource read and prompt get', async () => {
      const init = await sendRequest('initialize', { protocolVersion: SUPPORTED_PROTOCOL_VERSION, capabilities: {}, clientInfo: { name: 'test', version: '1.0.0' } }); expect(init.result).toHaveProperty('protocolVersion');
      const toolRes = await sendRequest(TOOLS_CALL_ENDPOINT, { name: 'echo', arguments: { message: 'e2e' } }); expect(toolRes.result).toEqual({ result: 'e2e' });
      const resourceRes = await sendRequest(RESOURCES_READ_ENDPOINT, { uri: 'file:///test.txt' }); expect(resourceRes.result).toHaveProperty('text');
      const promptRes = await sendRequest(PROMPTS_GET_ENDPOINT, { name: 'greeting', arguments: { name: 'E2E' } }); expect((promptRes.result as PromptResult).messages[0].content.text).toBe('Hello E2E');
    });
  });

  describe('Performance', () => {
    test('1000 concurrent requests within 5s and 200MB memory', async () => {
      const startTime = Date.now(), startMem = process.memoryUsage().heapUsed;
      const results = await Promise.all(Array(1000).fill(null).map((_, i) => sendRequest(TOOLS_CALL_ENDPOINT, { name: 'echo', arguments: { message: `req${i}` } })));
      const duration = Date.now() - startTime, memUsedMB = (process.memoryUsage().heapUsed - startMem) / 1024 / 1024;
      expect(results).toHaveLength(1000); expect(duration).toBeLessThan(5000); expect(memUsedMB).toBeLessThan(200);
    });
  });
});
