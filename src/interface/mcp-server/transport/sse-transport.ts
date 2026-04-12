/**
 * MCP-09: SSE Transport Layer - HTTP Server, SSE streaming, Session lifecycle
 */
import * as http from 'http';
import { randomUUID } from 'crypto';
import type { JSONRPCMessage } from '../protocol/jsonrpc';
import { MCPErrorCode } from '../protocol/jsonrpc';
import { MCPError } from '../protocol/errors';
import type { MessageTransport } from './message-adapter';

interface Session {
  id: string;
  res: http.ServerResponse;
  lastActivity: number;
  messageQueue: string[];
}

export class SSETransport implements MessageTransport {
  private server?: http.Server;
  private sessions = new Map<string, Session>();
  private readonly sessionTimeout = 300000;
  private cleanupInterval?: NodeJS.Timeout;
  onMessage?: (msg: JSONRPCMessage) => void;
  onError?: (error: Error) => void;
  onClose?: () => void;

  constructor(private port: number = 3000) {}

  async start(): Promise<void> {
    this.server = http.createServer((req, res) => this.handleRequest(req, res));
    this.cleanupInterval = setInterval(() => this.cleanupSessions(), 60000);
    return new Promise((resolve, reject) => {
      this.server!.listen(this.port, () => resolve());
      this.server!.on('error', (err) => reject(new MCPError(MCPErrorCode.TRANSPORT_ERROR, err.message)));
    });
  }

  private handleRequest(req: http.IncomingMessage, res: http.ServerResponse): void {
    const url = new URL(req.url || '/', `http://${req.headers.host}`);
    if (url.pathname === '/sse' && req.method === 'GET') {
      const sid = url.searchParams.get('sessionId') || this.createSession(res);
      this.handleSSE(sid, res);
    } else if (url.pathname === '/message' && req.method === 'POST') {
      this.handleMessage(req, res);
    } else if (url.pathname.startsWith('/session/') && req.method === 'DELETE') {
      this.closeSession(url.pathname.split('/')[2], res);
    } else {
      res.writeHead(404).end('Not Found');
    }
  }

  private createSession(res: http.ServerResponse): string {
    const id = randomUUID();
    this.sessions.set(id, { id, res, lastActivity: Date.now(), messageQueue: [] });
    return id;
  }

  private handleSSE(sessionId: string, res: http.ServerResponse): void {
    const session = this.sessions.get(sessionId);
    if (!session) { res.writeHead(401).end('Invalid session'); return; }
    res.writeHead(200, { 'Content-Type': 'text/event-stream', 'Cache-Control': 'no-cache' });
    session.messageQueue.forEach((data) => res.write(`data: ${data}\n\n`));
    session.messageQueue = [];
    res.on('close', () => this.sessions.delete(sessionId));
  }

  private handleMessage(req: http.IncomingMessage, res: http.ServerResponse): void {
    let body = '';
    req.on('data', (chunk) => (body += chunk));
    req.on('end', () => {
      try {
        const msg = JSON.parse(body) as JSONRPCMessage;
        this.sessions.forEach((s) => (s.lastActivity = Date.now()));
        this.onMessage?.(msg);
        res.writeHead(200).end('OK');
      } catch {
        this.onError?.(new MCPError(MCPErrorCode.TRANSPORT_ERROR, 'Invalid JSON'));
        res.writeHead(400).end('Invalid JSON');
      }
    });
  }

  private closeSession(sessionId: string, res: http.ServerResponse): void {
    const session = this.sessions.get(sessionId);
    if (session) { session.res.end(); this.sessions.delete(sessionId); }
    res.writeHead(session ? 200 : 404).end(session ? 'OK' : 'Not Found');
  }

  send(msg: JSONRPCMessage): void {
    const data = JSON.stringify(msg);
    this.sessions.forEach((s) => { if (!s.res.writableEnded) { s.res.write(`data: ${data}\n\n`); s.lastActivity = Date.now(); } });
  }

  private cleanupSessions(): void {
    const now = Date.now();
    this.sessions.forEach((s, id) => { if (now - s.lastActivity > this.sessionTimeout) { s.res.end(); this.sessions.delete(id); } });
  }

  close(): void {
    if (this.cleanupInterval) clearInterval(this.cleanupInterval);
    this.sessions.forEach((s) => s.res.destroy());
    this.sessions.clear();
    this.server?.close(() => this.onClose?.());
  }

  isConnected(): boolean { return this.server?.listening ?? false; }
}
