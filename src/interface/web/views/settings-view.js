(function (global) {
  'use strict';

  // V3X Day4-E: Settings View
  // Responsible for applying settings state to DOM elements (theme, font size, word wrap, auto save).
  // All DOM ids and class selectors remain unchanged from the original app.js implementation.

  var STORAGE_KEY = 'hajimi.settings';

  function applySettings(app) {
    var s = app.settings;

    // Theme
    var themeSelect = document.getElementById('settingTheme');
    if (themeSelect) themeSelect.value = s.theme;
    applyTheme(s.theme);

    // Font size
    var fontSizeInput = document.getElementById('settingFontSize');
    if (fontSizeInput) fontSizeInput.value = s.fontSize;
    document.documentElement.style.setProperty('--editor-font-size', s.fontSize + 'px');

    // Word wrap
    var wordWrapInput = document.getElementById('settingWordWrap');
    if (wordWrapInput) wordWrapInput.checked = s.wordWrap;

    // Auto save
    var autoSaveSelect = document.getElementById('settingAutoSave');
    if (autoSaveSelect) autoSaveSelect.value = s.autoSave;
  }

  function applyTheme(theme) {
    var root = document.documentElement;
    var effectiveTheme = theme;
    if (theme === 'system') {
      effectiveTheme = global.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
    } else if (theme === 'dark+' || theme === 'high-contrast') {
      effectiveTheme = 'dark';
    }
    root.setAttribute('data-theme', effectiveTheme);
  }

  var api = {
    storageKey: STORAGE_KEY,
    applySettings: applySettings,
    applyTheme: applyTheme,
  };

  global.HajimiSettingsView = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
