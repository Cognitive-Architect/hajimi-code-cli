const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const auditLogPath = path.join(repoRoot, 'src/interface/web/modules/audit-log.js');

class Element {
  constructor(tagName, id = '') {
    this.tagName = tagName.toUpperCase();
    this.id = id;
    this.children = [];
    this.parentNode = null;
    this.listeners = {};
    this.attributes = {};
    this.className = '';
    this._textContent = '';
  }

  get firstChild() {
    return this.children[0] || null;
  }

  set textContent(value) {
    this._textContent = value == null ? '' : String(value);
    this.children = [];
  }

  get textContent() {
    return this._textContent + this.children.map(child => child.textContent).join('');
  }

  get innerHTML() {
    const attrs = [];
    if (this.className) attrs.push(`class="${escapeHtml(this.className)}"`);
    for (const [key, value] of Object.entries(this.attributes)) {
      attrs.push(`${key}="${escapeHtml(value)}"`);
    }
    const open = attrs.length ? `<${this.tagName.toLowerCase()} ${attrs.join(' ')}>` : `<${this.tagName.toLowerCase()}>`;
    const body = escapeHtml(this._textContent) + this.children.map(child => child.outerHTML).join('');
    return body || (this.tagName === 'TBODY' ? '' : `${open}</${this.tagName.toLowerCase()}>`);
  }

  get outerHTML() {
    const attrs = [];
    if (this.className) attrs.push(`class="${escapeHtml(this.className)}"`);
    for (const [key, value] of Object.entries(this.attributes)) {
      attrs.push(`${key}="${escapeHtml(value)}"`);
    }
    const open = attrs.length ? `<${this.tagName.toLowerCase()} ${attrs.join(' ')}>` : `<${this.tagName.toLowerCase()}>`;
    return `${open}${escapeHtml(this._textContent)}${this.children.map(child => child.outerHTML).join('')}</${this.tagName.toLowerCase()}>`;
  }

  appendChild(child) {
    child.parentNode = this;
    this.children.push(child);
    return child;
  }

  removeChild(child) {
    const index = this.children.indexOf(child);
    if (index >= 0) {
      this.children.splice(index, 1);
      child.parentNode = null;
    }
    return child;
  }

  setAttribute(name, value) {
    this.attributes[name] = String(value);
  }

  addEventListener(type, handler) {
    this.listeners[type] = this.listeners[type] || [];
    this.listeners[type].push(handler);
  }

