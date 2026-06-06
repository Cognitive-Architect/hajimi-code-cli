const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const htmlPath = path.join(repoRoot, 'src/interface/web/index.html');
const securityDomPath = path.join(repoRoot, 'src/interface/web/modules/security-dom.js');
const sessionsPath = path.join(repoRoot, 'src/interface/web/modules/sessions.js');

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
}

class Element {
  constructor(document, id = '', tagName = 'div') {
    this.document = document;
    this.id = id;
    this.tagName = tagName.toUpperCase();
    this.children = [];
    this.listeners = {};
    this.dataset = {};
    this.className = '';
    this._innerHTML = '';
    this._textContent = '';
    this.classList = new ClassList(this);
  }

  set innerHTML(value) {
    this._innerHTML = String(value || '');
    this._textContent = '';
    this.children = [];
    if (this.id === 'sessionList') {
      this._hydrateSessionItems();
    }
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

  addEventListener(type, handler) {
    this.listeners[type] = this.listeners[type] || [];
    this.listeners[type].push(handler);
  }

  click() {
    for (const handler of this.listeners.click || []) {
      handler({ target: this });
    }
  }

  querySelectorAll(selector) {
    if (selector === '.session-item') {
      return this.children.filter((child) => child.classList.contains('session-item'));
    }
    return [];
  }

  _hydrateSessionItems() {
    const itemRe = /<div class="([^"]*session-item[^"]*)" data-session="([^"]+)">([\s\S]*?)<\/div>\s*<\/div>/g;
    let match;
    while ((match = itemRe.exec(this._innerHTML))) {
      const [, className, sessionId, body] = match;
      const item = new Element(this.document, '', 'div');
      item.className = className;
      item.dataset.session = decodeHtmlAttr(sessionId);
      item._innerHTML = body;
      this.children.push(item);
    }
  }
}

function decodeHtmlAttr(value) {
  return String(value)
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&amp;/g, '&');
}

function escapeHtmlForDom(value) {
  return String(value ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function createDocument() {
  const nodes = new Map();
  const document = {
    nodes,
    register(id, tagName = 'div') {
      const node = new Element(document, id, tagName);
      nodes.set(id, node);
      return node;
    },
    createElement(tagName) {
      return new Element(document, '', tagName);
    },
    getElementById(id) {
      return nodes.get(id) || null;
    },
  };

  document.register('sessionList');
  document.register('aiChatMessages');
  return document;
}

function createLocalStorage() {
  const values = new Map();
  return {
    getItem(key) {
      return values.has(key) ? values.get(key) : null;
    },
    setItem(key, value) {
      values.set(key, String(value));
    },
    removeItem(key) {
      values.delete(key);
    },
    clear() {
      values.clear();
    },
  };
}

function assertIndexHtmlHasSessionList() {
  const html = fs.readFileSync(htmlPath, 'utf8');
  assert.ok(/\bid=["']sessionList["']/.test(html), 'index.html should contain #sessionList');
  assert.ok(/class=["'][^"']*session-list/.test(html), 'index.html should keep session-list class');
}

function loadHarness() {
  const document = createDocument();
  const localStorage = createLocalStorage();
  const renderStateCalls = [];
  const renderedMessages = [];

  const context = {
    console,
    document,
    localStorage,
    Date,
    Math,
  };
  context.window = context;
  context.globalThis = context;
  vm.createContext(context);

  for (const file of [securityDomPath, sessionsPath]) {
    vm.runInContext(fs.readFileSync(file, 'utf8'), context, { filename: file });
  }

  const app = {
    chatSessions: [],
    chatMessages: [],
    activeSessionId: null,
    tokenStats: { promptTokens: 0, completionTokens: 0, estimatedTokens: 0 },
    cumulativeStats: { promptTokens: 0, completionTokens: 0, requestCount: 0 },
    escapeHtml(value) {
      return context.HajimiSecurityDom.escapeHtml(value);
    },
    escapeAttr(value) {
      return context.HajimiSecurityDom.escapeAttr(value);
    },
    renderLiveShellState(value) {
      renderStateCalls.push(value || '');
    },
    renderChatMessages() {
      renderedMessages.push(...this.chatMessages);
    },
    updateTokenDisplay() {},
    renderSessionList() {
      return context.HajimiSessions.renderSessionList(this);
    },
    saveChatSessions() {
      return context.HajimiSessions.saveChatSessions(this);
    },
    newChatSession() {
      return context.HajimiSessions.newChatSession(this);
    },
    switchSession(id) {
      return context.HajimiSessions.switchSession(this, id);
    },
  };

  return { context, document, localStorage, app, renderStateCalls, renderedMessages };
}

function main() {
  assertIndexHtmlHasSessionList();

  {
    const { context, document, app, renderStateCalls } = loadHarness();
    const sessionList = document.getElementById('sessionList');
    assert.ok(sessionList, 'test fixture should register #sessionList');

    context.HajimiSessions.renderSessionList(app);

    assert.ok(sessionList.innerHTML.includes('暂无会话'), 'empty state should render into #sessionList');
    assert.strictEqual(sessionList.querySelectorAll('.session-item').length, 0, 'empty state should not render session items');
    assert.strictEqual(renderStateCalls.length, 1, 'empty render should touch live shell state without backend session behavior');
  }

  {
    const { context, document, localStorage, app, renderedMessages } = loadHarness();
    const now = new Date(2026, 5, 6, 10, 15).getTime();
    localStorage.setItem(context.HajimiSessions.storageKey, JSON.stringify([
      {
        id: 'session-a',
        title: 'Active <script>alert(1)</script>',
        preview: 'Preview <img src=x onerror=1>',
        messages: [{ role: 'user', content: 'hello from localStorage' }],
        createdAt: now,
        updatedAt: now,
      },
      {
        id: 'session-b',
        title: 'Second session',
        preview: 'safe preview',
        messages: [],
        createdAt: now - 86400000,
        updatedAt: now - 86400000,
      },
    ]));

    context.HajimiSessions.loadChatSessions(app);

    const sessionList = document.getElementById('sessionList');
    const items = sessionList.querySelectorAll('.session-item');
    assert.strictEqual(app.activeSessionId, 'session-a', 'loadChatSessions should activate first localStorage session');
    assert.strictEqual(renderedMessages[0].content, 'hello from localStorage', 'loadChatSessions should use mock localStorage messages');
    assert.strictEqual(items.length, 2, 'session state should render two .session-item rows into #sessionList');
    assert.strictEqual(items[0].dataset.session, 'session-a', 'first rendered item should keep data-session id');
    assert.ok(items[0].classList.contains('active'), 'active session item should keep active class');
    assert.ok(sessionList.innerHTML.includes('Active &lt;script&gt;alert(1)&lt;/script&gt;'), 'session title should be escaped');
    assert.ok(sessionList.innerHTML.includes('Preview &lt;img src=x onerror=1&gt;'), 'session preview should be escaped');
    assert.ok(!sessionList.innerHTML.includes('<script>alert(1)</script>'), 'session title must not render raw script');
    assert.ok(!sessionList.innerHTML.includes('<img src=x onerror=1>'), 'session preview must not render raw image HTML');

    items[1].click();
    assert.strictEqual(app.activeSessionId, 'session-b', 'clicking a rendered item should switch to its localStorage-backed session');
  }

  console.log('day29 sessionList DOM smoke: PASS (6 scenarios)');
}

try {
  main();
} catch (error) {
  console.error(error);
  process.exit(1);
}
