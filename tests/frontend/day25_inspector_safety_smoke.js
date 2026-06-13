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

  add(name) {
    this.values.add(name);
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
    this.classList = new FakeClassList(attrs.classes || []);
    this.children = [];
    this.parentNode = null;
    this._className = this.classList.toString();
    this._innerHTML = '';
    this._textContent = '';
    if (attrs.className) this.className = attrs.className;
  }

  addEventListener(type, handler) {
    this.listeners[type] = this.listeners[type] || [];
    this.listeners[type].push(handler);
  }

  click() {
    for (const handler of this.listeners.click || []) {
      handler({ type: 'click', target: this, stopPropagation() {} });
    }
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
  return String(value ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function createHarness() {
  const document = new FakeDocument();
  const tabs = {
    task: new FakeElement(document, null, { dataset: { inspectorTab: 'task-detail' }, classes: ['active'] }),
    diff: new FakeElement(document, null, { dataset: { inspectorTab: 'diff-preview' } }),
    trace: new FakeElement(document, null, { dataset: { inspectorTab: 'agent-trace' } }),
    receipt: new FakeElement(document, null, { dataset: { inspectorTab: 'context-receipt' } }),
  };
  const panels = {
    task: new FakeElement(document, 'inspectorTaskDetail', { dataset: { inspectorPanel: 'task-detail' }, classes: ['active'] }),
    diff: new FakeElement(document, 'inspectorDiffContent', { dataset: { inspectorPanel: 'diff-preview' } }),
    trace: new FakeElement(document, 'inspectorTraceContent', { dataset: { inspectorPanel: 'agent-trace' } }),
    receipt: new FakeElement(document, 'contextReceiptPanel', { dataset: { inspectorPanel: 'context-receipt' } }),
  };

  document.setSelector('.inspector-tab', Object.values(tabs));
  document.setSelector('.inspector-panel', Object.values(panels));
  document.register('rightInspector', new FakeElement(document, 'rightInspector'));
  document.register('inspectorCloseBtn', new FakeElement(document, 'inspectorCloseBtn'));
  document.register('inspectorDiffContent', panels.diff);
  document.register('inspectorTraceContent', panels.trace);
  document.register('contextReceiptBody', new FakeElement(document, 'contextReceiptBody'));
  document.register('refreshReceiptBtn', new FakeElement(document, 'refreshReceiptBtn'));

  const context = {
    window: {},
    document,
    console,
    Date,
    alert: () => {
      throw new Error('compare alert should not be executed during readonly smoke');
    },
  };
  context.window = context;
  context.globalThis = context;
  vm.createContext(context);
  vm.runInContext(fs.readFileSync(inspectorViewPath, 'utf8'), context, { filename: 'inspector-view.js' });
  vm.runInContext(fs.readFileSync(inspectorControllerPath, 'utf8'), context, { filename: 'inspector-controller.js' });
  vm.runInContext(fs.readFileSync(inspectorPath, 'utf8'), context, { filename: 'inspector.js' });
  assert.strictEqual(typeof context.HajimiInspectorView?.setActiveTab, 'function', 'inspector view should expose setActiveTab');
  assert.strictEqual(typeof context.HajimiInspectorController?.showInspectorTab, 'function', 'inspector controller should expose showInspectorTab');

  const calls = [];
  const forbidden = (name) => () => {
    throw new Error(`${name} must not be executed during inspector safety smoke`);
  };
  const app = {
    calls,
    currentDiffFile: null,
    currentEditPayload: null,
    traceEvents: [],
    escapeHtml,
    showInspectorTab(tabId) {
      calls.push(['showInspectorTab', tabId]);
      context.HajimiInspector.showInspectorTab(app, tabId);
    },
    safeRenderInspectorDiffPreview() {
      calls.push(['safeRenderInspectorDiffPreview']);
      context.HajimiInspector.safeRenderInspectorDiffPreview(app);
    },
    safeRenderTraceInspector() {
      calls.push(['safeRenderTraceInspector']);
      context.HajimiInspector.safeRenderTraceInspector(app);
    },
    withInspectorGuard(label, renderFn) {
      calls.push(['withInspectorGuard', label]);
      context.HajimiInspector.withInspectorGuard(app, label, renderFn);
    },
    renderInspectorDiffPreview() {
      calls.push(['renderInspectorDiffPreview']);
      context.HajimiInspector.renderInspectorDiffPreview(app);
    },
    renderTraceInspector() {
      calls.push(['renderTraceInspector']);
      context.HajimiInspector.renderTraceInspector(app);
    },
    showGitDiff: forbidden('showGitDiff'),
    restoreCheckpoint: forbidden('restoreCheckpoint'),
    exportAllCheckpoints: forbidden('exportAllCheckpoints'),
    compareCheckpoints: forbidden('compareCheckpoints'),
    invokeTauri: forbidden('invokeTauri'),
    loadProviders: forbidden('loadProviders'),
    executeShellCommand: forbidden('executeShellCommand'),
  };

  return { context, document, tabs, panels, app };
}

function assertNoExecutableHtml(html, label) {
  assert.ok(!/<script\b/i.test(html), `${label} should not contain raw script tags`);
  assert.ok(!/<img\b/i.test(html), `${label} should not contain raw img tags`);
  assert.ok(!/<svg\b/i.test(html), `${label} should not contain raw svg tags`);
  assert.ok(!/<[^>]+\son\w+=/i.test(html), `${label} should not contain raw event handler attributes`);
}

async function main() {
  {
    const { context, document, tabs, panels, app } = createHarness();
    context.HajimiInspector.init(app);

    tabs.diff.click();
    assert.ok(tabs.diff.classList.contains('active'), 'diff tab should become active');
    assert.ok(panels.diff.classList.contains('active'), 'diff panel should become active');
    assert.ok(!tabs.task.classList.contains('active'), 'task tab should deactivate');
    assert.ok(document.getElementById('inspectorDiffContent').innerHTML.includes('选择文件或等待 Agent 建议修改后显示 Diff'), 'diff empty state should render');

    tabs.trace.click();
    assert.ok(tabs.trace.classList.contains('active'), 'trace tab should become active');
    assert.ok(panels.trace.classList.contains('active'), 'trace panel should become active');
    assert.ok(document.getElementById('inspectorTraceContent').innerHTML.includes('任务执行后显示 Trace'), 'trace empty state should render');
  }

  {
    const { context, document, app } = createHarness();
    app.currentEditPayload = {
      summary: '<img src=x onerror="globalThis.__xss=1">diff summary',
      hunks: [{
        file_path: 'src/<script>bad</script>.js',
        start_line: 12,
        old_lines: ['old <script>globalThis.__xss=1</script>'],
        new_lines: ['new <svg onload="globalThis.__xss=1">'],
      }],
    };

    context.HajimiInspector.renderInspectorDiffPreview(app);
    const html = document.getElementById('inspectorDiffContent').innerHTML;
    assert.ok(html.includes('&lt;img'), 'diff summary should be escaped');
    assert.ok(html.includes('&lt;script&gt;bad&lt;/script&gt;'), 'diff file path should be escaped');
    assert.ok(html.includes('&lt;svg'), 'diff new lines should be escaped');
    assertNoExecutableHtml(html, 'diff preview');
  }

  {
    const { context, document, app } = createHarness();
    app.traceEvents = [{
      step_type: 'Plan',
      step: '<img src=x onerror="globalThis.__xss=1">Plan',
      iteration: '<script>1</script>',
      details: 'details <svg onload="globalThis.__xss=1">',
      timestamp: 1,
    }];

    context.HajimiInspector.renderTraceInspector(app);
    const html = document.getElementById('inspectorTraceContent').innerHTML;
    assert.ok(html.includes('&lt;img'), 'trace step should be escaped');
    assert.ok(html.includes('&lt;script&gt;1&lt;/script&gt;'), 'trace iteration should be escaped');
    assert.ok(html.includes('&lt;svg'), 'trace details should be escaped');
    assertNoExecutableHtml(html, 'trace inspector');
  }

  {
    const { context, document, app } = createHarness();
    context.HajimiInspector.renderContextReceiptPanel(app, null);
    assert.ok(document.getElementById('contextReceiptBody').innerHTML.includes('暂无上下文小票'), 'receipt empty state should render');

    context.HajimiInspector.renderContextReceiptPanel(app, {
      mode: '<script>mode</script>',
      maxContextTokens: 128000,
      inputBudget: 64000,
      estimatedInputTokens: 123,
      longContextMode: true,
      includedBlocks: [{ name: 'included' }],
      omittedBlocks: [{
        name: '<img src=x onerror="globalThis.__xss=1">',
        reason: '<svg onload="globalThis.__xss=1">',
        tokenEstimate: 7,
      }],
      bridgeRole: '<script>planner</script>',
      model: '<img src=x onerror="globalThis.__xss=1">',
      providerId: '<svg onload="globalThis.__xss=1">',
      timestamp: 1,
    });
    const html = document.getElementById('contextReceiptBody').innerHTML;
    assert.ok(html.includes('Context Receipt'), 'receipt content should render');
    assert.ok(html.includes('&lt;img'), 'receipt malicious names should be escaped');
    assert.ok(html.includes('&lt;svg'), 'receipt malicious reasons should be escaped');
    assert.ok(html.includes('&lt;script&gt;planner&lt;/script&gt;'), 'receipt bridge role should be escaped');
    assertNoExecutableHtml(html, 'context receipt');
  }

  console.log('day25 inspector safety smoke: PASS (4 scenarios)');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
