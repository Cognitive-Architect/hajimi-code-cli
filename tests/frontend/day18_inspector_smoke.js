const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const inspectorPath = path.join(repoRoot, 'src/interface/web/modules/inspector.js');
const inspectorViewPath = path.join(repoRoot, 'src/interface/web/views/inspector-view.js');
const inspectorControllerPath = path.join(repoRoot, 'src/interface/web/controllers/inspector-controller.js');

class FakeClassList {
  constructor(initial = []) {
    this.values = new Set(initial);
  }

  toggle(name, active) {
    if (active) this.values.add(name);
    else this.values.delete(name);
  }

  contains(name) {
    return this.values.has(name);
  }

  toString() {
    return Array.from(this.values).join(' ');
  }
}

class FakeElement {
  constructor(document, id = null, attrs = {}) {
    this.document = document;
    this.tagName = attrs.tagName || 'div';
    this.id = id;
    this.dataset = attrs.dataset || {};
    this.style = { cssText: '' };
    this.listeners = {};
    this.children = [];
    this.parentNode = null;
    this.classList = new FakeClassList(attrs.classes || []);
    this._className = this.classList.toString();
    this._innerHTML = '';
    this._textContent = '';
    if (attrs.className) this.className = attrs.className;
  }

  addEventListener(type, handler) {
    this.listeners[type] = this.listeners[type] || [];
    this.listeners[type].push(handler);
  }

  async click() {
    let result;
    for (const handler of this.listeners.click || []) {
      result = handler({ type: 'click', target: this, stopPropagation() {} });
    }
    return result;
  }

  appendChild(child) {
    this.children.push(child);
    child.parentNode = this;
    this.document.registerTree(child);
    return child;
  }

  replaceChildren(...children) {
    this.children = [];
    this._innerHTML = '';
    this._textContent = '';
    children.forEach(child => this.appendChild(child));
  }

  removeChild(child) {
    this.children = this.children.filter(item => item !== child);
    child.parentNode = null;
    return child;
  }

  get firstChild() {
    return this.children[0] || null;
  }

  set className(value) {
    this._className = String(value || '');
    this.classList = new FakeClassList(this._className.split(/\s+/).filter(Boolean));
  }

  get className() {
    return this._className || this.classList.toString();
  }

  set textContent(value) {
    this._textContent = String(value ?? '');
    this._innerHTML = '';
    this.children = [];
  }

  get textContent() {
    if (this.children.length) return this.children.map(child => child.textContent).join('');
    return this._textContent;
  }

  set innerHTML(value) {
    this._innerHTML = String(value);
    this._textContent = '';
    this.children = [];
    if (this._innerHTML.includes('id="inspectorOldDiffBtn"')) {
      this.document.register('inspectorOldDiffBtn', new FakeElement(this.document, 'inspectorOldDiffBtn'));
    }
  }

  get innerHTML() {
    if (this._innerHTML) return this._innerHTML;
    if (this.children.length) return this.children.map(child => child.serialize()).join('');
    return escapeHtml(this._textContent);
  }

  serialize() {
    const attrs = [];
    if (this.id) attrs.push(`id="${escapeHtml(this.id)}"`);
    if (this.className) attrs.push(`class="${escapeHtml(this.className)}"`);
    if (this.style.cssText) attrs.push(`style="${escapeHtml(this.style.cssText)}"`);
    Object.keys(this.dataset || {}).forEach(key => {
      attrs.push(`data-${key}="${escapeHtml(this.dataset[key])}"`);
    });
    const attrText = attrs.length ? ` ${attrs.join(' ')}` : '';
    return `<${this.tagName}${attrText}>${this.innerHTML}</${this.tagName}>`;
  }

