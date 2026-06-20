const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const appPath = path.join(repoRoot, 'src/interface/web/app.js');
const appStatePath = path.join(repoRoot, 'src/interface/web/app/app-state.js');
const bootstrapPath = path.join(repoRoot, 'src/interface/web/app/bootstrap.js');

function loadHarness() {
  const document = {
    addEventListener() {},
    getElementById() { return null; },
    querySelector() { return null; },
    querySelectorAll() { return []; },
  };
  const window = {
    HajimiCommandPaletteCatalog: {
      createCommandPaletteCatalog(app) {
        return [
          { id: 'palette', label: '命令面板', key: 'Ctrl+Shift+P', action: () => app.showCommandPalette() },
          { id: 'view.settings', label: '视图: 显示设置', key: 'Ctrl+Shift+S', action: () => app.showSidebar('settings') },
        ];
      },
    },
  };
  const context = {
    window,
    document,
    console,
    Promise,
    Error,
    Date,
    setTimeout,
    clearTimeout,
  };
  context.globalThis = context;
  vm.createContext(context);

  vm.runInContext(fs.readFileSync(appStatePath, 'utf8'), context, { filename: 'app-state.js' });
  vm.runInContext(fs.readFileSync(bootstrapPath, 'utf8'), context, { filename: 'bootstrap.js' });

  const appSource = fs.readFileSync(appPath, 'utf8');
  const appOnlySource = appSource.slice(0, appSource.indexOf('\n// D3-MINIMAL-FIX'));
  vm.runInContext(appOnlySource, context, { filename: 'app.js' });

  return context;
}

async function main() {
  const context = loadHarness();

  assert.ok(context.window.HajimiAppState, 'HajimiAppState should mount in Node/browser-compatible context');
  assert.strictEqual(typeof context.window.HajimiAppState.createDefaultAppState, 'function', 'createDefaultAppState should be exported');
  assert.strictEqual(typeof context.window.HajimiAppState.applyDefaultAppState, 'function', 'applyDefaultAppState should be exported');
  assert.ok(context.window.HajimiAppBootstrap, 'HajimiAppBootstrap should mount in Node/browser-compatible context');
  assert.strictEqual(typeof context.window.HajimiAppBootstrap.runAppBootstrap, 'function', 'runAppBootstrap should be exported');

  const app = context.window.app;
  assert.ok(app, 'app.js should still attach window.app');
  assert.strictEqual(app.tabs, undefined, 'app.js should not inline default state before applyDefaultAppState');

  context.window.HajimiAppState.applyDefaultAppState(app);
  assert.ok(Array.isArray(app.tabs) && app.tabs.length === 0, 'default tabs state should be injected');
  assert.strictEqual(app.sidebarView, 'ai-chat', 'default sidebarView should be preserved');
  assert.strictEqual(app.panelView, 'terminal', 'default panelView should be preserved');
  assert.strictEqual(app.settings.theme, 'dark', 'default settings should be preserved');
  assert.strictEqual(app.tokenStats.estimatedTokens, 0, 'default tokenStats should be preserved');
  assert.ok(app.extensions.some((ext) => ext.id === 'hajimi-agent'), 'default extensions should be preserved');

  const calls = [];
  const asyncCalls = [];
  const record = (name) => () => { calls.push(name); };
  [
    'setupDialogTrace',
    'setupActivityBar',
    'setupChat',
    'setupCommandPalette',
    'setupKeyboardShortcuts',
    'setupStatusBar',
    'setupTraceTabs',
    'setupSessionReplay',
    'setupFileTreeToolbar',
    'setupAgentTrace',
    'loadSettings',
    'setupSystemThemeListener',
    'loadLayoutSizes',
    'loadChatSessions',
    'loadProviders',
    'setupModelPicker',
    'setupProviderSettings',
    'loadProfiles',
    'setupProfileSettings',
    'setupAuditLog',
    'loadCumulativeFromBackend',
    'setupAgentProvider',
    'setupMcpSettings',
    'setupGovernance',
    'setupSessionBrowser',
    'setupResourceDashboard',
    'setupInspector',
    'setupSettingsTabs',
    'setupMoreMenus',
    'setupLiveShellControls',
    'setupReceiptPanel',
    'renderLiveShellState',
    'updateGitBranch',
    'showCommandPalette',
    'showSidebar',
  ].forEach((name) => {
    app[name] = record(name);
  });
  app.initWorkspace = () => {
    calls.push('initWorkspace');
    return Promise.resolve().then(() => asyncCalls.push('initWorkspace:resolved'));
  };
  app.loadFileTree = () => {
    calls.push('loadFileTree');
  };

  app.init();
  await Promise.resolve();
  await Promise.resolve();

  assert.deepStrictEqual(calls.slice(0, 10), [
    'setupDialogTrace',
    'setupActivityBar',
    'setupChat',
    'setupCommandPalette',
    'setupKeyboardShortcuts',
    'setupStatusBar',
    'setupTraceTabs',
    'setupSessionReplay',
    'setupFileTreeToolbar',
    'setupAgentTrace',
  ], 'bootstrap should preserve the leading init order');
  assert.ok(calls.includes('loadSettings'), 'bootstrap should still call loadSettings');
  assert.ok(calls.includes('setupSettingsTabs'), 'bootstrap should still call setupSettingsTabs');
  assert.ok(calls.includes('setupChat'), 'bootstrap should still call setupChat');
  assert.ok(calls.includes('setupCommandPalette'), 'bootstrap should still call setupCommandPalette');
  assert.ok(calls.includes('setupProviderSettings'), 'bootstrap should still call setupProviderSettings without changing provider semantics');
  assert.ok(asyncCalls.includes('initWorkspace:resolved'), 'initWorkspace promise should still resolve');
  assert.ok(calls.includes('loadFileTree'), 'bootstrap should still call loadFileTree after initWorkspace resolves');
  assert.strictEqual(app.commands.length, 2, 'bootstrap should prefer command palette catalog module when present');
  assert.strictEqual(app.commands[0].id, 'palette', 'command catalog module result should be preserved');

  const fallbackCatalog = context.window.HajimiAppBootstrap.createFallbackCommandCatalog(app);
  const fallbackIds = fallbackCatalog.map((command) => command.id);
  for (const id of ['file.open', 'view.settings', 'palette', 'chat.new', 'providers.refresh', 'agent.status', 'edit.history']) {
    assert.ok(fallbackIds.includes(id), `fallback command catalog should include ${id}`);
  }

  console.log('day33 app bootstrap smoke: PASS (state/bootstrap/init/catalog scenarios)');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
