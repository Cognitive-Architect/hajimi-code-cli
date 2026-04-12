#!/usr/bin/env node
/**
 * MCP-11: CLI工具与配置系统
 * 命令: init/start/stop | 配置: .mcp.json | 验证: zod
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import * as z from 'zod';
import { MCPServer } from './lifecycle';
import { createTransport, StdioTransportConfig } from './transport/message-adapter';

// 配置Schema（zod验证）
const ConfigSchema = z.object({
  server: z.object({
    name: z.string(),
    version: z.string(),
    transport: z.enum(['stdio', 'sse']),
    port: z.number().optional(),
  }),
  capabilities: z.object({
    tools: z.boolean().default(true),
    resources: z.boolean().default(true),
    prompts: z.boolean().default(true),
  }),
});

type Config = z.infer<typeof ConfigSchema>;

// CLI命令实现
class MCPCLI {
  private configPath = '.mcp.json';
  private server?: MCPServer;

  async init(): Promise<void> {
    try {
      const defaultConfig = {
        server: { name: 'mcp-server', version: '1.0.0', transport: 'stdio' as const },
        capabilities: { tools: true, resources: true, prompts: true }
      };
      await fs.writeFile(this.configPath, JSON.stringify(defaultConfig, null, 2));
      process.stderr.write('Created .mcp.json\n');
    } catch (err) {
      process.stderr.write(`Failed to create config: ${(err as Error).message}\n`);
      process.exit(1);
    }
  }

  async start(): Promise<void> {
    try {
      const config = await this.loadConfig();
      this.server = new MCPServer({ name: config.server.name, version: config.server.version });
      if (config.server.transport === 'stdio') {
        const transportConfig: StdioTransportConfig = { type: 'stdio', command: process.argv[0], args: [] };
        const transport = createTransport(transportConfig);
        this.server.attach(transport);
      }
      this.setupSignalHandlers();
      process.stderr.write(`MCP server '${config.server.name}' started\n`);
    } catch (err) {
      process.stderr.write(`Failed to start server: ${(err as Error).message}\n`);
      process.exit(1);
    }
  }

  async stop(): Promise<void> {
    await this.gracefulShutdown();
  }

  private async loadConfig(): Promise<Config> {
    try {
      const content = await fs.readFile(this.configPath, 'utf8');
      const parsed = JSON.parse(content);
      return ConfigSchema.parse(parsed);
    } catch (err) {
      if ((err as NodeJS.ErrnoException).code === 'ENOENT') {
        throw new Error(`Config file not found: ${this.configPath}`);
      }
      throw err;
    }
  }

  private setupSignalHandlers(): void {
    process.on('SIGINT', () => this.gracefulShutdown());
    process.on('SIGTERM', () => this.gracefulShutdown());
  }

  private async gracefulShutdown(): Promise<void> {
    process.stderr.write('Shutting down...\n');
    this.server?.close();
    process.exit(0);
  }
}

function showHelp(): void {
  process.stdout.write(`
MCP Server CLI

Commands:
  init    Create default .mcp.json configuration
  start   Start MCP server with configuration
  stop    Stop running MCP server

Options:
  --help  Show this help message
`);
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const command = args[0];
  const cli = new MCPCLI();
  switch (command) {
    case 'init': await cli.init(); break;
    case 'start': await cli.start(); break;
    case 'stop': await cli.stop(); break;
    case '--help': showHelp(); break;
    default:
      process.stderr.write(`Unknown command: ${command}\n`);
      process.exit(1);
  }
}

main().catch(err => {
  process.stderr.write(`Error: ${err.message}\n`);
  process.exit(1);
});
