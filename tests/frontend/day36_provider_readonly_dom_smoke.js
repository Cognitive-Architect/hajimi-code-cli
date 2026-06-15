const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const indexPath = path.join(repoRoot, 'src/interface/web/index.html');
const providerServicePath = path.join(repoRoot, 'src/interface/web/services/provider-service.js');
const providerViewPath = path.join(repoRoot, 'src/interface/web/views/provider-view.js');
const providerControllerPath = path.join(repoRoot, 'src/interface/web/controllers/provider-controller.js');

class FakeElement {
  constructor(document, tagName = 'div', id = null) {
    this.document = document;
    this.tagName = tagName.toUpperCase();
    this.id = id;
    this.className = '';
    this.dataset = {};
    this.children = [];
    this.parentNode = null;
    this._textContent = '';
    this.title = '';
    this.style = {};
  }

  appendChild(child) {
    child.parentNode = this;
    this.children.push(child);
    return child;
  }

  removeChild(child) {
    const index = this.children.indexOf(child);
    if (index >= 0) this.children.splice(index, 1);
    child.parentNode = null;
    return child;
  }

  replaceChildren(...children) {
    this.children.forEach((child) => { child.parentNode = null; });
    this.children = [];
    children.forEach((child) => this.appendChild(child));
  }

  get firstChild() {
    return this.children[0] || null;
  }

  set textContent(value) {
    this._textContent = value == null ? '' : String(value);
    this.children = [];
  }

  get textContent() {
    return this._textContent + this.children.map((child) => child.textContent).join('');
  }

  querySelectorAll(selector) {
    const results = [];
    const visit = (node) => {
      if (matchesSelector(node, selector)) results.push(node);
      node.children.forEach(visit);
    };
    this.children.forEach(visit);
    return results;
  }
}

function matchesSelector(node, selector) {
  if (selector.startsWith('.')) {
    return node.className.split(/\s+/).includes(selector.slice(1));
  }
  if (selector.startsWith('[data-') && selector.endsWith(']')) {
    const key = selector.slice(6, -1).replace(/-([a-z])/g, (_, c) => c.toUpperCase());
    return Object.prototype.hasOwnProperty.call(node.dataset, key);
  }
  return false;
}

class FakeDocument {
  constructor() {
    this.byId = new Map();
  }

  createElement(tagName) {
    return new FakeElement(this, tagName);
  }

  getElementById(id) {
    return this.byId.get(id) || null;
  }

  register(id, tagName = 'div') {
    const el = new FakeElement(this, tagName, id);
    this.byId.set(id, el);
    return el;
  }
}

function loadProviderModules(document) {
  const context = {
    window: { document },
    document,
    module: { exports: {} },
    exports: {},
    console,
  };
  context.globalThis = context;
  vm.createContext(context);

  vm.runInContext(fs.readFileSync(providerServicePath, 'utf8'), context, { filename: 'provider-service.js' });
  const serviceExports = context.module.exports;
  context.module = { exports: {} };
  context.exports = {};

  vm.runInContext(fs.readFileSync(providerViewPath, 'utf8'), context, { filename: 'provider-view.js' });
  const viewExports = context.module.exports;
  context.module = { exports: {} };
  context.exports = {};

  vm.runInContext(fs.readFileSync(providerControllerPath, 'utf8'), context, { filename: 'provider-controller.js' });
  const controllerExports = context.module.exports;

  return { context, serviceExports, viewExports, controllerExports };
}

function createForbiddenSpy(name, calls) {
  return function forbiddenProviderCall() {
    calls.push(name);
    throw new Error(`${name} must not be called by Provider readonly smoke`);
  };
}

function assertIndexHasProviderShell() {
  const html = fs.readFileSync(indexPath, 'utf8');
  [
    'providerListTab',
    'providerModal',
    'providerForm',
    'providerName',
    'providerApiKey',
    'saveProvider',
    'testProviderBtn',
    'backupModal',
  ].forEach((id) => {
    assert.ok(html.includes(`id="${id}"`), `index.html should contain #${id}`);
  });
}

