// ============================================================
// Hajimi Resource Dashboard View
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

  function renderMetrics(metrics) {
    setMetric('metricIterationTab', metrics?.iteration_count != null ? metrics.iteration_count : 'N/A');
    setMetric('metricBlackboardTab', metrics?.blackboard_size != null ? metrics.blackboard_size : 'N/A');
    setMetric('metricEditCountTab', metrics?.edit_count != null ? metrics.edit_count : '0');
  }

  function renderUnavailable() {
    setMetric('metricIterationTab', 'N/A');
    setMetric('metricBlackboardTab', 'N/A');
    setMetric('metricEditCountTab', 'N/A');
  }

  const api = {
    setMetric,
    renderMetrics,
    renderUnavailable,
  };

  global.HajimiDashboardView = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
