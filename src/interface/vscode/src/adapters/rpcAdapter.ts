// ADAPTER: Week 21 RpcClient logic adapted for VSCode extension
// Cross-project import isolation - local adapter layer
// Reuses Week 21 heartbeat/reconnect logic, VSCode-specific paths

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
  error?: { code: number; message: string; data?: unknown };
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

/** Pending request tracking */
interface PendingRequest<TResponse> {
  resolve: (value: TResponse) => void;
  reject: (error: Error) => void;
  timer: ReturnType<typeof setTimeout>;
}

/** State change listener */
type StateChangeListener = () => void;

/** VSCode-specific RPC client - adapts Week 21 logic for extension context */
export class VsCodeRpcClient {
  private ws: WebSocket | null = null;
  private state: ConnectionState = 'disconnected';
  private requestId = 0;
  private pendingRequests = new Map<number, PendingRequest<unknown>>();
  private reconnectAttempts = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private heartbeatTimer: ReturnType<typeof setTimeout> | null = null;
  private stateChangeListeners = new Set<StateChangeListener>();

  constructor(
    private readonly url: string,
    private readonly options: {
      reconnectInterval?: number;
      requestTimeout?: number;
      maxReconnectAttempts?: number;
      heartbeatInterval?: number;
    } = {}
  ) {}

  /** Establish WebSocket connection with auto-reconnect */
  async connect(): Promise<void> {
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
  async send<TResponse>(message: RpcMessage): Promise<TResponse> {
    return new Promise((resolve, reject) => {
      if (this.state !== 'connected' || !this.ws) {
        reject(new Error('RPC client not connected'));
        return;
      }

      const timeout = this.options.requestTimeout ?? 30000;
      const timer = setTimeout(() => {
        this.pendingRequests.delete(message.id!);
        reject(new Error(`RPC request timeout after ${timeout}ms`));
      }, timeout);

      this.pendingRequests.set(message.id!, { resolve: resolve as (value: unknown) => void, reject, timer });
      this.ws.send(JSON.stringify(message));
    });
  }

  /** Send RPC notification (no response expected) */
  notify(notification: RpcNotification): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(notification));
    }
  }

  /** Get next request ID */
  getNextId(): number {
    return ++this.requestId;
  }

  /** Get current connection state */
  getState(): ConnectionState {
    return this.state;
  }

  /** Subscribe to state changes */
  onStateChange(listener: StateChangeListener): () => void {
    this.stateChangeListeners.add(listener);
    return () => this.stateChangeListeners.delete(listener);
  }

  /** Close connection gracefully */
  disconnect(): void {
    this.stopHeartbeat();
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.ws?.close();
    this.ws = null;
    this.setState('disconnected');
  }

  private setState(newState: ConnectionState): void {
    if (this.state !== newState) {
      this.state = newState;
      this.emitStateChange();
    }
  }

  private emitStateChange(): void {
    for (const listener of this.stateChangeListeners) {
      listener();
    }
  }

  private handleMessage(data: string): void {
    try {
      const response = JSON.parse(data) as RpcResponse;
      if (response.id === null || response.id === undefined) return;

      const pending = this.pendingRequests.get(response.id);
      if (!pending) return;

      clearTimeout(pending.timer);
      this.pendingRequests.delete(response.id);

      if (response.error) {
        pending.reject(new Error(response.error.message));
      } else {
        pending.resolve(response.result as unknown);
      }
    } catch {
      // Ignore invalid JSON
    }
  }

  private handleDisconnect(): void {
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
        this.connect().catch(() => {});
      }, delay);
    }
  }

  private startHeartbeat(): void {
    const interval = this.options.heartbeatInterval ?? 30000;
    this.heartbeatTimer = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({ jsonrpc: '2.0', method: 'ping', id: this.getNextId() }));
      }
    }, interval);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }
}
