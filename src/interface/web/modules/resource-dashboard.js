// ============================================================
// Hajimi Resource Dashboard Readonly Module
// ============================================================

(function (global) {
  function getDocument() {
    return global.document || (typeof document !== 'undefined' ? document : null);
  }

  function setMetric(id, value) {
    const el = getDocument()?.getElementById(id);
    if (el) el.textContent = value;
  }

  function setupResourceDashboard(app) {
    app.updateMetrics();
    app.metricsInterval = global.setInterval(() => app.updateMetrics(), 3000);
  }

  async function updateMetrics(app) {
    if (!app?.isTauriAvailable?.()) {
      setMetric('metricIterationTab', 'N/A');
      setMetric('metricBlackboardTab', 'N/A');
      setMetric('metricEditCountTab', 'N/A');
      return;
    }
    try {
      const metrics = await app.invokeTauri('get_resource_metrics');
      setMetric('metricIterationTab', metrics.iteration_count != null ? metrics.iteration_count : 'N/A');
      setMetric('metricBlackboardTab', metrics.blackboard_size != null ? metrics.blackboard_size : 'N/A');
      setMetric('metricEditCountTab', metrics.edit_count != null ? metrics.edit_count : '0');
    } catch (error) {}
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
