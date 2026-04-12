/** RPC message structure (mirrored from Week 21) */
export interface RpcMessage {
    jsonrpc: '2.0';
    method: string;
    params?: unknown;
    id?: number;
}
/** RPC response structure */
export interface RpcResponse {
    jsonrpc: '2.0';
    result?: unknown;
    error?: {
        code: number;
        message: string;
        data?: unknown;
    };
    id?: number;
}
/** RPC notification structure */
export interface RpcNotification {
    jsonrpc: '2.0';
    method: string;
    params?: unknown;
}
/** Connection states */
type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'reconnecting';
/** State change listener */
type StateChangeListener = () => void;
/** VSCode-specific RPC client - adapts Week 21 logic for extension context */
export declare class VsCodeRpcClient {
    private readonly url;
    private readonly options;
    private ws;
    private state;
    private requestId;
    private pendingRequests;
    private reconnectAttempts;
    private reconnectTimer;
    private heartbeatTimer;
    private stateChangeListeners;
    constructor(url: string, options?: {
        reconnectInterval?: number;
        requestTimeout?: number;
        maxReconnectAttempts?: number;
        heartbeatInterval?: number;
    });
    /** Establish WebSocket connection with auto-reconnect */
    connect(): Promise<void>;
    /** Send RPC request with timeout and type safety */
    send<TResponse>(message: RpcMessage): Promise<TResponse>;
    /** Send RPC notification (no response expected) */
    notify(notification: RpcNotification): void;
    /** Get next request ID */
    getNextId(): number;
    /** Get current connection state */
    getState(): ConnectionState;
    /** Subscribe to state changes */
    onStateChange(listener: StateChangeListener): () => void;
    /** Close connection gracefully */
    disconnect(): void;
    private setState;
    private emitStateChange;
    private handleMessage;
    private handleDisconnect;
    private startHeartbeat;
    private stopHeartbeat;
}
export {};
//# sourceMappingURL=rpcAdapter.d.ts.map