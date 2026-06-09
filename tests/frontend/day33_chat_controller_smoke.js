const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const chatControllerPath = path.join(repoRoot, 'src/interface/web/controllers/chat-controller.js');

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
    this.style = {};
    this.listeners = {};
    this.scrollHeight = 100;
  }

  addEventListener(type, handler) {
    this.listeners[type] = handler;
  }

  dispatchEvent(event) {
    if (this.listeners[event.type]) {
      this.listeners[event.type](event);
    }
  }

  trigger(type, eventData = {}) {
    if (this.listeners[type]) {
      this.listeners[type]({
        preventDefault() {},
        ...eventData
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

class FakeEvent {
  constructor(type) {
    this.type = type;
  }
}

async function main() {
  const document = new FakeDocument();
  const context = { window: {}, document, console, Event: FakeEvent };
  context.window = context;
  context.globalThis = context;
  vm.createContext(context);

  // Load chat-controller
  vm.runInContext(fs.readFileSync(chatControllerPath, 'utf8'), context, { filename: 'chat-controller.js' });

  // 1. Verify mount
  assert.ok(context.HajimiChatController, 'HajimiChatController should be mounted');

  // 2. Setup mock elements
  const chatInput = new FakeElement(document, 'aiChatInput');
  const chatSendBtn = new FakeElement(document, 'aiChatSendBtn');
  const addContextBtn = new FakeElement(document, 'addContextBtn');
  const clearContextBtn = new FakeElement(document, 'clearContextBtn');
  const editModeBtn = new FakeElement(document, 'editModeBtn');
  const newChatBtn = new FakeElement(document, 'newChatBtn');

  document.register('aiChatInput', chatInput);
  document.register('aiChatSendBtn', chatSendBtn);
  document.register('addContextBtn', addContextBtn);
  document.register('clearContextBtn', clearContextBtn);
  document.register('editModeBtn', editModeBtn);
  document.register('newChatBtn', newChatBtn);

  let sendChatMessageCalls = 0;
  let showSidebarValue = null;
  let addChatMessageCalls = [];
  let clearChatContextCalls = 0;
  let newChatSessionCalls = 0;

  const app = {
    sendChatMessage() { sendChatMessageCalls++; },
    showSidebar(val) { showSidebarValue = val; },
    addChatMessage(role, text) { addChatMessageCalls.push({ role, text }); },
    clearChatContext() { clearChatContextCalls++; },
    newChatSession() { newChatSessionCalls++; },
    updateTokenDisplay() {}
  };

  // 3. Initialize Controller
  context.HajimiChatController.init(app);

  // 4. Trigger events and verify delegation
  chatInput.trigger('input');
  assert.ok(chatInput.style.height, 'input listener should adjust input element height');

  chatInput.trigger('keydown', { key: 'Enter', shiftKey: false });
  assert.strictEqual(sendChatMessageCalls, 1, 'Enter key should invoke sendChatMessage');

  chatSendBtn.trigger('click');
  assert.strictEqual(sendChatMessageCalls, 2, 'Clicking send button should invoke sendChatMessage');

  addContextBtn.trigger('click');
  assert.strictEqual(showSidebarValue, 'explorer', 'addContextBtn should navigate to explorer');
  assert.ok(addChatMessageCalls.length > 0, 'addContextBtn should prompt hint message');

  clearContextBtn.trigger('click');
  assert.strictEqual(clearChatContextCalls, 1, 'clearContextBtn should clear chat context');

  editModeBtn.trigger('click');
  assert.strictEqual(addChatMessageCalls[1].role, 'ai');

  newChatBtn.trigger('click');
  assert.strictEqual(newChatSessionCalls, 1, 'newChatBtn should start new session');

  console.log('day33 chat controller smoke: PASS');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
