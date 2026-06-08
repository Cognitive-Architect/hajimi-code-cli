const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const settingsPath = path.join(repoRoot, 'src/interface/web/modules/settings-panel.js');
const settingsViewPath = path.join(repoRoot, 'src/interface/web/views/settings-view.js');
const settingsControllerPath = path.join(repoRoot, 'src/interface/web/controllers/settings-controller.js');

class FakeClassList {
  constructor() {
    this.values = new Set();
  }

  toggle(name, active) {
    if (active) this.values.add(name);
    else this.values.delete(name);
  }

  contains(name) {
    return this.values.has(name);
  }
}

class FakeElement {
  constructor(document, id = null, attrs = {}) {
    this.document = document;
    this.id = id;
    this.dataset = attrs.dataset || {};
    this.style = {};
    this.listeners = {};
    this.classList = new FakeClassList();
    this.value = '';
    this.checked = false;
  }

  addEventListener(type, handler) {
    this.listeners[type] = handler;
  }

  async click() {
    if (this.listeners.click) {
      return this.listeners.click();
    }
    return undefined;
  }
}

class FakeDocument {
  constructor() {
    this.byId = new Map();
    this.bySelector = new Map();
    this.documentElement = {
      style: {
        setProperty() {},
      },
      setAttribute(name, value) {
        this._attrs = this._attrs || {};
        this._attrs[name] = value;
      },
      getAttribute(name) {
        return (this._attrs || {})[name] || null;
      },
      _attrs: {},
    };
  }

  register(id, el) {
    this.byId.set(id, el);
    return el;
  }

  setSelector(selector, elements) {
    this.bySelector.set(selector, elements);
  }

  getElementById(id) {
    return this.byId.get(id) || null;
  }

  querySelectorAll(selector) {
    return this.bySelector.get(selector) || [];
  }
}

