import * as vscode from 'vscode';
import { LspClient } from '../clients/LspClient';
export declare enum CommandId {
    OPEN_SIDEBAR = "hajimi.openSidebar",
    SEARCH_CODE = "hajimi.searchCode",
    TOGGLE_TERMINAL = "hajimi.toggleTerminal",
    RUN_TESTS = "hajimi.test.run",
    BUILD = "hajimi.build",
    GIT_COMMIT = "hajimi.git.commit",
    OPEN_ADR = "hajimi.adr.open"
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
export declare class CommandRegistry {
    private context;
    private lspClient;
    constructor(context: vscode.ExtensionContext, lspClient: LspClient);
    /**
     * Register a single command with VSCode and push disposable to context.subscriptions.
     */
    registerCommand(command: string, callback: (...args: unknown[]) => unknown): void;
    /**
     * Invoke an MCP tool via the real RPC bridge to Rust McpServer.handle_tools_call.
     *
     * No simulation, no setTimeout, no hard-coded success messages.
     * Uses LspClient.sendRequest('mcp/toolCall') for honest two-way communication.
     */
    private invokeMcpTool;
    /**
     * Register all retained commands.
     *
     * Previously this method contained 20+ explicit registrations plus a dispatcher loop
     * handling 40+ additional commands. After Week 6 hemostasis, only 7 commands remain.
     */
    registerAllCommands(): void;
}
//# sourceMappingURL=CommandRegistry.d.ts.map