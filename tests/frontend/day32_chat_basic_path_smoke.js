const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const appPath = path.join(repoRoot, 'src/interface/web/app.js');
const indexPath = path.join(repoRoot, 'src/interface/web/index.html');
const chatControllerPath = path.join(repoRoot, 'src/interface/web/controllers/chat-controller.js');
const chatViewPath = path.join(repoRoot, 'src/interface/web/views/chat-view.js');

class FakeClassList {
  constructor() {
    this.values = new Set();
  }
  add(name) { this.values.add(name); }
  remove(name) { this.values.delete(name); }
  contains(name) { return this.values.has(name); }
  toggle(name, force) {
    if (force === undefined) {
      if (this.values.has(name)) this.values.delete(name);
      else this.values.add(name);
      return;
    }
    if (force) this.values.add(name);
    else this.values.delete(name);
  }
}

class FakeElement {
  constructor(document, id = null) {
    this.document = document;
    this.id = id;
    this.classList = new FakeClassList();
    this.dataset = {};
    this.textContent = '';
    this.innerHTML = '';
    this.value = '';
    this.disabled = false;
    this.style = {};
    this.listeners = {};
    this.scrollHeight = 120;
    this.focusCalls = 0;
  }

  addEventListener(type, handler) {
    if (!this.listeners[type]) this.listeners[type] = [];
    this.listeners[type].push(handler);
  }

  dispatchEvent(event) {
    for (const handler of this.listeners[event.type] || []) {
      handler(event);
    }
  }

  trigger(type, eventData = {}) {
    const event = {
      type,
      defaultPrevented: false,
      preventDefault() {
        this.defaultPrevented = true;
      },
      ...eventData,
    };
    this.dispatchEvent(event);
    return event;
  }

