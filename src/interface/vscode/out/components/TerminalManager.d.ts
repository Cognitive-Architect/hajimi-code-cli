import * as vscode from 'vscode';
export declare class TerminalManager implements vscode.Disposable {
    private terminals;
    private counter;
    private readonly serverUrl;
    private readonly allowedProtocols;
    constructor(serverUrl?: string);
    private getValidatedHttpUrl;
    createTerminal(): vscode.Terminal;
    private createWebSocket;
    private setupWebSocket;
    sendText(terminal: vscode.Terminal, text: string): void;
    sendCommand(terminalId: number, command: string): boolean;
    getTerminal(id: number): vscode.Terminal | undefined;
    getActiveTerminals(): number[];
    dispose(): void;
}
//# sourceMappingURL=TerminalManager.d.ts.map