async function main() {
  const document = new FakeDocument();

  // --- Part 1: HajimiSettingsPanel (existing tests) ---
  const tabProviders = new FakeElement(document, null, { dataset: { tab: 'providers' } });
  const tabMcp = new FakeElement(document, null, { dataset: { tab: 'mcp' } });
  const panelProviders = new FakeElement(document, null, { dataset: { settingsPanel: 'providers' } });
  const panelMcp = new FakeElement(document, null, { dataset: { settingsPanel: 'mcp' } });

  document.setSelector('.settings-tab', [tabProviders, tabMcp]);
  document.setSelector('.settings-tab-panel', [panelProviders, panelMcp]);

  const context = { window: {}, document, console, localStorage: createFakeLocalStorage() };
  context.window = context;
  context.globalThis = context;
  context.matchMedia = function () {
    return { matches: false, addEventListener: function () {} };
  };
  vm.createContext(context);
  vm.runInContext(fs.readFileSync(settingsPath, 'utf8'), context, { filename: 'settings-panel.js' });

  let loadProvidersCount = 0;
  let loadAgentProvidersCount = 0;
  let loadMcpServersCount = 0;

  const app = {
    sidebarView: null,
    settings: { theme: 'dark', fontSize: 14, wordWrap: true, autoSave: 'off' },
    loadGitStatus() {},
    loadProviders() { loadProvidersCount += 1; },
    loadAgentProviders() { loadAgentProvidersCount += 1; },
    loadMcpServers() { loadMcpServersCount += 1; },
    loadCheckpoints() {},
    loadAuditLogs() {},
  };

  context.HajimiSettingsPanel.init(app);

  // Test switch tab
  await tabProviders.click();
  assert.strictEqual(loadProvidersCount, 1, 'clicking providers tab should load providers');
  assert.strictEqual(loadAgentProvidersCount, 1, 'clicking providers tab should load agent providers');
  assert.strictEqual(tabProviders.classList.contains('active'), true, 'providers tab should be active');
  assert.strictEqual(panelProviders.classList.contains('active'), true, 'providers panel should be active');
  assert.strictEqual(panelProviders.style.display, 'block', 'providers panel display style should be block');

  await tabMcp.click();
  assert.strictEqual(loadMcpServersCount, 1, 'clicking mcp tab should load mcp servers');
  assert.strictEqual(tabMcp.classList.contains('active'), true, 'mcp tab should be active');
  assert.strictEqual(panelMcp.classList.contains('active'), true, 'mcp panel should be active');
  assert.strictEqual(panelMcp.style.display, 'block', 'mcp panel display style should be block');
  assert.strictEqual(panelProviders.style.display, 'none', 'providers panel should be hidden');

  // Test showSidebar redirection
  context.HajimiSettingsPanel.showSidebar(app, 'models');
  assert.strictEqual(app.sidebarView, 'settings', 'models view should redirect to settings sidebar');
  assert.strictEqual(tabProviders.classList.contains('active'), true, 'redirect should activate providers tab');

  // --- Part 2: HajimiSettingsView + HajimiSettingsController delegation (Day4-E) ---
  vm.runInContext(fs.readFileSync(settingsViewPath, 'utf8'), context, { filename: 'settings-view.js' });
  vm.runInContext(fs.readFileSync(settingsControllerPath, 'utf8'), context, { filename: 'settings-controller.js' });

  assert.ok(context.HajimiSettingsView, 'HajimiSettingsView should be mounted on window');
  assert.ok(context.HajimiSettingsController, 'HajimiSettingsController should be mounted on window');
  assert.strictEqual(typeof context.HajimiSettingsController.loadSettings, 'function', 'loadSettings should be a function');
  assert.strictEqual(typeof context.HajimiSettingsController.saveSettings, 'function', 'saveSettings should be a function');
  assert.strictEqual(typeof context.HajimiSettingsController.applySettings, 'function', 'applySettings should be a function');
  assert.strictEqual(typeof context.HajimiSettingsController.applyTheme, 'function', 'applyTheme should be a function');
  assert.strictEqual(typeof context.HajimiSettingsController.setupSystemThemeListener, 'function', 'setupSystemThemeListener should be a function');
  assert.strictEqual(typeof context.HajimiSettingsController.bindSettingsEvents, 'function', 'bindSettingsEvents should be a function');

  // Test saveSettings + loadSettings round-trip
  const app2 = {
    settings: { theme: 'light', fontSize: 16, wordWrap: false, autoSave: 'afterDelay' },
  };
  context.HajimiSettingsController.saveSettings(app2);
  const stored = context.localStorage.getItem('hajimi.settings');
  assert.ok(stored, 'saveSettings should persist to localStorage');
  const parsed = JSON.parse(stored);
  assert.strictEqual(parsed.theme, 'light', 'saved theme should be light');
  assert.strictEqual(parsed.fontSize, 16, 'saved fontSize should be 16');

  const app3 = {
    settings: { theme: 'dark', fontSize: 14, wordWrap: true, autoSave: 'off' },
  };
  context.HajimiSettingsController.loadSettings(app3);
  assert.strictEqual(app3.settings.theme, 'light', 'loadSettings should restore theme from localStorage');
  assert.strictEqual(app3.settings.fontSize, 16, 'loadSettings should restore fontSize from localStorage');
  assert.strictEqual(app3.settings.wordWrap, false, 'loadSettings should restore wordWrap from localStorage');

  // Test applyTheme
  context.HajimiSettingsView.applyTheme('dark');
  assert.strictEqual(document.documentElement.getAttribute('data-theme'), 'dark', 'applyTheme dark should set data-theme to dark');

  context.HajimiSettingsView.applyTheme('light');
  assert.strictEqual(document.documentElement.getAttribute('data-theme'), 'light', 'applyTheme light should set data-theme to light');

  context.HajimiSettingsView.applyTheme('high-contrast');
  assert.strictEqual(document.documentElement.getAttribute('data-theme'), 'dark', 'applyTheme high-contrast should set data-theme to dark');

  // Test controller delegates applyTheme to view
  context.HajimiSettingsController.applyTheme('light');
  assert.strictEqual(document.documentElement.getAttribute('data-theme'), 'light', 'controller applyTheme should delegate to view');

  console.log('day19 settings panel smoke: PASS');
}

function createFakeLocalStorage() {
  const store = {};
  return {
    getItem(key) { return store[key] || null; },
    setItem(key, value) { store[key] = String(value); },
    removeItem(key) { delete store[key]; },
  };
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
