import * as vscode from 'vscode';
import { Tool } from '../data/tools';
import { LspClient } from '../clients/LspClient';
/**
 * TreeItem represents a node in the tree view
 * Supports category containers and tool leaf nodes
 */
export declare class TreeItem extends vscode.TreeItem {
    readonly tool: Tool | null;
    readonly children: TreeItem[];
    readonly collapsibleState: vscode.TreeItemCollapsibleState;
    constructor(tool: Tool | null, children: TreeItem[], collapsibleState: vscode.TreeItemCollapsibleState);
}
/**
 * TreeViewManager - TreeDataProvider for 56 tools
 * TOOL-DATA: Synced with ToolRegistry (ID-268)
 */
export declare class TreeViewManager implements vscode.TreeDataProvider<TreeItem> {
    private context;
    tools: Tool[];
    private _onDidChangeTreeData;
    readonly onDidChangeTreeData?: vscode.Event<TreeItem | undefined | null | void>;
    constructor(context: vscode.ExtensionContext, _lspClient: LspClient);
    private registerCommands;
    getTreeItem(element: TreeItem): vscode.TreeItem;
    getChildren(element?: TreeItem): Thenable<TreeItem[]>;
    private getCategoryItems;
    private getCategoryIcon;
    refresh(): void;
}
//# sourceMappingURL=TreeViewManager.d.ts.map