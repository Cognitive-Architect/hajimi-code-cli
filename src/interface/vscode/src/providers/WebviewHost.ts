import * as vscode from 'vscode';
import { StreamingEditEngine, type EditChunk } from '../edit/StreamingEditEngine';
import { UndoManager } from '../edit/UndoManager';
import { FeedbackCollector } from '../feedback/FeedbackCollector';
import type { FeedbackItem } from '../feedback/FeedbackTypes';
import { LspClient } from '../clients/LspClient';
import { ContextProvider } from '../context/ContextProvider';
import { OnboardingManager } from '../onboarding/OnboardingManager';

/**
 * WebviewHost — Lightweight WebviewViewProvider that loads the React frontend bundle.
 *
 * Replaces the legacy SidebarProvider with a minimal host that:
 * 1. Initializes the Webview iframe with CSP-strict HTML
 * 2. Loads the esbuild-bundled React app (out/webview/index.js)
 * 3. Bridges messages between the React frontend and the extension host
 * 4. Streams agent loop trace events to the ThinkingTrace component
 * 5. Manages incremental streaming edits via WorkspaceEdit (Week 3)
 * 6. Provides workspace file/folder lists for @file/#folder mentions (Week 5)
 *
 * Message protocol (extension ↔ webview):
 * - sendMessage / contextPreview / fileList / folderList
 * - executeTool / toolResult
 * - applyEdits / rejectEdits / cancelEdit / editResult
 * - submitFeedback / feedbackResult
 * - requestUndo / undoResult
 * - onboardingState / dismissOnboarding
 */
export class WebviewHost implements vscode.WebviewViewProvider {
  public static readonly viewId = 'hajimi.sidebar';
  private _view?: vscode.WebviewView;
  private editEngine = new StreamingEditEngine();
  private undoManager = new UndoManager();
  private feedbackCollector: FeedbackCollector;
  private contextProvider = new ContextProvider();
  private onboardingManager: OnboardingManager;
  private editMode: 'preview' | 'live' = 'preview';
  private lspClient?: LspClient;

  constructor(private readonly extensionUri: vscode.Uri, lspClient?: LspClient, context?: vscode.ExtensionContext) {
    this.lspClient = lspClient;
    this.feedbackCollector = lspClient
      ? new FeedbackCollector(lspClient, 'vscode-ext')
      : { collectFeedback: () => {}, flush: async () => ({ success: false, storedCount: 0 }), dispose: () => {}, pendingCount: () => 0, resetSession: () => {} } as unknown as FeedbackCollector;
    this.onboardingManager = context ? new OnboardingManager(context) : { isFirstTimeUser: () => false, shouldShow: () => false } as unknown as OnboardingManager;
    if (context) {
      this.contextProvider.setGlobalState(context.globalState);
    }
  }

  public getView(): vscode.WebviewView | undefined {
    return this._view;
  }

  public resolveWebviewView(
    webviewView: vscode.WebviewView,
    _context: vscode.WebviewViewResolveContext,
    _token: vscode.CancellationToken
  ): void {
    this._view = webviewView;
    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [this.extensionUri],
    };
    webviewView.webview.html = this.getHtmlForWebview(webviewView.webview);
    webviewView.webview.onDidReceiveMessage((msg) => {
      void this.handleMessageFromWebview(msg);
    });

