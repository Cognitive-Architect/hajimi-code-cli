// LSP-LIFECYCLE: initialize → initialized → [operations] → shutdown → exit
// TYPE-SAFETY: LSP standard client with Week 21 RpcClient reuse
// Zero 'any' types - full type safety from LSP types

import * as vscode from 'vscode';
import { VsCodeRpcClient, RpcMessage } from '../adapters/rpcAdapter';
import type {
  InitializeParams, InitializeResult, InitializedParams,
  DidOpenTextDocumentParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
  CompletionParams, CompletionList, HoverParams, Hover,
  DefinitionParams, Location, PublishDiagnosticsParams,
  ShutdownParams, ExitParams, NotificationHandler, RequestHandler,
} from '../types/lsp';

/** LSP message ID tracker for race condition protection */
interface MessageIdManager {
  current: number;
  increment(): number;
}

/** LSP standard client implementation using Week 21 RpcClient */
export class LspClient implements vscode.Disposable {
  private rpcClient: VsCodeRpcClient;
  private notificationHandlers = new Map<string, NotificationHandler<unknown>[]>();
  private requestHandlers = new Map<string, RequestHandler<unknown, unknown>>();
  private messageId: MessageIdManager = { current: 0, increment(): number { return ++this.current; } };
  private isInitialized = false;
  private isShutdown = false;

  /** Create LSP client with RpcClient from Week 21 */
  constructor(private readonly serverUrl: string) {
    this.rpcClient = new VsCodeRpcClient(this.serverUrl, { requestTimeout: 60000, heartbeatInterval: 30000 });
  }

  /** Establish connection to LSP server */
  async connect(): Promise<void> {
    await this.rpcClient.connect();
  }

  /**
   * LSP Initialize - first phase of lifecycle
   * Client declares capabilities and receives server capabilities
   */
  async initialize(params: InitializeParams): Promise<InitializeResult> {
    if (this.isInitialized) {
      throw new Error('LSP client already initialized');
    }

    const id = this.messageId.increment();
    const response = await this.sendRequest<InitializeResult>('initialize', params, id);
    this.isInitialized = true;

    // Send initialized notification to complete handshake
    await this.sendNotification('initialized', {} as InitializedParams);

    return response;
  }

  /** Register notification handler for LSP server-to-client messages */
  onNotification<T>(method: string, handler: NotificationHandler<T>): void {
    const handlers = this.notificationHandlers.get(method) ?? [];
    handlers.push(handler as NotificationHandler<unknown>);
    this.notificationHandlers.set(method, handlers);
  }

  /** Register request handler for LSP server-to-client requests */
  onRequest<TParams, TResult>(method: string, handler: RequestHandler<TParams, TResult>): void {
    this.requestHandlers.set(method, handler as RequestHandler<unknown, unknown>);
  }

  /** textDocument/didOpen - notify server of document opening */
  async textDocumentDidOpen(params: DidOpenTextDocumentParams): Promise<void> {
    this.checkInitialized();
    await this.sendNotification('textDocument/didOpen', params);
  }

  /** textDocument/didChange - notify server of document changes */
  async textDocumentDidChange(params: DidChangeTextDocumentParams): Promise<void> {
    this.checkInitialized();
    await this.sendNotification('textDocument/didChange', params);
  }

  /** textDocument/didClose - notify server of document closing */
  async textDocumentDidClose(params: DidCloseTextDocumentParams): Promise<void> {
    this.checkInitialized();
    await this.sendNotification('textDocument/didClose', params);
  }

  /** textDocument/completion - request completion items */
  async textDocumentCompletion(params: CompletionParams): Promise<CompletionList> {
    this.checkInitialized();
    const id = this.messageId.increment();
    return this.sendRequest<CompletionList>('textDocument/completion', params, id);
  }

  /** textDocument/hover - request hover information */
  async textDocumentHover(params: HoverParams): Promise<Hover | null> {
    this.checkInitialized();
    const id = this.messageId.increment();
    return this.sendRequest<Hover | null>('textDocument/hover', params, id);
  }

  /** textDocument/definition - request definition location */
  async textDocumentDefinition(params: DefinitionParams): Promise<Location | Location[] | null> {
    this.checkInitialized();
    const id = this.messageId.increment();
    return this.sendRequest<Location | Location[] | null>('textDocument/definition', params, id);
  }

  /** textDocument/publishDiagnostics - handle server diagnostic notifications */
  onPublishDiagnostics(handler: NotificationHandler<PublishDiagnosticsParams>): void {
    this.onNotification('textDocument/publishDiagnostics', handler);
  }

  /** LSP Shutdown - initiate graceful shutdown */
  async shutdown(): Promise<void> {
    this.checkInitialized();
    if (this.isShutdown) {
      return;
    }
    const id = this.messageId.increment();
    await this.sendRequest<void>('shutdown', {} as ShutdownParams, id);
    this.isShutdown = true;
  }

  /** LSP Exit - final notification before closing connection */
  async exit(): Promise<void> {
    await this.sendNotification('exit', {} as ExitParams);
    this.dispose();
  }

  /** Complete LSP lifecycle shutdown sequence */
  async stop(): Promise<void> {
    await this.shutdown();
    await this.exit();
  }

  /** Check if client is initialized, throw if not */
  private checkInitialized(): void {
    if (!this.isInitialized) {
      throw new Error('LSP client not initialized. Call initialize() first.');
    }
  }

  /** Send JSON-RPC request using VsCodeRpcClient adapter */
  private async sendRequest<TResult>(method: string, params: unknown, id: number): Promise<TResult> {
    const message: RpcMessage = { jsonrpc: '2.0', method, params, id };
    return this.rpcClient.send<TResult>(message);
  }

  /** Send custom JSON-RPC request (for MCP tool calls and extensions) */
  async sendCustomRequest<TResult>(method: string, params: unknown): Promise<TResult> {
    const id = this.messageId.increment();
    return this.sendRequest<TResult>(method, params, id);
  }

  /** Send JSON-RPC notification (no response expected) */
  private async sendNotification(method: string, params: unknown): Promise<void> {
    this.rpcClient.notify({ jsonrpc: '2.0', method, params });
  }

  /** Get current message ID for debugging */
  getRequestId(): number {
    return this.messageId.current;
  }

  /** Check if client is in initialized state */
  getInitializedState(): boolean {
    return this.isInitialized;
  }

  /** Check if client is in shutdown state */
  getShutdownState(): boolean {
    return this.isShutdown;
  }

  /** Dispose resources - connection cleanup */
  dispose(): void {
    // Clean up RpcClient connection
    this.rpcClient.disconnect();
    this.notificationHandlers.clear();
    this.requestHandlers.clear();
    this.isInitialized = false;
  }
}

/** Factory function for creating LSP client instances */
export function createLspClient(serverUrl: string): LspClient {
  return new LspClient(serverUrl);
}
