"use strict";
// Hajimi VSCode Extension - Main Entry Point
// Week 23: ToolPicker, OutputLogger, TerminalManager, StatusBar integration
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const SidebarProvider_1 = require("./providers/SidebarProvider");
const TreeViewManager_1 = require("./managers/TreeViewManager");
const CommandRegistry_1 = require("./registry/CommandRegistry");
const LspClient_1 = require("./clients/LspClient");
// Extension activation entry point
function activate(context) {
    // Initialize core components
    const lspClient = new LspClient_1.LspClient('ws://localhost:8080');
    const commandRegistry = new CommandRegistry_1.CommandRegistry(context, lspClient);
    const treeViewManager = new TreeViewManager_1.TreeViewManager(context, lspClient);
    const sidebarProvider = new SidebarProvider_1.SidebarProvider(context.extensionUri);
    commandRegistry.registerAllCommands();
    // Register sidebar webview
    context.subscriptions.push(vscode.window.registerWebviewViewProvider('hajimi.sidebar', sidebarProvider));
    // Register tree view for tools
    vscode.window.createTreeView('hajimi.tools', {
        treeDataProvider: treeViewManager,
        showCollapseAll: true
    });
    // LSP client connection initiated by CommandRegistry constructor
    // Week 5 MCP Expansion: All MCP tools (15 mapped to engine/tool-system 38 impls)
    // registered via mcp.rs bridge. Full coverage in registry + .mcp.json. Deps pinned.
    // See src/engine/tool-system/src/mcp.rs and registry.rs for details.
    // Store in extension context for global access
    context.subscriptions.push(lspClient);
}
// Extension deactivation cleanup
function deactivate() {
    // Cleanup handled by context.subscriptions
}
//# sourceMappingURL=extension.js.map