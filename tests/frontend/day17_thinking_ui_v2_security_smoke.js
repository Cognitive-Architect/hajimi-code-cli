const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const securityDomPath = path.join(repoRoot, 'src/interface/web/modules/security-dom.js');
const thinkingUiPath = path.join(repoRoot, 'src/interface/web/modules/thinking-ui.js');
const appJsPath = path.join(repoRoot, 'src/interface/web/app.js');

function escapeHtmlForDom(value) {
  return String(value == null ? '' : value)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

function createElement() {
  let html = '';
  let text = '';
  return {
    dataset: {},
    style: {},
    children: [],
    className: '',
    classList: {
      add() {},
      remove() {},
      toggle() {},
      contains() { return false; },
    },
    appendChild(child) {
      this.children.push(child);
      return child;
    },
    addEventListener() {},
    setAttribute(name, value) {
      this[name] = String(value);
    },
    querySelector() {
      return null;
    },
    set textContent(value) {
      text = String(value == null ? '' : value);
      html = '';
    },
    get textContent() {
      return text;
    },
    set innerHTML(value) {
      html = String(value == null ? '' : value);
      text = '';
    },
    get innerHTML() {
      return html || escapeHtmlForDom(text);
    },
  };
}

let alertCount = 0;
const context = {
  console,
  window: null,
  document: {
    createElement,
    querySelector() {
      return null;
    },
  },
  alert() {
    alertCount += 1;
  },
};
context.window = context;
context.globalThis = context;
vm.createContext(context);
vm.runInContext(fs.readFileSync(securityDomPath, 'utf8'), context, { filename: securityDomPath });
vm.runInContext(fs.readFileSync(thinkingUiPath, 'utf8'), context, { filename: thinkingUiPath });

const app = {
  safeText(value) {
    return context.HajimiSecurityDom.safeText(value);
  },
  escapeHtml(value) {
    return context.HajimiSecurityDom.escapeHtml(value);
  },
  escapeAttr(value) {
    return context.HajimiSecurityDom.escapeAttr(value);
  },
  sanitizeUrl(url) {
    if (!url) return null;
    const trimmed = String(url).trim().toLowerCase();
    if (trimmed.startsWith('http://') || trimmed.startsWith('https://') || trimmed.startsWith('mailto:')) {
      return String(url).trim();
    }
    return null;
  },
  formatText(text) {
    let html = this.safeText(text)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
    html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
    html = html.replace(/`(.+?)`/g, '<code>$1</code>');
    html = html.replace(/```([\s\S]*?)```/g, '<pre><code>$1</code></pre>');
    html = html.replace(/\n/g, '<br>');
    return html;
  },
  renderMarkdown(text) {
    let html = this.safeText(text)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
    html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
    html = html.replace(/`(.+?)`/g, '<code>$1</code>');
    html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (match, label, url) => {
      const safe = this.sanitizeUrl(url);
      return safe ? `<a href="${this.escapeAttr(safe)}" target="_blank" rel="noopener">${label}</a>` : `<span>${label}</span>`;
    });
    html = html.replace(/\n/g, '<br>');
    return html;
  },
};

function createPanel() {
  return {
    root: createElement(),
    header: createElement(),
    icon: createElement(),
    title: createElement(),
    meta: createElement(),
    toggle: createElement(),
    content: createElement(),
    app,
  };
}

function assertNoExecutableHtml(label, html) {
  assert(!/<script\b/i.test(html), `${label} must not render script tags`);
  assert(!/<img\b/i.test(html), `${label} must not render img tags`);
  assert(!/<svg\b/i.test(html), `${label} must not render svg tags`);
  assert(!/<[^>]+\son\w+\s*=/i.test(html), `${label} must not render event handler attributes`);
  assert(!/<a\b[^>]*href\s*=\s*["']?\s*javascript:/i.test(html), `${label} must not render javascript: href`);
}

const ordinary = context.HajimiThinkingUI.parseStreamEvent('', { chunk: 'hello' });
assert.strictEqual(ordinary.response, 'hello', 'ordinary chunks should go to response');
assert.strictEqual(ordinary.thinking, null, 'ordinary chunks should not update thinking');

const thinkingTag = context.HajimiThinkingUI.parseStreamEvent('', { chunk: '<thinking>plan</thinking><response>done</response>' });
assert.strictEqual(thinkingTag.thinking, 'plan', '<thinking> should update thinking');
assert.strictEqual(thinkingTag.response, 'done', '<response> should update response');

const thinkTag = context.HajimiThinkingUI.parseStreamEvent('', { chunk: '<think>short plan</think>answer' });
assert.strictEqual(thinkTag.thinking, 'short plan', '<think> should update thinking');
assert.strictEqual(thinkTag.response, 'answer', '<think> close should expose following response text');

const thinkingContent = context.HajimiThinkingUI.parseStreamEvent('', { thinking_content: 'trace plan' });
assert.strictEqual(thinkingContent.thinking, 'trace plan', 'thinking_content should update thinking only');
assert.strictEqual(thinkingContent.response, null, 'thinking_content should not update response');

const streamError = context.HajimiThinkingUI.parseStreamEvent('', { error: 'backend failed' });
assert.strictEqual(streamError.error, 'backend failed', 'event.error should be surfaced inline');
assert.strictEqual(streamError.state, 'error');

const appJs = fs.readFileSync(appJsPath, 'utf8');
const streamChatBlock = appJs.match(/async streamChat[\s\S]*?\n  generateDemoResponse/);
assert(streamChatBlock, 'streamChat block should be discoverable');
assert(streamChatBlock[0].includes('parseStreamEvent'), 'streamChat should use parseStreamEvent');
assert(!streamChatBlock[0].includes('showErrorToast'), 'streamChat must not call showErrorToast');
assert(appJs.includes('assistant-inline-error'), 'inline response error class should be present');

const payloads = [
  '<script>alert(1)</script>',
  '<img src=x onerror=alert(1)>',
  '<svg onload=alert(1)></svg>',
  '<a href="javascript:alert(1)">click</a>',
  '**bold** `<img src=x onerror=alert(1)>`',
];

for (const payload of payloads) {
  const userBubble = app.formatText(payload);
  const response = app.formatText(payload);
  const panel = createPanel();
  context.HajimiThinkingUI.setThinkingContent(panel, payload);
  assertNoExecutableHtml(`user bubble ${payload}`, userBubble);
  assertNoExecutableHtml(`response ${payload}`, response);
  assertNoExecutableHtml(`thinking ${payload}`, panel.content.innerHTML);
}

const blockedMarkdownLink = app.renderMarkdown('[click](javascript:alert(1))');
assert(!blockedMarkdownLink.includes('<a '), 'javascript: markdown link should not become a clickable anchor');
assert.strictEqual(alertCount, 0, 'payload smoke should not call alert');

console.log('day17 thinking ui v2 security smoke: PASS');