  focus() {
    this.focusCalls += 1;
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

  register(id) {
    const el = new FakeElement(this, id);
    this.byId.set(id, el);
    return el;
  }
}

class FakeEvent {
  constructor(type) {
    this.type = type;
  }
}

function assertScriptOrder() {
  const html = fs.readFileSync(indexPath, 'utf8');
  const chatViewIndex = html.indexOf('views/chat-view.js');
  const chatControllerIndex = html.indexOf('controllers/chat-controller.js');
  const appIndex = html.indexOf('app.js');
  assert.ok(chatViewIndex !== -1, 'index.html should load views/chat-view.js');
  assert.ok(chatControllerIndex !== -1, 'index.html should load controllers/chat-controller.js');
  assert.ok(appIndex !== -1, 'index.html should load app.js');
  assert.ok(chatViewIndex < appIndex, 'chat-view.js should load before app.js');
  assert.ok(chatControllerIndex < appIndex, 'chat-controller.js should load before app.js');
}

function assertAppDelegation() {
  const appSource = fs.readFileSync(appPath, 'utf8');
  const setupChatIndex = appSource.indexOf('setupChat()');
  const delegateIndex = appSource.indexOf('window.HajimiChatController', setupChatIndex);
  const fallbackInputIndex = appSource.indexOf("document.getElementById('aiChatInput')", setupChatIndex);
  assert.ok(setupChatIndex !== -1, 'app.js should still define setupChat() wrapper');
  assert.ok(delegateIndex !== -1, 'setupChat() should delegate to HajimiChatController');
  assert.ok(fallbackInputIndex !== -1, 'setupChat() fallback should still exist');
  assert.ok(delegateIndex < fallbackInputIndex, 'HajimiChatController delegation should happen before inline fallback');
}

function assertControllerIsBasicOnly() {
  const controllerSource = fs.readFileSync(chatControllerPath, 'utf8');
  for (const forbidden of ['streamChat', 'invokeTauri', 'getTauriInvoke', 'providerConfigs', 'activeProviderId']) {
    assert.ok(!controllerSource.includes(forbidden), `chat-controller.js should not touch ${forbidden}`);
  }
}

async function main() {
  assertScriptOrder();
  assertAppDelegation();
  assertControllerIsBasicOnly();

  const document = new FakeDocument();
  const context = { window: {}, document, console, Event: FakeEvent, setTimeout };
  context.window = context;
  context.globalThis = context;
  vm.createContext(context);

  vm.runInContext(fs.readFileSync(chatViewPath, 'utf8'), context, { filename: 'chat-view.js' });
  vm.runInContext(fs.readFileSync(chatControllerPath, 'utf8'), context, { filename: 'chat-controller.js' });

  assert.ok(context.HajimiChatView, 'HajimiChatView should be mounted');
  assert.ok(context.HajimiChatController, 'HajimiChatController should be mounted');

  const chatInput = document.register('aiChatInput');
  const chatSendBtn = document.register('aiChatSendBtn');
  document.register('slashPalette');
  const modelSelectBtn = document.register('modelSelectBtn');
  const addContextBtn = document.register('addContextBtn');
  const clearContextBtn = document.register('clearContextBtn');
  const editModeBtn = document.register('editModeBtn');
  const newChatBtn = document.register('newChatBtn');
  const newSessionBtn = document.register('newSessionBtn');

  let slashHandleInputCalls = 0;
  let slashSelectedConfig = null;
  context.HajimiSlashPalette = {
    createSlashPalette(config) {
      slashSelectedConfig = config;
      return {
        handleInput() { slashHandleInputCalls += 1; },
        handleKeyDown() { return false; },
        isOpen() { return false; },
        close() {},
      };
    },
  };

  let sendCalls = 0;
  let modelPickerCalls = 0;
  let explorerCalls = 0;
  let clearContextCalls = 0;
  let newSessionCalls = 0;
  const chatMessages = [];
  const app = {
    getSlashCommands() {
      return [{ trigger: '/tools', executeMode: 'direct', riskLevel: 'low' }];
    },
    sendChatMessage() { sendCalls += 1; },
    openModelPicker() { modelPickerCalls += 1; },
    showSidebar(area) {
      if (area === 'explorer') explorerCalls += 1;
    },
    addChatMessage(role, text) { chatMessages.push({ role, text }); },
    clearChatContext() { clearContextCalls += 1; },
    newChatSession() { newSessionCalls += 1; },
    updateTokenDisplay() {},
    streamChat() { throw new Error('streamChat must not be called by chat basic shell smoke'); },
    invokeTauri() { throw new Error('invokeTauri must not be called by chat basic shell smoke'); },
  };

  context.HajimiChatController.init(app);

  chatInput.value = 'hello';
  chatInput.trigger('input');
  assert.ok(chatInput.style.height.endsWith('px'), 'input listener should resize the chat input');
  assert.strictEqual(slashHandleInputCalls, 1, 'input listener should notify slash palette when present');

  const shiftEnter = chatInput.trigger('keydown', { key: 'Enter', shiftKey: true });
  assert.strictEqual(shiftEnter.defaultPrevented, false, 'Shift+Enter should not submit');
  assert.strictEqual(sendCalls, 0, 'Shift+Enter should not call sendChatMessage');

  const enter = chatInput.trigger('keydown', { key: 'Enter', shiftKey: false });
  assert.strictEqual(enter.defaultPrevented, true, 'Enter should prevent default newline');
  assert.strictEqual(sendCalls, 1, 'Enter should call app.sendChatMessage once');

  chatSendBtn.trigger('click');
  assert.strictEqual(sendCalls, 2, 'send button should call app.sendChatMessage');

  modelSelectBtn.trigger('click');
  assert.strictEqual(modelPickerCalls, 1, 'model picker button should delegate to app.openModelPicker');

  addContextBtn.trigger('click');
  assert.strictEqual(explorerCalls, 1, 'add context button should show explorer');
  assert.ok(chatMessages.some((msg) => msg.role === 'ai'), 'add context button should add a helper message');

  clearContextBtn.trigger('click');
  assert.strictEqual(clearContextCalls, 1, 'clear context button should delegate to app.clearChatContext');

  editModeBtn.trigger('click');
  assert.ok(chatMessages.length >= 2, 'edit mode button should add an AI helper message');

  newChatBtn.trigger('click');
  newSessionBtn.trigger('click');
  assert.strictEqual(newSessionCalls, 2, 'session buttons should delegate to app.newChatSession');

  slashSelectedConfig.onSelect({ trigger: '/tools', executeMode: 'direct', riskLevel: 'low' });
  assert.strictEqual(chatInput.value, '/tools', 'low-risk slash select should fill chat input');
  assert.strictEqual(sendCalls, 3, 'low-risk direct slash select should only call app.sendChatMessage');

  slashSelectedConfig.onSelect({ trigger: '/agent', executeMode: 'fill', riskLevel: 'high' });
  assert.strictEqual(chatInput.value, '/agent', 'high-risk slash select should fill chat input');
  assert.strictEqual(sendCalls, 3, 'high-risk slash select should not auto-submit');

  console.log('day32 chat basic path smoke: PASS');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
