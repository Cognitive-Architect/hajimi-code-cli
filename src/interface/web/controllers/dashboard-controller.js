// ============================================================
// Hajimi Resource Dashboard Controller
// ============================================================

(function (global) {
  'use strict';

  function getView() {
    return global.HajimiDashboardView;
  }

  function setupResourceDashboard(app) {
    app.updateMetrics();
    app.metricsInterval = global.setInterval(() => app.updateMetrics(), 3000);
  }

  async function updateMetrics(app) {
    const view = getView();
    if (!app?.isTauriAvailable?.()) {
      view?.renderUnavailable?.();
      return;
    }

    try {
      const metrics = await app.invokeTauri('get_resource_metrics');
      view?.renderMetrics?.(metrics);
    } catch (error) {}
  }

  const api = {
    setupResourceDashboard,
    updateMetrics,
  };

  global.HajimiDashboardController = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
