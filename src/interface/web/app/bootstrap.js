'use strict';

(function attachAppBootstrap(root) {
  function createFallbackCommandCatalog(app) {
    return [
      { id: 'file.open', label: '文件: 打开文件', key: 'Ctrl+O', action: () => app.openFilePrompt() },
      { id: 'file.openFolder', label: '文件: 打开文件夹', key: 'Ctrl+K Ctrl+O', action: () => app.openFolder() },
      { id: 'view.chat-sessions', label: '视图: 显示会话列表', key: 'Ctrl+Shift+C', action: () => app.showSidebar('chat-sessions') },
      { id: 'view.explorer', label: '视图: 显示文件', key: 'Ctrl+Shift+E', action: () => app.showSidebar('explorer') },
      { id: 'view.providers', label: '视图: 显示模型设置', key: 'Ctrl+Shift+M', action: () => { app.showSidebar('settings'); app.switchSettingsTab('providers'); } },
      { id: 'view.governance', label: '视图: 显示治理控制', key: 'Ctrl+Shift+G', action: () => { app.showSidebar('settings'); app.switchSettingsTab('governance'); } },
      { id: 'view.audit', label: '视图: 显示审计日志', key: 'Ctrl+Shift+Y', action: () => { app.showSidebar('settings'); app.switchSettingsTab('audit'); } },
      { id: 'view.settings', label: '视图: 显示设置', key: 'Ctrl+Shift+S', action: () => app.showSidebar('settings') },
      { id: 'palette', label: '命令面板', key: 'Ctrl+Shift+P', action: () => app.showCommandPalette() },
      { id: 'chat.new', label: '对话: 新会话', key: '', action: () => app.newChatSession() },
      { id: 'git.commit', label: 'Git: 提交', key: '', action: () => app.gitCommit() },
      { id: 'providers.refresh', label: '模型: 刷新提供商列表', key: '', action: () => app.loadProviders() },
      { id: 'audit.log', label: '系统: 刷新审计日志', key: '', action: () => app.loadAuditLogs() },
      { id: 'system.resources', label: '系统: 打开资源监控', key: '', action: () => { app.showSidebar('settings'); app.switchSettingsTab('audit'); } },
      { id: 'session.export', label: '会话: 导出所有检查点', key: '', action: () => app.exportAllCheckpoints() },
      { id: 'trace.clear', label: 'Trace: 清空', key: '', action: () => app.clearTraceCards() },
      { id: 'trace.pause', label: 'Trace: 暂停/继续', key: '', action: () => app.toggleTracePause() },
      { id: 'agent.refactor', label: '@agent refactor — 重构选中代码', key: '', action: () => app.runAgentCommand('@agent refactor selection') },
      { id: 'agent.review-pr', label: '@agent review-pr — 审查 PR', key: '', action: () => app.runAgentCommand('@agent review-pr') },
      { id: 'agent.continue', label: '@agent continue-background — 后台继续', key: '', action: () => app.runAgentCommand('@agent continue-background') },
      { id: 'agent.pause', label: '@agent pause — 暂停 Agent', key: '', action: () => app.runAgentCommand('@agent pause') },
      { id: 'agent.status', label: '@agent status — Agent 状态', key: '', action: () => app.runAgentCommand('@agent status') },
      { id: 'edit.history', label: '编辑: 显示编辑历史', key: '', action: () => app.showEditHistoryTab() },
    ];
  }

  function buildCommandCatalog(app) {
    const commandCatalogFactory = root.HajimiCommandPaletteCatalog?.createCommandPaletteCatalog;
    return commandCatalogFactory ? commandCatalogFactory(app) : createFallbackCommandCatalog(app);
  }

  function runAppBootstrap(app) {
    app.setupDialogTrace();
    app.setupActivityBar();
    app.setupChat();
    app.setupCommandPalette();
    app.setupKeyboardShortcuts();
    app.setupStatusBar();
    app.setupTraceTabs();
    app.setupSessionReplay();
    app.setupFileTreeToolbar();
    app.setupAgentTrace();
    app.loadSettings();
    app.setupSystemThemeListener();
    app.loadLayoutSizes();
    app.initWorkspace().then(() => {
      app.loadFileTree();
    });
    app.loadChatSessions();
    app.loadProviders();
    app.setupModelPicker();
    app.setupProviderSettings();
    app.loadProfiles();
    app.setupProfileSettings();
    app.setupAuditLog();
    app.loadCumulativeFromBackend();
    app.setupAgentProvider();
    app.setupMcpSettings();
    app.setupGovernance();
    app.setupSessionBrowser();
    app.setupResourceDashboard();
    app.setupInspector();
    app.setupSettingsTabs();
    app.setupMoreMenus();
    app.setupLiveShellControls();
    app.setupReceiptPanel();
    app.renderLiveShellState('就绪');
    app.updateGitBranch();
    app.commands = buildCommandCatalog(app);
  }

  const api = {
    createFallbackCommandCatalog,
    buildCommandCatalog,
    runAppBootstrap,
  };

  root.HajimiAppBootstrap = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
