const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const appPath = path.join(repoRoot, 'src/interface/web/app.js');
const htmlPath = path.join(repoRoot, 'src/interface/web/index.html');

class ClassList {
  constructor(el) {
    this.el = el;
  }

  _items() {
    return new Set((this.el.className || '').split(/\s+/).filter(Boolean));
  }

  add(...names) {
    const items = this._items();
    names.forEach((name) => items.add(name));
    this.el.className = Array.from(items).join(' ');
  }

  remove(...names) {
    const items = this._items();
    names.forEach((name) => items.delete(name));
    this.el.className = Array.from(items).join(' ');
  }

  contains(name) {
    return this._items().has(name);
  }
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

  focus() {
    this.focused = true;
  }

  scrollIntoView() {}

  set innerHTML(value) {
    this._innerHTML = String(value || '');
    this.children = [];
    if (this.id === 'commandList') {
      this._hydrateCommandItems();
    }
  }

  get innerHTML() {
    return this._innerHTML;
  }

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
      return this.children.filter((child) => child.classList.contains('command-item'));
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
    target: overrides.target,
    defaultPrevented: false,
    preventDefault() {
      this.defaultPrevented = true;
    },
  };
}

function createDocument() {
  const document = {
    elements: new Map(),
    addEventListener() {},
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

function loadAppHarness() {
  const document = createDocument();
  const window = {
    HajimiSecurityDom: {
      safeText(value) {
        return String(value ?? '');
      },
      escapeHtml(value) {
        return String(value ?? '')
          .replace(/&/g, '&amp;')
          .replace(/</g, '&lt;')
          .replace(/>/g, '&gt;')
          .replace(/"/g, '&quot;')
          .replace(/'/g, '&#39;');
      },
      escapeAttr(value) {
        return this.escapeHtml(value);
      },
    },
  };
  const context = {
    window,
    document,
    console,
    setTimeout,
    clearTimeout,
    Promise,
    Math,
  };
  context.globalThis = context;
  vm.createContext(context);

  const source = fs.readFileSync(appPath, 'utf8');
  const appOnlySource = source.slice(0, source.indexOf('\n// D3-MINIMAL-FIX'));
  vm.runInContext(appOnlySource, context, { filename: 'app.js' });

  const calls = [];
  const app = context.window.app;
  app.commands = [
    { id: 'view.settings', label: '视图: 显示设置', key: 'Ctrl+Shift+S', action: () => calls.push(['view.settings']) },
    { id: 'palette', label: '命令面板', key: 'Ctrl+Shift+P', action: () => calls.push(['palette']) },
    { id: 'chat.new', label: '对话: 新会话', key: '', action: () => calls.push(['chat.new']) },
  ];

  return { app, document, calls };
}

function assertHtmlContract() {
  const html = fs.readFileSync(htmlPath, 'utf8');
  for (const id of ['commandPalette', 'commandInput', 'commandList']) {
    assert.ok(
      new RegExp(`id=["']${id}["']`).test(html),
      `index.html should contain #${id}`
    );
  }
}

async function main() {
  assertHtmlContract();

  const { app, document, calls } = loadAppHarness();
  const palette = document.getElementById('commandPalette');
  const input = document.getElementById('commandInput');
  const list = document.getElementById('commandList');

  assert.ok(palette, 'test DOM should contain commandPalette');
  assert.ok(input, 'test DOM should contain commandInput');
  assert.ok(list, 'test DOM should contain commandList');

  app.setupCommandPalette();
  app.showCommandPalette();

  assert.strictEqual(palette.classList.contains('active'), true, 'showCommandPalette should open the palette');
  assert.strictEqual(input.value, '', 'showCommandPalette should clear commandInput');
  assert.strictEqual(input.focused, true, 'showCommandPalette should focus commandInput');
  assert.ok(list.querySelectorAll('.command-item').length >= 3, 'showCommandPalette should render command candidates');

  input.value = '设置';
  input.dispatchEvent(createEvent('input'));
  const filteredItems = list.querySelectorAll('.command-item');
  assert.strictEqual(filteredItems.length, 1, 'typing should filter command candidates');
  assert.strictEqual(filteredItems[0].dataset.id, 'view.settings', 'filter should keep matching command id');
  assert.ok(list.innerHTML.includes('视图: 显示设置'), 'filtered list should render matching label');

  input.dispatchEvent(createEvent('keydown', { key: 'Escape' }));
  assert.strictEqual(palette.classList.contains('active'), false, 'Escape should close the palette');

  app.showCommandPalette();
  input.value = '设置';
  input.dispatchEvent(createEvent('input'));
  const safeCandidate = list.querySelectorAll('.command-item')[0];
  safeCandidate.dispatchEvent(createEvent('click'));

  assert.deepStrictEqual(calls, [['view.settings']], 'clicking a safe candidate should reach action wiring once');
  assert.strictEqual(palette.classList.contains('active'), false, 'candidate click should close the palette');

  console.log('day28 command palette DOM smoke: PASS (5 scenarios)');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
