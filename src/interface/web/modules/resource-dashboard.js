// ============================================================
// Hajimi Resource Dashboard Readonly Module
// ============================================================

(function (global) {
  'use strict';

  function getDocument() {
    return global.document || (typeof document !== 'undefined' ? document : null);
  }

  function setMetric(id, value) {
    const el = getDocument()?.getElementById(id);
    if (el) el.textContent = value;
  }

  const compatView = {
    renderUnavailable() {
      setMetric('metricIterationTab', 'N/A');
      setMetric('metricBlackboardTab', 'N/A');
      setMetric('metricEditCountTab', 'N/A');
    },
    renderMetrics(metrics) {
      setMetric('metricIterationTab', metrics?.iteration_count != null ? metrics.iteration_count : 'N/A');
      setMetric('metricBlackboardTab', metrics?.blackboard_size != null ? metrics.blackboard_size : 'N/A');
      setMetric('metricEditCountTab', metrics?.edit_count != null ? metrics.edit_count : '0');
    },
  };

  const compatController = {
    setupResourceDashboard(app) {
      app.updateMetrics();
      app.metricsInterval = global.setInterval(() => app.updateMetrics(), 3000);
    },
    async updateMetrics(app) {
      const view = global.HajimiDashboardView || compatView;
      if (!app?.isTauriAvailable?.()) {
        view.renderUnavailable();
        return;
      }
      try {
        const metrics = await app.invokeTauri('get_resource_metrics');
        view.renderMetrics(metrics);
      } catch (error) {}
    },
  };

  function getController() {
    return global.HajimiDashboardController || compatController;
  }

  function setupResourceDashboard(app) {
    return getController().setupResourceDashboard(app);
  }

  async function updateMetrics(app) {
    return getController().updateMetrics(app);
  }

  global.HajimiResourceDashboard = {
    setupResourceDashboard,
    updateMetrics,
  };

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
      setupResourceDashboard,
      updateMetrics,
    };
  }
})(typeof window !== 'undefined' ? window : globalThis);
