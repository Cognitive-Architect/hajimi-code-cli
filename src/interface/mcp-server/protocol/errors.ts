/**
 * MCP-04: Error Handling System
 */

import { JSONRPCErrorCode, MCPErrorCode, JSONRPCError, createErrorResponse } from './jsonrpc';

export class MCPError extends Error {
  constructor(
    public readonly code: number,
    message: string,
    public readonly context?: unknown,
    public readonly retryable: boolean = false
  ) {
    super(message);
    this.name = 'MCPError';
  }

  toJSONRPCError(id: number | string | null = null): JSONRPCError {
    return createErrorResponse(id, this.code, this.message, this.context);
  }

  get userMessage(): string {
    const map: Record<number, string> = {
      [JSONRPCErrorCode.PARSE_ERROR]: '数据解析失败',
      [JSONRPCErrorCode.INVALID_REQUEST]: '请求格式错误',
      [JSONRPCErrorCode.METHOD_NOT_FOUND]: '方法不存在',
      [JSONRPCErrorCode.INVALID_PARAMS]: '参数错误',
      [JSONRPCErrorCode.INTERNAL_ERROR]: '内部错误',
      [MCPErrorCode.INITIALIZATION_ERROR]: '初始化失败',
      [MCPErrorCode.CAPABILITY_ERROR]: '能力不匹配',
      [MCPErrorCode.TRANSPORT_ERROR]: '传输错误',
      [MCPErrorCode.TIMEOUT_ERROR]: '请求超时',
      [MCPErrorCode.SERVER_NOT_FOUND]: '服务未找到',
      [MCPErrorCode.RESOURCE_NOT_FOUND]: '资源不存在',
      [MCPErrorCode.TOOL_EXECUTION_ERROR]: '工具执行失败',
      [MCPErrorCode.PROMPT_RENDER_ERROR]: 'Prompt渲染失败',
    };
    return map[this.code] || `未知错误 (code: ${this.code})`;
  }

  isRetryable(): boolean { return this.retryable; }
}

export class MCPParseError extends MCPError {
  constructor(message: string) {
    super(JSONRPCErrorCode.PARSE_ERROR, message, undefined, false);
    this.name = 'MCPParseError';
  }
}

export class MCPInvalidRequestError extends MCPError {
  constructor(message: string, context?: unknown) {
    super(JSONRPCErrorCode.INVALID_REQUEST, message, context, false);
    this.name = 'MCPInvalidRequestError';
  }
}

export class MCPMethodNotFoundError extends MCPError {
  constructor(method: string) {
    super(JSONRPCErrorCode.METHOD_NOT_FOUND, `Method not found: ${method}`, undefined, false);
    this.name = 'MCPMethodNotFoundError';
  }
}

export class MCPTransportError extends MCPError {
  constructor(message: string) {
    super(MCPErrorCode.TRANSPORT_ERROR, message, undefined, true);
    this.name = 'MCPTransportError';
  }
}

export class MCPTimeoutError extends MCPError {
  constructor(operation: string, timeoutMs: number) {
    super(MCPErrorCode.TIMEOUT_ERROR, `Operation "${operation}" timed out after ${timeoutMs}ms`, undefined, true);
    this.name = 'MCPTimeoutError';
  }
}

export function errorFromJSONRPC(error: JSONRPCError['error']): MCPError {
  switch (error.code) {
    case JSONRPCErrorCode.PARSE_ERROR: return new MCPParseError(error.message);
    case JSONRPCErrorCode.INVALID_REQUEST: return new MCPInvalidRequestError(error.message, error.data);
    case JSONRPCErrorCode.METHOD_NOT_FOUND: return new MCPMethodNotFoundError(error.message);
    case MCPErrorCode.TRANSPORT_ERROR: return new MCPTransportError(error.message);
    case MCPErrorCode.TIMEOUT_ERROR: return new MCPTimeoutError('unknown', 0);
    default: return new MCPError(error.code, error.message, error.data);
  }
}
