'use strict';

(function (global) {
  function normalizeProviderConfig(config) {
    const cfg = config || {};
    return {
      id: cfg.id || '',
      name: cfg.name || cfg.id || 'Unnamed Provider',
      model: cfg.model || '',
      providerType: cfg.providerType || cfg.provider_type || 'openai-compatible',
      baseUrl: cfg.baseUrl || cfg.base_url || '',
      hasSavedKey: Boolean(cfg.hasSavedKey || cfg.hasApiKey || cfg.has_api_key),
    };
  }

  function normalizeProviderConfigs(configs) {
    if (!Array.isArray(configs)) return [];
    return configs.map(normalizeProviderConfig);
  }

  function formatProviderMeta(config) {
    const cfg = normalizeProviderConfig(config);
    return [
      cfg.model,
      cfg.baseUrl,
      cfg.hasSavedKey ? 'API Key 已保存' : '未保存 API Key',
    ].filter(Boolean).join(' · ');
  }

  function getProviderSource(workspacePath) {
    if (workspacePath) {
      return { type: 'workspace', label: 'workspace', title: workspacePath };
    }
    return { type: 'global', label: 'global', title: '' };
  }

  const api = {
    normalizeProviderConfig,
    normalizeProviderConfigs,
    formatProviderMeta,
    getProviderSource,
  };

  global.HajimiProviderService = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
