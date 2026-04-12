"use strict";
// LSP-LIFECYCLE: initialize → initialized → [operations] → shutdown → exit
// TYPE-SAFETY: LSP standard client with Week 21 RpcClient reuse
// Zero 'any' types - full type safety from LSP types
Object.defineProperty(exports, "__esModule", { value: true });
exports.LspClient = void 0;
exports.createLspClient = createLspClient;
const rpcAdapter_1 = require("../adapters/rpcAdapter");
/** LSP standard client implementation using Week 21 RpcClient */
class LspClient {
    /** Create LSP client with RpcClient from Week 21 */
    constructor(serverUrl) {
        this.serverUrl = serverUrl;
        this.notificationHandlers = new Map();
        this.requestHandlers = new Map();
        this.messageId = { current: 0, increment() { return ++this.current; } };
        this.isInitialized = false;
        this.isShutdown = false;
        this.rpcClient = new rpcAdapter_1.VsCodeRpcClient(this.serverUrl, { requestTimeout: 60000, heartbeatInterval: 30000 });
    }
    /** Establish connection to LSP server */
    async connect() {
        await this.rpcClient.connect();
    }
    /**
     * LSP Initialize - first phase of lifecycle
     * Client declares capabilities and receives server capabilities
     */
    async initialize(params) {
        if (this.isInitialized) {
            throw new Error('LSP client already initialized');
        }
        const id = this.messageId.increment();
        const response = await this.sendRequest('initialize', params, id);
        this.isInitialized = true;
        // Send initialized notification to complete handshake
        await this.sendNotification('initialized', {});
        return response;
    }
    /** Register notification handler for LSP server-to-client messages */
    onNotification(method, handler) {
        const handlers = this.notificationHandlers.get(method) ?? [];
        handlers.push(handler);
        this.notificationHandlers.set(method, handlers);
    }
    /** Register request handler for LSP server-to-client requests */
    onRequest(method, handler) {
        this.requestHandlers.set(method, handler);
    }
    /** textDocument/didOpen - notify server of document opening */
    async textDocumentDidOpen(params) {
        this.checkInitialized();
        await this.sendNotification('textDocument/didOpen', params);
    }
    /** textDocument/didChange - notify server of document changes */
    async textDocumentDidChange(params) {
        this.checkInitialized();
        await this.sendNotification('textDocument/didChange', params);
    }
    /** textDocument/didClose - notify server of document closing */
    async textDocumentDidClose(params) {
        this.checkInitialized();
        await this.sendNotification('textDocument/didClose', params);
    }
    /** textDocument/completion - request completion items */
    async textDocumentCompletion(params) {
        this.checkInitialized();
        const id = this.messageId.increment();
        return this.sendRequest('textDocument/completion', params, id);
    }
    /** textDocument/hover - request hover information */
    async textDocumentHover(params) {
        this.checkInitialized();
        const id = this.messageId.increment();
        return this.sendRequest('textDocument/hover', params, id);
    }
    /** textDocument/definition - request definition location */
    async textDocumentDefinition(params) {
        this.checkInitialized();
        const id = this.messageId.increment();
        return this.sendRequest('textDocument/definition', params, id);
    }
    /** textDocument/publishDiagnostics - handle server diagnostic notifications */
    onPublishDiagnostics(handler) {
        this.onNotification('textDocument/publishDiagnostics', handler);
    }
    /** LSP Shutdown - initiate graceful shutdown */
    async shutdown() {
        this.checkInitialized();
        if (this.isShutdown) {
            return;
        }
        const id = this.messageId.increment();
        await this.sendRequest('shutdown', {}, id);
        this.isShutdown = true;
    }
    /** LSP Exit - final notification before closing connection */
    async exit() {
        await this.sendNotification('exit', {});
        this.dispose();
    }
    /** Complete LSP lifecycle shutdown sequence */
    async stop() {
        await this.shutdown();
        await this.exit();
    }
    /** Check if client is initialized, throw if not */
    checkInitialized() {
        if (!this.isInitialized) {
            throw new Error('LSP client not initialized. Call initialize() first.');
        }
    }
    /** Send JSON-RPC request using VsCodeRpcClient adapter */
    async sendRequest(method, params, id) {
        const message = { jsonrpc: '2.0', method, params, id };
        return this.rpcClient.send(message);
    }
    /** Send JSON-RPC notification (no response expected) */
    async sendNotification(method, params) {
        this.rpcClient.notify({ jsonrpc: '2.0', method, params });
    }
    /** Get current message ID for debugging */
    getRequestId() {
        return this.messageId.current;
    }
    /** Check if client is in initialized state */
    getInitializedState() {
        return this.isInitialized;
    }
    /** Check if client is in shutdown state */
    getShutdownState() {
        return this.isShutdown;
    }
    /** Dispose resources - connection cleanup */
    dispose() {
        // Clean up RpcClient connection
        this.rpcClient.disconnect();
        this.notificationHandlers.clear();
        this.requestHandlers.clear();
        this.isInitialized = false;
    }
}
exports.LspClient = LspClient;
/** Factory function for creating LSP client instances */
function createLspClient(serverUrl) {
    return new LspClient(serverUrl);
}
//# sourceMappingURL=LspClient.js.map