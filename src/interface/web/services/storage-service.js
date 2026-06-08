(function (global) {
  'use strict';

  // V3X Day4-F/G: Settings + Sessions Storage Service
  // Provides clean, isolated storage get/set methods for the settings and sessions domains.
  // All other storage domains (layout, stats, mcp, extensions) remain untouched and kept in-place.

  var SETTINGS_KEY = 'hajimi.settings';
  var SESSIONS_KEY = 'hajimi_chat_sessions';

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

  function getSessions() {
    try {
      var raw = localStorage.getItem(SESSIONS_KEY);
      return raw ? JSON.parse(raw) : null;
    } catch (e) {
      console.error('StorageService.getSessions error:', e);
      return null;
    }
  }

  function setSessions(sessions) {
    try {
      localStorage.setItem(SESSIONS_KEY, JSON.stringify(sessions));
    } catch (e) {
      console.error('StorageService.setSessions error:', e);
    }
  }

  var api = {
    getSettings: getSettings,
    setSettings: setSettings,
    getSessions: getSessions,
    setSessions: setSessions,
  };

  global.HajimiStorageService = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
