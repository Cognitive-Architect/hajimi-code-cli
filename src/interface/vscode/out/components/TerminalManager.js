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
exports.TerminalManager = void 0;
const vscode = __importStar(require("vscode"));
class TerminalManager {
    constructor(serverUrl = 'ws://localhost:8080') {
        this.terminals = new Map();
        this.counter = 0;
        this.allowedProtocols = ['http:', 'https:', 'ws:', 'wss:'];
        this.serverUrl = serverUrl;
    }
    getValidatedHttpUrl(wsUrl) {
        try {
            const url = new URL(wsUrl);
            if (!this.allowedProtocols.includes(url.protocol)) {
                throw new Error(`Invalid protocol: ${url.protocol}`);
            }
            return wsUrl.replace('ws://', 'http://').replace('wss://', 'https://');
        }
        catch {
            return 'http://localhost:8080';
        }
    }
    createTerminal() {
        this.counter++;
        const id = this.counter;
        const name = `hajimi-terminal-${id}`;
        const env = {
            HAJIMI_SERVER_URL: this.getValidatedHttpUrl(this.serverUrl)
        };
        const terminal = vscode.window.createTerminal({ name, env });
        const ws = this.createWebSocket();
        this.terminals.set(id, { terminal, ws, id });
        this.setupWebSocket(ws, id);
        terminal.show();
        return terminal;
    }
    createWebSocket() {
        return new (require('ws'))(this.serverUrl);
    }
    setupWebSocket(ws, id) {
        ws.on('open', () => {
            const session = this.terminals.get(id);
            if (session)
                this.sendText(session.terminal, `echo "[Hajimi] Connected"`);
        });
        ws.on('message', (data) => {
            const session = this.terminals.get(id);
            if (session)
                this.sendText(session.terminal, `echo "[Server] ${String(data)}"`);
        });
        ws.on('close', () => this.terminals.delete(id));
    }
    sendText(terminal, text) {
        terminal.sendText(text, true);
    }
    sendCommand(terminalId, command) {
        const session = this.terminals.get(terminalId);
        if (session) {
            this.sendText(session.terminal, command);
            return true;
        }
        return false;
    }
    getTerminal(id) {
        return this.terminals.get(id)?.terminal;
    }
    getActiveTerminals() {
        return Array.from(this.terminals.keys());
    }
    dispose() {
        for (const session of this.terminals.values()) {
            session.ws.close();
            session.terminal.dispose();
        }
        this.terminals.clear();
    }
}
exports.TerminalManager = TerminalManager;
//# sourceMappingURL=TerminalManager.js.map