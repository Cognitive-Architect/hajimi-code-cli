"use strict";
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
exports.SidebarProvider = void 0;
const vscode = __importStar(require("vscode"));
/**
 * SidebarProvider — Hajimi sidebar with 7 real commands.
 * Week 6 hemostasis: reduced from 64 commands → 7 real commands.
 * Sidebar tool list now strictly matches CommandRegistry enum.
 */
class SidebarProvider {
    constructor(extensionUri) {
        this.extensionUri = extensionUri;
        /**
         * 7 real commands — each maps to a CommandRegistry entry.
         * invokeMcpTool calls vscode.commands.executeCommand(`hajimi.${id}`).
         * Registered commands: hajimi.openSidebar, hajimi.searchCode, hajimi.toggleTerminal,
         * hajimi.test.run, hajimi.build, hajimi.git.commit, hajimi.adr.open.
         */
        this.tools = [
            // Built-in VSCode API commands
            { id: 'openSidebar', name: 'Open Sidebar', icon: 'layout-sidebar-left', category: 'core' },
            { id: 'searchCode', name: 'Search Code', icon: 'search', category: 'core' },
            { id: 'toggleTerminal', name: 'Toggle Terminal', icon: 'terminal', category: 'core' },
            // Real MCP commands (RPC to Rust backend)
            { id: 'test.run', name: 'Run Tests', icon: 'beaker', category: 'mcp' },
            { id: 'build', name: 'Build', icon: 'tools', category: 'mcp' },
            { id: 'git.commit', name: 'Git Commit', icon: 'git-commit', category: 'mcp' },
            { id: 'adr.open', name: 'Open ADR', icon: 'book', category: 'mcp' },
        ];
    }
    /** Get current webview view instance */
    getView() { return this._view; }
    /**
     * Resolve WebView view - SAFETY: WebView CSP strict
     */
    resolveWebviewView(webviewView, _context, _token) {
        this._view = webviewView;
        webviewView.webview.options = {
            enableScripts: true,
            localResourceRoots: [this.extensionUri],
        };
        webviewView.webview.html = this.getHtmlForWebview(webviewView.webview);
        webviewView.webview.onDidReceiveMessage((msg) => { void this.handleMessage(msg); });
    }
    /** Handle messages from WebView */
    async handleMessage(message) {
        if (message.command === 'executeTool') {
            const tool = this.tools.find((t) => t.id === String(message.data ?? ''));
            if (tool)
                await this.invokeMcpTool(tool.id, tool.name);
        }
    }
    /** Invoke MCP tool via VSCode command registry */
    async invokeMcpTool(toolId, toolName) {
        try {
            await vscode.commands.executeCommand(`hajimi.${toolId}`);
            vscode.window.showInformationMessage(`Completed: ${toolName}`);
        }
        catch (err) {
            vscode.window.showErrorMessage(`Tool ${toolName} failed: ${err instanceof Error ? err.message : String(err)}`);
        }
    }
    /**
     * Generate HTML for WebView
     * SAFETY: WebView CSP strict - nonce-based CSP, vscode-webview://
     */
    getHtmlForWebview(_webview) {
        const nonce = this.getNonce();
        const buttons = this.tools.map((t) => `<button class="btn" data-id="${t.id}" title="${t.name}"><span class="codicon codicon-${t.icon}"></span><span>${t.name}</span></button>`).join('');
        return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'nonce-${nonce}'; style-src 'unsafe-inline';">
  <title>Hajimi</title>
  <style>
    *{box-sizing:border-box;margin:0;padding:0}
    body{font-family:var(--vscode-font-family);background:var(--vscode-sidebar-background);color:var(--vscode-foreground);padding:8px}
    h2{font-size:13px;font-weight:600;padding:8px 4px;border-bottom:1px solid var(--vscode-panel-border);margin-bottom:10px}
    .grid{display:grid;grid-template-columns:repeat(2,1fr);gap:5px}
    .btn{display:flex;align-items:center;gap:5px;padding:7px;background:var(--vscode-button-secondaryBackground);color:var(--vscode-button-secondaryForeground);border:none;border-radius:3px;cursor:pointer;font-size:10px}
    .btn:hover{background:var(--vscode-button-secondaryHoverBackground)}
  </style>
</head>
<body>
  <h2>Hajimi Tools (${this.tools.length})</h2>
  <div class="grid">${buttons}</div>
  <script nonce="${nonce}">
    const vscode=acquireVsCodeApi();
    document.querySelectorAll('.btn').forEach(b=>b.addEventListener('click',()=>{
      vscode.postMessage({command:'executeTool',data:b.getAttribute('data-id')});
    }));
  </script>
</body>
</html>`;
    }
    /** Generate nonce for CSP */
    getNonce() {
        const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
        return Array.from({ length: 32 }, () => chars.charAt(Math.floor(Math.random() * chars.length))).join('');
    }
}
exports.SidebarProvider = SidebarProvider;
SidebarProvider.viewId = 'hajimi.sidebar';
//# sourceMappingURL=SidebarProvider.js.map