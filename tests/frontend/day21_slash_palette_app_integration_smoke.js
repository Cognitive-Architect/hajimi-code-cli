const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const appPath = path.join(repoRoot, 'src/interface/web/app.js');
const catalogPath = path.join(repoRoot, 'src/interface/web/modules/slash-command-catalog.js');

class ClassList {
  constructor(el) {
    this.el = el;
  }

  _items() {
    return new Set((this.el.className || '').split(/\s+/).filter(Boolean));
  }

  add(...names) {
    const items = this._items();
    names.forEach(name => items.add(name));
    this.el.className = Array.from(items).join(' ');
  }

  remove(...names) {
    const items = this._items();
    names.forEach(name => items.delete(name));
    this.el.className = Array.from(items).join(' ');
  }

  contains(name) {
    return this._items().has(name);
  }
}

class Element {
  constructor(id = '', tagName = 'div') {
    this.id = id;
    this.tagName = tagName.toUpperCase();
    this.listeners = {};
    this.style = {};
    this.className = '';
    this.value = '';
    this.selectionStart = 0;
    this.scrollHeight = 20;
    this.disabled = false;
    this.focused = false;
    this.classList = new ClassList(this);
  }

  addEventListener(type, handler) {
    this.listeners[type] = this.listeners[type] || [];
    this.listeners[type].push(handler);
  }

  dispatchEvent(event) {
    event.target = event.target || this;
    for (const handler of this.listeners[event.type] || []) {
      handler(event);
    }
    return !event.defaultPrevented;
  }

  focus() {
    this.focused = true;
  }
}

function createEvent(type, overrides = {}) {
  return {
    type,
    key: overrides.key,
    shiftKey: Boolean(overrides.shiftKey),
    bubbles: Boolean(overrides.bubbles),
    defaultPrevented: false,
    preventDefault() {
      this.defaultPrevented = true;
    },
  };
}

function createDocument() {
  const elements = new Map();
  const ids = [
    'aiChatInput',
    'aiChatSendBtn',
    'slashPalette',
    'modelSelectBtn',
    'addContextBtn',
    'clearContextBtn',
    'editModeBtn',
    'newChatBtn',
    'newSessionBtn',
  ];

  ids.forEach(id => {
    elements.set(id, new Element(id, id === 'aiChatInput' ? 'textarea' : 'button'));
  });

  return {
    elements,
    getElementById(id) {
      return elements.get(id) || null;
    },
  };
}

function loadCatalogModule() {
  delete require.cache[require.resolve(catalogPath)];
  return require(catalogPath);
}

function loadAppHarness(options = {}) {
  const document = createDocument();
  const catalogModule = options.catalogModule === false ? null : loadCatalogModule();
  const paletteCalls = [];

  const window = {
    __HAJIMI_FLAGS__: options.flags || {},
  };

  if (catalogModule) {
    window.HajimiSlashCommandCatalog = catalogModule;
  }

  if (options.withPalette !== false) {
    window.HajimiSlashPalette = {
      createSlashPalette(createOptions) {
        paletteCalls.push(createOptions);
        return {
          handleInputCalls: 0,
          handleInput() {
            this.handleInputCalls += 1;
          },
          handleKeyDown() {
            return false;
          },
          isOpen() {
            return false;
          },
          close() {},
        };
      },
    };
  }

  const context = {
    window,
    document,
    console,
    Event: createEvent,
    Math,
  };
  context.globalThis = context;
  vm.createContext(context);

  const source = fs.readFileSync(appPath, 'utf8');
  const appOnlySource = source.slice(0, source.indexOf('\n// D3-MINIMAL-FIX'));
  vm.runInContext(appOnlySource, context, { filename: 'app.js' });

  const app = context.window.app;
  let sendCount = 0;
  app.updateTokenDisplay = () => {};
  app.openModelPicker = () => {};
  app.showSidebar = () => {};
  app.addChatMessage = () => {};
  app.clearChatContext = () => {};
  app.newChatSession = () => {};
  app.sendChatMessage = () => {
    sendCount += 1;
  };

  return {
    app,
    document,
    paletteCalls,
    get sendCount() {
      return sendCount;
    },
  };
}

