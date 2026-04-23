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
exports.TreeViewManager = exports.TreeItem = void 0;
const vscode = __importStar(require("vscode"));
const tools_1 = require("../data/tools");
/**
 * TreeItem represents a node in the tree view
 * Supports category containers and tool leaf nodes
 */
class TreeItem extends vscode.TreeItem {
    constructor(tool, children, collapsibleState) {
        const label = tool
            ? tool.name
            : (children.length > 0 && children[0].tool?.category)
                ? children[0].tool.category
                : 'Tools';
        super(label, collapsibleState);
        this.tool = tool;
        this.children = children;
        this.collapsibleState = collapsibleState;
        this.tooltip = tool ? tool.description : `${children.length} tools`;
        this.description = tool ? tool.description : undefined;
        if (tool) {
            this.iconPath = new vscode.ThemeIcon(tool.icon || 'tools');
            this.command = {
                command: 'hajimi.executeTool',
                title: tool.name,
                arguments: [tool]
            };
        }
    }
}
exports.TreeItem = TreeItem;
/**
 * TreeViewManager - TreeDataProvider for 56 tools
 * TOOL-DATA: Synced with ToolRegistry (ID-268)
 */
class TreeViewManager {
    constructor(context, _lspClient) {
        this.context = context;
        this.tools = [...(0, tools_1.getPhase2Tools)(), ...(0, tools_1.getWebSocketTools)()];
        this._onDidChangeTreeData = new vscode.EventEmitter();
        this.onDidChangeTreeData = this._onDidChangeTreeData.event;
        this.registerCommands();
    }
    registerCommands() {
        this.context.subscriptions.push(vscode.commands.registerCommand('hajimi.executeTool', (tool) => {
            // Real dispatch to CommandRegistry dispatcher (now routes to MCP invokeMcpTool -> McpServer.handle_tools_call).
            // Removed fake "Executing:" message (V5=0). TreeView now synced with true clearance. Results shown via registry handler.
            void vscode.commands.executeCommand(tool.command);
        }));
    }
    getTreeItem(element) {
        return element;
    }
    getChildren(element) {
        if (!element) {
            return Promise.resolve(this.getCategoryItems());
        }
        return Promise.resolve(element.children);
    }
    getCategoryItems() {
        const categories = new Map();
        for (const tool of this.tools) {
            const list = categories.get(tool.category) || [];
            list.push(tool);
            categories.set(tool.category, list);
        }
        const result = [];
        for (const [category, tools] of categories) {
            const children = tools.map((tool) => new TreeItem(tool, [], vscode.TreeItemCollapsibleState.None));
            const categoryItem = new TreeItem(null, children, vscode.TreeItemCollapsibleState.Expanded);
            categoryItem.iconPath = new vscode.ThemeIcon(this.getCategoryIcon(category));
            result.push(categoryItem);
        }
        return result.sort((a, b) => (a.children[0]?.tool?.category || '').localeCompare(b.children[0]?.tool?.category || ''));
    }
    getCategoryIcon(category) {
        const icons = {
            search: 'search',
            git: 'git-branch',
            build: 'tools',
            code: 'code',
            websocket: 'broadcast'
        };
        return icons[category] || 'circle-filled';
    }
    refresh() {
        this._onDidChangeTreeData.fire();
    }
}
exports.TreeViewManager = TreeViewManager;
//# sourceMappingURL=TreeViewManager.js.map