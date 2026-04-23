import * as vscode from 'vscode';
import * as path from 'path';

/** Editor context snapshot captured from the active text editor.
 *
 *  Includes file metadata, language ID, selected text, and cursor position.
 *  Used by the agent to understand what the user is currently working on.
 */
export interface EditorContext {
  uri: string;
  language: string;
  fileName: string;
  selectedText: string;
  cursorLine: number;
  selectionStart: number;
  selectionEnd: number;
  totalLines: number;
}

/** Workspace file entry for @file mention resolution. */
export interface WorkspaceFile {
  relativePath: string;
  absolutePath: string;
  language: string;
}

/** ContextProvider — Automatically reads the current editor state
 *  (active file + selected code) and formats it for injection into
 *  agent messages. Also provides workspace file enumeration for
 *  @file and @folder mention resolution.
 *
 *  Features:
 *  - getCurrentContext: snapshot of active editor
 *  - getFullFileContext: fallback when selection is empty
 *  - autoInject: appends formatted context to user messages
 *  - getWorkspaceFiles: lists files for @file completion
 *  - resolveMention: resolves @file or #folder to actual paths
 */
export class ContextProvider {
  private readonly maxSelectionLength = 4096;
  private fileCache: WorkspaceFile[] = [];
  private cacheTimestamp = 0;
  private readonly cacheTtlMs = 30000;
  private readonly globalStateCacheKey = 'hajimi.workspaceFileCache';
  private globalState?: vscode.ExtensionContext['globalState'];

  /** Capture the current editor context. Returns null if no editor open. */
  public getCurrentContext(): EditorContext | null {
    const editor = vscode.window.activeTextEditor;
    if (!editor) return null;
    const doc = editor.document;
    const sel = editor.selection;
    const selectedText = doc.getText(sel);
    return {
      uri: doc.uri.toString(),
      language: doc.languageId,
      fileName: doc.fileName,
      selectedText: selectedText.length > this.maxSelectionLength
        ? selectedText.slice(0, this.maxSelectionLength) + '\n... [truncated]'
        : selectedText,
      cursorLine: sel.active.line,
      selectionStart: sel.start.line,
      selectionEnd: sel.end.line,
      totalLines: doc.lineCount,
    };
  }

  /** Format editor context as a markdown code block for message injection. */
  public formatContext(ctx: EditorContext): string {
    const hasSelection = ctx.selectionStart !== ctx.selectionEnd || ctx.selectedText.length > 0;
    const header = hasSelection
      ? `Selected code in \`${path.basename(ctx.fileName)}\` (lines ${ctx.selectionStart + 1}-${ctx.selectionEnd + 1}):`
      : `Current file: \`${path.basename(ctx.fileName)}\` (${ctx.language}, ${ctx.totalLines} lines)`;
    return `${header}\n\`\`\`${ctx.language}\n${ctx.selectedText || '// No selection'}\n\`\`\``;
  }

  /** Auto-inject context into a user message. Returns the augmented message. */
  public autoInject(message: string, ctx?: EditorContext | null): string {
    if (!ctx || message.includes('@context-off')) return message;
    const formatted = this.formatContext(ctx);
    return `${message}\n\n---\n${formatted}`;
  }

  /** Check if the active selection is empty (just a cursor, no highlighted text). */
  public isEmptySelection(): boolean {
    const editor = vscode.window.activeTextEditor;
    if (!editor) return true;
    return editor.selection.isEmpty;
  }

  /** Fallback: get full file content when selection is empty. */
  public getFullFileContext(): EditorContext | null {
    const editor = vscode.window.activeTextEditor;
    if (!editor) return null;
    const doc = editor.document;
    return {
      uri: doc.uri.toString(),
      language: doc.languageId,
      fileName: doc.fileName,
      selectedText: doc.getText(),
      cursorLine: editor.selection.active.line,
      selectionStart: 0,
      selectionEnd: doc.lineCount - 1,
      totalLines: doc.lineCount,
    };
  }

  /** List workspace files for @file mention completion.
   *  Results are cached for 30 seconds to avoid repeated disk scans.
   *  DEBT-W5-PERF-CACHE: Caches are persisted to globalState for survival across restarts.
   */
  public async getWorkspaceFiles(): Promise<WorkspaceFile[]> {
    const now = Date.now();
    if (now - this.cacheTimestamp < this.cacheTtlMs && this.fileCache.length > 0) {
      return this.fileCache;
    }
    // Attempt to restore from persistent cache (globalState / IndexedDB-style)
    const persisted = this.restoreFromGlobalState();
    if (persisted.length > 0 && now - this.cacheTimestamp < this.cacheTtlMs * 10) {
      this.fileCache = persisted;
      return persisted;
    }
    const files: WorkspaceFile[] = [];
    const folders = vscode.workspace.workspaceFolders ?? [];
    for (const folder of folders) {
      const pattern = new vscode.RelativePattern(folder, '**/*.{rs,ts,tsx,js,jsx,json,md,toml,yaml,yml}');
      const uris = await vscode.workspace.findFiles(pattern, '**/node_modules/**', 200);
      for (const uri of uris) {
        const relative = path.relative(folder.uri.fsPath, uri.fsPath);
        files.push({ relativePath: relative, absolutePath: uri.fsPath, language: path.extname(uri.fsPath) });
      }
    }
    this.fileCache = files;
    this.cacheTimestamp = now;
    this.saveToGlobalState(files);
    return files;
  }

  /** Bind globalState for persistent cache storage. */
  public setGlobalState(globalState: vscode.ExtensionContext['globalState']): void {
    this.globalState = globalState;
  }

  private saveToGlobalState(files: WorkspaceFile[]): void {
    if (!this.globalState) return;
    try {
      this.globalState.update(this.globalStateCacheKey, { timestamp: Date.now(), files });
    } catch { /* silent */ }
  }

  private restoreFromGlobalState(): WorkspaceFile[] {
    if (!this.globalState) return [];
    try {
      const raw = this.globalState.get<{ timestamp: number; files: WorkspaceFile[] }>(this.globalStateCacheKey);
      if (raw && Array.isArray(raw.files)) {
        this.cacheTimestamp = raw.timestamp;
        return raw.files;
      }
    } catch { /* silent */ }
    return [];
  }

  /** Resolve an @file mention to its absolute path. */
  public async resolveFileMention(name: string): Promise<string | null> {
    const files = await this.getWorkspaceFiles();
    const match = files.find((f) => f.relativePath === name || path.basename(f.relativePath) === name);
    return match?.absolutePath ?? null;
  }

  /** Resolve a #folder mention to its absolute path. */
  public async resolveFolderMention(name: string): Promise<string | null> {
    const folders = vscode.workspace.workspaceFolders ?? [];
    for (const folder of folders) {
      const candidate = path.join(folder.uri.fsPath, name);
      try {
        const stat = await vscode.workspace.fs.stat(vscode.Uri.file(candidate));
        if (stat.type === vscode.FileType.Directory) return candidate;
      } catch { /* not found */ }
    }
    return null;
  }

  /** Clear the file cache (useful after large workspace changes). */
  public clearCache(): void {
    this.fileCache = [];
    this.cacheTimestamp = 0;
  }
}
