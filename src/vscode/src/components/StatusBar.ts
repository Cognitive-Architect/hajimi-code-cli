// StatusBar: VSCode status bar for connection/tools display
import * as vscode from 'vscode';
import { VsCodeRpcClient } from '../adapters/rpcAdapter';

interface StatusConfig {
  text: string;
  tooltip: string;
  color: string;
  icon: string;
}

/** StatusBar manager - displays connection state and tool count */
export class StatusBar implements vscode.Disposable {
  private statusItem: vscode.StatusBarItem;
  private rpcClient: VsCodeRpcClient;
  private readonly toolCount = 56;
  private unsubscribeStateChange?: () => void;

  constructor(rpcClient: VsCodeRpcClient) {
    this.rpcClient = rpcClient;
    this.statusItem = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Right,
      100
    );
    this.statusItem.command = 'hajimi.showStatusMenu';
    this.statusItem.show();
    this.setupEventSubscription();
  }

  update(): void {
    const state = this.rpcClient.getState();
    const cfg = this.getStatusConfig(state);
    this.statusItem.text = `$(${cfg.icon}) ${cfg.text}`;
    this.statusItem.tooltip = cfg.tooltip;
    this.statusItem.color = cfg.color;
  }

  private getStatusConfig(state: string): StatusConfig {
    if (state === 'connected') {
      return {
        text: `${this.toolCount} Tools`,
        tooltip: 'Hajimi: Connected - Click for options',
        color: '#89d185',
        icon: 'plug'
      };
    }
    if (state === 'connecting' || state === 'reconnecting') {
      return {
        text: 'Connecting...',
        tooltip: 'Hajimi: Reconnecting',
        color: '#cca700',
        icon: 'sync~spin'
      };
    }
    return {
      text: 'Offline',
      tooltip: 'Hajimi: Disconnected - Click to reconnect',
      color: '#808080',
      icon: 'circle-filled'
    };
  }

  private setupEventSubscription(): void {
    this.update();
    this.unsubscribeStateChange = this.rpcClient.onStateChange(() => this.update());
  }

  dispose(): void {
    this.unsubscribeStateChange?.();
    this.statusItem.dispose();
  }
}
