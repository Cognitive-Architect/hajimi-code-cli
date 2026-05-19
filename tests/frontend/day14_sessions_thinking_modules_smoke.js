const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const securityDomPath = path.join(repoRoot, 'src/interface/web/modules/security-dom.js');
const sessionsPath = path.join(repoRoot, 'src/interface/web/modules/sessions.js');
const thinkingUiPath = path.join(repoRoot, 'src/interface/web/modules/thinking-ui.js');

class ClassList {
  constructor(el) {
    this.el = el;
  }

  _items() {
    return new Set((this.el.className || '').split(/\s+/).filter(Boolean));
  }

  contains(name) {
    return this._items().has(name);
  }

  add(name) {
    const items = this._items();
    items.add(name);
    this.el.className = Array.from(items).join(' ');
  }

  remove(name) {
    const items = this._items();
    items.delete(name);
    this.el.className = Array.from(items).join(' ');
  }
}

class Element {
  constructor(tagName) {
    this.tagName = tagName.toUpperCase();
    this.children = [];
    this.parentNode = null;
    this.style = {};
    this.dataset = {};
    this.listeners = {};
    this.className = '';
    this.id = '';
    this.title = '';
    this._textContent = '';
    this.scrollTop = 0;
    this.scrollHeight = 0;
    this._innerHTML = '';
    this.classList = new ClassList(this);
  }

  set innerHTML(value) {
    this._innerHTML = String(value || '');
    this._textContent = '';
    this.children = [];
    this._hydrateCommonChildren(this._innerHTML);
  }

  get innerHTML() {
    return this._innerHTML || escapeHtmlForDom(this._textContent);
  }

  set textContent(value) {
    this._textContent = String(value ?? '');
    this._innerHTML = '';
    this.children = [];
  }

  get textContent() {
    return this._textContent;
  }

  appendChild(child) {
    child.parentNode = this;
    this.children.push(child);
    return child;
  }

  insertBefore(child, ref) {
    child.parentNode = this;
    const idx = ref ? this.children.indexOf(ref) : -1;
    if (idx === -1) this.children.unshift(child);
    else this.children.splice(idx, 0, child);
    return child;
  }

  remove() {
    if (!this.parentNode) return;
    this.parentNode.children = this.parentNode.children.filter(child => child !== this);
    this.parentNode = null;
  }

  addEventListener(type, handler) {
    this.listeners[type] = this.listeners[type] || [];
    this.listeners[type].push(handler);
  }

  click() {
    for (const handler of this.listeners.click || []) {
      handler({ target: this, stopPropagation() {} });
    }
  }

  querySelector(selector) {
    if (selector === '.chat-message.ai:last-child') {
      return this.children.slice().reverse().find(child => {
        const classes = (child.className || '').split(/\s+/);
        return classes.includes('chat-message') && classes.includes('ai');
      }) || null;
    }

    if (selector.includes('[data-tab="trace"]')) {
      return findFirst(this, el => el.classList.contains('trace-tab') && el.dataset.tab === 'trace');
    }

    if (selector.startsWith('.')) {
      const className = selector.slice(1).split(':')[0];
      return findFirst(this, el => el.classList.contains(className));
    }

    return null;
  }

  querySelectorAll(selector) {
    if (!selector.startsWith('.')) return [];
    const className = selector.slice(1).split(':')[0];
    const out = [];
    collect(this, el => el.classList.contains(className), out);
    return out;
  }

  _hydrateCommonChildren(html) {
    const classMatches = html.matchAll(/class="([^"]+)"/g);
    for (const match of classMatches) {
      const child = new Element('div');
      child.className = match[1];
      this.appendChild(child);
    }

    const sessionMatches = html.matchAll(/class="session-item[^"]*" data-session="([^"]+)"/g);
    for (const match of sessionMatches) {
      const child = new Element('div');
      child.className = 'session-item';
      child.dataset.session = match[1];
      this.appendChild(child);
    }

    const details = this.querySelector('.operation-summary-details');
    if (details) details.dataset.lazy = 'true';
  }
}

