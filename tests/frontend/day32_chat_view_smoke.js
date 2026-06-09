const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const chatViewPath = path.join(repoRoot, 'src/interface/web/views/chat-view.js');
const thinkingUiPath = path.join(repoRoot, 'src/interface/web/modules/thinking-ui.js');

class FakeClassList {
  constructor() {
    this.values = new Set();
  }
  add(name) { this.values.add(name); }
  remove(name) { this.values.delete(name); }
  contains(name) { return this.values.has(name); }
  toggle(name, val) {
    if (val === undefined) {
      if (this.values.has(name)) this.values.delete(name);
      else this.values.add(name);
    } else {
      if (val) this.values.add(name);
      else this.values.delete(name);
    }
  }
}

class FakeElement {
  constructor(document, id = null) {
    this.document = document;
    this.id = id;
    this.classList = new FakeClassList();
    this.textContent = '';
    this.innerHTML = '';
    this.dataset = {};
    this.style = {};
    this.scrollTop = 0;
    this.scrollHeight = 100;
    this.listeners = {};
  }

  addEventListener(type, handler) {
    this.listeners[type] = handler;
  }

  appendChild(el) {
    return el;
  }

  querySelectorAll(selector) {
    return [];
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
  const context = { window: {}, document, console, setTimeout };
  context.window = context;
  context.globalThis = context;
  vm.createContext(context);

  // Load thinking-ui
  vm.runInContext(fs.readFileSync(thinkingUiPath, 'utf8'), context, { filename: 'thinking-ui.js' });
  // Load chat-view
  vm.runInContext(fs.readFileSync(chatViewPath, 'utf8'), context, { filename: 'chat-view.js' });

  // 1. Verify mount
  assert.ok(context.HajimiChatView, 'HajimiChatView should be mounted');

  // 2. Setup mock container
  const container = new FakeElement(document, 'aiChatMessages');
  document.register('aiChatMessages', container);

  const app = {
    safeText(t) { return t; },
    formatText(t) { return `Formatted: ${t}`; }
  };

  // 3. Test createAssistantTurn
  const turn = context.HajimiChatView.createAssistantTurn(app);
  assert.ok(turn, 'createAssistantTurn should return a turn object');
  assert.ok(turn.id, 'turn should have an ID');
  assert.ok(turn.root, 'turn should have root element');
  assert.ok(turn.responseEl, 'turn should have responseEl');

  // 4. Test hasSessionThinking
  assert.strictEqual(context.HajimiChatView.hasSessionThinking({ role: 'user' }), false);
  assert.strictEqual(context.HajimiChatView.hasSessionThinking({ role: 'assistant', thinkingContent: '...' }), true);

  // 5. Test snapshotAssistantTurn
  const snapshot = context.HajimiChatView.snapshotAssistantTurn(app, turn);
  assert.strictEqual(snapshot.thinkingState, 'empty');
  assert.strictEqual(snapshot.responseState, 'pending');

  // 6. Test createAssistantSessionMessage
  const sessionMsg = context.HajimiChatView.createAssistantSessionMessage(app, 'hello', turn);
  assert.strictEqual(sessionMsg.role, 'assistant');
  assert.strictEqual(sessionMsg.content, 'hello');
  assert.strictEqual(sessionMsg.thinkingState, 'empty');

  // 7. Test updateTurnThinking
  context.HajimiChatView.updateTurnThinking(app, turn, { content: 'my thoughts', state: 'done' });
  assert.strictEqual(turn.state.thinking.content, 'my thoughts');
  assert.strictEqual(turn.state.thinking.state, 'done');

  // 8. Test updateTurnResponse
  context.HajimiChatView.updateTurnResponse(app, turn, { state: 'done', content: 'final response' });
  assert.strictEqual(turn.state.response.state, 'done');
  assert.strictEqual(turn.state.response.content, 'final response');
  assert.ok(turn.responseEl.innerHTML.includes('Formatted: final response'));

  // 9. Test addChatMessage
  context.HajimiChatView.addChatMessage(app, 'user', 'hello user');
  // It shouldn't crash and should call app.formatText

  console.log('day32 chat view smoke: PASS');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
