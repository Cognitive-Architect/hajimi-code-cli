'use strict';

(function attachAppState(root) {
  function createDefaultSettings() {
    return {
      theme: 'dark',
      fontSize: 14,
      wordWrap: true,
      autoSave: 'off',
    };
  }

  function createDefaultExtensions() {
    return [
      { id: 'rust', name: 'Rust', desc: 'Rust 语言支持', version: '1.0.0', publisher: 'rust-lang', icon: 'R', iconColor: 'var(--fg-cyan)', installed: true },
      { id: 'hajimi-agent', name: 'Hajimi 智能体', desc: 'AI 助手集成', version: '0.3.0', publisher: 'hajimi', icon: 'H', iconColor: 'var(--fg-magenta)', installed: true },
      { id: 'toml', name: 'TOML', desc: 'TOML 语言支持', version: '0.1.0', publisher: '应用市场', icon: 'T', iconColor: 'var(--fg-green)', installed: false },
      { id: 'python', name: 'Python', desc: 'Python 语言支持', version: '1.2.0', publisher: 'microsoft', icon: 'P', iconColor: 'var(--fg-cyan)', installed: false },
      { id: 'go', name: 'Go', desc: 'Go 语言支持', version: '0.5.0', publisher: 'golang', icon: 'G', iconColor: 'var(--fg-cyan)', installed: false },
      { id: 'docker', name: 'Docker', desc: 'Dockerfile 和 Compose 支持', version: '1.0.0', publisher: 'microsoft', icon: 'D', iconColor: 'var(--fg-cyan)', installed: false },
    ];
  }

  function createDefaultAppState() {
    return {
      tabs: [],
      activeTab: null,
      sidebarView: 'ai-chat',
      panelView: 'terminal',
      panelCollapsed: false,
      isProcessing: false,
      commands: [],
      providerConfigs: [],
      activeProviderId: null,
      editingProviderId: null,
      currentWorkspace: null,
      fileTree: null,
      commandHistory: [],
      commandHistoryIndex: -1,
      slashPalette: null,
      settings: createDefaultSettings(),
      chatContextFiles: [],
      chatMessages: [],
      chatSessions: [],
      activeSessionId: null,
      autoCompact: true,
      isAutoCompacting: false,
      tokenStats: { promptTokens: 0, completionTokens: 0, estimatedTokens: 0 },
      cumulativeStats: { promptTokens: 0, completionTokens: 0, requestCount: 0 },
      showCumulative: false,
      mcpServers: [],
      traceEvents: [],
      tracePaused: false,
      traceChannel: null,
      extensions: createDefaultExtensions(),
      installedExtensions: [],
    };
  }

  function applyDefaultAppState(app) {
    if (!app || typeof app !== 'object') return app;
    return Object.assign(app, createDefaultAppState());
  }

  const api = {
    createDefaultSettings,
    createDefaultExtensions,
    createDefaultAppState,
    applyDefaultAppState,
  };

  root.HajimiAppState = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
