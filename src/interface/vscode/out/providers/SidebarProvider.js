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
 * SidebarProvider - Hajimi main panel with 56 tool shortcuts
 * SAFETY: WebView CSP strict - vscode-webview:// protocol only
 */
class SidebarProvider {
    constructor(extensionUri) {
        this.extensionUri = extensionUri;
        /** 56 tools: 8 categories × 7 tools each */
        this.tools = [
            // Gen (8)
            { id: 'gen-code', name: 'Gen Code', icon: 'code', category: 'gen' },
            { id: 'gen-test', name: 'Gen Tests', icon: 'beaker', category: 'gen' },
            { id: 'gen-docs', name: 'Gen Docs', icon: 'book', category: 'gen' },
            { id: 'gen-types', name: 'Gen Types', icon: 'symbol-type', category: 'gen' },
            { id: 'gen-api', name: 'Gen API', icon: 'globe', category: 'gen' },
            { id: 'gen-sql', name: 'Gen SQL', icon: 'database', category: 'gen' },
            { id: 'gen-regex', name: 'Gen Regex', icon: 'regex', category: 'gen' },
            { id: 'gen-config', name: 'Gen Config', icon: 'gear', category: 'gen' },
            // Analysis (8)
            { id: 'analyze-code', name: 'Analyze Code', icon: 'search', category: 'analysis' },
            { id: 'analyze-deps', name: 'Analyze Deps', icon: 'package', category: 'analysis' },
            { id: 'analyze-security', name: 'Security', icon: 'shield', category: 'analysis' },
            { id: 'analyze-perf', name: 'Performance', icon: 'speed', category: 'analysis' },
            { id: 'analyze-coverage', name: 'Coverage', icon: 'coverage', category: 'analysis' },
            { id: 'analyze-complexity', name: 'Complexity', icon: 'graph', category: 'analysis' },
            { id: 'analyze-smell', name: 'Code Smell', icon: 'warning', category: 'analysis' },
            { id: 'analyze-imports', name: 'Imports', icon: 'link', category: 'analysis' },
            // Refactor (8)
            { id: 'refactor-extract', name: 'Extract', icon: 'symbol-method', category: 'refactor' },
            { id: 'refactor-rename', name: 'Rename', icon: 'symbol-variable', category: 'refactor' },
            { id: 'refactor-inline', name: 'Inline', icon: 'symbol-constant', category: 'refactor' },
            { id: 'refactor-move', name: 'Move', icon: 'file-move', category: 'refactor' },
            { id: 'refactor-optimize', name: 'Optimize', icon: 'lightbulb', category: 'refactor' },
            { id: 'refactor-format', name: 'Format', icon: 'edit', category: 'refactor' },
            { id: 'refactor-sort', name: 'Sort', icon: 'list-ordered', category: 'refactor' },
            { id: 'refactor-convert', name: 'Convert', icon: 'sync', category: 'refactor' },
            // Explain (8)
            { id: 'explain-code', name: 'Explain Code', icon: 'comment', category: 'explain' },
            { id: 'explain-error', name: 'Explain Error', icon: 'error', category: 'explain' },
            { id: 'explain-regex', name: 'Explain Regex', icon: 'regex', category: 'explain' },
            { id: 'explain-sql', name: 'Explain SQL', icon: 'database', category: 'explain' },
            { id: 'explain-algo', name: 'Explain Algo', icon: 'symbol-structure', category: 'explain' },
            { id: 'explain-diff', name: 'Explain Diff', icon: 'diff', category: 'explain' },
            { id: 'explain-commit', name: 'Explain Commit', icon: 'git-commit', category: 'explain' },
            { id: 'explain-api', name: 'Explain API', icon: 'globe', category: 'explain' },
            // Fix (8)
            { id: 'fix-bug', name: 'Fix Bug', icon: 'bug', category: 'fix' },
            { id: 'fix-lint', name: 'Fix Lint', icon: 'check', category: 'fix' },
            { id: 'fix-types', name: 'Fix Types', icon: 'symbol-type', category: 'fix' },
            { id: 'fix-imports', name: 'Fix Imports', icon: 'link', category: 'fix' },
            { id: 'fix-merge', name: 'Fix Merge', icon: 'git-merge', category: 'fix' },
            { id: 'fix-deps', name: 'Fix Deps', icon: 'package', category: 'fix' },
            { id: 'fix-security', name: 'Fix Security', icon: 'shield', category: 'fix' },
            { id: 'fix-perf', name: 'Fix Perf', icon: 'speed', category: 'fix' },
            // Chat (8)
            { id: 'chat-general', name: 'Chat', icon: 'comment-discussion', category: 'chat' },
            { id: 'chat-code', name: 'Code Chat', icon: 'code', category: 'chat' },
            { id: 'chat-debug', name: 'Debug Chat', icon: 'debug', category: 'chat' },
            { id: 'chat-review', name: 'Review', icon: 'eye', category: 'chat' },
            { id: 'chat-pair', name: 'Pair', icon: 'person', category: 'chat' },
            { id: 'chat-learn', name: 'Learn', icon: 'mortar-board', category: 'chat' },
            { id: 'chat-arch', name: 'Arch', icon: 'symbol-structure', category: 'chat' },
            { id: 'chat-design', name: 'Design', icon: 'paintbrush', category: 'chat' },
            // Utils (8)
            { id: 'util-translate', name: 'Translate', icon: 'globe', category: 'util' },
            { id: 'util-summarize', name: 'Summarize', icon: 'list-flat', category: 'util' },
            { id: 'util-complete', name: 'Complete', icon: 'sparkle', category: 'util' },
            { id: 'util-snippets', name: 'Snippets', icon: 'symbol-snippet', category: 'util' },
            { id: 'util-commit', name: 'Git Commit', icon: 'git-commit', category: 'util' },
            { id: 'util-pr', name: 'PR Draft', icon: 'git-pull-request', category: 'util' },
            { id: 'util-readme', name: 'README', icon: 'book', category: 'util' },
            { id: 'util-changelog', name: 'Changelog', icon: 'history', category: 'util' },
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
        // SAFETY: WebView CSP strict
        webviewView.webview.html = this.getHtmlForWebview(webviewView.webview);
        webviewView.webview.onDidReceiveMessage((msg) => { void this.handleMessage(msg); });
    }
    /** Handle messages from WebView */
    async handleMessage(message) {
        if (message.command === 'executeTool') {
            const tool = this.tools.find((t) => t.id === String(message.data ?? ''));
            if (tool)
                await vscode.window.showInformationMessage(`Executing: ${tool.name}`);
        }
    }
    /**
     * Generate HTML for WebView
     * SAFETY: WebView CSP strict - nonce-based CSP, vscode-webview://
     */
    getHtmlForWebview(_webview) {
        const nonce = this.getNonce();
        // Build tool buttons using map for type safety
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
  <h2>Hajimi Tools (56)</h2>
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