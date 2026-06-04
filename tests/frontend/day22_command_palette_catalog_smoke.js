const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const appPath = path.join(repoRoot, 'src/interface/web/app.js');
const catalogPath = path.join(repoRoot, 'src/interface/web/modules/command-palette-catalog.js');

const expectedIds = [
  'file.open',
  'file.openFolder',
  'view.chat-sessions',
  'view.explorer',
  'view.providers',
  'view.governance',
  'view.audit',
  'view.settings',
  'palette',
  'chat.new',
  'git.commit',
  'providers.refresh',
  'audit.log',
  'system.resources',
  'session.export',
  'trace.clear',
  'trace.pause',
  'agent.refactor',
  'agent.review-pr',
  'agent.continue',
  'agent.pause',
  'agent.status',
  'edit.history',
];

function loadCatalogModule() {
  const context = {
    window: {},
    module: { exports: {} },
    exports: {},
    console,
  };
  context.globalThis = context;
  vm.createContext(context);
  vm.runInContext(fs.readFileSync(catalogPath, 'utf8'), context, { filename: 'command-palette-catalog.js' });
  assert.strictEqual(
    context.window.HajimiCommandPaletteCatalog?.createCommandPaletteCatalog,
    context.module.exports.createCommandPaletteCatalog,
    'browser global and module.exports should expose the same factory'
  );
  return context.module.exports;
}

function createFakeApp() {
  const calls = [];
  const app = {
    calls,
    openFilePrompt: () => calls.push(['openFilePrompt']),
    openFolder: () => calls.push(['openFolder']),
    showSidebar: (view) => calls.push(['showSidebar', view]),
    switchSettingsTab: (tab) => calls.push(['switchSettingsTab', tab]),
    showCommandPalette: () => calls.push(['showCommandPalette']),
    newChatSession: () => calls.push(['newChatSession']),
    gitCommit: () => calls.push(['gitCommit']),
    loadProviders: () => calls.push(['loadProviders']),
    loadAuditLogs: () => calls.push(['loadAuditLogs']),
    exportAllCheckpoints: () => calls.push(['exportAllCheckpoints']),
    clearTraceCards: () => calls.push(['clearTraceCards']),
    toggleTracePause: () => calls.push(['toggleTracePause']),
    runAgentCommand: (command) => calls.push(['runAgentCommand', command]),
    showEditHistoryTab: () => calls.push(['showEditHistoryTab']),
  };
  return app;
}

function getCommand(commands, id) {
  const command = commands.find(item => item.id === id);
  assert.ok(command, `expected command id ${id}`);
  return command;
}

function loadAppWithoutCatalog() {
  const window = {};
  const document = {
    addEventListener() {},
    getElementById() {
      return null;
    },
    querySelector() {
      return null;
    },
    querySelectorAll() {
      return [];
    },
  };
  const context = {
    window,
    document,
    console,
    setTimeout,
    clearTimeout,
    Promise,
  };
  context.globalThis = context;
  vm.createContext(context);

  const source = fs.readFileSync(appPath, 'utf8');
  const appOnlySource = source.slice(0, source.indexOf('\n// D3-MINIMAL-FIX'));
  vm.runInContext(appOnlySource, context, { filename: 'app.js' });

  const app = context.window.app;
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
  ].forEach((name) => {
    app[name] = () => {};
  });
  app.initWorkspace = () => Promise.resolve();
  app.loadFileTree = () => {};

  return app;
}

async function main() {
  const { createCommandPaletteCatalog } = loadCatalogModule();
  assert.strictEqual(typeof createCommandPaletteCatalog, 'function', 'catalog factory should be exported');

  const app = createFakeApp();
  const commands = createCommandPaletteCatalog(app);
  assert.ok(Array.isArray(commands), 'catalog should return an array');
  assert.strictEqual(
    JSON.stringify(commands.map(command => command.id)),
    JSON.stringify(expectedIds),
    'command order should match the original inline catalog'
  );

  for (const command of commands) {
    assert.strictEqual(typeof command.label, 'string', `${command.id} should keep label`);
    assert.ok(command.label.length > 0, `${command.id} should keep non-empty label`);
    assert.strictEqual(typeof command.key, 'string', `${command.id} should keep key field`);
    assert.strictEqual(typeof command.action, 'function', `${command.id} action should be a function`);
  }

  getCommand(commands, 'palette').action();
  assert.deepStrictEqual(app.calls.pop(), ['showCommandPalette'], 'palette should call showCommandPalette');

  getCommand(commands, 'view.settings').action();
  assert.deepStrictEqual(app.calls.pop(), ['showSidebar', 'settings'], 'view.settings should call showSidebar(settings)');

  getCommand(commands, 'providers.refresh').action();
  assert.deepStrictEqual(app.calls.pop(), ['loadProviders'], 'providers.refresh should call loadProviders');

  getCommand(commands, 'audit.log').action();
  assert.deepStrictEqual(app.calls.pop(), ['loadAuditLogs'], 'audit.log should call loadAuditLogs');

  const fallbackApp = loadAppWithoutCatalog();
  assert.doesNotThrow(() => fallbackApp.init(), 'app init should keep inline fallback when catalog module is missing');
  assert.strictEqual(
    JSON.stringify(fallbackApp.commands.map(command => command.id)),
    JSON.stringify(expectedIds),
    'inline fallback should preserve command ids and order'
  );

  console.log('day22 command palette catalog smoke: PASS (7 scenarios)');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
