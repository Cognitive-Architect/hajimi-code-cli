/**
 * MCP-02: Stdio Transport Layer
 * NDJSON framing, cross-spawn Windows compatibility, process lifecycle management
 */

import { spawn } from 'cross-spawn';
import { ChildProcess } from 'child_process';
import { JSONRPCMessage, serializeMessage, parseMessage, JSONRPCParseError } from '../protocol/jsonrpc';
import { MCPTransportError, MCPParseError } from '../protocol/errors';

export interface StdioTransportConfig {
  command: string;
  args?: string[];
  env?: Record<string, string>;
  timeout?: number;
  cwd?: string;
}

export class StdioTransport {
  private proc?: ChildProcess;
  private buffer: Buffer = Buffer.alloc(0);
  private connected = false;

  onMessage?: (msg: JSONRPCMessage) => void;
  onError?: (error: Error) => void;
  onClose?: () => void;

  get pid(): number | undefined {
    return this.proc?.pid;
  }

  connect(config: StdioTransportConfig): void {
    if (this.connected) {
      throw new MCPTransportError('Transport already connected');
    }

    const { command, args = [], env = {}, cwd } = config;

    // Spawn child process with cross-spawn for Windows compatibility
    this.proc = spawn(command, args, {
      stdio: ['pipe', 'pipe', 'pipe'],
      env: { ...process.env, ...env },
      cwd,
    });

    // Handle stdout data for NDJSON framing
    this.proc.stdout!.on('data', (data: Buffer) => {
      this.buffer = Buffer.concat([this.buffer, data]);
      this.processBuffer();
    });

    // Handle stderr output
    this.proc.stderr!.on('data', (data: Buffer) => {
      this.onError?.(new MCPTransportError(`stderr: ${data.toString('utf8').trim()}`));
    });

    // Handle process exit
    this.proc.on('exit', (code) => {
      this.connected = false;
      if (code !== 0 && code !== null) {
        this.onError?.(new MCPTransportError(`Process exited with code ${code}`));
      }
    });

    // Handle process close
    this.proc.on('close', () => {
      this.connected = false;
      this.onClose?.();
    });

    // Handle spawn errors
    this.proc.on('error', (err) => {
      this.connected = false;
      this.onError?.(new MCPTransportError(`Spawn error: ${err.message}`));
    });

    this.connected = true;
  }

  /**
   * Process buffer for NDJSON framing (newline-delimited JSON)
   * Handles sticky packet and split packet scenarios
   */
  private processBuffer(): void {
    let index: number;
    while ((index = this.buffer.indexOf(0x0a)) !== -1) {
      const line = this.buffer.slice(0, index).toString('utf8');
      this.buffer = this.buffer.slice(index + 1);
      if (line.trim()) {
        try {
          const msg = parseMessage(line);
          this.onMessage?.(msg);
        } catch (e) {
          this.onError?.(new MCPParseError(`Invalid JSON: ${line}`));
        }
      }
    }
  }

  send(msg: JSONRPCMessage): void {
    if (!this.connected || !this.proc?.stdin) {
      throw new MCPTransportError('Transport not connected');
    }
    const data = serializeMessage(msg) + '\n';
    this.proc.stdin.write(data, 'utf8');
  }

  close(): void {
    this.connected = false;
    if (this.proc) {
      this.proc.kill();
      this.proc = undefined;
    }
    this.buffer = Buffer.alloc(0);
  }

  isConnected(): boolean {
    return this.connected;
  }
}
