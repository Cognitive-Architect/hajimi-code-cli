"use strict";
// ADAPTER: Week 21 RpcClient logic adapted for VSCode extension
// Cross-project import isolation - local adapter layer
// Reuses Week 21 heartbeat/reconnect logic, VSCode-specific paths
Object.defineProperty(exports, "__esModule", { value: true });
exports.VsCodeRpcClient = void 0;
/** VSCode-specific RPC client - adapts Week 21 logic for extension context */
class VsCodeRpcClient {
    constructor(url, options = {}) {
        this.url = url;
        this.options = options;
        this.ws = null;
        this.state = 'disconnected';
        this.requestId = 0;
        this.pendingRequests = new Map();
        this.reconnectAttempts = 0;
        this.reconnectTimer = null;
        this.heartbeatTimer = null;
        this.stateChangeListeners = new Set();
    }
    /** Establish WebSocket connection with auto-reconnect */
    async connect() {
        return new Promise((resolve, reject) => {
            if (this.ws?.readyState === WebSocket.OPEN) {
                resolve();
                return;
            }
            this.setState('connecting');
            this.ws = new WebSocket(this.url);
            this.ws.onopen = () => {
                this.setState('connected');
                this.reconnectAttempts = 0;
                this.startHeartbeat();
                resolve();
            };
            this.ws.onclose = () => this.handleDisconnect();
            this.ws.onerror = (err) => reject(new Error(`WebSocket error: ${err.type}`));
            this.ws.onmessage = (event) => this.handleMessage(event.data);
        });
    }
    /** Send RPC request with timeout and type safety */
    async send(message) {
        return new Promise((resolve, reject) => {
            if (this.state !== 'connected' || !this.ws) {
                reject(new Error('RPC client not connected'));
                return;
            }
            const timeout = this.options.requestTimeout ?? 30000;
            const timer = setTimeout(() => {
                this.pendingRequests.delete(message.id);
                reject(new Error(`RPC request timeout after ${timeout}ms`));
            }, timeout);
            this.pendingRequests.set(message.id, { resolve: resolve, reject, timer });
            this.ws.send(JSON.stringify(message));
        });
    }
    /** Send RPC notification (no response expected) */
    notify(notification) {
        if (this.ws?.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(notification));
        }
    }
    /** Get next request ID */
    getNextId() {
        return ++this.requestId;
    }
    /** Get current connection state */
    getState() {
        return this.state;
    }
    /** Subscribe to state changes */
    onStateChange(listener) {
        this.stateChangeListeners.add(listener);
        return () => this.stateChangeListeners.delete(listener);
    }
    /** Close connection gracefully */
    disconnect() {
        this.stopHeartbeat();
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }
        this.ws?.close();
        this.ws = null;
        this.setState('disconnected');
    }
    setState(newState) {
        if (this.state !== newState) {
            this.state = newState;
            this.emitStateChange();
        }
    }
    emitStateChange() {
        for (const listener of this.stateChangeListeners) {
            listener();
        }
    }
    handleMessage(data) {
        try {
            const response = JSON.parse(data);
            if (response.id === null || response.id === undefined)
                return;
            const pending = this.pendingRequests.get(response.id);
            if (!pending)
                return;
            clearTimeout(pending.timer);
            this.pendingRequests.delete(response.id);
            if (response.error) {
                pending.reject(new Error(response.error.message));
            }
            else {
                pending.resolve(response.result);
            }
        }
        catch {
            // Ignore invalid JSON
        }
    }
    handleDisconnect() {
        this.setState('disconnected');
        this.stopHeartbeat();
        // Reject pending requests
        for (const pending of this.pendingRequests.values()) {
            clearTimeout(pending.timer);
            pending.reject(new Error('Connection lost'));
        }
        this.pendingRequests.clear();
        // Schedule reconnect with exponential backoff
        const maxAttempts = this.options.maxReconnectAttempts ?? 10;
        if (this.reconnectAttempts < maxAttempts) {
            this.setState('reconnecting');
            const baseDelay = this.options.reconnectInterval ?? 5000;
            const delay = Math.min(baseDelay * 2 ** this.reconnectAttempts, 30000);
            this.reconnectTimer = setTimeout(() => {
                this.reconnectAttempts++;
                this.connect().catch(() => { });
            }, delay);
        }
    }
    startHeartbeat() {
        const interval = this.options.heartbeatInterval ?? 30000;
        this.heartbeatTimer = setInterval(() => {
            if (this.ws?.readyState === WebSocket.OPEN) {
                this.ws.send(JSON.stringify({ jsonrpc: '2.0', method: 'ping', id: this.getNextId() }));
            }
        }, interval);
    }
    stopHeartbeat() {
        if (this.heartbeatTimer) {
            clearInterval(this.heartbeatTimer);
            this.heartbeatTimer = null;
        }
    }
}
exports.VsCodeRpcClient = VsCodeRpcClient;
//# sourceMappingURL=rpcAdapter.js.map