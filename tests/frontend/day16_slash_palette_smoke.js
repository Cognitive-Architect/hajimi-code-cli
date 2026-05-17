const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const slashPalettePath = path.join(repoRoot, 'src/interface/web/modules/slash-palette.js');

class ClassList {
  constructor(el) {
    this.el = el;
  }

  _items() {
    return new Set((this.el.className || '').split(/\s+/).filter(Boolean));
  }

  add(...names) {
    const items = this._items();
    for (const name of names) items.add(name);
    this.el.className = Array.from(items).join(' ');
  }

  remove(...names) {
    const items = this._items();
    for (const name of names) items.delete(name);
    this.el.className = Array.from(items).join(' ');
  }

  contains(name) {
    return this._items().has(name);
  }
}

class Element {
  constructor(tagName) {
    this.tagName = tagName.toUpperCase();
    this.children = [];
    this.parentNode = null;
    this.dataset = {};
    this.listeners = {};
    this.attributes = {};
    this.style = {};
    this.className = '';
    this.disabled = false;
    this.type = '';
    this.value = '';
    this.selectionStart = 0;
    this._textContent = '';
    this.classList = new ClassList(this);
  }

  get firstChild() {
    return this.children[0] || null;
  }

  set textContent(value) {
    this._textContent = String(value ?? '');
    this.children = [];
  }

  get textContent() {
    return this._textContent + this.children.map(child => child.textContent).join('');
  }