  querySelectorAll(selector) {
    const matches = [];
    const classNames = selector.startsWith('.')
      ? selector.slice(1).split('.').filter(Boolean)
      : [];

    function visit(node) {
      if (classNames.length && classNames.every(name => node.classList.contains(name))) {
        matches.push(node);
      }
      node.children.forEach(visit);
    }

    this.children.forEach(visit);
    return matches;
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

  createElement(tagName) {
    return new FakeElement(this, null, { tagName });
  }

  registerTree(el) {
    if (el.id) this.register(el.id, el);
    el.children.forEach(child => this.registerTree(child));
  }

  querySelectorAll(selector) {
    return this.bySelector.get(selector) || [];
  }
}

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

async function main() {
  const document = new FakeDocument();
  const tabTask = new FakeElement(document, null, { dataset: { inspectorTab: 'task-detail' } });
  const tabDiff = new FakeElement(document, null, { dataset: { inspectorTab: 'diff-preview' } });
  const panelTask = new FakeElement(document, 'inspectorTaskDetail', { dataset: { inspectorPanel: 'task-detail' } });
  const panelDiff = new FakeElement(document, 'inspectorDiffContent', { dataset: { inspectorPanel: 'diff-preview' } });
  const panelTrace = new FakeElement(document, 'inspectorTraceContent', { dataset: { inspectorPanel: 'agent-trace' } });

  document.setSelector('.inspector-tab', [tabTask, tabDiff]);
  document.setSelector('.inspector-panel', [panelTask, panelDiff, panelTrace]);
  document.register('rightInspector', new FakeElement(document, 'rightInspector'));
  document.register('inspectorCloseBtn', new FakeElement(document, 'inspectorCloseBtn'));
  document.register('inspectorDiffContent', panelDiff);
  document.register('contextReceiptBody', new FakeElement(document, 'contextReceiptBody'));
  document.register('refreshReceiptBtn', new FakeElement(document, 'refreshReceiptBtn'));

  const context = { window: {}, document, console };
  context.window = context;
  context.globalThis = context;
  vm.createContext(context);
  vm.runInContext(fs.readFileSync(inspectorViewPath, 'utf8'), context, { filename: 'inspector-view.js' });
  vm.runInContext(fs.readFileSync(inspectorControllerPath, 'utf8'), context, { filename: 'inspector-controller.js' });
  vm.runInContext(fs.readFileSync(inspectorPath, 'utf8'), context, { filename: 'inspector.js' });
  assert.strictEqual(typeof context.HajimiInspectorView?.setActiveTab, 'function', 'inspector view should expose setActiveTab');
  assert.strictEqual(typeof context.HajimiInspectorController?.showInspectorTab, 'function', 'inspector controller should expose showInspectorTab');

  let diffRenderCount = 0;
  let receiptCalls = 0;
  let oldDiffFile = null;
  const app = {
    currentDiffFile: null,
    currentEditPayload: null,
    escapeHtml,
    isTauriAvailable: () => true,
    invokeTauri: async (command) => {
      assert.strictEqual(command, 'get_latest_receipt');
      receiptCalls += 1;
      return {
        mode: 'Verified',
        maxContextTokens: 128000,
        inputBudget: 64000,
        estimatedInputTokens: 1234,
        longContextMode: true,
        includedBlocks: [{ name: 'file-a' }],
        omittedBlocks: [{ name: 'file-b', reason: 'budget', tokenEstimate: 10 }],
        bridgeRole: 'planner',
        model: 'test-model',
        providerId: 'test-provider',
        timestamp: 1,
      };
    },
    showInspectorTab(tabId) {
      context.HajimiInspector.showInspectorTab(app, tabId);
    },
    safeRenderInspectorDiffPreview() {
      diffRenderCount += 1;
    },
    safeRenderTraceInspector() {},
    showGitDiff(file) {
      oldDiffFile = file;
    },
    renderContextReceiptPanel(receipt) {
      context.HajimiInspector.renderContextReceiptPanel(app, receipt);
    },
  };

  context.HajimiInspector.init(app);
  await tabDiff.click();
  assert.strictEqual(diffRenderCount, 1, 'diff tab should call safeRenderInspectorDiffPreview');
  assert.strictEqual(tabDiff.classList.contains('active'), true, 'diff tab should become active');
  assert.strictEqual(panelDiff.classList.contains('active'), true, 'diff panel should become active');

  await document.getElementById('inspectorCloseBtn').click();
  assert.strictEqual(document.getElementById('rightInspector').style.display, 'none', 'close button should hide inspector');

  context.HajimiInspector.openDiffPreview(app, 'src/main.rs');
  context.HajimiInspector.renderInspectorDiffPreview(app);
  const fallback = document.getElementById('inspectorOldDiffBtn');
  assert.ok(fallback, 'diff fallback button should be registered');
  await fallback.click();
  assert.strictEqual(oldDiffFile, 'src/main.rs', 'diff fallback should call showGitDiff with current file');

  context.HajimiInspector.setupReceiptPanel(app);
  assert.strictEqual(receiptCalls, 1, 'setupReceiptPanel should load receipt once');
  await document.getElementById('refreshReceiptBtn').click();
  assert.strictEqual(receiptCalls, 2, 'refresh button should reload receipt');
  assert.ok(
    document.getElementById('contextReceiptBody').innerHTML.includes('Context Receipt'),
    'receipt panel should render Context Receipt content',
  );

  console.log('day18 inspector module smoke: PASS');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
