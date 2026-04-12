/**
 * MCP-07: Prompts Capability
 * Prompt接口 + prompts/list/get端点 + 参数模板替换
 */

import type { JSONRPCMessage, JSONRPCRequest } from '../protocol/jsonrpc';
import { JSONRPCErrorCode } from '../protocol/jsonrpc';
import { MCPError } from '../protocol/errors';
import type { MessageTransport } from '../transport/message-adapter';

export interface PromptArgument {
  name: string;
  description?: string;
  required?: boolean;
  type?: 'string' | 'number' | 'boolean';
}

export interface Prompt {
  name: string;
  description: string;
  arguments?: PromptArgument[];
}

export interface PromptMessage {
  role: 'user' | 'assistant' | 'system';
  content: { type: 'text' | 'image'; text?: string; data?: string; mimeType?: string };
}

export interface PromptTemplate {
  name: string;
  description: string;
  arguments: PromptArgument[];
  render: (args: Record<string, string>) => PromptMessage[];
}

export const PROMPTS_LIST_ENDPOINT = 'prompts/list';
export const PROMPTS_GET_ENDPOINT = 'prompts/get';

export class PromptsCapability {
  private prompts = new Map<string, PromptTemplate>();
  private transport?: MessageTransport;

  register(template: PromptTemplate): void {
    this.prompts.set(template.name, template);
  }

  attach(transport: MessageTransport): void {
    this.transport = transport;
    transport.onMessage = (msg: JSONRPCMessage) => this.handleMessage(msg);
  }

  private handleMessage(msg: JSONRPCMessage): void {
    if (!('method' in msg) || msg.method === undefined) return;
    const req = msg as JSONRPCRequest;
    if (req.method === PROMPTS_LIST_ENDPOINT) {
      this.sendResponse(req.id, this.handleList());
    } else if (req.method === PROMPTS_GET_ENDPOINT) {
      const p = req.params as { name?: string; arguments?: Record<string, string> } | undefined;
      try { this.sendResponse(req.id, this.handleGet(p?.name || '', p?.arguments)); }
      catch (e) { this.sendError(req.id, e as MCPError); }
    }
  }

  private sendResponse(id: number | string, result: unknown): void {
    if (this.transport?.isConnected()) this.transport.send({ jsonrpc: '2.0', id, result });
  }

  private sendError(id: number | string, error: MCPError): void {
    if (this.transport?.isConnected()) this.transport.send(error.toJSONRPCError(id));
  }

  private handleList(): { prompts: Prompt[] } {
    return { prompts: Array.from(this.prompts.values()).map(p => ({ name: p.name, description: p.description, arguments: p.arguments })) };
  }

  private handleGet(name: string, args?: Record<string, string>): { messages: PromptMessage[]; description: string } {
    const prompt = this.prompts.get(name);
    if (!prompt) throw new MCPError(JSONRPCErrorCode.METHOD_NOT_FOUND, `Prompt not found: ${name}`);
    this.validateArguments(prompt.arguments, args);
    return { messages: prompt.render(args || {}), description: prompt.description };
  }

  private validateArguments(defs: PromptArgument[], args?: Record<string, string>): void {
    for (const d of defs) {
      if (d.required && !args?.[d.name]) throw new MCPError(JSONRPCErrorCode.INVALID_PARAMS, `Missing required argument: ${d.name}`);
      if (args?.[d.name] && d.type === 'number' && isNaN(Number(args[d.name]))) throw new MCPError(JSONRPCErrorCode.INVALID_PARAMS, `Invalid type for ${d.name}: expected number`);
      if (args?.[d.name] && d.type === 'boolean' && !['true', 'false'].includes(args[d.name])) throw new MCPError(JSONRPCErrorCode.INVALID_PARAMS, `Invalid type for ${d.name}: expected boolean`);
    }
  }
}

const HTML_ESC: Record<string, string> = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#x27;' };

export function renderTemplate(template: string, args: Record<string, string>): string {
  return template.replace(/\{\{(\w+)\}\}/g, (_, k: string) => (args[k] !== undefined ? args[k].replace(/[&<>"']/g, c => HTML_ESC[c] || c) : ''));
}
