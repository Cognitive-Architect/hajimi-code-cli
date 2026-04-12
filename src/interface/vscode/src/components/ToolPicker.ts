import * as vscode from 'vscode';
import { Tool, getPhase2Tools, getWebSocketTools } from '../data/tools';

interface ToolItem extends vscode.QuickPickItem {
  toolId: string;
  command: string;
}

export class ToolPicker {
  private static instance: ToolPicker | undefined;
  private recentlyUsed: string[] = [];
  private allTools: Tool[] = [];
  private outputChannel: vscode.OutputChannel;
  private static readonly MAX_RECENT = 10;
  private static readonly TOOL_PREFIX = 'hajimi.tools.';

  private constructor() {
    this.outputChannel = vscode.window.createOutputChannel('Hajimi ToolPicker');
    this.allTools = [...getPhase2Tools(), ...getWebSocketTools()];
  }

  static getInstance(): ToolPicker {
    if (!ToolPicker.instance) ToolPicker.instance = new ToolPicker();
    return ToolPicker.instance;
  }

  private fuzzySearch(query: string, tool: Tool): boolean {
    const q = query.toLowerCase().replace(/\s+/g, '');
    const text = (tool.name + tool.description + tool.category).toLowerCase();
    let i = 0;
    for (const c of text) if (i < q.length && c === q[i]) i++;
    return i === q.length;
  }

  private categoryFilter(tools: Tool[], category: string | undefined): Tool[] {
    if (!category || category === 'all') return tools;
    return tools.filter(t => t.category === category);
  }

  private toItem(tool: Tool): ToolItem {
    return {
      label: `$(${tool.icon}) ${tool.name}`,
      description: tool.category,
      detail: tool.description,
      toolId: `${ToolPicker.TOOL_PREFIX}${tool.id}`,
      command: tool.command
    };
  }

  private getItems(category?: string): ToolItem[] {
    const filtered = this.categoryFilter(this.allTools, category);
    return filtered.sort((a, b) => {
      const ia = this.recentlyUsed.indexOf(a.id), ib = this.recentlyUsed.indexOf(b.id);
      if (ia !== -1 && ib !== -1) return ia - ib;
      if (ia !== -1) return -1;
      if (ib !== -1) return 1;
      return a.name.localeCompare(b.name);
    }).map(t => this.toItem(t));
  }

  private addRecent(toolId: string): void {
    const cleanId = toolId.replace(ToolPicker.TOOL_PREFIX, '');
    this.recentlyUsed = [cleanId, ...this.recentlyUsed.filter(id => id !== cleanId)].slice(0, ToolPicker.MAX_RECENT);
  }

  async show(): Promise<string | undefined> {
    const qp = vscode.window.createQuickPick<ToolItem>();
    qp.placeholder = 'Select a tool (type to search)...';
    qp.items = this.getItems();
    return new Promise(resolve => {
      qp.onDidChangeValue(v => {
        qp.items = v ? this.allTools.filter(t => this.fuzzySearch(v, t)).map(t => this.toItem(t)) : this.getItems();
      });
      qp.onDidAccept(() => {
        const s = qp.selectedItems[0];
        if (s) { this.addRecent(s.toolId); this.outputChannel.appendLine(`Selected: ${s.toolId}`); }
        qp.dispose(); resolve(s?.toolId);
      });
      qp.onDidHide(() => { qp.dispose(); resolve(undefined); });
      qp.show();
    });
  }

  showByCategory(category: string): Promise<string | undefined> {
    const qp = vscode.window.createQuickPick<ToolItem>();
    qp.placeholder = `Select a ${category} tool...`;
    qp.items = this.getItems(category);
    return new Promise(resolve => {
      qp.onDidAccept(() => {
        const s = qp.selectedItems[0];
        if (s) this.addRecent(s.toolId);
        qp.dispose(); resolve(s?.toolId);
      });
      qp.onDidHide(() => { qp.dispose(); resolve(undefined); });
      qp.show();
    });
  }
}
