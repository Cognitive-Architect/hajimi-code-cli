// ============================================================
// Hajimi Inspector Shell Controller
// ============================================================

(function (global) {
  'use strict';

  function getView() {
    return global.HajimiInspectorView;
  }

  function init(app) {
    const view = getView();
    view?.bindTabs?.((tabId) => app.showInspectorTab(tabId));
    view?.bindClose?.(() => view.setVisible(false));
  }

  function showInspectorTab(app, tabId) {
    getView()?.setActiveTab?.(tabId, (activeTabId) => {
      if (activeTabId === 'diff-preview') app.safeRenderInspectorDiffPreview();
      if (activeTabId === 'agent-trace') app.safeRenderTraceInspector();
    });
  }

  function withInspectorGuard(app, label, renderFn) {
    try {
      renderFn();
    } catch (e) {
      console.warn(`Inspector render skipped (${label}):`, e);
    }
  }

  function safeUpdateTaskDetails(app, statusText) {
    app.withInspectorGuard('task details', () => app.updateTaskDetails(statusText));
  }

  function safeRenderContextFiles(app) {
    app.withInspectorGuard('context files', () => app.renderContextFiles());
  }

  function safeRenderModelInfo(app) {
    app.withInspectorGuard('model info', () => app.renderModelInfo());
  }

  function safeRenderInspectorDiffPreview(app) {
    app.withInspectorGuard('diff preview', () => app.renderInspectorDiffPreview());
  }

  function safeRenderTraceInspector(app) {
    app.withInspectorGuard('trace summary', () => app.renderTraceInspector());
  }

  function openDiffPreview(app, file = null) {
    if (file) app.currentDiffFile = file;
    getView()?.setVisible?.(true);
    app.showInspectorTab('diff-preview');
  }

  function updateTaskDetails(app, statusText) {
    const text = statusText || (app.isProcessing ? '处理中...' : '就绪');
    app.renderInspectorTaskStatus(text);
    app.renderChatShellStatus(text);
    app.renderInspectorSessionStats();
    if (typeof app.renderInspectorOperationSummary === 'function') {
      app.renderInspectorOperationSummary();
    }
  }

  function renderTaskSteps(app) {
    app.renderInspectorSessionStats();
  }

  const api = {
    init,
    showInspectorTab,
    withInspectorGuard,
    safeUpdateTaskDetails,
    safeRenderContextFiles,
    safeRenderModelInfo,
    safeRenderInspectorDiffPreview,
    safeRenderTraceInspector,
    openDiffPreview,
    updateTaskDetails,
    renderTaskSteps,
  };

  global.HajimiInspectorController = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