function triggerById(commands, id) {
  const item = commands.find(command => command.id === id);
  assert.ok(item, `expected slash command id ${id}`);
  return item;
}

async function main() {
  {
    const { app, paletteCalls } = loadAppHarness();
    app.setupChat();
    assert.ok(app.slashPalette, 'setupChat should create this.slashPalette');
    assert.strictEqual(paletteCalls.length, 1, 'createSlashPalette should be called once');
    assert.strictEqual(typeof paletteCalls[0].getCommands, 'function', 'getCommands should be provided');
  }

  {
    const harness = loadAppHarness({ flags: { slashPaletteEnabled: false } });
    harness.app.setupChat();
    assert.strictEqual(harness.app.slashPalette, null, 'feature flag should skip palette creation');
    const input = harness.document.getElementById('aiChatInput');
    const enter = createEvent('keydown', { key: 'Enter' });
    input.dispatchEvent(enter);
    assert.strictEqual(enter.defaultPrevented, true, 'normal Enter send should still prevent default');
    assert.strictEqual(harness.sendCount, 1, 'normal Enter send should still call sendChatMessage');
  }

  {
    const harness = loadAppHarness();
    harness.app.setupChat();
    const options = harness.paletteCalls[0];
    const compact = triggerById(options.getCommands(), 'compact');
    options.onSelect(compact);
    const input = harness.document.getElementById('aiChatInput');
    assert.strictEqual(input.value, '/compact', '/compact should fill the input');
    assert.strictEqual(input.focused, true, 'selection should focus the input');
    assert.strictEqual(harness.app.slashPalette.handleInputCalls, 1, 'selection should dispatch input');
    assert.strictEqual(harness.sendCount, 0, '/compact should not auto-send');
  }

  {
    const harness = loadAppHarness();
    harness.app.setupChat();
    const options = harness.paletteCalls[0];
    const tools = triggerById(options.getCommands(), 'tools');
    options.onSelect(tools);
    assert.strictEqual(harness.document.getElementById('aiChatInput').value, '/tools');
    assert.strictEqual(harness.sendCount, 1, '/tools should auto-send only because it is direct + low risk');
  }

  {
    const harness = loadAppHarness();
    harness.app.setupChat();
    const options = harness.paletteCalls[0];
    for (const id of ['tool', 'agent']) {
      harness.document.getElementById('aiChatInput').value = '';
      options.onSelect(triggerById(options.getCommands(), id));
      assert.strictEqual(harness.sendCount, 0, `/${id} should not auto-send`);
    }
    assert.strictEqual(harness.document.getElementById('aiChatInput').value, '/agent ');
  }

  {
    const harness = loadAppHarness({ withPalette: false });
    assert.doesNotThrow(() => harness.app.setupChat(), 'missing HajimiSlashPalette should not throw');
    assert.strictEqual(harness.app.slashPalette, null, 'missing module should leave palette unset');
  }

  {
    const { app } = loadAppHarness();
    const triggers = app.getSlashCommands().map(command => command.trigger);
    const expectedTriggers = [
      '/tools',
      '/providers',
      '/tool',
      '/chat',
      '/mcp',
      '/search',
      '/git',
      '/extensions',
      '/compact',
      '/agent',
    ];
    assert.strictEqual(
      JSON.stringify(triggers),
      JSON.stringify(expectedTriggers),
      'extracted catalog should preserve trigger order'
    );
    assert.ok(!triggers.includes('/代理'), 'Chinese aliases must not masquerade as rendered triggers');
  }

  {
    const { app } = loadAppHarness({ catalogModule: false });
    const triggers = app.getSlashCommands().map(command => command.trigger);
    assert.ok(triggers.includes('/tools'), 'inline fallback should still return slash commands');
    assert.ok(triggers.includes('/agent'), 'inline fallback should keep /agent');
  }

  console.log('day21 slash palette app integration smoke: PASS (8 scenarios)');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
