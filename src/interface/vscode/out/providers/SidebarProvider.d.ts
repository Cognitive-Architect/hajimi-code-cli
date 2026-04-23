import * as vscode from 'vscode';
/**
 * SidebarProvider — Hajimi sidebar with 7 real commands.
 * Week 6 hemostasis: reduced from 64 commands → 7 real commands.
 * Sidebar tool list now strictly matches CommandRegistry enum.
 */
export declare class SidebarProvider implements vscode.WebviewViewProvider {
    private readonly extensionUri;
    static readonly viewId = "hajimi.sidebar";
    private _view?;
    /**
     * 7 real commands — each maps to a CommandRegistry entry.
     * invokeMcpTool calls vscode.commands.executeCommand(`hajimi.${id}`).
     * Registered commands: hajimi.openSidebar, hajimi.searchCode, hajimi.toggleTerminal,
     * hajimi.test.run, hajimi.build, hajimi.git.commit, hajimi.adr.open.
     */
    private readonly tools;
    constructor(extensionUri: vscode.Uri);
    /** Get current webview view instance */
    getView(): vscode.WebviewView | undefined;
    /**
     * Resolve WebView view - SAFETY: WebView CSP strict
     */
    resolveWebviewView(webviewView: vscode.WebviewView, _context: vscode.WebviewViewResolveContext, _token: vscode.CancellationToken): void;
    /** Handle messages from WebView */
    private handleMessage;
    /** Invoke MCP tool via VSCode command registry */
    private invokeMcpTool;
    /**
     * Generate HTML for WebView
     * SAFETY: WebView CSP strict - nonce-based CSP, vscode-webview://
     */
    private getHtmlForWebview;
    /** Generate nonce for CSP */
    private getNonce;
}
//# sourceMappingURL=SidebarProvider.d.ts.map