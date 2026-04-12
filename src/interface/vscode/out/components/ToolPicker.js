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
exports.ToolPicker = void 0;
const vscode = __importStar(require("vscode"));
const tools_1 = require("../data/tools");
class ToolPicker {
    constructor() {
        this.recentlyUsed = [];
        this.allTools = [];
        this.outputChannel = vscode.window.createOutputChannel('Hajimi ToolPicker');
        this.allTools = [...(0, tools_1.getPhase2Tools)(), ...(0, tools_1.getWebSocketTools)()];
    }
    static getInstance() {
        if (!ToolPicker.instance)
            ToolPicker.instance = new ToolPicker();
        return ToolPicker.instance;
    }
    fuzzySearch(query, tool) {
        const q = query.toLowerCase().replace(/\s+/g, '');
        const text = (tool.name + tool.description + tool.category).toLowerCase();
        let i = 0;
        for (const c of text)
            if (i < q.length && c === q[i])
                i++;
        return i === q.length;
    }
    categoryFilter(tools, category) {
        if (!category || category === 'all')
            return tools;
        return tools.filter(t => t.category === category);
    }
    toItem(tool) {
        return {
            label: `$(${tool.icon}) ${tool.name}`,
            description: tool.category,
            detail: tool.description,
            toolId: `${ToolPicker.TOOL_PREFIX}${tool.id}`,
            command: tool.command
        };
    }
    getItems(category) {
        const filtered = this.categoryFilter(this.allTools, category);
        return filtered.sort((a, b) => {
            const ia = this.recentlyUsed.indexOf(a.id), ib = this.recentlyUsed.indexOf(b.id);
            if (ia !== -1 && ib !== -1)
                return ia - ib;
            if (ia !== -1)
                return -1;
            if (ib !== -1)
                return 1;
            return a.name.localeCompare(b.name);
        }).map(t => this.toItem(t));
    }
    addRecent(toolId) {
        const cleanId = toolId.replace(ToolPicker.TOOL_PREFIX, '');
        this.recentlyUsed = [cleanId, ...this.recentlyUsed.filter(id => id !== cleanId)].slice(0, ToolPicker.MAX_RECENT);
    }
    async show() {
        const qp = vscode.window.createQuickPick();
        qp.placeholder = 'Select a tool (type to search)...';
        qp.items = this.getItems();
        return new Promise(resolve => {
            qp.onDidChangeValue(v => {
                qp.items = v ? this.allTools.filter(t => this.fuzzySearch(v, t)).map(t => this.toItem(t)) : this.getItems();
            });
            qp.onDidAccept(() => {
                const s = qp.selectedItems[0];
                if (s) {
                    this.addRecent(s.toolId);
                    this.outputChannel.appendLine(`Selected: ${s.toolId}`);
                }
                qp.dispose();
                resolve(s?.toolId);
            });
            qp.onDidHide(() => { qp.dispose(); resolve(undefined); });
            qp.show();
        });
    }
    showByCategory(category) {
        const qp = vscode.window.createQuickPick();
        qp.placeholder = `Select a ${category} tool...`;
        qp.items = this.getItems(category);
        return new Promise(resolve => {
            qp.onDidAccept(() => {
                const s = qp.selectedItems[0];
                if (s)
                    this.addRecent(s.toolId);
                qp.dispose();
                resolve(s?.toolId);
            });
            qp.onDidHide(() => { qp.dispose(); resolve(undefined); });
            qp.show();
        });
    }
}
exports.ToolPicker = ToolPicker;
ToolPicker.MAX_RECENT = 10;
ToolPicker.TOOL_PREFIX = 'hajimi.tools.';
//# sourceMappingURL=ToolPicker.js.map