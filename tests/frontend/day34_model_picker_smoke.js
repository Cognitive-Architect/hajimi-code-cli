const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const viewPath = path.join(repoRoot, 'src/interface/web/views/model-picker-view.js');
const controllerPath = path.join(repoRoot, 'src/interface/web/controllers/model-picker-controller.js');

class FakeClassList {
  constructor() {
    this.values = new Set();
  }
  add(name) { this.values.add(name); }
  remove(name) { this.values.delete(name); }
  contains(name) { return this.values.has(name); }
}

class FakeElement {
  constructor(document, id = null) {
    this.document = document;
    this.id = id;
    this.classList = new FakeClassList();
    this.textContent = '';
    this.innerHTML = '';
    this.dataset = {};
    this.listeners = {};
  }

  addEventListener(type, handler) {
    this.listeners[type] = handler;
  }

  querySelectorAll(selector) {
    return [];
  }

  trigger(type, ev = {}) {
    if (this.listeners[type]) {
      this.listeners[type]({
        preventDefault() {},
        stopPropagation() {},
        target: this,
        ...ev
      });
    }
  }
}

class FakeDocument {
  constructor() {
    this.byId = new Map();
  }

  createElement(tag) {
    return new FakeElement(this);
  }

  getElementById(id) {
    return this.byId.get(id) || null;
  }

  register(id, el) {
    this.byId.set(id, el);
    return el;
  }
}

async function main() {
  const document = new FakeDocument();
  const context = { window: {}, document, console };
  context.window = context;
  context.globalThis = context;
  vm.createContext(context);

  // Load view & controller
  vm.runInContext(fs.readFileSync(viewPath, 'utf8'), context, { filename: 'model-picker-view.js' });
  vm.runInContext(fs.readFileSync(controllerPath, 'utf8'), context, { filename: 'model-picker-controller.js' });

  assert.ok(context.HajimiModelPickerView, 'HajimiModelPickerView should be mounted');
  assert.ok(context.HajimiModelPickerController, 'HajimiModelPickerController should be mounted');

  // Setup DOM elements
  const btn = new FakeElement(document, 'modelSelectBtn');
  const closeBtn = new FakeElement(document, 'modelPickerClose');
  const addBtn = new FakeElement(document, 'modelPickerAddBtn');
  const modal = new FakeElement(document, 'modelPickerModal');
  const body = new FakeElement(document, 'modelPickerBody');

  document.register('modelSelectBtn', btn);
  document.register('modelPickerClose', closeBtn);
  document.register('modelPickerAddBtn', addBtn);
  document.register('modelPickerModal', modal);
  document.register('modelPickerBody', body);

  let openProviderModalCalls = 0;
  let selectProviderCalls = [];
  let deleteProviderConfigCalls = [];

  const app = {
    providerConfigs: [
      { id: 'gpt-4', name: 'GPT-4', model: 'gpt-4', providerType: 'openai' }
    ],
    activeProviderId: 'gpt-4',
    escapeHtml(t) { return t; },
    escapeAttr(t) { return t; },
    renderSidebarModelSummary() {},
    openProviderModal() { openProviderModalCalls++; },
    selectProvider(id) { selectProviderCalls.push(id); },
    deleteProviderConfig(id) { deleteProviderConfigCalls.push(id); },
    closeModelPicker() { modal.classList.remove('active'); }
  };

  // 1. Test renderModelButton
  context.HajimiModelPickerView.renderModelButton(app);
  assert.strictEqual(btn.textContent, 'GPT-4');

  // 2. Test setupModelPicker
  context.HajimiModelPickerController.setupModelPicker(app);

  btn.trigger('click');
  assert.ok(modal.classList.contains('active'), 'Click select btn should show modal');

  closeBtn.trigger('click');
  assert.strictEqual(modal.classList.contains('active'), false, 'Click close btn should hide modal');

  addBtn.trigger('click');
  assert.strictEqual(openProviderModalCalls, 1, 'Click add btn should open provider modal');

  // 3. Test renderModelPicker
  context.HajimiModelPickerView.renderModelPicker(app);
  // Just verify no crash and some HTML produced
  assert.ok(body.innerHTML.includes('gpt-4'), 'Should render model picker elements');

  console.log('day34 model picker smoke: PASS');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
