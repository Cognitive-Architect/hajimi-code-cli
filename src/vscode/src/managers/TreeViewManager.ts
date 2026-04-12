import * as vscode from 'vscode';
import { Tool, getPhase2Tools, getWebSocketTools } from '../data/tools';

/**
 * TreeItem represents a node in the tree view
 * Supports category containers and tool leaf nodes
 */
export class TreeItem extends vscode.TreeItem {
  constructor(
    public readonly tool: Tool | null,
    public readonly children: TreeItem[],
    public readonly collapsibleState: vscode.TreeItemCollapsibleState
  ) {
    const label = tool 
      ? tool.name 
      : (children.length > 0 && children[0].tool?.category) 
        ? children[0].tool.category 
        : 'Tools';
    super(label, collapsibleState);
    this.tooltip = tool ? tool.description : `${children.length} tools`;
    this.description = tool ? tool.description : undefined;
    
    if (tool) {
      this.iconPath = new vscode.ThemeIcon(tool.icon);
      this.command = {
        command: 'hajimi.executeTool',
        title: tool.name,
        arguments: [tool]
      };
    }
  }
}

/**
 * TreeViewManager - TreeDataProvider for 56 tools
 * TOOL-DATA: Synced with ToolRegistry (ID-268)
 */
export class TreeViewManager implements vscode.TreeDataProvider<TreeItem> {
  tools: Tool[] = [...getPhase2Tools(), ...getWebSocketTools()];
  
  private _onDidChangeTreeData: vscode.EventEmitter<TreeItem | undefined | null | void> 
    = new vscode.EventEmitter<TreeItem | undefined | null | void>();
  
  readonly onDidChangeTreeData?: vscode.Event<TreeItem | undefined | null | void> 
    = this._onDidChangeTreeData.event;

  constructor(private context: vscode.ExtensionContext) {
    this.registerCommands();
  }

  private registerCommands(): void {
    this.context.subscriptions.push(
      vscode.commands.registerCommand('hajimi.executeTool', (tool: Tool) => {
        vscode.commands.executeCommand(tool.command);
        vscode.window.showInformationMessage(`Executing: ${tool.name}`);
      })
    );
  }

  getTreeItem(element: TreeItem): vscode.TreeItem {
    return element;
  }

  getChildren(element?: TreeItem): Thenable<TreeItem[]> {
    if (!element) {
      return Promise.resolve(this.getCategoryItems());
    }
    return Promise.resolve(element.children);
  }

  private getCategoryItems(): TreeItem[] {
    const categories: Map<string, Tool[]> = new Map();
    
    for (const tool of this.tools) {
      const list = categories.get(tool.category) || [];
      list.push(tool);
      categories.set(tool.category, list);
    }

    const result: TreeItem[] = [];
    for (const [category, tools] of categories) {
      const children: TreeItem[] = tools.map((tool) => 
        new TreeItem(tool, [], vscode.TreeItemCollapsibleState.None)
      );
      const categoryItem = new TreeItem(
        null,
        children,
        vscode.TreeItemCollapsibleState.Expanded
      );
      categoryItem.iconPath = new vscode.ThemeIcon(this.getCategoryIcon(category));
      result.push(categoryItem);
    }
    
    return result.sort((a, b) => 
      (a.children[0]?.tool?.category || '').localeCompare(
        b.children[0]?.tool?.category || ''
      )
    );
  }

  private getCategoryIcon(category: string): string {
    const icons: Record<string, string> = {
      search: 'search',
      git: 'git-branch',
      build: 'tools',
      code: 'code',
      websocket: 'broadcast'
    };
    return icons[category] || 'circle-filled';
  }

  refresh(): void {
    this._onDidChangeTreeData.fire();
  }
}
