/**
 * MCP-08: Lifecycle Management - initialize → initialized → operation → shutdown → exit
 * State machine: disconnected → connecting → connected → closing → closed
 */

import type { JSONRPCMessage, JSONRPCRequest, JSONRPCNotification, JSONRPCResponse } from './protocol/jsonrpc';
import { MCPError } from './protocol/errors';
import { MCPErrorCode } from './protocol/jsonrpc';
import type { MessageTransport } from './transport/message-adapter';
import type { ToolsCapability } from './capabilities/tools';
import type { ResourcesCapability } from './capabilities/resources';
import type { PromptsCapability } from './capabilities/prompts';

export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'closing' | 'closed';
export const SUPPORTED_PROTOCOL_VERSION = '2024-11-05';

interface ClientCapabilities { tools?: { listChanged?: boolean }; resources?: { listChanged?: boolean; subscribe?: boolean }; prompts?: { listChanged?: boolean }; }
export interface ServerCapabilities { tools?: { listChanged?: boolean }; resources?: { listChanged?: boolean; subscribe?: boolean }; prompts?: { listChanged?: boolean }; }
interface InitializeRequest { protocolVersion: string; capabilities: ClientCapabilities; clientInfo: { name: string; version: string }; }
interface InitializeResult { protocolVersion: string; capabilities: ServerCapabilities; serverInfo: { name: string; version: string }; }

export class MCPServer {
  private state: ConnectionState = 'disconnected';
  private transport?: MessageTransport;
  private readonly capabilities: ServerCapabilities = {};

  constructor(private readonly serverInfo: { name: string; version: string }, private readonly tools?: ToolsCapability, private readonly resources?: ResourcesCapability, private readonly prompts?: PromptsCapability) {
    if (tools) this.capabilities.tools = { listChanged: true };
    if (resources) this.capabilities.resources = { listChanged: true, subscribe: true };
    if (prompts) this.capabilities.prompts = { listChanged: true };
  }

  get connectionState(): ConnectionState { return this.state; }

  attach(transport: MessageTransport): void {
    if (this.state !== 'disconnected') throw new MCPError(MCPErrorCode.CAPABILITY_ERROR, 'Server already attached');
    this.transport = transport;
    transport.onMessage = (msg) => this.handleMessage(msg);
    transport.onClose = () => { this.state = 'closed'; };
  }

  private handleMessage(msg: JSONRPCMessage): void {
    if (!('method' in msg) || !msg.method) return;
    const req = msg as JSONRPCRequest;
    switch (req.method) {
      case 'initialize': this.handleInitialize(req); break;
      case 'shutdown': this.handleShutdown(req); break;
      default: this.handleOperation(req);
    }
  }

  private handleInitialize(req: JSONRPCRequest): void {
    if (this.state !== 'disconnected') { this.sendError(req.id, new MCPError(MCPErrorCode.CAPABILITY_ERROR, 'Already initialized')); return; }
    this.state = 'connecting';
    const params = req.params as InitializeRequest;
    if (params.protocolVersion !== SUPPORTED_PROTOCOL_VERSION) {
      this.state = 'disconnected'; this.sendError(req.id, new MCPError(MCPErrorCode.CAPABILITY_ERROR, `Version mismatch: ${params.protocolVersion}`)); return;
    }
    const result: InitializeResult = { protocolVersion: SUPPORTED_PROTOCOL_VERSION, capabilities: this.capabilities, serverInfo: this.serverInfo };
    this.sendResponse(req.id, result); this.state = 'connected'; this.sendNotification('initialized', {});
    if (this.transport) { this.tools?.attach(this.transport); this.resources?.attach(this.transport); this.prompts?.attach(this.transport); }
  }

  private handleOperation(req: JSONRPCRequest): void {
    if (this.state !== 'connected') { this.sendError(req.id, new MCPError(MCPErrorCode.CAPABILITY_ERROR, 'Server not initialized')); return; }
  }

  private handleShutdown(req: JSONRPCRequest): void {
    if (this.state !== 'connected') { this.sendError(req.id, new MCPError(MCPErrorCode.CAPABILITY_ERROR, 'Not connected')); return; }
    this.state = 'closing'; this.sendResponse(req.id, {}); this.sendNotification('exit', {}); this.state = 'closed'; this.transport?.close();
  }

  private sendResponse(id: number | string, result: unknown): void {
    if (!this.transport?.isConnected()) return;
    this.transport.send({ jsonrpc: '2.0', id, result } as JSONRPCResponse);
  }

  private sendError(id: number | string, err: MCPError): void {
    if (!this.transport?.isConnected()) return;
    this.transport.send(err.toJSONRPCError(id));
  }

  private sendNotification(method: string, params: unknown): void {
    if (!this.transport?.isConnected()) return;
    this.transport.send({ jsonrpc: '2.0', method, params } as JSONRPCNotification);
  }

  close(): void {
    if (this.state === 'disconnected' || this.state === 'closed') return;
    this.state = 'closing'; this.transport?.close(); this.state = 'closed';
  }
}