async function main() {
  assertIndexHasProviderShell();

  const sourceText = [
    fs.readFileSync(providerServicePath, 'utf8'),
    fs.readFileSync(providerViewPath, 'utf8'),
    fs.readFileSync(providerControllerPath, 'utf8'),
  ].join('\n');
  assert.ok(!sourceText.includes('invokeTauri'), 'readonly provider modules must not call invokeTauri');
  assert.ok(!sourceText.includes('saveProviderConfig'), 'readonly provider modules must not call saveProviderConfig');
  assert.ok(!sourceText.includes('deleteProviderConfig'), 'readonly provider modules must not call deleteProviderConfig');
  assert.ok(!sourceText.includes('probe_provider_context_capacity'), 'readonly provider modules must not call provider probe');
  assert.ok(!sourceText.includes('validate_provider'), 'readonly provider modules must not call provider validation');
  assert.ok(!sourceText.includes('keyring'), 'readonly provider modules must not touch keyring text/path');
  assert.ok(!fs.readFileSync(providerViewPath, 'utf8').includes('innerHTML'), 'provider view should use safe DOM, not innerHTML');

  {
    const document = new FakeDocument();
    const providerList = document.register('providerListTab');
    const { context, serviceExports, viewExports, controllerExports } = loadProviderModules(document);

    assert.strictEqual(typeof context.window.HajimiProviderService.normalizeProviderConfigs, 'function', 'service global should mount');
    assert.strictEqual(typeof context.window.HajimiProviderView.renderProviderListReadOnly, 'function', 'view global should mount');
    assert.strictEqual(typeof context.window.HajimiProviderController.renderProviderListReadOnly, 'function', 'controller global should mount');
    assert.strictEqual(typeof serviceExports.formatProviderMeta, 'function', 'service should support Node module exports');
    assert.strictEqual(typeof viewExports.renderProviderListReadOnly, 'function', 'view should support Node module exports');
    assert.strictEqual(typeof controllerExports.renderProviderListReadOnly, 'function', 'controller should support Node module exports');

    const forbiddenCalls = [];
    const app = {
      currentWorkspace: 'F:\\demo-workspace',
      providerConfigs: [
        {
          id: 'deepseek',
          name: 'DeepSeek',
          model: 'deepseek-chat',
          providerType: 'openai-compatible',
          baseUrl: 'https://api.deepseek.com/v1',
          hasApiKey: true,
          apiKey: 'SHOULD_NOT_RENDER',
        },
        {
          id: 'evil',
          name: '<img src=x onerror="globalThis.__providerXss=1">',
          model: '<script>globalThis.__providerXss=1</script>',
          baseUrl: 'https://example.test',
          has_api_key: false,
        },
      ],
      saveProviderConfig: createForbiddenSpy('saveProviderConfig', forbiddenCalls),
      deleteProviderConfig: createForbiddenSpy('deleteProviderConfig', forbiddenCalls),
      exportProviderBackup: createForbiddenSpy('exportProviderBackup', forbiddenCalls),
      importProviderBackup: createForbiddenSpy('importProviderBackup', forbiddenCalls),
      openBackupModal: createForbiddenSpy('openBackupModal', forbiddenCalls),
      invokeTauri: createForbiddenSpy('invokeTauri', forbiddenCalls),
      openProviderModal: createForbiddenSpy('openProviderModal', forbiddenCalls),
    };

    const result = context.window.HajimiProviderController.renderProviderListReadOnly(app);
    assert.strictEqual(result.rendered, true, 'controller should render provider list');
    assert.strictEqual(result.count, 2, 'controller should render two readonly providers');
    assert.strictEqual(result.readonly, true, 'controller result should be readonly');
    assert.ok(providerList.textContent.includes('DeepSeek'), 'provider name should render');
    assert.ok(providerList.textContent.includes('deepseek-chat'), 'provider model should render');
    assert.ok(providerList.textContent.includes('API Key 已保存'), 'saved key status should render without the secret');
    assert.ok(providerList.textContent.includes('未保存 API Key'), 'missing key status should render');
    assert.ok(providerList.textContent.includes('workspace'), 'workspace source tag should render');
    assert.ok(!providerList.textContent.includes('SHOULD_NOT_RENDER'), 'API key secret must not render');
    assert.strictEqual(context.window.__providerXss, undefined, 'malicious provider strings must not execute');
    assert.strictEqual(providerList.querySelectorAll('[data-provider-edit]').length, 0, 'readonly render must not add edit buttons');
    assert.strictEqual(providerList.querySelectorAll('[data-provider-delete]').length, 0, 'readonly render must not add delete buttons');
    assert.strictEqual(forbiddenCalls.length, 0, 'readonly render must not call provider write/probe/backup/keyring paths');
  }

  {
    const document = new FakeDocument();
    const providerList = document.register('providerListTab');
    const { context } = loadProviderModules(document);
    const result = context.window.HajimiProviderController.renderProviderListReadOnly({
      currentWorkspace: '',
      providerConfigs: [],
    });
    assert.strictEqual(result.rendered, true, 'empty provider list should render');
    assert.strictEqual(result.count, 0, 'empty provider list count should be 0');
    assert.strictEqual(result.readonly, true, 'empty provider result should be readonly');
    assert.ok(providerList.textContent.includes('暂无自定义模型'), 'empty state should be readable');
    assert.ok(providerList.textContent.includes('global'), 'global source tag should render for empty workspace');
  }

  {
    const document = new FakeDocument();
    const { context } = loadProviderModules(document);
    assert.doesNotThrow(
      () => context.window.HajimiProviderController.renderProviderListReadOnly({ providerConfigs: [{ id: 'x' }] }),
      'missing providerListTab should no-op safely'
    );
  }

  console.log('day36 provider readonly dom smoke: PASS (DOM shell + read-only view/controller/service)');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
