import * as vscode from 'vscode';
import { VsCodeRpcClient } from '../adapters/rpcAdapter';
/** StatusBar manager - displays connection state and tool count */
export declare class StatusBar implements vscode.Disposable {
    private statusItem;
    private rpcClient;
    private readonly toolCount;
    private unsubscribeStateChange?;
    constructor(rpcClient: VsCodeRpcClient);
    update(): void;
    private getStatusConfig;
    private setupEventSubscription;
    dispose(): void;
}
//# sourceMappingURL=StatusBar.d.ts.map