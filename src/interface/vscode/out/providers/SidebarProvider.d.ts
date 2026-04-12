import * as vscode from 'vscode';
/**
 * SidebarProvider - Hajimi main panel with 56 tool shortcuts
 * SAFETY: WebView CSP strict - vscode-webview:// protocol only
 */
export declare class SidebarProvider implements vscode.WebviewViewProvider {
    private readonly extensionUri;
    static readonly viewId = "hajimi.sidebar";
    private _view?;
    /** 56 tools: 8 categories × 7 tools each */
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
    /**
     * Generate HTML for WebView
     * SAFETY: WebView CSP strict - nonce-based CSP, vscode-webview://
     */
    private getHtmlForWebview;
    /** Generate nonce for CSP */
    private getNonce;
}
//# sourceMappingURL=SidebarProvider.d.ts.map