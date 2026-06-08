(function (global) {
  'use strict';

  // V3X Day4-E: Settings Controller
  // Responsible for loading/saving settings from localStorage,
  // binding DOM events for theme/fontSize/wordWrap/autoSave,
  // and setting up system theme listener.
  // Delegates DOM rendering to settings-view.js (HajimiSettingsView).

  var STORAGE_KEY = 'hajimi.settings';

  function ensureSettingsView() {
    if (global.HajimiSettingsView) {
      return global.HajimiSettingsView;
    }
    return null;
  }

  function loadSettings(app) {
    try {
      var raw = localStorage.getItem(STORAGE_KEY);
      if (raw) {
        var saved = JSON.parse(raw);
        app.settings = Object.assign({}, app.settings, saved);
      }
    } catch (e) {
      console.error('loadSettings error:', e);
    }
    applySettings(app);
    bindSettingsEvents(app);
  }

  function saveSettings(app) {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(app.settings));
    } catch (e) {
      console.error('saveSettings error:', e);
    }
  }

  function applySettings(app) {
    var view = ensureSettingsView();
    if (view) {
      view.applySettings(app);
    }
  }

  function applyTheme(theme) {
    var view = ensureSettingsView();
    if (view) {
      view.applyTheme(theme);
    }
  }

  function setupSystemThemeListener(app) {
    var mediaQuery = global.matchMedia('(prefers-color-scheme: light)');
    mediaQuery.addEventListener('change', function () {
      if (app.settings.theme === 'system') {
        applyTheme('system');
      }
    });
  }

  function bindSettingsEvents(app) {
    var themeSelect = document.getElementById('settingTheme');
    var fontSizeInput = document.getElementById('settingFontSize');
    var wordWrapInput = document.getElementById('settingWordWrap');
    var autoSaveSelect = document.getElementById('settingAutoSave');

    if (themeSelect) {
      themeSelect.addEventListener('change', function () {
        app.settings.theme = themeSelect.value;
        applyTheme(app.settings.theme);
        saveSettings(app);
      });
    }

    if (fontSizeInput) {
      fontSizeInput.addEventListener('change', function () {
        var val = parseInt(fontSizeInput.value);
        if (val >= 8 && val <= 32) {
          app.settings.fontSize = val;
          document.documentElement.style.setProperty('--editor-font-size', val + 'px');
          saveSettings(app);
        }
      });
    }

    if (wordWrapInput) {
      wordWrapInput.addEventListener('change', function () {
        app.settings.wordWrap = wordWrapInput.checked;
        saveSettings(app);
      });
    }

    if (autoSaveSelect) {
      autoSaveSelect.addEventListener('change', function () {
        app.settings.autoSave = autoSaveSelect.value;
        saveSettings(app);
      });
    }
  }

  var api = {
    storageKey: STORAGE_KEY,
    loadSettings: loadSettings,
    saveSettings: saveSettings,
    applySettings: applySettings,
    applyTheme: applyTheme,
    setupSystemThemeListener: setupSystemThemeListener,
    bindSettingsEvents: bindSettingsEvents,
  };

  global.HajimiSettingsController = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