function escapeHtmlForDom(value) {
  return String(value ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function findFirst(root, predicate) {
  for (const child of root.children) {
    if (predicate(child)) return child;
    const nested = findFirst(child, predicate);
    if (nested) return nested;
  }
  return null;
}

function collect(root, predicate, out) {
  for (const child of root.children) {
    if (predicate(child)) out.push(child);
    collect(child, predicate, out);
  }
}

function createDocument() {
  const nodes = new Map();
  return {
    createElement: tagName => new Element(tagName),
    getElementById(id) {
      if (!nodes.has(id)) {
        const node = new Element('div');
        node.id = id;
        nodes.set(id, node);
      }
      return nodes.get(id);
    },
    querySelector(selector) {
      for (const node of nodes.values()) {
        const found = node.querySelector(selector);
        if (found) return found;
      }
      return null;
    },
  };
}

function createLocalStorage() {
  const data = new Map();
  return {
    getItem(key) {
      return data.has(key) ? data.get(key) : null;
    },
    setItem(key, value) {
      data.set(key, String(value));
    },
    removeItem(key) {
      data.delete(key);
    },
    clear() {
      data.clear();
    },
  };
}

const document = createDocument();
const localStorage = createLocalStorage();
const invokeCalls = [];
const channels = [];

class Channel {
  constructor() {
    channels.push(this);
    this.onmessage = null;
  }
}

const context = {
  console,
  document,
  localStorage,
  requestAnimationFrame(fn) {
    fn();
    return 1;
  },
  cancelAnimationFrame() {},
  confirm() {
    return true;
  },
  setTimeout(fn) {
    fn();
    return 1;
  },
  clearTimeout() {},
  __TAURI__: {
    core: {
      Channel,
      invoke(cmd, args) {
        invokeCalls.push({ cmd, args });
        return Promise.resolve();
      },
    },
  },
};
context.window = context;
context.globalThis = context;
vm.createContext(context);

for (const file of [securityDomPath, sessionsPath, thinkingUiPath]) {
  vm.runInContext(fs.readFileSync(file, 'utf8'), context, { filename: file });
}

function createApp() {
  const app = {
    chatSessions: [],
    chatMessages: [],
    activeSessionId: null,
    tokenStats: { promptTokens: 0, completionTokens: 0, estimatedTokens: 0 },
    cumulativeStats: { promptTokens: 0, completionTokens: 0, requestCount: 0 },
    traceEvents: [],
    tracePaused: false,
    sidebarView: 'agent-trace',
    replayEvents: [],
    replayIndex: -1,
    renderedMessages: [],
    currentDiffFile: null,
    safeText(value) {
      return context.HajimiSecurityDom.safeText(value);
    },
    escapeHtml(value) {
      return context.HajimiSecurityDom.escapeHtml(value);
    },
    escapeAttr(value) {
      return context.HajimiSecurityDom.escapeAttr(value);
    },
    renderMarkdown(value) {
      return this.escapeHtml(value);
    },
    addChatMessage(role, content) {
      this.renderedMessages.push({ role, content, legacy: true });
      const container = document.getElementById('aiChatMessages');
      const message = document.createElement('div');
      message.className = `chat-message ${role === 'ai' || role === 'assistant' ? 'ai' : role}`;
      message.innerHTML = `<div class="chat-message-body">${this.escapeHtml(content)}</div>`;
      container.appendChild(message);
      return message;
    },
    renderChatMessageFromSession(msg) {
      this.renderedMessages.push({
        role: msg.role,
        content: msg.content,
        restoredThinking: Object.prototype.hasOwnProperty.call(msg, 'thinkingContent'),
        thinkingContent: msg.thinkingContent || '',
        thinkingState: msg.thinkingState || '',
        thinkingElapsedMs: msg.thinkingElapsedMs || 0,
      });
      return this.addChatMessage(msg.role, msg.content);
    },
    updateTokenDisplay() {},
    safeRenderTraceInspector() {},
    onEditProposed() {},
    openDiffPreview(file) {
      this.openedDiffFile = file;
    },
    updateReplayStatus() {
      context.HajimiThinkingUI.updateReplayStatus(this);
    },
    renderReplayThinking(container, thinking) {
      context.HajimiThinkingUI.renderReplayThinking(this, container, thinking);
    },
    buildTimelineEvent(type, payload) {
      return context.HajimiThinkingUI.buildTimelineEvent(type, payload);
    },
    generateOperationReason(summary, toolName) {
      return context.HajimiThinkingUI.generateOperationReason(summary, toolName);
    },
    createOperationSummaryBar(summary, toolName) {
      return context.HajimiThinkingUI.createOperationSummaryBar(this, summary, toolName);
    },
    renderOperationDiffPreview(container, summary) {
      return context.HajimiThinkingUI.renderOperationDiffPreview(this, container, summary);
    },
    toggleDetails(bar) {
      return context.HajimiThinkingUI.toggleDetails(this, bar);
    },
    renderTraceCards() {
      return context.HajimiThinkingUI.renderTraceCards(this);
    },
    updateOperationSummary(summary, toolName) {
      return context.HajimiThinkingUI.updateOperationSummary(this, summary, toolName);
    },
    updateOperationProgress(text) {
      return context.HajimiThinkingUI.updateOperationProgress(text);
    },
    toggleThinking(block) {
      return context.HajimiThinkingUI.toggleThinking(block);
    },
    newChatSession() {
      return context.HajimiSessions.newChatSession(this);
    },
    loadChatSessions() {
      return context.HajimiSessions.loadChatSessions(this);
    },
    saveChatSessions() {
      return context.HajimiSessions.saveChatSessions(this);
    },
    switchSession(id) {
      return context.HajimiSessions.switchSession(this, id);
    },
    renderChatMessages() {
      return context.HajimiSessions.renderChatMessages(this);
    },
    renderSessionList() {
      return context.HajimiSessions.renderSessionList(this);
    },
  };
  return app;
}

assert(context.HajimiSessions, 'HajimiSessions should be mounted');
assert(context.HajimiThinkingUI, 'HajimiThinkingUI should be mounted');

const app = createApp();
context.HajimiSessions.newChatSession(app);
const firstSessionId = app.activeSessionId;
assert(firstSessionId, 'newChatSession should create an active session');

app.chatMessages = [{ role: 'user', content: 'A prompt for reload' }];
context.HajimiSessions.saveChatSessions(app);
context.HajimiSessions.newChatSession(app);
const secondSessionId = app.activeSessionId;
assert.notStrictEqual(firstSessionId, secondSessionId, 'A/B sessions should have different ids');

context.HajimiSessions.switchSession(app, firstSessionId);
assert.strictEqual(app.activeSessionId, firstSessionId, 'switchSession should restore the target session');
assert.strictEqual(app.chatMessages[0].content, 'A prompt for reload');

const restored = createApp();
context.HajimiSessions.loadChatSessions(restored);
assert(restored.activeSessionId, 'loadChatSessions should restore an active session');
assert(localStorage.getItem('hajimi_chat_sessions'), 'legacy localStorage key should be used');

app.chatMessages = [{
  role: 'assistant',
  content: 'answer with restored thinking',
  thinkingContent: 'restored plan',
  thinkingState: 'done',
  thinkingElapsedMs: 2100,
  responseState: 'done',
}];
context.HajimiSessions.saveChatSessions(app);
const savedSession = JSON.parse(localStorage.getItem('hajimi_chat_sessions'))
  .find(session => session.id === app.activeSessionId);
assert(savedSession.messages[0].thinkingContent === 'restored plan', 'new assistant messages should persist thinkingContent');
assert(savedSession.messages[0].thinkingState === 'done', 'new assistant messages should persist thinkingState');
assert(savedSession.messages[0].thinkingElapsedMs === 2100, 'new assistant messages should persist thinkingElapsedMs');

localStorage.setItem('hajimi_chat_sessions', JSON.stringify([{
  id: 'compat-session',
  title: 'compat',
  preview: 'old',
  createdAt: Date.now(),
  updatedAt: Date.now(),
  messages: [
    { role: 'user', content: 'old user message', timestamp: Date.now() },
    { role: 'assistant', content: 'old assistant message', timestamp: Date.now() },
    {
      role: 'assistant',
      content: 'new assistant message',
      timestamp: Date.now(),
      thinkingContent: 'new thinking',
      thinkingState: 'done',
      thinkingElapsedMs: 1200,
      responseState: 'done',
    },
  ],
}]));
const compat = createApp();
context.HajimiSessions.loadChatSessions(compat);
assert.strictEqual(compat.activeSessionId, 'compat-session', 'compat session should load');
const restoredUser = compat.renderedMessages.find(msg => msg.content === 'old user message');
const restoredOldAssistant = compat.renderedMessages.find(msg => msg.content === 'old assistant message' && !msg.restoredThinking);
const restoredNewAssistant = compat.renderedMessages.find(msg => msg.content === 'new assistant message' && msg.restoredThinking);
assert(restoredUser, 'old user message should render');
assert(restoredOldAssistant, 'old assistant message without thinking fields should render as legacy');
assert(restoredNewAssistant, 'new assistant message with thinking fields should use restore branch');
assert.strictEqual(restoredNewAssistant.thinkingContent, 'new thinking', 'restored thinkingContent should be preserved');

const parsed = context.HajimiThinkingUI.parseThinkingStream('<thinking>plan</thinking><response>done</response>');
assert.strictEqual(parsed.thinking, 'plan');
assert.strictEqual(parsed.response, 'done');
assert.strictEqual(parsed.state, 'response');

const thinkingPanel = context.HajimiThinkingUI.createThinkingPanel(app, { state: 'empty', collapsed: true });
assert.strictEqual(thinkingPanel.root.dataset.state, 'empty', 'thinking panel should start in empty state');
assert.strictEqual(thinkingPanel.root.dataset.collapsed, 'true', 'thinking panel should start collapsed');
assert.strictEqual(thinkingPanel.title.textContent, '未返回显式思考过程', 'empty state should be explicit');
thinkingPanel.header.click();
assert.strictEqual(thinkingPanel.root.dataset.collapsed, 'false', 'thinking panel header should expand panel');
context.HajimiThinkingUI.setThinkingContent(thinkingPanel, '<img src=x onerror=alert(1)>');
assert.strictEqual(thinkingPanel.root.dataset.state, 'thinking', 'content should move panel to thinking state');
assert(thinkingPanel.content.innerHTML.includes('&lt;img'), 'thinking content should be escaped');
assert(!thinkingPanel.content.innerHTML.includes('<img'), 'thinking content must not render raw image tags');
assert(!thinkingPanel.content.innerHTML.includes('onerror=alert(1)>'), 'thinking content must not render raw event attributes');
context.HajimiThinkingUI.setThinkingState(thinkingPanel, 'done', { elapsedMs: 2100 });
assert.strictEqual(thinkingPanel.root.dataset.state, 'done', 'thinking panel should support done state');
assert.strictEqual(thinkingPanel.title.textContent, '已思考');
context.HajimiThinkingUI.setThinkingState(thinkingPanel, 'error');
assert.strictEqual(thinkingPanel.root.dataset.state, 'error', 'thinking panel should support error state');

app.traceEvents = [{
  step_type: 'Plan',
  step: '<bad>',
  details: '<script>alert(1)</script>',
  iteration: 7,
  plan_summary: '<b>plan</b>',
}];
context.HajimiThinkingUI.renderTraceCards(app);
const traceHtml = document.getElementById('tracePanel').innerHTML;
assert(traceHtml.includes('&lt;bad&gt;'), 'trace card should escape step text');
assert(traceHtml.includes('&lt;script&gt;alert(1)&lt;/script&gt;'), 'trace card should escape details');
assert(!traceHtml.includes('<script>alert(1)</script>'), 'trace card must not render raw script tags');

const summary = {
  files_edited: 1,
  files_created: 1,
  commands_run: 1,
  total_diff_lines: 3,
  files: [{ path: 'src/example.js', status: 'modified' }],
};
const bar = context.HajimiThinkingUI.createOperationSummaryBar(app, summary, 'edit_file');
assert(bar, 'operation summary bar should be created for non-empty summary');
assert(bar.innerHTML.includes('已编辑 1 个文件'), 'operation summary should report edited files');
const preview = document.createElement('div');
context.HajimiThinkingUI.renderOperationDiffPreview(app, preview, summary);
assert(preview.innerHTML.includes('src/example.js'), 'diff preview should render file paths from operation_summary');

context.HajimiThinkingUI.startTraceSubscription(app);
assert.strictEqual(invokeCalls.at(-1).cmd, 'subscribe_agent_trace', 'trace subscription should use real Tauri command name');
assert.strictEqual(channels.length, 1, 'trace subscription should create a Tauri Channel');

context.HajimiThinkingUI.startSessionReplay(app, [{ checkpoint_id: 'cp-1', step_type: 'Checkpoint', summary: 'saved' }], 0);
assert.strictEqual(app.replayEvents[0].source, 'edit_history+checkpoint');
context.HajimiThinkingUI.replayStep(app, 0);
assert(document.getElementById('tracePanel').innerHTML || document.getElementById('tracePanel').children.length > 0, 'replay should render into trace panel');

app.traceEvents = [{ thinking_content: 'thinking' }, { operation_summary: summary }, { tool_name: 'read_file' }];
assert.strictEqual(context.HajimiThinkingUI.getTimelineEvents(app, 'thinking').length, 1);
assert.strictEqual(context.HajimiThinkingUI.getTimelineEvents(app, 'action').length, 2);

console.log('day14 sessions/thinking modules smoke: PASS');
