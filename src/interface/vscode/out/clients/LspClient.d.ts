import * as vscode from 'vscode';
import type { InitializeParams, InitializeResult, DidOpenTextDocumentParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams, CompletionParams, CompletionList, HoverParams, Hover, DefinitionParams, Location, PublishDiagnosticsParams, NotificationHandler, RequestHandler } from '../types/lsp';
/** LSP standard client implementation using Week 21 RpcClient */
export declare class LspClient implements vscode.Disposable {
    private readonly serverUrl;
    private rpcClient;
    private notificationHandlers;
    private requestHandlers;
    private messageId;
    private isInitialized;
    private isShutdown;
    /** Create LSP client with RpcClient from Week 21 */
    constructor(serverUrl: string);
    /** Establish connection to LSP server */
    connect(): Promise<void>;
    /**
     * LSP Initialize - first phase of lifecycle
     * Client declares capabilities and receives server capabilities
     */
    initialize(params: InitializeParams): Promise<InitializeResult>;
    /** Register notification handler for LSP server-to-client messages */
    onNotification<T>(method: string, handler: NotificationHandler<T>): void;
    /** Register request handler for LSP server-to-client requests */
    onRequest<TParams, TResult>(method: string, handler: RequestHandler<TParams, TResult>): void;
    /** textDocument/didOpen - notify server of document opening */
    textDocumentDidOpen(params: DidOpenTextDocumentParams): Promise<void>;
    /** textDocument/didChange - notify server of document changes */
    textDocumentDidChange(params: DidChangeTextDocumentParams): Promise<void>;
    /** textDocument/didClose - notify server of document closing */
    textDocumentDidClose(params: DidCloseTextDocumentParams): Promise<void>;
    /** textDocument/completion - request completion items */
    textDocumentCompletion(params: CompletionParams): Promise<CompletionList>;
    /** textDocument/hover - request hover information */
    textDocumentHover(params: HoverParams): Promise<Hover | null>;
    /** textDocument/definition - request definition location */
    textDocumentDefinition(params: DefinitionParams): Promise<Location | Location[] | null>;
    /** textDocument/publishDiagnostics - handle server diagnostic notifications */
    onPublishDiagnostics(handler: NotificationHandler<PublishDiagnosticsParams>): void;
    /** LSP Shutdown - initiate graceful shutdown */
    shutdown(): Promise<void>;
    /** LSP Exit - final notification before closing connection */
    exit(): Promise<void>;
    /** Complete LSP lifecycle shutdown sequence */
    stop(): Promise<void>;
    /** Check if client is initialized, throw if not */
    private checkInitialized;
    /** Send JSON-RPC request using VsCodeRpcClient adapter */
    private sendRequest;
    /** Send JSON-RPC notification (no response expected) */
    private sendNotification;
    /** Get current message ID for debugging */
    getRequestId(): number;
    /** Check if client is in initialized state */
    getInitializedState(): boolean;
    /** Check if client is in shutdown state */
    getShutdownState(): boolean;
    /** Dispose resources - connection cleanup */
    dispose(): void;
}
/** Factory function for creating LSP client instances */
export declare function createLspClient(serverUrl: string): LspClient;
//# sourceMappingURL=LspClient.d.ts.map