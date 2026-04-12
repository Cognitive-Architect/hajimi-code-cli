/**
 * MCP-05: Tools Capability
 * Tool接口定义 + tools/list/call端点 + JSON Schema验证(zod)
 */

import * as z from 'zod';
import type { JSONRPCMessage, JSONRPCRequest } from '../protocol/jsonrpc';
import { MCPError, MCPMethodNotFoundError } from '../protocol/errors';
import { JSONRPCErrorCode, MCPErrorCode, isRequest, createErrorResponse } from '../protocol/jsonrpc';
import type { MessageTransport } from '../transport/message-adapter';

/** Tool接口定义 */
export interface Tool<TParams = unknown, TResult = unknown> {
  name: string;
  description: string;
  inputSchema: z.ZodType<TParams>;
  handler: (params: TParams) => TResult | Promise<TResult>;
}

/** 工具列表响应 */
interface ToolListItem {
  name: string;
  description: string;
  inputSchema: object;
}

/** 端点常量 */
export const TOOLS_LIST_ENDPOINT = 'tools/list';
export const TOOLS_CALL_ENDPOINT = 'tools/call';

/** Tools管理器 */
export class ToolsCapability {
  private tools = new Map<string, Tool<unknown, unknown>>();
  private transport?: MessageTransport;

  /** 注册工具（名称唯一性检查） */
  register<TParams, TResult>(tool: Tool<TParams, TResult>): void {
    if (this.tools.has(tool.name)) {
      throw new MCPError(MCPErrorCode.CAPABILITY_ERROR, `Tool '${tool.name}' already registered`);
    }
    this.tools.set(tool.name, tool as Tool<unknown, unknown>);
  }

  /** 绑定传输层 */
  attach(transport: MessageTransport): void {
    this.transport = transport;
    transport.onMessage = (msg: JSONRPCMessage) => this.handleMessage(msg);
  }

  /** 消息路由处理 */
  private handleMessage(msg: JSONRPCMessage): void {
    if (!isRequest(msg)) return;
    const req = msg as JSONRPCRequest;
    if (req.method === TOOLS_LIST_ENDPOINT) {
      this.sendResponse(req.id, this.handleList());
    } else if (req.method === TOOLS_CALL_ENDPOINT) {
      const { name, arguments: args } = (req.params || {}) as { name?: string; arguments?: unknown };
      this.handleCall(name || '', args).then(
        result => this.sendResponse(req.id, result),
        error => this.sendError(req.id, error)
      );
    }
  }

  /** 发送响应 */
  private sendResponse(id: number | string, result: unknown): void {
    if (!this.transport) return;
    this.transport.send({ jsonrpc: '2.0', id, result });
  }

  /** 发送错误 */
  private sendError(id: number | string, error: MCPError): void {
    if (!this.transport) return;
    this.transport.send(error.toJSONRPCError(id));
  }

  /** 处理tools/list请求 */
  private handleList(): { tools: ToolListItem[] } {
    const tools: ToolListItem[] = [];
    for (const tool of this.tools.values()) {
      tools.push({
        name: tool.name,
        description: tool.description,
        inputSchema: this.zodToJSONSchema(tool.inputSchema)
      });
    }
    return { tools };
  }

  /** 处理tools/call请求 */
  private async handleCall(name: string, args: unknown): Promise<unknown> {
    const tool = this.tools.get(name);
    if (!tool) {
      throw new MCPMethodNotFoundError(`tools/call: '${name}'`);
    }
    let validated: unknown;
    try {
      validated = tool.inputSchema.parse(args);
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Invalid parameters';
      throw new MCPError(JSONRPCErrorCode.INVALID_PARAMS, message, { tool: name });
    }
    try {
      return await tool.handler(validated);
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      throw new MCPError(MCPErrorCode.TOOL_EXECUTION_ERROR, message, { tool: name });
    }
  }

  /** zod schema转JSON Schema */
  private zodToJSONSchema(schema: z.ZodType): object {
    return z.toJSONSchema(schema);
  }
}
