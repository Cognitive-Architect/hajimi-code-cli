/**
 * MCP-01: JSON-RPC 2.0 Protocol
 */

export interface JSONRPCRequest { jsonrpc: '2.0'; id: number | string; method: string; params?: unknown; }
export interface JSONRPCResponse { jsonrpc: '2.0'; id: number | string | null; result: unknown; }
export interface JSONRPCError { jsonrpc: '2.0'; id: number | string | null; error: { code: number; message: string; data?: unknown }; }
export interface JSONRPCNotification { jsonrpc: '2.0'; method: string; params?: unknown; }
export type JSONRPCMessage = JSONRPCRequest | JSONRPCResponse | JSONRPCError | JSONRPCNotification;

export enum JSONRPCErrorCode {
  PARSE_ERROR = -32700, INVALID_REQUEST = -32600, METHOD_NOT_FOUND = -32601,
  INVALID_PARAMS = -32602, INTERNAL_ERROR = -32603
}

export enum MCPErrorCode {
  INITIALIZATION_ERROR = -32000, CAPABILITY_ERROR = -32001, TRANSPORT_ERROR = -32002,
  TIMEOUT_ERROR = -32003, SERVER_NOT_FOUND = -32004, RESOURCE_NOT_FOUND = -32005,
  TOOL_EXECUTION_ERROR = -32006, PROMPT_RENDER_ERROR = -32007
}

export class JSONRPCParseError extends Error {
  constructor(message: string, public readonly causeError?: Error) {
    super(message);
    this.name = 'JSONRPCParseError';
  }
}

export function isRequest(msg: JSONRPCMessage): msg is JSONRPCRequest {
  return 'method' in msg && 'id' in msg && msg.id !== undefined && msg.id !== null;
}

export function isResponse(msg: JSONRPCMessage): msg is JSONRPCResponse {
  return 'result' in msg && !('error' in msg);
}

export function isError(msg: JSONRPCMessage): msg is JSONRPCError {
  return 'error' in msg;
}

export function isNotification(msg: JSONRPCMessage): msg is JSONRPCNotification {
  return 'method' in msg && !('id' in msg);
}

export function serializeMessage(msg: JSONRPCMessage): string {
  try {
    return JSON.stringify(msg);
  } catch (e) {
    throw new JSONRPCParseError('Failed to serialize', e instanceof Error ? e : undefined);
  }
}

export function parseMessage(data: string): JSONRPCMessage {
  let parsed: unknown;
  try {
    parsed = JSON.parse(data);
  } catch (e) {
    throw new JSONRPCParseError('Parse error', e instanceof Error ? e : undefined);
  }

  if (typeof parsed !== 'object' || parsed === null) {
    throw new JSONRPCParseError('Invalid request: Expected object');
  }

  const obj = parsed as Record<string, unknown>;
  if (obj.jsonrpc !== '2.0') {
    throw new JSONRPCParseError('Invalid jsonrpc version');
  }

  const msg = obj as unknown as JSONRPCMessage;
  if (isNotification(msg)) return msg;
  if (isRequest(msg)) return msg;
  if (isError(msg)) return msg;
  if (isResponse(msg)) return msg;

  throw new JSONRPCParseError('Unknown message type');
}

export function createErrorResponse(
  id: number | string | null, code: number, message: string, data?: unknown
): JSONRPCError {
  return { jsonrpc: '2.0', id, error: { code, message, data } };
}
