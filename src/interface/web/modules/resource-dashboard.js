// ============================================================
// Hajimi Resource Dashboard Readonly Module
// ============================================================

(function (global) {
  'use strict';

  function getController() {
    return global.HajimiDashboardController || null;
  }

  function setupResourceDashboard(app) {
    return getController()?.setupResourceDashboard?.(app);
  }

  async function updateMetrics(app) {
    return getController()?.updateMetrics?.(app);
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