  dispatchEvent(event) {
    event.target = event.target || this;
    for (const handler of this.listeners[event.type] || []) {
      handler(event);
    }
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

function createDocument(options = {}) {
  const elements = new Map();
  if (options.refresh !== false) {
    elements.set('refreshAuditBtnTab', new Element('button', 'refreshAuditBtnTab'));
  }
  if (options.body !== false) {
    elements.set('auditLogBodyTab', new Element('tbody', 'auditLogBodyTab'));
  }
  return {
    elements,
    createElement(tagName) {
      return new Element(tagName);
    },
    getElementById(id) {
      return elements.get(id) || null;
    },
  };
}

function loadAuditModule(document) {
  const context = {
    window: { document },
    document,
    module: { exports: {} },
    exports: {},
    console,
  };
  context.globalThis = context;
  vm.createContext(context);
  vm.runInContext(fs.readFileSync(auditLogPath, 'utf8'), context, { filename: 'audit-log.js' });
  assert.strictEqual(
    context.window.HajimiAuditLog?.loadAuditLogs,
    context.module.exports.loadAuditLogs,
    'browser global and module.exports should expose loadAuditLogs'
  );
  assert.strictEqual(
    context.window.HajimiAuditLog?.setupAuditLog,
    context.module.exports.setupAuditLog,
    'browser global and module.exports should expose setupAuditLog'
  );
  return context.module.exports;
}

function createApp(options = {}) {
  const calls = [];
  return {
    calls,
    isTauriAvailable: () => options.tauri !== false,
    loadAuditLogs: () => calls.push(['loadAuditLogs']),
    invokeTauri: async (command, args) => {
      calls.push(['invokeTauri', command, args]);
      return options.logs || [];
    },
  };
}

function statusSpan(row) {
  return row.children[3].children[0];
}

async function main() {
  {
    const document = createDocument();
    const { setupAuditLog } = loadAuditModule(document);
    const app = createApp();
    setupAuditLog(app);
    const refresh = document.getElementById('refreshAuditBtnTab');
    assert.strictEqual(refresh.listeners.click.length, 1, 'setupAuditLog should bind refresh click');
    refresh.dispatchEvent({ type: 'click' });
    assert.deepStrictEqual(app.calls, [['loadAuditLogs']], 'refresh click should call app.loadAuditLogs');
  }

  {
    const document = createDocument();
    const { loadAuditLogs } = loadAuditModule(document);
    const app = createApp({ tauri: false, logs: [{ providerName: 'openai' }] });
    await loadAuditLogs(app);
    assert.deepStrictEqual(app.calls, [], 'Tauri unavailable should return without invoking backend');
  }

  {
    const document = createDocument();
    const { loadAuditLogs } = loadAuditModule(document);
    await loadAuditLogs(createApp({ logs: [] }));
    const tbody = document.getElementById('auditLogBodyTab');
    assert.strictEqual(tbody.children.length, 1, 'empty logs should render one row');
    assert.ok(tbody.innerHTML.includes('暂无记录'), 'empty logs should render no-records text');
    assert.strictEqual(tbody.children[0].children[0].className, 'audit-empty', 'empty cell should keep class');
  }

  {
    const document = createDocument();
    const { loadAuditLogs } = loadAuditModule(document);
    const logs = [
      { providerName: 'OpenAI', model: 'gpt-4.1', timestamp: '2026-06-04T01:02:03.000Z', status: 'completed' },
      { provider_name: 'Anthropic', model: 'claude', timestamp: '2026-06-04T02:03:04.000Z', status: 'failed' },
    ];
    const app = createApp({ logs });
    await loadAuditLogs(app);
    const tbody = document.getElementById('auditLogBodyTab');
    assert.strictEqual(app.calls[0][0], 'invokeTauri', 'should invoke Tauri once');
    assert.strictEqual(app.calls[0][1], 'get_audit_logs', 'should only call get_audit_logs');
    assert.strictEqual(app.calls[0][2].limit, 100, 'should keep audit log limit');
    assert.strictEqual(app.calls[0][2].offset, 0, 'should keep audit log offset');
    assert.strictEqual(tbody.children.length, 2, 'multiple logs should render multiple rows');
    assert.ok(tbody.textContent.includes('OpenAI'), 'providerName should render');
    assert.ok(tbody.textContent.includes('Anthropic'), 'provider_name should render');
    assert.ok(tbody.textContent.includes('gpt-4.1'), 'model should render');
    assert.ok(tbody.textContent.includes('completed'), 'completed status should render');
    assert.ok(tbody.textContent.includes('failed'), 'failed status should render');
    assert.strictEqual(tbody.children[0].children[3].textContent, 'completed', 'completed status cell should not include filler text');
    assert.strictEqual(tbody.children[1].children[3].textContent, 'failed', 'failed status cell should not include filler text');
    assert.ok(statusSpan(tbody.children[0]).className.includes('audit-status-ok'), 'completed should use ok class');
    assert.ok(statusSpan(tbody.children[1]).className.includes('audit-status-err'), 'failed should use err class');
  }

  {
    const document = createDocument();
    const { loadAuditLogs } = loadAuditModule(document);
    const logs = [{
      providerName: '<img src=x onerror="globalThis.__auditXss=1">',
      model: '<script>globalThis.__auditXss=1</script>',
      timestamp: 'bad-date',
      status: '<svg onload="globalThis.__auditXss=1">',
    }];
    await loadAuditLogs(createApp({ logs }));
    const html = document.getElementById('auditLogBodyTab').innerHTML;
    assert.ok(html.includes('&lt;img'), 'provider should be escaped');
    assert.ok(html.includes('&lt;script&gt;'), 'model should be escaped');
    assert.ok(html.includes('&lt;svg'), 'status should be escaped');
    assert.ok(!html.includes('<img src='), 'provider HTML should not become an element');
    assert.ok(!html.includes('<script>'), 'model HTML should not become an element');
  }

  {
    const document = createDocument({ body: false });
    const { loadAuditLogs } = loadAuditModule(document);
    await assert.doesNotReject(() => loadAuditLogs(createApp({ logs: [{ providerName: 'OpenAI' }] })), 'missing auditLogBodyTab should not throw');
  }

  console.log('day23 audit log smoke: PASS (9 scenarios)');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
