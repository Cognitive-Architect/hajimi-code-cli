import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext): void {
  const disposable = vscode.commands.registerCommand('hajimi.openSidebar', () => {
    vscode.window.showInformationMessage('Hajimi: open the web UI with `npx serve src/interface/web`');
  });
  context.subscriptions.push(disposable);
}

export function deactivate(): void {}
