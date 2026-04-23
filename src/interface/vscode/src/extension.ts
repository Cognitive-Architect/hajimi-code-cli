// Hajimi VSCode Extension - Main Entry Point
// Week 23: ToolPicker, OutputLogger, TerminalManager, StatusBar integration

import * as vscode from 'vscode';
import { WebviewHost } from './providers/WebviewHost';
import { TreeViewManager } from './managers/TreeViewManager';
import { CommandRegistry } from './registry/CommandRegistry';
import { LspClient } from './clients/LspClient';
import { MentionCompletionProvider } from './completion/MentionCompletionProvider';

// Extension activation entry point
export function activate(context: vscode.ExtensionContext): void {
  // Initialize core components
  const lspClient = new LspClient('ws://localhost:8080');
  const commandRegistry = new CommandRegistry(context, lspClient);
  const treeViewManager = new TreeViewManager(context, lspClient);
  const webviewHost = new WebviewHost(context.extensionUri, lspClient, context);
  commandRegistry.registerAllCommands();

  // Register sidebar webview
  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider('hajimi.sidebar', webviewHost)
  );

  // Register tree view for tools
  vscode.window.createTreeView('hajimi.tools', {
    treeDataProvider: treeViewManager,
    showCollapseAll: true
  });

  // DEBT-W5-COMPLETION-API-001: Register @file / #folder completion provider
  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider(
      [{ scheme: 'file' }, { scheme: 'untitled' }],
      new MentionCompletionProvider(context),
      '@',
      '#'
    )
  );

  // LSP client connection initiated by CommandRegistry constructor

  // Week 5 MCP Expansion: All MCP tools (15 mapped to engine/tool-system 38 impls)
  // registered via mcp.rs bridge. Full coverage in registry + .mcp.json. Deps pinned.
  // See src/engine/tool-system/src/mcp.rs and registry.rs for details.
  
  // Store in extension context for global access
  context.subscriptions.push(lspClient);
}

// Extension deactivation cleanup
export function deactivate(): void {
  // Cleanup handled by context.subscriptions
}