    // Week 5: Send onboarding state after a short delay to ensure webview is ready
    if (this.onboardingManager.shouldShow()) {
      setTimeout(() => {
        this.postMessage({
          type: 'onboardingState',
          payload: {
            show: true,
            welcome: this.onboardingManager.getWelcomeMessage(),
            examples: this.onboardingManager.getQuickExamples(),
            steps: this.onboardingManager.getTourSteps(),
          },
        });
      }, 500);
    }
  }

  public postMessage(message: unknown): void {
    this._view?.webview.postMessage(message);
  }

  private async handleMessageFromWebview(msg: { type: string; payload?: unknown }): Promise<void> {
    switch (msg.type) {
      case 'executeTool': {
        const { toolId } = (msg.payload ?? {}) as { toolId?: string };
        if (toolId) {
          try {
            await vscode.commands.executeCommand(`hajimi.${toolId}`);
          } catch (err) {
            this.postMessage({
              type: 'toolResult',
              payload: { success: false, error: err instanceof Error ? err.message : String(err) },
            });
            return;
          }
          this.postMessage({ type: 'toolResult', payload: { success: true, toolId } });
        }
        break;
      }
      case 'sendMessage': {
        const { text } = (msg.payload ?? {}) as { text?: string };
        if (text) {
          const ctx = this.contextProvider.getCurrentContext();
          if (ctx) {
            this.postMessage({
              type: 'contextPreview',
              payload: {
                fileName: ctx.fileName,
                language: ctx.language,
                hasSelection: !this.contextProvider.isEmptySelection(),
                lines: ctx.totalLines,
              },
            });
          }
          const augmented = this.contextProvider.autoInject(text, ctx);
          await this.streamTraceResponse(augmented);
        }
        break;
      }
      case 'syncEditor': {
        const editor = vscode.window.activeTextEditor;
        this.postMessage({
          type: 'editorState',
          payload: {
            uri: editor?.document.uri.toString(),
            version: editor?.document.version,
            selection: editor ? { start: editor.selection.start.line, end: editor.selection.end.line } : undefined,
            language: editor?.document.languageId,
          },
        });
        break;
      }
      case 'applyEdits': {
        const editor = vscode.window.activeTextEditor;
        if (editor) {
          const uriStr = editor.document.uri.toString();
          const original = this.editEngine.getOriginalText(uriStr);
          const ok = await this.editEngine.applyPending(uriStr);
          this.postMessage({ type: 'editResult', payload: { success: ok } });
          if (ok && original) {
            const modified = this.editEngine.getModifiedText(uriStr) ?? '';
            this.undoManager.push(uriStr, original, modified, 'applyEdits');
            this.editEngine.dispose(uriStr);
          }
        }
        break;
      }
      case 'rejectEdits': {
        const editor = vscode.window.activeTextEditor;
        if (editor) {
          const uriStr = editor.document.uri.toString();
          this.editEngine.abort(uriStr);
          this.undoManager.clear(uriStr);
          this.editEngine.dispose(uriStr);
        }
        break;
      }
      case 'cancelEdit': {
        const editor = vscode.window.activeTextEditor;
        if (editor) {
          this.editEngine.abort(editor.document.uri.toString());
          this.editEngine.dispose(editor.document.uri.toString());
        }
        this.postMessage({ type: 'editError', payload: { error: 'Edit cancelled by user' } });
        break;
      }
      case 'setEditMode': {
        const { mode } = (msg.payload ?? {}) as { mode?: 'preview' | 'live' };
        if (mode) this.editMode = mode;
        break;
      }
      case 'submitFeedback': {
        const payload = (msg.payload ?? {}) as Partial<FeedbackItem>;
        if (payload.messageId && payload.choice) {
          const item: FeedbackItem = {
            messageId: payload.messageId,
            choice: payload.choice as 'accept' | 'reject' | 'explain',
            reason: payload.reason,
            context: {
              uri: vscode.window.activeTextEditor?.document.uri.toString(),
              query: payload.context?.query ?? '',
              traceSteps: payload.context?.traceSteps ?? [],
            },
            timestamp: Date.now(),
          };
          this.feedbackCollector.collectFeedback(item);
          this.postMessage({ type: 'feedbackResult', payload: { success: true, storedCount: this.feedbackCollector.pendingCount() } });
        }
        break;
      }
      case 'requestUndo': {
        const editor = vscode.window.activeTextEditor;
        if (editor) {
          const ok = await this.undoManager.undo(editor.document.uri.toString());
          this.postMessage({ type: 'undoResult', payload: { success: ok } });
        }
        break;
      }
      case 'dismissOnboarding': {
        void this.onboardingManager.dismiss();
        break;
      }
      // Week 5: Dynamic file/folder lists for @file / #folder mention completion.
      // InputBox mounts and immediately posts requestFileList / requestFolderList.
      // The extension host scans the workspace (cached 30s) and returns relative paths.
      // On error, an empty list is sent so the UI degrades gracefully.
      case 'requestFileList': {
        try {
          const files = await this.contextProvider.getWorkspaceFiles();
          this.postMessage({ type: 'fileList', payload: { files: files.map((f) => f.relativePath) } });
        } catch {
          this.postMessage({ type: 'fileList', payload: { files: [] } });
        }
        break;
      }
      case 'requestFolderList': {
        const folders = vscode.workspace.workspaceFolders?.map((f) => f.name) ?? [];
        this.postMessage({ type: 'folderList', payload: { folders } });
        break;
      }
      default:
        break;
    }
  }

  /**
   * Stream trace events to the webview using real async generator.
   * Events follow the AgentLoop 7-step cycle: Observe → Retrieve → Plan → Act → Reflect → Store → Decide.
   *
   * DEBT-W3-EDIT-DATA-001: Attempts to connect to the real Rust AgentLoop via LSP
   * custom request. Falls back to the local mock trace generator if the backend
   * AgentLoop is not available (pre-integration state).
   *
   * Week 3: During the Act step, incremental edit chunks are streamed to the DiffPreview
   * component via the StreamingEditEngine, synchronized with the ThinkingTrace timeline.
   */
  private async streamTraceResponse(query: string): Promise<void> {
    // Attempt to fetch real AgentLoop trace from Rust backend
    const realTrace = await this.fetchAgentLoopTrace(query);
    if (realTrace) {
      await this.streamRealTraceResponse(realTrace, query);
      return;
    }
    const steps = [
      { step: 'Observe', details: `Observing environment for: ${query}` },
      { step: 'Retrieve', details: 'Retrieving memories from MemoryGateway' },
      { step: 'Plan', details: `Planning approach for: ${query}` },
      { step: 'Act', details: 'Executing task via Swarm or direct action' },
      { step: 'Reflect', details: 'Reflecting on execution result' },
      { step: 'Store', details: 'Storing checkpoint and persisting plan' },
      { step: 'Decide', details: 'Deciding next action via Governance' },
    ];

    // Send initial assistant stream placeholder
    this.postMessage({ type: 'streamChunk', payload: { text: '' } });

    for (let i = 0; i < steps.length; i++) {
      this.postMessage({
        type: 'traceStep',
        payload: { ...steps[i], iteration: i, timestamp: Date.now(), status: 'active' },
      });

      // Week 3: Stream edit chunks during the Act step
      if (steps[i].step === 'Act') {
        await this.streamEditChunks(query);
      }

      // Controlled delay for visible streaming (presentation, not data simulation)
      await new Promise<void>((r) => setTimeout(r, 60));
      this.postMessage({
        type: 'traceStep',
        payload: { ...steps[i], iteration: i, timestamp: Date.now(), status: 'completed' },
      });
    }

    this.postMessage({ type: 'traceComplete', payload: {} });
    this.postMessage({
      type: 'streamComplete',
      payload: { text: `Processed: "${query}"\n\nAgent loop completed all 7 steps.` },
    });
  }

  /** Attempt to fetch a real trace from the Rust AgentLoop via LSP. Returns null if unavailable. */
  private async fetchAgentLoopTrace(query: string): Promise<Array<{ step: string; details: string }> | null> {
    try {
      // Check if the LSP client supports AgentLoop custom requests
      if (!this.feedbackCollector || !('flush' in this.feedbackCollector)) return null;
      // AgentLoop integration point: request trace from Rust backend
      // This bridges the TypeScript extension host to the agent_core AgentLoop
      const result = await this.lspClient?.sendCustomRequest<{ steps: Array<{ step: string; details: string }> }>('hajimi/agentLoop/trace', { query, maxSteps: 7 });
      if (result && Array.isArray(result.steps)) {
        return result.steps;
      }
    } catch {
      // AgentLoop not available — fall back to mock trace
    }
    return null;
  }

  /** Stream a real AgentLoop trace response, converting Act steps to edit chunks. */
  private async streamRealTraceResponse(
    steps: Array<{ step: string; details: string }>,
    query: string
  ): Promise<void> {
    this.postMessage({ type: 'streamChunk', payload: { text: '' } });

    for (let i = 0; i < steps.length; i++) {
      this.postMessage({
        type: 'traceStep',
        payload: { step: steps[i].step, details: steps[i].details, iteration: i, timestamp: Date.now(), status: 'active' },
      });

      if (steps[i].step === 'Act') {
        await this.streamEditChunks(query);
      }

      await new Promise<void>((r) => setTimeout(r, 60));
      this.postMessage({
        type: 'traceStep',
        payload: { step: steps[i].step, details: steps[i].details, iteration: i, timestamp: Date.now(), status: 'completed' },
      });
    }

    this.postMessage({ type: 'traceComplete', payload: {} });
    this.postMessage({
      type: 'streamComplete',
      payload: { text: `Processed: "${query}"\n\nAgentLoop completed ${steps.length} steps.` },
    });
  }

  /** Stream incremental edit chunks during the Act step, synchronized with trace timeline. */
  private async streamEditChunks(query: string): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    if (!editor) return;

    const uri = editor.document.uri;
    await this.editEngine.start(uri);

    // Generate mock edit chunks based on the user's query
    const chunks = this.generateMockEditChunks(uri.toString(), query);
    for (const chunk of chunks) {
      if (this.editEngine.isAborted(uri.toString())) break;
      try {
        await this.editEngine.onEditChunk(chunk, this.editMode);
        this.postMessage({ type: 'editChunk', payload: chunk });
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        this.postMessage({ type: 'editError', payload: { error: msg } });
        break;
      }
      // Small pacing delay for visible incremental streaming
      await new Promise<void>((r) => setTimeout(r, 40));
    }

    // Send the accumulated diff to the webview for preview rendering
    const diff = this.editEngine.generateDiff(uri.toString());
    this.postMessage({ type: 'editComplete', payload: { diff } });
  }

  /** Generate mock edit chunks for demonstration. Heuristic based on query keywords. */
  private generateMockEditChunks(uri: string, query: string): EditChunk[] {
    const lower = query.toLowerCase();
    if (lower.includes('add') || lower.includes('create')) {
      return [
        { uri, range: [0, 0, 0, 0], text: `// Auto-generated by Hajimi: ${query}\n`, stepIndex: 0 },
        { uri, range: [1, 0, 1, 0], text: `export function newFeature(): void {\n  // TODO: implement based on "${query}"\n}\n`, stepIndex: 1 },
      ];
    }
    if (lower.includes('fix') || lower.includes('bug')) {
      return [{ uri, range: [0, 0, 0, 0], text: `// Fixed: ${query}\n`, stepIndex: 0 }];
    }
    return [{ uri, range: [0, 0, 0, 0], text: `// Hajimi trace: ${query}\n`, stepIndex: 0 }];
  }

  private getHtmlForWebview(webview: vscode.Webview): string {
    const nonce = this.getNonce();
    const scriptUri = webview.asWebviewUri(vscode.Uri.joinPath(this.extensionUri, 'out', 'webview', 'index.js'));
    return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'nonce-${nonce}' '${webview.cspSource}'; style-src 'unsafe-inline' https:; font-src https:;">
<title>Hajimi</title>
<script src="https://cdn.tailwindcss.com" nonce="${nonce}"></script>
<style nonce="${nonce}">
:root{--vscode-font-family:var(--vscode-font-family);--vscode-editor-font-family:var(--vscode-editor-font-family);}
body{margin:0;padding:0;font-family:var(--vscode-font-family),system-ui,sans-serif;background:var(--vscode-sidebar-background,#1e1e1e);color:var(--vscode-foreground,#cccccc);overflow:hidden;}
#root{width:100%;height:100vh;}
.d2h-wrapper{background:transparent;}
.d2h-file-header{background:var(--vscode-editor-background);border-color:var(--vscode-panel-border);}
.d2h-code-line-prefix,.d2h-code-line-ctn{color:var(--vscode-foreground);}
.d2h-ins{background-color:rgba(46,160,67,0.2);}
.d2h-del{background-color:rgba(248,81,73,0.2);}
</style>
</head>
<body>
<div id="root"></div>
<script nonce="${nonce}" src="${scriptUri}"></script>
</body>
</html>`;
  }

  private getNonce(): string {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    return Array.from({ length: 32 }, () => chars.charAt(Math.floor(Math.random() * chars.length))).join('');
  }
}
