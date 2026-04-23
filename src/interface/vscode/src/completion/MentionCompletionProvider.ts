import * as vscode from 'vscode';
import { ContextProvider } from '../context/ContextProvider';

/**
 * MentionCompletionProvider — VSCode-native CompletionItemProvider for @file and #folder mentions.
 *
 * Replaces the postMessage-based file list request (DEBT-W5-COMPLETION-API-001)
 * with a native VSCode completion provider that triggers on `@` and `#` characters.
 *
 * Features:
 * - @file: lists workspace files matching the typed prefix
 * - #folder: lists workspace folders matching the typed prefix
 * - Icons and detail labels for each completion item
 * - Falls back to empty list on workspace scan errors
 *
 * The existing InputBox postMessage fallback remains functional for webview-embedded usage.
 */
export class MentionCompletionProvider implements vscode.CompletionItemProvider {
  private contextProvider: ContextProvider;

  constructor(context: vscode.ExtensionContext) {
    this.contextProvider = new ContextProvider();
    this.contextProvider.setGlobalState(context.globalState);
  }

  public async provideCompletionItems(
    document: vscode.TextDocument,
    position: vscode.Position,
    _token: vscode.CancellationToken,
    context: vscode.CompletionContext
  ): Promise<vscode.CompletionItem[]> {
    const lineText = document.lineAt(position).text.slice(0, position.character);
    const triggerChar = context.triggerCharacter;

    if (triggerChar === '@' || lineText.endsWith('@')) {
      return this.provideFileCompletions(lineText);
    }

    if (triggerChar === '#' || lineText.endsWith('#')) {
      return this.provideFolderCompletions(lineText);
    }

    return [];
  }

  private async provideFileCompletions(lineText: string): Promise<vscode.CompletionItem[]> {
    try {
      const files = await this.contextProvider.getWorkspaceFiles();
      const prefix = this.extractPrefix(lineText, '@');
      const filtered = prefix
        ? files.filter((f) => f.relativePath.toLowerCase().includes(prefix.toLowerCase()))
        : files;
      return filtered.slice(0, 50).map((f) => {
        const item = new vscode.CompletionItem(f.relativePath, vscode.CompletionItemKind.File);
        item.detail = `${f.language} file`;
        item.insertText = f.relativePath;
        item.sortText = `0-${f.relativePath}`;
        return item;
      });
    } catch {
      return [];
    }
  }

  private async provideFolderCompletions(lineText: string): Promise<vscode.CompletionItem[]> {
    try {
      const folders = vscode.workspace.workspaceFolders ?? [];
      const prefix = this.extractPrefix(lineText, '#');
      const filtered = prefix
        ? folders.filter((f) => f.name.toLowerCase().includes(prefix.toLowerCase()))
        : folders;
      return filtered.map((f) => {
        const item = new vscode.CompletionItem(f.name, vscode.CompletionItemKind.Folder);
        item.detail = `Workspace folder`;
        item.insertText = f.name;
        item.sortText = `1-${f.name}`;
        return item;
      });
    } catch {
      return [];
    }
  }

  private extractPrefix(lineText: string, trigger: string): string | null {
    const idx = lineText.lastIndexOf(trigger);
    if (idx === -1) return null;
    const after = lineText.slice(idx + 1);
    return after.length > 0 ? after : null;
  }
}
