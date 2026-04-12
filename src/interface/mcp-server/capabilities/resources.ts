/** MCP-06: Resources Capability - Resource接口 + resources/list/read端点 + URI解析 + 订阅机制 */

import type { JSONRPCMessage, JSONRPCRequest, JSONRPCNotification } from '../protocol/jsonrpc';
import { MCPErrorCode, JSONRPCErrorCode } from '../protocol/jsonrpc';
import { MCPError } from '../protocol/errors';
import type { MessageTransport } from '../transport/message-adapter';

/** Resource接口定义 */
export interface Resource {
  uri: string; name: string; description?: string; mimeType?: string; metadata?: Record<string, unknown>;
}

/** 资源内容 */
export interface ResourceContent {
  uri: string; mimeType?: string; text?: string; blob?: string;
}

/** URI解析结果 */
interface ParsedURI { scheme: string; path: string; query?: Record<string, string>; }

export type ResourceContentProvider = (uri: string) => Promise<ResourceContent> | ResourceContent;

/** Resources管理器 */
export class ResourcesCapability {
  private resources = new Map<string, Resource>();
  private subscribers = new Set<string>();
  private providers = new Map<string, ResourceContentProvider>();
  private transport?: MessageTransport;

  register(resource: Resource, provider?: ResourceContentProvider): void {
    this.validateURI(resource.uri);
    this.resources.set(resource.uri, resource);
    if (provider) this.providers.set(resource.uri, provider);
    this.notifyListChanged();
  }

  unregister(uri: string): void {
    this.resources.delete(uri); this.providers.delete(uri); this.subscribers.delete(uri); this.notifyListChanged();
  }

  attach(transport: MessageTransport): void {
    this.transport = transport;
    transport.onMessage = (msg: JSONRPCMessage) => this.handleMessage(msg);
  }

  private handleMessage(msg: JSONRPCMessage): void {
    if (!('method' in msg) || msg.method === undefined) return;
    const req = msg as JSONRPCRequest;
    const p = req.params as { uri?: string } | undefined;
    switch (req.method) {
      case RESOURCES_LIST_ENDPOINT: this.sendResponse(req.id, { resources: Array.from(this.resources.values()) }); break;
      case RESOURCES_READ_ENDPOINT: this.handleRead(p?.uri).then(r => this.sendResponse(req.id, r), e => this.sendError(req.id, e)); break;
      case RESOURCES_SUBSCRIBE_ENDPOINT: if (p?.uri) this.subscribers.add(p.uri); this.sendResponse(req.id, { subscribed: true }); break;
    }
  }

  private sendResponse(id: number | string, result: unknown): void {
    if (this.transport?.isConnected()) this.transport.send({ jsonrpc: '2.0', id, result });
  }

  private sendError(id: number | string, err: MCPError): void {
    if (this.transport?.isConnected()) this.transport.send(err.toJSONRPCError(id));
  }

  private async handleRead(uri: string | undefined): Promise<ResourceContent> {
    if (!uri) throw new MCPError(JSONRPCErrorCode.INVALID_PARAMS, 'URI is required');
    const r = this.resources.get(uri);
    if (!r) throw new MCPError(MCPErrorCode.RESOURCE_NOT_FOUND, `Resource not found: ${uri}`);
    const p = this.providers.get(uri);
    return p ? await p(uri) : { uri, mimeType: r.mimeType };
  }

  private notifyListChanged(): void {
    if (!this.transport?.isConnected()) return;
    this.transport.send({ jsonrpc: '2.0', method: 'notifications/resources/list_changed' } as JSONRPCNotification);
  }

  notifyResourceChanged(uri: string): void {
    if (!this.transport?.isConnected() || !this.subscribers.has(uri)) return;
    this.transport.send({ jsonrpc: '2.0', method: 'notifications/resources/updated', params: { uri } } as JSONRPCNotification);
  }

  private validateURI(uri: string): void {
    try { const url = new URL(uri); if (!url.protocol || url.protocol === ':') throw new Error(); }
    catch { throw new MCPError(JSONRPCErrorCode.INVALID_PARAMS, `Invalid URI: ${uri}`); }
  }

  parseURI(uri: string): ParsedURI {
    try {
      const url = new URL(uri);
      const query: Record<string, string> = {}; url.searchParams.forEach((v, k) => query[k] = v);
      return { scheme: url.protocol.slice(0, -1), path: url.pathname, query: Object.keys(query).length ? query : undefined };
    } catch { throw new MCPError(JSONRPCErrorCode.INVALID_PARAMS, `Invalid URI: ${uri}`); }
  }
}

/** 端点常量 */
export const RESOURCES_LIST_ENDPOINT = 'resources/list';
export const RESOURCES_READ_ENDPOINT = 'resources/read';
export const RESOURCES_SUBSCRIBE_ENDPOINT = 'resources/subscribe';
