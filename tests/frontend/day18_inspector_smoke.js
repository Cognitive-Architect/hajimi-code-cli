const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const inspectorPath = path.join(repoRoot, 'src/interface/web/modules/inspector.js');

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
    this._innerHTML = '';
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

  set innerHTML(value) {
    this._innerHTML = String(value);
    if (this._innerHTML.includes('id="inspectorOldDiffBtn"')) {
      this.document.register('inspectorOldDiffBtn', new FakeElement(this.document, 'inspectorOldDiffBtn'));
    }
  }

  get innerHTML() {
    return this._innerHTML;
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
  vm.runInContext(fs.readFileSync(inspectorPath, 'utf8'), context, { filename: 'inspector.js' });

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
