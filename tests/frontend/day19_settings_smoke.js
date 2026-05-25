const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const settingsPath = path.join(repoRoot, 'src/interface/web/modules/settings-panel.js');

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

  const tabProviders = new FakeElement(document, null, { dataset: { tab: 'providers' } });
  const tabMcp = new FakeElement(document, null, { dataset: { tab: 'mcp' } });
  const panelProviders = new FakeElement(document, null, { dataset: { settingsPanel: 'providers' } });
  const panelMcp = new FakeElement(document, null, { dataset: { settingsPanel: 'mcp' } });

  document.setSelector('.settings-tab', [tabProviders, tabMcp]);
  document.setSelector('.settings-tab-panel', [panelProviders, panelMcp]);

  const context = { window: {}, document, console };
  context.window = context;
  context.globalThis = context;
  vm.createContext(context);
  vm.runInContext(fs.readFileSync(settingsPath, 'utf8'), context, { filename: 'settings-panel.js' });

  let loadProvidersCount = 0;
  let loadAgentProvidersCount = 0;
  let loadMcpServersCount = 0;
  let sidebarView = null;

  const app = {
    sidebarView: null,
    loadGitStatus() {},
    loadProviders() {
      loadProvidersCount += 1;
    },
    loadAgentProviders() {
      loadAgentProvidersCount += 1;
    },
    loadMcpServers() {
      loadMcpServersCount += 1;
    },
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

  console.log('day19 settings panel smoke: PASS');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
