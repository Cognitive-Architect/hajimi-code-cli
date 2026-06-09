const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const feedbackViewPath = path.join(repoRoot, 'src/interface/web/views/feedback-view.js');
const markdownServicePath = path.join(repoRoot, 'src/interface/web/services/markdown-service.js');
const topbarViewPath = path.join(repoRoot, 'src/interface/web/views/topbar-view.js');

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
    this.title = '';
  }
}

class FakeDocument {
  constructor() {
    this.byId = new Map();
    this.body = {
      appendChild: (el) => {
        this.byId.set(el.id, el);
        return el;
      }
    };
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

  // Load modules
  vm.runInContext(fs.readFileSync(feedbackViewPath, 'utf8'), context, { filename: 'feedback-view.js' });
  vm.runInContext(fs.readFileSync(markdownServicePath, 'utf8'), context, { filename: 'markdown-service.js' });
  vm.runInContext(fs.readFileSync(topbarViewPath, 'utf8'), context, { filename: 'topbar-view.js' });

  // 1. Verify mounts
  assert.ok(context.HajimiFeedbackView, 'HajimiFeedbackView should be mounted');
  assert.ok(context.HajimiMarkdownService, 'HajimiMarkdownService should be mounted');
  assert.ok(context.HajimiTopbarView, 'HajimiTopbarView should be mounted');

  // 2. Test FeedbackView (showErrorToast & hideErrorToast)
  context.HajimiFeedbackView.showErrorToast('Test Error');
  const toast = document.getElementById('errorToast');
  assert.ok(toast, 'showErrorToast should create toast element');
  assert.strictEqual(toast.textContent, 'Test Error', 'toast content should match');
  assert.strictEqual(toast.classList.contains('active'), true, 'toast should have active class');

  context.HajimiFeedbackView.hideErrorToast();
  assert.strictEqual(toast.classList.contains('active'), false, 'hideErrorToast should remove active class');

  // 3. Test MarkdownService
  // test url sanitization (NEG-001)
  assert.strictEqual(context.HajimiMarkdownService.sanitizeUrl('javascript:alert(1)'), null, 'javascript: should be blocked');
  assert.strictEqual(context.HajimiMarkdownService.sanitizeUrl('HTTPS://EXAMPLE.COM'), 'HTTPS://EXAMPLE.COM', 'https should be accepted');
  assert.strictEqual(context.HajimiMarkdownService.sanitizeUrl('http://google.com'), 'http://google.com', 'http should be accepted');
  assert.strictEqual(context.HajimiMarkdownService.sanitizeUrl('mailto:test@test.com'), 'mailto:test@test.com', 'mailto should be accepted');

  // test formatText
  // Mock window.HajimiSecurityDom
  context.HajimiSecurityDom = {
    safeText(text) { return text; },
    escapeAttr(text) { return text; }
  };
  const formatted = context.HajimiMarkdownService.formatText('**bold** and `code`');
  assert.ok(formatted.includes('<strong>bold</strong>'), 'bold should render');
  assert.ok(formatted.includes('<code>code</code>'), 'code should render');

  // test renderMarkdown
  const md = context.HajimiMarkdownService.renderMarkdown('[label](https://site.com)');
  assert.ok(md.includes('<a href="https://site.com"'), 'links should render');

  // 4. Test TopbarView
  const app = {
    currentWorkspace: 'C:\\projects\\my-project',
    isTauriAvailable() { return true; },
    escapeHtml(t) { return t; },
    runShellCommand(cmd, args) {
      return Promise.resolve({ stdout: 'main\n' });
    },
    renderLiveShellState() {
      this.liveShellStateCalled = true;
    },
    renderChatShellStatus() {},
    renderSidebarFileSummary() {},
    renderSidebarModelSummary() {},
    renderSidebarMcpSummary() {},
    renderInspectorTaskStatus() {},
    renderInspectorSessionStats() {}
  };

  const projectEl = new FakeElement(document, 'topBarProject');
  document.register('topBarProject', projectEl);

  context.HajimiTopbarView.renderTopBarWorkspace(app);
  assert.strictEqual(projectEl.textContent, 'my-project', 'workspace folder name should be extracted');
  assert.strictEqual(projectEl.title, 'C:\\projects\\my-project', 'workspace full path should be set as title');

  // test renderLiveShellState
  context.HajimiTopbarView.renderLiveShellState(app, 'Running');
  assert.strictEqual(projectEl.textContent, 'my-project');

  // test updateGitBranch
  const statusBranch = new FakeElement(document, 'statusBranch');
  const topBarBranch = new FakeElement(document, 'topBarBranch');
  const sidebarGitBranch = new FakeElement(document, 'sidebarGitBranch');
  document.register('statusBranch', statusBranch);
  document.register('topBarBranch', topBarBranch);
  document.register('sidebarGitBranch', sidebarGitBranch);

  context.HajimiTopbarView.updateGitBranch(app);
  // wait for microtask queue
  await new Promise(resolve => setTimeout(resolve, 10));

  assert.ok(statusBranch.innerHTML.includes('main'), 'branch name should update on statusBranch');
  assert.strictEqual(topBarBranch.textContent, 'main', 'branch name should update on topBarBranch');
  assert.strictEqual(sidebarGitBranch.textContent, 'main', 'branch name should update on sidebarGitBranch');
  assert.strictEqual(app.liveShellStateCalled, true, 'renderLiveShellState should be triggered');

  console.log('day31 feedback markdown topbar smoke: PASS');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