  get innerHTML() {
    return this.textContent
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  set innerHTML(value) {
    throw new Error(`innerHTML should not be used in slash palette smoke: ${value}`);
  }

  appendChild(child) {
    child.parentNode = this;
    this.children.push(child);
    return child;
  }

  removeChild(child) {
    const index = this.children.indexOf(child);
    if (index >= 0) {
      this.children.splice(index, 1);
      child.parentNode = null;
    }
    return child;
  }

  setAttribute(name, value) {
    this.attributes[name] = String(value);
  }

  getAttribute(name) {
    return this.attributes[name] ?? null;
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
  }

  click() {
    this.dispatchEvent(createEvent('click'));
  }

  focus() {
    this.focused = true;
  }

  querySelectorAll(selector) {
    if (!selector.startsWith('.')) return [];
    const classNames = selector
      .slice(1)
      .split(':')[0]
      .split('.')
      .filter(Boolean);
    const out = [];
    collect(this, el => classNames.every(className => el.classList.contains(className)), out);
    return out;
  }

  querySelector(selector) {
    return this.querySelectorAll(selector)[0] || null;
  }
}

function collect(root, predicate, out) {
  for (const child of root.children) {
    if (predicate(child)) out.push(child);
    collect(child, predicate, out);
  }
}

function createDocument() {
  return {
    createElement: tagName => new Element(tagName),
  };
}

function createEvent(type, overrides = {}) {
  return {
    type,
    key: overrides.key,
    defaultPrevented: false,
    preventDefault() {
      this.defaultPrevented = true;
    },
  };
}

function keyEvent(key) {
  return createEvent('keydown', { key });
}

function loadSlashPalette(document) {
  const context = {
    window: {},
    document,
    console,
    module: { exports: {} },
  };
  context.window.window = context.window;
  context.window.document = document;
  vm.createContext(context);
  vm.runInContext(fs.readFileSync(slashPalettePath, 'utf8'), context, {
    filename: 'slash-palette.js',
  });
  return context.module.exports.createSlashPalette || context.window.HajimiSlashPalette.createSlashPalette;
}

function commandSet() {
  return [
    { id: 'tools', trigger: '/tools', title: 'List tools', description: 'Show tools', category: 'tool', riskLevel: 'low', enabled: true, executeMode: 'direct' },
    { id: 'compact', trigger: '/compact', title: 'Compact context', description: 'Compress context', category: 'context', riskLevel: 'medium', enabled: true, executeMode: 'fill' },
    { id: 'danger', trigger: '/danger', title: '<img src=x onerror=1>', description: 'safe <script>alert(1)</script>', category: 'tool', riskLevel: 'high', enabled: true, executeMode: 'fill' },
    { id: 'disabled', trigger: '/disabled', title: 'Disabled command', description: 'Should never execute', category: 'test', riskLevel: 'low', enabled: false },
  ];
}

function createHarness() {
  const document = createDocument();
  const inputEl = new Element('textarea');
  const containerEl = new Element('div');
  const selected = [];
  const closed = [];
  const createSlashPalette = loadSlashPalette(document);
  const palette = createSlashPalette({
    inputEl,
    containerEl,
    getCommands: commandSet,
    onSelect(item) {
      selected.push(item);
    },
    onClose(reason) {
      closed.push(reason);
    },
  });

  return { inputEl, containerEl, palette, selected, closed };
}

function renderedTriggers(containerEl) {
  return containerEl.querySelectorAll('.slash-palette-trigger').map(el => el.textContent);
}

function activeTrigger(containerEl) {
  const active = containerEl.querySelector('.slash-palette-item.active');
  return active ? active.querySelector('.slash-palette-trigger').textContent : null;
}

async function main() {
  {
    const { inputEl, containerEl, palette } = createHarness();
    inputEl.value = '/';
    inputEl.selectionStart = 1;
    palette.handleInput();
    assert.strictEqual(palette.isOpen(), true, 'slash token should open the palette');
    assert.strictEqual(containerEl.classList.contains('hidden'), false, 'open palette should be visible');
    assert.ok(renderedTriggers(containerEl).includes('/tools'), 'open palette should render commands');
  }

  {
    const { inputEl, containerEl, palette } = createHarness();
    inputEl.value = '/c';
    inputEl.selectionStart = 2;
    palette.handleInput();
    assert.deepStrictEqual(renderedTriggers(containerEl), ['/compact'], '/c should filter to compact');
  }

  {
    const { inputEl, containerEl, palette } = createHarness();
    inputEl.value = '/';
    inputEl.selectionStart = 1;
    palette.handleInput();
    assert.strictEqual(activeTrigger(containerEl), '/tools', 'first enabled item should be active');
    const down = keyEvent('ArrowDown');
    assert.strictEqual(palette.handleKeyDown(down), true, 'ArrowDown should be handled');
    assert.strictEqual(down.defaultPrevented, true, 'ArrowDown should prevent default');
    assert.strictEqual(activeTrigger(containerEl), '/compact', 'ArrowDown should move active item');
    const up = keyEvent('ArrowUp');
    palette.handleKeyDown(up);
    assert.strictEqual(activeTrigger(containerEl), '/tools', 'ArrowUp should wrap active item back');
  }

  {
    const { inputEl, palette, selected } = createHarness();
    inputEl.value = '/c';
    inputEl.selectionStart = 2;
    palette.handleInput();
    const enter = keyEvent('Enter');
    assert.strictEqual(palette.handleKeyDown(enter), true, 'Enter should select active item');
    assert.strictEqual(enter.defaultPrevented, true, 'Enter selection should prevent ordinary send');
    assert.strictEqual(selected.length, 1, 'Enter should call onSelect once');
    assert.strictEqual(selected[0].id, 'compact', 'Enter should select filtered active command');
    assert.strictEqual(palette.isOpen(), false, 'Enter selection should close the palette');
  }

  {
    const { inputEl, palette, closed } = createHarness();
    inputEl.value = '/tools';
    inputEl.selectionStart = 6;
    palette.handleInput();
    const escape = keyEvent('Escape');
    assert.strictEqual(palette.handleKeyDown(escape), true, 'Escape should be handled');
    assert.strictEqual(escape.defaultPrevented, true, 'Escape should prevent default');
    assert.strictEqual(inputEl.value, '/tools', 'Escape should preserve input');
    assert.strictEqual(closed.at(-1), 'escape', 'Escape should close with escape reason');
    assert.strictEqual(palette.isOpen(), false, 'Escape should close the palette');
  }

  {
    const { inputEl, containerEl, palette, selected } = createHarness();
    inputEl.value = '/disabled';
    inputEl.selectionStart = 9;
    palette.handleInput();
    const enter = keyEvent('Enter');
    assert.strictEqual(palette.handleKeyDown(enter), false, 'disabled active item should not handle Enter');
    assert.strictEqual(enter.defaultPrevented, false, 'disabled Enter should not prevent default');
    assert.strictEqual(selected.length, 0, 'disabled command must not execute');
    const disabledRow = containerEl.querySelector('.slash-palette-item.disabled');
    assert.ok(disabledRow, 'disabled row should render as disabled');
    disabledRow.click();
    assert.strictEqual(selected.length, 0, 'disabled click must not execute');
  }

  {
    const { inputEl, containerEl, palette } = createHarness();
    inputEl.value = '/danger';
    inputEl.selectionStart = 7;
    palette.handleInput();
    assert.ok(containerEl.textContent.includes('<img src=x onerror=1>'), 'malicious title should exist as textContent');
    assert.ok(containerEl.textContent.includes('safe <script>alert(1)</script>'), 'malicious description should exist as textContent');
    assert.strictEqual(containerEl.querySelectorAll('.slash-palette-item').length, 1, 'safe DOM render should keep one candidate row');
    assert.ok(containerEl.innerHTML.includes('&lt;img src=x onerror=1&gt;'), 'innerHTML view should contain escaped text');
  }

  {
    const { inputEl, containerEl, palette } = createHarness();
    inputEl.value = 'hello';
    inputEl.selectionStart = 5;
    palette.handleInput();
    assert.strictEqual(palette.isOpen(), false, 'blank or non-slash input should keep palette closed');
    assert.strictEqual(containerEl.classList.contains('hidden'), true, 'closed palette should be hidden');
  }

  console.log('day16 slash palette smoke: PASS (8 scenarios)');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
