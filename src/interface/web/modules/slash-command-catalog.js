(function (global) {
  'use strict';

  function createSlashCommandCatalog() {
    return [
      { id: 'tools', trigger: '/tools', title: 'List tools', description: 'Show available backend tools', category: 'tool', riskLevel: 'low', enabled: true, executeMode: 'direct', keywords: ['list', 'backend', '工具', '列出'] },
      { id: 'providers', trigger: '/providers', title: 'List providers', description: 'Show configured model providers', category: 'model', riskLevel: 'low', enabled: true, executeMode: 'direct', keywords: ['models', '模型', '提供商'] },
      { id: 'tool', trigger: '/tool', title: 'Run tool', description: 'Fill /tool <name> {json_args}', category: 'tool', riskLevel: 'high', enabled: true, executeMode: 'fill', insertText: '/tool ', keywords: ['运行', '执行', '工具'] },
      { id: 'chat', trigger: '/chat', title: 'Chat with provider', description: 'Fill /chat <provider> <prompt>', category: 'model', riskLevel: 'medium', enabled: true, executeMode: 'fill', insertText: '/chat ', keywords: ['聊天', '对话'] },
      { id: 'mcp', trigger: '/mcp', title: 'MCP command', description: 'Fill /mcp list/init/invoke', category: 'mcp', riskLevel: 'medium', enabled: true, executeMode: 'fill', insertText: '/mcp ', keywords: ['mcp'] },
      { id: 'search', trigger: '/search', title: 'Search workspace', description: 'Fill /search <pattern>', category: 'search', riskLevel: 'low', enabled: true, executeMode: 'fill', insertText: '/search ', keywords: ['搜索', '查找'] },
      { id: 'git', trigger: '/git', title: 'Git helper', description: 'Fill /git status/diff/commit', category: 'git', riskLevel: 'medium', enabled: true, executeMode: 'fill', insertText: '/git ', keywords: ['git', '版本控制'] },
      { id: 'extensions', trigger: '/extensions', title: 'List extensions', description: 'Show available extensions', category: 'extension', riskLevel: 'low', enabled: true, executeMode: 'direct', keywords: ['plugins', '扩展', '插件'] },
      { id: 'compact', trigger: '/compact', title: 'Compact context', description: 'Fill compact command for explicit submit', category: 'context', riskLevel: 'medium', enabled: true, executeMode: 'fill', keywords: ['压缩', '精简', '上下文'] },
      { id: 'agent', trigger: '/agent', title: 'Run agent task', description: 'Fill /agent <goal>', category: 'agent', riskLevel: 'high', enabled: true, executeMode: 'fill', insertText: '/agent ', keywords: ['代理', '智能体', 'agent', '任务'] },
    ];
  }

  global.HajimiSlashCommandCatalog = {
    createSlashCommandCatalog,
  };

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = { createSlashCommandCatalog };
  }
})(typeof window !== 'undefined' ? window : globalThis);
