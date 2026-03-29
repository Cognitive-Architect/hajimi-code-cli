/**
 * MCP-03: Transport Abstraction Layer
 * MessageTransport统一接口 + createTransport工厂函数 + 双模式预留（stdio/sse）
 * 
 * 本模块定义MCP传输层的抽象接口，支持两种传输模式：
 * - stdio: 通过标准输入输出与子进程通信
 * - sse: 通过Server-Sent Events与HTTP端点通信（MCP-10预留）
 */

import type { JSONRPCMessage } from '../protocol/jsonrpc';
import { MCPError } from '../protocol/errors';
import { MCPErrorCode, JSONRPCErrorCode } from '../protocol/jsonrpc';

/** 连接状态类型 */
export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'closing';

/** 传输类型：stdio标准输入输出 / sse服务器推送事件 */
export type TransportType = 'stdio' | 'sse';

/**
 * MessageTransport统一接口（6方法）
 * 所有传输实现必须遵循此接口
 */
export interface MessageTransport {
  /** 发送JSON-RPC消息到对端 */
  send(msg: JSONRPCMessage): void;
  /** 消息接收回调 */
  onMessage?: (msg: JSONRPCMessage) => void;
  /** 错误处理回调 */
  onError?: (error: Error) => void;
  /** 连接关闭回调 */
  onClose?: () => void;
  /** 关闭传输连接 */
  close(): void;
  /** 检查当前连接状态 */
  isConnected(): boolean;
}

/** 带状态的传输接口（增强版） */
export interface StatefulTransport extends MessageTransport {
  /** 当前连接状态 */
  state: ConnectionState;
  /** 状态变更回调 */
  onStateChange?: (state: ConnectionState) => void;
}

/** 基础传输配置 */
export interface TransportConfig {
  /** 传输类型 */
  type: TransportType;
  /** 超时时间（毫秒） */
  timeout?: number;
}

/** stdio传输配置：用于启动子进程并通过stdin/stdout通信 */
export interface StdioTransportConfig extends TransportConfig {
  type: 'stdio';
  /** 可执行命令 */
  command: string;
  /** 命令行参数 */
  args?: string[];
  /** 环境变量 */
  env?: Record<string, string>;
  /** 工作目录 */
  cwd?: string;
}

/** SSE传输配置：用于连接HTTP SSE端点（MCP-10预留） */
export interface SSETransportConfig extends TransportConfig {
  type: 'sse';
  /** SSE端点URL */
  url: string;
  /** 自定义请求头 */
  headers?: Record<string, string>;
}

/** 传输配置联合类型 */
export type TransportConfigUnion = StdioTransportConfig | SSETransportConfig;

/**
 * 创建传输实例的工厂函数
 * @param config 传输配置
 * @returns MessageTransport实例
 * @throws MCPError 当传输类型未知或未实现时
 */
export function createTransport(config: TransportConfigUnion): MessageTransport {
  switch (config.type) {
    case 'stdio': {
      // 动态导入避免与MCP-02循环依赖，MCP-02将实现StdioTransport类
      const { StdioTransport } = require('./stdio-transport');
      const transport = new StdioTransport();
      transport.connect(config);
      return transport;
    }
    case 'sse':
      // MCP-10将实现SSE传输
      throw new MCPError(MCPErrorCode.CAPABILITY_ERROR, 'SSE transport not implemented (MCP-10)');
    default:
      throw new MCPError(JSONRPCErrorCode.INVALID_PARAMS, `Unknown transport type: ${(config as any).type}`);
  }
}

/** 类型守卫：检查是否为stdio配置 */
export function isStdioConfig(config: TransportConfigUnion): config is StdioTransportConfig {
  return config.type === 'stdio';
}

/** 类型守卫：检查是否为SSE配置 */
export function isSSEConfig(config: TransportConfigUnion): config is SSETransportConfig {
  return config.type === 'sse';
}
