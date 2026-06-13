const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const inspectorPath = path.join(repoRoot, 'src/interface/web/modules/inspector.js');
const inspectorSource = fs.readFileSync(inspectorPath, 'utf8');

class FakeClassList {
  constructor(initial = []) {
    this.values = new Set(initial);
  }

  contains(name) {
    return this.values.has(name);
  }
}

class FakeElement {
  constructor(document, tagName = 'div') {
    this.document = document;
    this.tagName = tagName;
    this.id = '';
    this.dataset = {};
    this.style = { cssText: '' };
    this.children = [];
    this.listeners = {};
    this.classList = new FakeClassList();
    this._className = '';
    this._textContent = '';
  }

  set className(value) {
    this._className = String(value || '');
    this.classList = new FakeClassList(this._className.split(/\s+/).filter(Boolean));
  }

  get className() {
    return this._className;
  }

  set textContent(value) {
    this._textContent = String(value ?? '');
    this.children = [];
  }

  get textContent() {
    if (this.children.length) return this.children.map(child => child.textContent).join('');
    return this._textContent;
  }

  appendChild(child) {
    this.children.push(child);
    this.document.registerTree(child);
    return child;
  }

  replaceChildren(...children) {
    this.children = [];
    this._textContent = '';
    children.forEach(child => this.appendChild(child));
  }

  removeChild(child) {
    this.children = this.children.filter(item => item !== child);
    return child;
  }

  get firstChild() {
    return this.children[0] || null;
  }

  addEventListener(type, handler) {
    this.listeners[type] = this.listeners[type] || [];
    this.listeners[type].push(handler);
  }

  get innerHTML() {
    if (this.children.length) return this.children.map(child => child.serialize()).join('');
    return escapeHtml(this._textContent);
  }

  serialize() {
    const attrs = [];
    if (this.id) attrs.push(`id="${escapeHtml(this.id)}"`);
    if (this.className) attrs.push(`class="${escapeHtml(this.className)}"`);
    if (this.style.cssText) attrs.push(`style="${escapeHtml(this.style.cssText)}"`);
    Object.keys(this.dataset).forEach(key => {
      attrs.push(`data-${key}="${escapeHtml(this.dataset[key])}"`);
    });
    const attrText = attrs.length ? ` ${attrs.join(' ')}` : '';
    return `<${this.tagName}${attrText}>${this.innerHTML}</${this.tagName}>`;
  }

  querySelectorAll(selector) {
    const requiredClasses = selector.startsWith('.')
      ? selector.slice(1).split('.').filter(Boolean)
      : [];
    const matches = [];

    function visit(node) {
      if (requiredClasses.length && requiredClasses.every(name => node.classList.contains(name))) {
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
  }

  createElement(tagName) {
    return new FakeElement(this, tagName);
  }

  register(id, element) {
    element.id = id;
    this.byId.set(id, element);
    return element;
  }

  registerTree(element) {
    if (element.id) this.byId.set(element.id, element);
    element.children.forEach(child => this.registerTree(child));
  }

  getElementById(id) {
    return this.byId.get(id) || null;
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

function assertNoExecutableHtml(html, label) {
  assert.ok(!/<script\b/i.test(html), `${label} should not contain raw script tags`);
  assert.ok(!/<img\b/i.test(html), `${label} should not contain raw img tags`);
  assert.ok(!/<svg\b/i.test(html), `${label} should not contain raw svg tags`);
  assert.ok(!/<[^>]+\son\w+=/i.test(html), `${label} should not contain raw event handlers`);
}

function createHarness() {
  const document = new FakeDocument();
  document.register('inspectorDiffContent', document.createElement('div'));
  document.register('inspectorTraceContent', document.createElement('div'));
  document.register('contextReceiptBody', document.createElement('div'));

  const context = {
    window: {},
    document,
    console,
    Date,
    alert: () => {
      throw new Error('compare alert should not run in safe DOM smoke');
    },
  };
  context.window = context;
  context.globalThis = context;
  vm.createContext(context);
  vm.runInContext(inspectorSource, context, { filename: 'inspector.js' });

  const forbidden = (name) => () => {
    throw new Error(`${name} must not run in safe DOM smoke`);
  };
  const app = {
    currentDiffFile: null,
    currentEditPayload: null,
    traceEvents: [],
    escapeHtml,
    showGitDiff: forbidden('showGitDiff'),
    restoreCheckpoint: forbidden('restoreCheckpoint'),
  };

  return { context, document, app };
}

function main() {
  assert.ok(
    !/\b(?:innerHTML|outerHTML|insertAdjacentHTML)\b/.test(inspectorSource),
    'inspector module should not use HTML sink APIs after safe DOM rewrite',
  );

  const { context, document, app } = createHarness();
  const malicious = '<img src=x onerror="globalThis.__xss=1"><script>globalThis.__xss=1</script><svg onload="globalThis.__xss=1">';

  app.currentEditPayload = {
    summary: malicious,
    hunks: [{
      file_path: `src/${malicious}.js`,
      start_line: malicious,
      old_lines: [malicious],
      new_lines: [malicious],
    }],
  };
  context.HajimiInspector.renderInspectorDiffPreview(app);
  const diffHtml = document.getElementById('inspectorDiffContent').innerHTML;
  assert.ok(diffHtml.includes('&lt;img'), 'diff preview should serialize malicious text as escaped text');
  assertNoExecutableHtml(diffHtml, 'diff preview');

  app.traceEvents = [{
    step_type: 'Plan',
    step: malicious,
    iteration: 7,
    details: malicious,
    timestamp: 1,
  }];
  context.HajimiInspector.renderTraceInspector(app);
  const traceHtml = document.getElementById('inspectorTraceContent').innerHTML;
  assert.ok(traceHtml.includes('&lt;script&gt;'), 'trace should serialize malicious text as escaped text');
  assertNoExecutableHtml(traceHtml, 'trace inspector');

  context.HajimiInspector.renderContextReceiptPanel(app, {
    mode: malicious,
    maxContextTokens: 1,
    inputBudget: 1,
    estimatedInputTokens: 1,
    longContextMode: true,
    includedBlocks: [{}],
    omittedBlocks: [{ name: malicious, reason: malicious, tokenEstimate: 1 }],
    bridgeRole: malicious,
    model: malicious,
    providerId: malicious,
    timestamp: 1,
  });
  const receiptHtml = document.getElementById('contextReceiptBody').innerHTML;
  assert.ok(receiptHtml.includes('&lt;svg'), 'context receipt should serialize malicious text as escaped text');
  assertNoExecutableHtml(receiptHtml, 'context receipt');

  console.log('day35 inspector rendering safe dom smoke: PASS');
}

try {
  main();
} catch (error) {
  console.error(error);
  process.exit(1);
}
