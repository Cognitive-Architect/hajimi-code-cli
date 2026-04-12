import * as vscode from 'vscode';

interface WebSocketLike {
  send(data: string): void;
  close(): void;
  on(event: string, handler: (data: unknown) => void): void;
}

interface TerminalSession {
  terminal: vscode.Terminal;
  ws: WebSocketLike;
  id: number;
}

export class TerminalManager implements vscode.Disposable {
  private terminals = new Map<number, TerminalSession>();
  private counter = 0;
  private readonly serverUrl: string;
  private readonly allowedProtocols = ['http:', 'https:', 'ws:', 'wss:'];

  constructor(serverUrl: string = 'ws://localhost:8080') {
    this.serverUrl = serverUrl;
  }

  private getValidatedHttpUrl(wsUrl: string): string {
    try {
      const url = new URL(wsUrl);
      if (!this.allowedProtocols.includes(url.protocol)) {
        throw new Error(`Invalid protocol: ${url.protocol}`);
      }
      return wsUrl.replace('ws://', 'http://').replace('wss://', 'https://');
    } catch {
      return 'http://localhost:8080';
    }
  }

  createTerminal(): vscode.Terminal {
    this.counter++;
    const id = this.counter;
    const name = `hajimi-terminal-${id}`;
    const env: { [key: string]: string } = {
      HAJIMI_SERVER_URL: this.getValidatedHttpUrl(this.serverUrl)
    };
    const terminal = vscode.window.createTerminal({ name, env });
    const ws = this.createWebSocket();
    this.terminals.set(id, { terminal, ws, id });
    this.setupWebSocket(ws, id);
    terminal.show();
    return terminal;
  }

  private createWebSocket(): WebSocketLike {
    return new (require('ws'))(this.serverUrl) as WebSocketLike;
  }

  private setupWebSocket(ws: WebSocketLike, id: number): void {
    ws.on('open', () => {
      const session = this.terminals.get(id);
      if (session) this.sendText(session.terminal, `echo "[Hajimi] Connected"`);
    });
    ws.on('message', (data: unknown) => {
      const session = this.terminals.get(id);
      if (session) this.sendText(session.terminal, `echo "[Server] ${String(data)}"`);
    });
    ws.on('close', () => this.terminals.delete(id));
  }

  sendText(terminal: vscode.Terminal, text: string): void {
    terminal.sendText(text, true);
  }

  sendCommand(terminalId: number, command: string): boolean {
    const session = this.terminals.get(terminalId);
    if (session) { this.sendText(session.terminal, command); return true; }
    return false;
  }

  getTerminal(id: number): vscode.Terminal | undefined {
    return this.terminals.get(id)?.terminal;
  }

  getActiveTerminals(): number[] {
    return Array.from(this.terminals.keys());
  }

  dispose(): void {
    for (const session of this.terminals.values()) {
      session.ws.close();
      session.terminal.dispose();
    }
    this.terminals.clear();
  }
}
