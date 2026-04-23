import * as vscode from 'vscode';
import { LspClient } from '../clients/LspClient';

//! WEEK6-VSCODE-COMMAND-HEMOSTASIS: Reduced from 64 commands to 8 real commands.
//! Removed all stub/dispatcher-loop commands. Only保留有真实后端实现或VSCode原生API的命令.
//! 4 built-in (VSCode API) + 4 MCP (real RPC via invokeMcpTool).
//! Criteria for retention:
//!   - Built-in: commands that delegate to VSCode native APIs (zero external dependency)
//!   - MCP: commands whose toolName exists in engine/tool-system ToolRegistry

export enum CommandId {
  OPEN_SIDEBAR = 'hajimi.openSidebar',
  SEARCH_CODE = 'hajimi.searchCode',
  TOGGLE_TERMINAL = 'hajimi.toggleTerminal',
  RUN_TESTS = 'hajimi.test.run',
  BUILD = 'hajimi.build',
  GIT_COMMIT = 'hajimi.git.commit',
  OPEN_ADR = 'hajimi.adr.open',
}

/**
 * CommandRegistry binds VSCode command palette entries to real implementations.
 *
 * Week 6 redesign:
 * - Eliminated 56 stub commands that only showed "Executing..." toast with no backend.
 * - Eliminated generic dispatcher loop that mapped command IDs to MCP tool names via
 *   string manipulation (fragile and untyped).
 * - Retained only commands with verified backend implementations in engine/tool-system.
 *
 * All remaining commands fall into one of two categories:
 * 1. Native VSCode API delegation (openSidebar, searchCode, toggleTerminal)
 * 2. Explicit MCP tool invocation via real RPC bridge (runTests, build, gitCommit, openAdr)
 */
export class CommandRegistry {
  constructor(
    private context: vscode.ExtensionContext,
    private lspClient: LspClient
  ) {
    // Ensure LSP client is connected for real RPC (Week 9 true bridge)
    this.lspClient.connect().catch((err: Error) => {
      console.error('[CommandRegistry] LSP connect failed:', err.message);
    });
  }

  /**
   * Register a single command with VSCode and push disposable to context.subscriptions.
   */
  registerCommand(command: string, callback: (...args: unknown[]) => unknown): void {
    this.context.subscriptions.push(vscode.commands.registerCommand(command, callback));
  }

  /**
   * Invoke an MCP tool via the real RPC bridge to Rust McpServer.handle_tools_call.
   *
   * No simulation, no setTimeout, no hard-coded success messages.
   * Uses LspClient.sendRequest('mcp/toolCall') for honest two-way communication.
   */
  private async invokeMcpTool(toolName: string, args: unknown[] = []): Promise<unknown> {
    try {
      const result = await this.lspClient.sendCustomRequest<unknown>('mcp/toolCall', {
        tool: toolName,
        arguments: args
      });
      return result;
    } catch (error: any) {
      vscode.window.showErrorMessage(`RPC Error (${toolName}): ${error.message}`);
      throw error;
    }
  }

  /**
   * Register all retained commands.
   *
   * Previously this method contained 20+ explicit registrations plus a dispatcher loop
   * handling 40+ additional commands. After Week 6 hemostasis, only 7 commands remain.
   */
  registerAllCommands(): void {
    // ── Built-in VSCode commands ──────────────────────────────────────────────
    // These delegate directly to VSCode's own command palette. They require no
    // Hajimi backend and serve as zero-dependency entry points.

    this.registerCommand(CommandId.OPEN_SIDEBAR, () =>
      vscode.commands.executeCommand('workbench.view.extension.hajimi'));

    this.registerCommand(CommandId.SEARCH_CODE, () =>
      vscode.commands.executeCommand('workbench.action.findInFiles'));

    this.registerCommand(CommandId.TOGGLE_TERMINAL, () =>
      vscode.commands.executeCommand('workbench.action.terminal.toggleTerminal'));

    // ── Real MCP commands ─────────────────────────────────────────────────────
    // Each toolName below MUST exist in engine/tool-system's ToolRegistry.
    // If a tool is missing on the Rust side, the RPC call will return an honest
    // "Tool not found" error rather than a fake success toast.

    this.registerCommand(CommandId.RUN_TESTS, async () => {
      return this.invokeMcpTool('run_tests');
    });

    this.registerCommand(CommandId.BUILD, async () => {
      return this.invokeMcpTool('build');
    });

    this.registerCommand(CommandId.GIT_COMMIT, async () => {
      return this.invokeMcpTool('git_commit');
    });

    this.registerCommand(CommandId.OPEN_ADR, async () => {
      return this.invokeMcpTool('open_adr');
    });
  }
}
