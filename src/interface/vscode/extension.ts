import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';

export function activate(context: vscode.ExtensionContext) {
  // Register the command to open an ADR by URI or debt_id.
  const openAdrCommand = vscode.commands.registerCommand(
    'command.openAdr',
    async (uri?: vscode.Uri, debt_id?: string) => {
      try {
        let targetUri = uri;
        if (!targetUri && debt_id) {
          const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
          if (!workspaceRoot) {
            vscode.window.showErrorMessage('No workspace folder open.');
            return;
          }
          const candidate = path.join(workspaceRoot, 'docs', 'adr', `${debt_id}.md`);
          if (!fs.existsSync(candidate)) {
            vscode.window.showErrorMessage(`ADR file not found: ${candidate}`);
            return;
          }
          targetUri = vscode.Uri.file(candidate);
        }
        if (!targetUri) {
          vscode.window.showErrorMessage('No ADR file or DEBT- identifier provided.');
          return;
        }
        const doc = await vscode.workspace.openTextDocument(targetUri);
        await vscode.window.showTextDocument(doc);
      } catch (err) {
        vscode.window.showErrorMessage(`Failed to open ADR: ${err}`);
      }
    }
  );

  // Register the command to jump from a DEBT- reference in the active editor.
  const gotoAdrCommand = vscode.commands.registerCommand(
    'command.gotoAdr',
    async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showInformationMessage('No active editor.');
        return;
      }
      const lineText = editor.document.lineAt(editor.selection.active.line).text;
      const match = lineText.match(/(DEBT-[A-Z0-9-]+)/);
      if (!match) {
        vscode.window.showWarningMessage('No debt_id (DEBT-*) found on current line.');
        return;
      }
      const debt_id = match[1];
      vscode.commands.executeCommand('command.openAdr', undefined, debt_id);
    }
  );

  context.subscriptions.push(openAdrCommand, gotoAdrCommand);
}

export function deactivate() {}
