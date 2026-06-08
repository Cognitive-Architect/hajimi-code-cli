(function (global) {
  'use strict';

  // V3X Day4-F: Settings Storage Service (Settings Only)
  // Provides clean, isolated storage get/set methods for the settings domain.
  // All other storage domains (sessions, layout, etc.) remain untouched and kept in-place.

  var SETTINGS_KEY = 'hajimi.settings';

  function getSettings() {
    try {
      var raw = localStorage.getItem(SETTINGS_KEY);
      return raw ? JSON.parse(raw) : null;
    } catch (e) {
      console.error('StorageService.getSettings error:', e);
      return null;
    }
  }

  function setSettings(settings) {
    try {
      localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
    } catch (e) {
      console.error('StorageService.setSettings error:', e);
    }
  }

  var api = {
    getSettings: getSettings,
    setSettings: setSettings,
  };

  global.HajimiStorageService = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
