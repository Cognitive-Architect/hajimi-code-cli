/**
 * Day 30 — Command Palette Delegation Path Smoke
 *
 * V3X Day 3-B-1: prove the NEW module delegation path works end-to-end.
 *
 * Load order inside the VM context:
 *   1. command-palette-view.js  → sets window.HajimiCommandPaletteView
 *   2. command-controller.js    → sets window.HajimiCommandController
 *   3. app.js                   → setupCommandPalette picks delegation branch
 *
 * This test does NOT touch index.html, modules/**, styles/**, style.css,
 * main.rs, Provider, Keyring, Shell, Checkpoint, or Agent streaming.
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const viewPath = path.join(repoRoot, 'src/interface/web/views/command-palette-view.js');
const ctrlPath = path.join(repoRoot, 'src/interface/web/controllers/command-controller.js');
const appPath = path.join(repoRoot, 'src/interface/web/app.js');

// ── Minimal DOM mock (matches day28 patterns) ──

class ClassList {
  constructor(el) { this.el = el; }
  _items() { return new Set((this.el.className || '').split(/\s+/).filter(Boolean)); }
  add(...names) {
    const items = this._items();
    names.forEach((n) => items.add(n));
    this.el.className = Array.from(items).join(' ');
  }
  remove(...names) {
    const items = this._items();
    names.forEach((n) => items.delete(n));
    this.el.className = Array.from(items).join(' ');
  }
  contains(name) { return this._items().has(name); }
}

class Element {
  constructor(document, id = '', tagName = 'div') {
    this.document = document;
    this.id = id;
    this.tagName = tagName.toUpperCase();
    this.listeners = {};
    this.children = [];
    this.dataset = {};
    this.style = {};
    this.className = '';
    this.value = '';
    this.focused = false;
    this._innerHTML = '';
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
  focus() { this.focused = true; }
  scrollIntoView() {}
  set innerHTML(value) {
    this._innerHTML = String(value || '');
    this.children = [];
    if (this.id === 'commandList') {
      this._hydrateCommandItems();
    }
  }
  get innerHTML() { return this._innerHTML; }
  _hydrateCommandItems() {
    const itemRe = /<div class="([^"]*command-item[^"]*)" data-index="([^"]+)" data-id="([^"]+)">([\s\S]*?)<\/div>/g;
    let match;
    while ((match = itemRe.exec(this._innerHTML))) {
      const [, className, index, id, body] = match;
      const item = new Element(this.document, '', 'div');
      item.className = className;
      item.dataset.index = index;
      item.dataset.id = decodeHtmlAttr(id);
      item._innerHTML = body;
      item.parentElement = this;
      this.children.push(item);
    }
  }
  querySelectorAll(selector) {
    if (selector === '.command-item') {
      return this.children.filter((c) => c.classList.contains('command-item'));
    }
    return [];
  }
}

function decodeHtmlAttr(value) {
  return String(value)
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&amp;/g, '&');
}

function createEvent(type, overrides = {}) {
  return {
    type,
    key: overrides.key,
    ctrlKey: overrides.ctrlKey || false,
    shiftKey: overrides.shiftKey || false,
    target: overrides.target,
    defaultPrevented: false,
    preventDefault() { this.defaultPrevented = true; },
  };
}

function createDocument() {
  const globalListeners = {};
  const document = {
    elements: new Map(),
    addEventListener(type, handler) {
      globalListeners[type] = globalListeners[type] || [];
      globalListeners[type].push(handler);
    },
    _dispatchGlobal(event) {
      for (const handler of globalListeners[event.type] || []) {
        handler(event);
      }
    },
    getElementById(id) {
      return this.elements.get(id) || null;
    },
    querySelector(selector) {
      if (selector === '.command-item.selected') {
        return this.querySelectorAll('.command-item').find((el) => el.classList.contains('selected')) || null;
      }
      return null;
    },
    querySelectorAll(selector) {
      if (selector === '.command-item') {
        const list = this.getElementById('commandList');
        return list ? list.querySelectorAll(selector) : [];
      }
      return [];
    },
    createElement(tagName) {
      return new Element(this, '', tagName);
    },
    body: { appendChild() {} },
  };

  [
    ['commandPalette', 'div', 'command-palette hidden'],
    ['commandInput', 'input', 'command-input'],
    ['commandList', 'div', 'command-list'],
  ].forEach(([id, tagName, className]) => {
    const el = new Element(document, id, tagName);
    el.className = className;
    document.elements.set(id, el);
  });

  return document;
}

function loadDelegationHarness() {
  const document = createDocument();
  const window = {
    HajimiSecurityDom: {
      safeText(value) { return String(value ?? ''); },
      escapeHtml(value) {
        return String(value ?? '')
          .replace(/&/g, '&amp;')
          .replace(/</g, '&lt;')
          .replace(/>/g, '&gt;')
          .replace(/"/g, '&quot;')
          .replace(/'/g, '&#39;');
      },
      escapeAttr(value) { return this.escapeHtml(value); },
    },
  };

  const context = {
    window,
    document,
    console,
    setTimeout,
    clearTimeout,
    clearInterval,
    setInterval,
    Promise,
    Math,
    Date,
    Array,
    Set,
    Map,
    Object,
    String,
    JSON,
    parseInt,
    confirm: () => false,
    Blob: function Blob() {},
    URL: { createObjectURL() { return ''; }, revokeObjectURL() {} },
  };
  context.globalThis = context;
  vm.createContext(context);

  // Step 1: Load command-palette-view.js → sets window.HajimiCommandPaletteView
  const viewSource = fs.readFileSync(viewPath, 'utf8');
  vm.runInContext(viewSource, context, { filename: 'command-palette-view.js' });

  // Step 2: Load command-controller.js → sets window.HajimiCommandController
  const ctrlSource = fs.readFileSync(ctrlPath, 'utf8');
  vm.runInContext(ctrlSource, context, { filename: 'command-controller.js' });

  // Step 3: Load app.js (up to D3-MINIMAL-FIX marker, same as day28)
  const appSource = fs.readFileSync(appPath, 'utf8');
  const appOnlySource = appSource.slice(0, appSource.indexOf('\n// D3-MINIMAL-FIX'));
  vm.runInContext(appOnlySource, context, { filename: 'app.js' });

  const app = context.window.app;

  // Inject test commands
  const calls = [];
  app.commands = [
    { id: 'view.settings', label: '视图: 显示设置', key: 'Ctrl+Shift+S', action: () => calls.push(['view.settings']) },
    { id: 'palette', label: '命令面板', key: 'Ctrl+Shift+P', action: () => calls.push(['palette']) },
    { id: 'chat.new', label: '对话: 新会话', key: '', action: () => calls.push(['chat.new']) },
  ];

  // Stub non-palette methods that init() calls
  app.showSidebar = (v) => calls.push(['showSidebar', v]);
  app.toggleSidebar = () => calls.push(['toggleSidebar']);

  return { app, document, context, calls };
}

async function main() {
  let passed = 0;

  // ── Scenario 1: Module globals exist ──
  const { app, document, context, calls } = loadDelegationHarness();
  assert.ok(context.window.HajimiCommandPaletteView, 'window.HajimiCommandPaletteView should exist');
  assert.strictEqual(
    typeof context.window.HajimiCommandPaletteView.createCommandPaletteView, 'function',
    'createCommandPaletteView should be a function'
  );
  assert.ok(context.window.HajimiCommandController, 'window.HajimiCommandController should exist');
  assert.strictEqual(
    typeof context.window.HajimiCommandController.createCommandController, 'function',
    'createCommandController should be a function'
  );
  passed++;

  // ── Scenario 2: setupCommandPalette takes delegation branch ──
  assert.strictEqual(app._commandPaletteView, undefined, '_commandPaletteView should not exist before setup');
  assert.strictEqual(app._commandController, undefined, '_commandController should not exist before setup');

  app.setupCommandPalette();

  assert.ok(app._commandPaletteView, '_commandPaletteView should exist after setupCommandPalette');
  assert.strictEqual(typeof app._commandPaletteView.show, 'function', '_commandPaletteView.show should be function');
  assert.strictEqual(typeof app._commandPaletteView.hide, 'function', '_commandPaletteView.hide should be function');
  assert.strictEqual(typeof app._commandPaletteView.renderList, 'function', '_commandPaletteView.renderList should be function');
  assert.strictEqual(typeof app._commandPaletteView.navigate, 'function', '_commandPaletteView.navigate should be function');
  assert.strictEqual(typeof app._commandPaletteView.executeSelected, 'function', '_commandPaletteView.executeSelected should be function');

  assert.ok(app._commandController, '_commandController should exist after setupCommandPalette');
  assert.strictEqual(typeof app._commandController.setup, 'function', '_commandController.setup should be function');
  assert.strictEqual(typeof app._commandController.setupKeyboardShortcuts, 'function', '_commandController.setupKeyboardShortcuts should be function');
  passed++;

  // ── Scenario 3: showCommandPalette delegation ──
  const palette = document.getElementById('commandPalette');
  const input = document.getElementById('commandInput');
  const list = document.getElementById('commandList');

  app.showCommandPalette();

  assert.strictEqual(palette.classList.contains('active'), true, 'showCommandPalette delegation should add active class');
  assert.strictEqual(input.value, '', 'showCommandPalette delegation should clear input');
  assert.strictEqual(input.focused, true, 'showCommandPalette delegation should focus input');
  assert.ok(list.querySelectorAll('.command-item').length >= 3, 'showCommandPalette delegation should render all commands');
  passed++;

  // ── Scenario 4: renderCommandList delegation ──
  app.renderCommandList('设置');
  const filteredItems = list.querySelectorAll('.command-item');
  assert.strictEqual(filteredItems.length, 1, 'renderCommandList delegation should filter to 1 result');
  assert.strictEqual(filteredItems[0].dataset.id, 'view.settings', 'filtered result should be view.settings');
  assert.ok(list.innerHTML.includes('视图: 显示设置'), 'filtered list should contain matching label');
  passed++;

  // ── Scenario 5: hideCommandPalette delegation ──
  app.hideCommandPalette();
  assert.strictEqual(palette.classList.contains('active'), false, 'hideCommandPalette delegation should remove active');
  passed++;

  // ── Scenario 6: navigateCommandList delegation ──
  app.showCommandPalette(); // re-open to get full list
  const allItems = list.querySelectorAll('.command-item');
  assert.ok(allItems.length >= 3, 'should have at least 3 items for navigation test');
  assert.ok(allItems[0].classList.contains('selected'), 'first item should be selected initially');

  app.navigateCommandList(1);
  assert.ok(allItems[1].classList.contains('selected'), 'navigateCommandList(1) should select second item');
  assert.strictEqual(allItems[0].classList.contains('selected'), false, 'first item should lose selection');
  passed++;

  // ── Scenario 7: executeSelectedCommand delegation ──
  calls.length = 0;
  app.executeSelectedCommand();
  const executedCmd = app.commands.find((c) => c.id === allItems[1].dataset.id);
  assert.ok(executedCmd, 'selected item should correspond to a valid command');
  assert.strictEqual(palette.classList.contains('active'), false, 'executeSelectedCommand should close palette');
  assert.strictEqual(calls.length, 1, 'executeSelectedCommand should trigger exactly one action');
  passed++;

  // ── Scenario 8: setupKeyboardShortcuts delegation ──
  app.setupKeyboardShortcuts();

  // Simulate Ctrl+Shift+P → should open palette
  palette.classList.remove('active');
  document._dispatchGlobal(createEvent('keydown', { key: 'P', ctrlKey: true, shiftKey: true }));
  assert.strictEqual(palette.classList.contains('active'), true, 'Ctrl+Shift+P via delegation should open palette');

  // Simulate Escape → should close palette
  document._dispatchGlobal(createEvent('keydown', { key: 'Escape' }));
  assert.strictEqual(palette.classList.contains('active'), false, 'Escape via delegation should close palette');

  // Simulate Ctrl+Shift+E → should call showSidebar('explorer')
  calls.length = 0;
  document._dispatchGlobal(createEvent('keydown', { key: 'E', ctrlKey: true, shiftKey: true }));
  assert.deepStrictEqual(calls, [['showSidebar', 'explorer']], 'Ctrl+Shift+E should call showSidebar(explorer)');
  passed++;

  // ── Scenario 9: click-to-execute via delegation ──
  calls.length = 0;
  app.showCommandPalette();
  app.renderCommandList('设置');
  const settingsItem = list.querySelectorAll('.command-item')[0];
  assert.ok(settingsItem, 'settings item should exist after filter');
  settingsItem.dispatchEvent(createEvent('click'));
  assert.strictEqual(palette.classList.contains('active'), false, 'clicking item via delegation should close palette');
  assert.deepStrictEqual(calls, [['view.settings']], 'clicking item should invoke action');
  passed++;

  // ── Scenario 10: controller input events ──
  calls.length = 0;
  app.showCommandPalette();
  // Type into commandInput → should trigger renderList via controller binding
  input.value = '对话';
  input.dispatchEvent(createEvent('input'));
  const chatItems = list.querySelectorAll('.command-item');
  assert.strictEqual(chatItems.length, 1, 'input event via controller should filter list');
  assert.strictEqual(chatItems[0].dataset.id, 'chat.new', 'filtered result should be chat.new');

  // Press Enter → should executeSelected
  input.dispatchEvent(createEvent('keydown', { key: 'Enter' }));
  assert.deepStrictEqual(calls, [['chat.new']], 'Enter via controller should execute selected command');
  assert.strictEqual(palette.classList.contains('active'), false, 'Enter should close palette');
  passed++;

  console.log(`day30 command palette delegation smoke: PASS (${passed} scenarios)`);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
