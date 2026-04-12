"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.StatusBar = void 0;
// StatusBar: VSCode status bar for connection/tools display
const vscode = __importStar(require("vscode"));
/** StatusBar manager - displays connection state and tool count */
class StatusBar {
    constructor(rpcClient) {
        this.toolCount = 56;
        this.rpcClient = rpcClient;
        this.statusItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
        this.statusItem.command = 'hajimi.showStatusMenu';
        this.statusItem.show();
        this.setupEventSubscription();
    }
    update() {
        const state = this.rpcClient.getState();
        const cfg = this.getStatusConfig(state);
        this.statusItem.text = `$(${cfg.icon}) ${cfg.text}`;
        this.statusItem.tooltip = cfg.tooltip;
        this.statusItem.color = cfg.color;
    }
    getStatusConfig(state) {
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
    setupEventSubscription() {
        this.update();
        this.unsubscribeStateChange = this.rpcClient.onStateChange(() => this.update());
    }
    dispose() {
        this.unsubscribeStateChange?.();
        this.statusItem.dispose();
    }
}
exports.StatusBar = StatusBar;
//# sourceMappingURL=StatusBar.js.map