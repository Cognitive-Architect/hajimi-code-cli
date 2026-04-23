import type { ToolDef } from '../types/webview';

export const TOOLS: ToolDef[] = [
  { id: 'openSidebar', name: 'Open Sidebar', icon: 'layout-sidebar-left', category: 'core' },
  { id: 'searchCode', name: 'Search Code', icon: 'search', category: 'core' },
  { id: 'toggleTerminal', name: 'Toggle Terminal', icon: 'terminal', category: 'core' },
  { id: 'test.run', name: 'Run Tests', icon: 'beaker', category: 'mcp' },
  { id: 'build', name: 'Build', icon: 'tools', category: 'mcp' },
  { id: 'git.commit', name: 'Git Commit', icon: 'git-commit', category: 'mcp' },
  { id: 'adr.open', name: 'Open ADR', icon: 'book', category: 'mcp' },
];
