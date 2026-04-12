// Hajimi VSCode Extension - Main Entry Point
// Week 23: ToolPicker, OutputLogger, TerminalManager, StatusBar integration

import * as vscode from 'vscode';
import { SidebarProvider } from './providers/SidebarProvider';
import { TreeViewManager } from './managers/TreeViewManager';
import { CommandRegistry } from './registry/CommandRegistry';
import { LspClient } from './clients/LspClient';

// Extension activation entry point
export function activate(context: vscode.ExtensionContext): void {
  // Initialize core components
  const sidebarProvider = new SidebarProvider(context.extensionUri);
  const treeViewManager = new TreeViewManager(context);
  const commandRegistry = new CommandRegistry(context);
  commandRegistry.registerAllCommands();
  
  // Register sidebar webview
  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider('hajimi.sidebar', sidebarProvider)
  );
  
  // Register tree view for 56 tools
  vscode.window.createTreeView('hajimi.tools', {
    treeDataProvider: treeViewManager,
    showCollapseAll: true
  });
  
  // Initialize LSP client connection
  const lspClient = new LspClient('ws://localhost:8080');
  
  // Week 23 components will be initialized here
  // - ToolPicker: QuickPick tool selector
  // - OutputLogger: Structured logging panel
  // - TerminalManager: Integrated terminal
  // - StatusBar: Connection status display
  
  // Store in extension context for global access
  context.subscriptions.push(lspClient);
}

// Extension deactivation cleanup
export function deactivate(): void {
  // Cleanup handled by context.subscriptions
}
