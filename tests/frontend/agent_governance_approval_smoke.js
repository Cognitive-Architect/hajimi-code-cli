const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const appJsPath = path.join(repoRoot, 'src/interface/web/app.js');

// Simple DOM Mocking framework suitable for app.js loading
class FakeClassList {
  constructor() {
    this.values = new Set();
  }
  add(name) { this.values.add(name); }
  remove(name) { this.values.delete(name); }
  contains(name) { return this.values.has(name); }
}

class FakeElement {
  constructor(document, tagName = 'div', id = null, attrs = {}) {
    this.document = document;
    this.tagName = tagName;
    this.id = id;
    this.className = attrs.className || '';
    this.listeners = {};
    this.classList = new FakeClassList();
    this._innerHTML = '';
    this.value = attrs.value || '';
    this.queriedElements = {};
  }

  addEventListener(type, handler) {
    this.listeners[type] = handler;
  }

  async click() {
    if (this.listeners.click) {
      return this.listeners.click();
    }
    return undefined;
  }

  remove() {
    if (this.document) {
      this.document.bodyElements = this.document.bodyElements.filter(el => el !== this);
    }
  }

  set innerHTML(value) {
    this._innerHTML = String(value);
  }

  get innerHTML() {
    return this._innerHTML;
  }

  querySelector(selector) {
    if (this.queriedElements[selector]) {
      return this.queriedElements[selector];
    }
    
    if (selector === '.approve-btn') {
      const btn = new FakeElement(this.document, 'button');
      btn.className = 'premium-approval-btn approve-btn';
      this.queriedElements[selector] = btn;
      return btn;
    }
    if (selector === '.reject-btn') {
      const btn = new FakeElement(this.document, 'button');
      btn.className = 'premium-approval-btn reject-btn';
      this.queriedElements[selector] = btn;
      return btn;
    }
    return null;
  }
}

class FakeDocument {
  constructor() {
    this.byId = new Map();
    this.bodyElements = [];
  }

  register(id, el) {
    this.byId.set(id, el);
    return el;
  }

  getElementById(id) {
    return this.byId.get(id) || null;
  }

  createElement(tagName) {
    return new FakeElement(this, tagName);
  }

  get body() {
    const doc = this;
    return {
      appendChild(element) {
        doc.bodyElements.push(element);
      }
    };
  }
}

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function createContext() {
  const document = new FakeDocument();
  document.register('pauseLoopBtnTab', new FakeElement(document, 'button'));
  document.register('resumeLoopBtnTab', new FakeElement(document, 'button'));
  document.register('approvalLevelSelectTab', new FakeElement(document, 'select'));
  document.register('injectMemoryBtnTab', new FakeElement(document, 'button'));
  document.register('injectMemoryKeyTab', new FakeElement(document, 'input'));
  document.register('injectMemoryValueTab', new FakeElement(document, 'input'));
  document.register('updatePlanBtnTab', new FakeElement(document, 'button'));
  document.register('updatePlanInputTab', new FakeElement(document, 'input'));

  const context = {
    window: {},
    document,
    console,
    globalThis: {},
    setTimeout: (fn) => fn(),
  };
  context.window = context;
  context.globalThis = context;
  return context;
}

function loadApp(context) {
  vm.createContext(context);
  let appJsCode = fs.readFileSync(appJsPath, 'utf8');
  // Elegant string replacements to disable auto-exec of init blocks
  appJsCode = appJsCode.replace('bindZombieBtns(); // one-time bind post-init', '// bindZombieBtns() disabled');
  appJsCode = appJsCode.replace('app.init(); // Initialize the app', '// app.init() disabled');
  vm.runInContext(appJsCode, context, { filename: 'app.js' });
}

// 1. Verify subscription on setupGovernance
async function testSetupGovernanceSubscribesToApprovalRequest() {
  const ctx = createContext();
  
  let listenCalled = false;
  ctx.window.HajimiTauri = {
    isAvailable: () => true,
    listen(eventName, handler) {
      listenCalled = true;
      assert.strictEqual(eventName, 'approval_request');
      return Promise.resolve(() => {});
    }
  };

  loadApp(ctx);
  const app = ctx.window.app;
  app.escapeHtml = escapeHtml;
  app.showErrorToast = () => {};

  app.setupGovernance();

  assert.ok(listenCalled, 'HajimiTauri.listen should be invoked on setupGovernance');
  
  // Wait for promise resolution
  await new Promise(resolve => setImmediate(resolve));
  assert.strictEqual(app._governanceListenerInstalled, true, 'governance listener flag should be marked as installed');

  // Verify Idempotency - double setupGovernance should not listen again
  let secondListenCalled = false;
  ctx.window.HajimiTauri.listen = () => { secondListenCalled = true; return Promise.resolve(() => {}); };
  app.setupGovernance();
  assert.strictEqual(secondListenCalled, false, 'idempotency guard should prevent duplicate listener registration');

  console.log('  testSetupGovernanceSubscribesToApprovalRequest: PASS');
}

// 2. Verify modal rendering on fake event delivery
async function testApprovalRequestRendersPremiumOverlay() {
  const ctx = createContext();
  
  let eventHandler = null;
  ctx.window.HajimiTauri = {
    isAvailable: () => true,
    listen(eventName, handler) {
      eventHandler = handler;
      return Promise.resolve(() => {});
    }
  };

  loadApp(ctx);
  const app = ctx.window.app;
  app.escapeHtml = escapeHtml;

  app.setupGovernance();
  await new Promise(resolve => setImmediate(resolve));

  assert.ok(eventHandler, 'eventHandler should be captured');

  // Emit fake approval request
  eventHandler({
    payload: {
      request_id: 'req-999',
      action_type: 'write_file',
      risk_score: 0.95,
      description: 'Attempt to overwrite Cargo.toml'
    }
  });

  const overlay = ctx.document.bodyElements.find(el => el.id === 'approval-overlay-req-999');
  assert.ok(overlay, 'modal overlay should be appended to document.body');
  assert.strictEqual(overlay.className, 'premium-approval-overlay', 'overlay class should be premium-approval-overlay');
  assert.ok(overlay.innerHTML.includes('安全治理审核'), 'modal header title should render');
  assert.ok(overlay.innerHTML.includes('高危'), 'risk score text should render as High Risk');
  assert.ok(overlay.innerHTML.includes('95分'), 'risk score point should render accurately');
  assert.ok(overlay.innerHTML.includes('write_file'), 'action type should render');
  assert.ok(overlay.innerHTML.includes('Attempt to overwrite Cargo.toml'), 'risk description should render');

  console.log('  testApprovalRequestRendersPremiumOverlay: PASS');
}

// 3. Verify Approve click invokes resolve_agent_approval with approved=true
async function testApprovalApproveInvokesResolveAgentApprovalTrue() {
  const ctx = createContext();
  
  let eventHandler = null;
  ctx.window.HajimiTauri = {
    isAvailable: () => true,
    listen(eventName, handler) {
      eventHandler = handler;
      return Promise.resolve(() => {});
    }
  };

  loadApp(ctx);
  const app = ctx.window.app;
  app.escapeHtml = escapeHtml;
  app.showToast = () => {};
  app.showErrorToast = () => {};

  let tauriCommand = null;
  let tauriArgs = null;
  app.invokeTauri = async (cmd, args) => {
    tauriCommand = cmd;
    tauriArgs = args;
    return Promise.resolve();
  };

  app.setupGovernance();
  await new Promise(resolve => setImmediate(resolve));

  // Emit fake approval request
  eventHandler({
    payload: {
      request_id: 'req-approve',
      action_type: 'write_file',
      risk_score: 0.8,
      description: 'Write dangerous file'
    }
  });

  const overlay = ctx.document.bodyElements.find(el => el.id === 'approval-overlay-req-approve');
  assert.ok(overlay, 'overlay must exist');

  const approveBtn = overlay.querySelector('.approve-btn');
  assert.ok(approveBtn, 'approve button must be created inside modal');
  
  // DOM click trigger
  await approveBtn.click();

  assert.strictEqual(tauriCommand, 'resolve_agent_approval', 'tauri command resolve_agent_approval should be invoked');
  assert.strictEqual(tauriArgs.requestId, 'req-approve', 'requestId should match payload');
  assert.strictEqual(tauriArgs.request_id, 'req-approve', 'snake_case request_id fallback should match payload');
  assert.strictEqual(tauriArgs.approved, true, 'approved parameter must be true');

  console.log('  testApprovalApproveInvokesResolveAgentApprovalTrue: PASS');
}

// 4. Verify Reject click invokes resolve_agent_approval with approved=false
async function testApprovalRejectInvokesResolveAgentApprovalFalse() {
  const ctx = createContext();
  
  let eventHandler = null;
  ctx.window.HajimiTauri = {
    isAvailable: () => true,
    listen(eventName, handler) {
      eventHandler = handler;
      return Promise.resolve(() => {});
    }
  };

  loadApp(ctx);
  const app = ctx.window.app;
  app.escapeHtml = escapeHtml;
  app.showToast = () => {};
  app.showErrorToast = () => {};

  let tauriCommand = null;
  let tauriArgs = null;
  app.invokeTauri = async (cmd, args) => {
    tauriCommand = cmd;
    tauriArgs = args;
    return Promise.resolve();
  };

  app.setupGovernance();
  await new Promise(resolve => setImmediate(resolve));

  // Emit fake approval request
  eventHandler({
    payload: {
      request_id: 'req-reject',
      action_type: 'write_file',
      risk_score: 0.8,
      description: 'Write dangerous file'
    }
  });

  const overlay = ctx.document.bodyElements.find(el => el.id === 'approval-overlay-req-reject');
  assert.ok(overlay, 'overlay must exist');

  const rejectBtn = overlay.querySelector('.reject-btn');
  assert.ok(rejectBtn, 'reject button must be created inside modal');
  
  // DOM click trigger
  await rejectBtn.click();

  assert.strictEqual(tauriCommand, 'resolve_agent_approval', 'tauri command resolve_agent_approval should be invoked');
  assert.strictEqual(tauriArgs.requestId, 'req-reject', 'requestId should match payload');
  assert.strictEqual(tauriArgs.request_id, 'req-reject', 'snake_case request_id fallback should match payload');
  assert.strictEqual(tauriArgs.approved, false, 'approved parameter must be false');

  console.log('  testApprovalRejectInvokesResolveAgentApprovalFalse: PASS');
}

// 5. Verify missing approval listener shows diagnostic error toast
async function testMissingApprovalListenerShowsDiagnostic() {
  const ctx = createContext();
  
  // Mock missing or failing event listen API
  ctx.window.HajimiTauri = {
    isAvailable: () => true,
    listen(eventName, handler) {
      return Promise.reject(new Error('Tauri event listen unavailable'));
    }
  };

  loadApp(ctx);
  const app = ctx.window.app;
  app.escapeHtml = escapeHtml;

  let toastErrorMsg = null;
  app.showErrorToast = (msg) => {
    toastErrorMsg = msg;
  };

  app.setupGovernance();
  await new Promise(resolve => setImmediate(resolve));

  assert.strictEqual(toastErrorMsg, 'Approval UI unavailable: cannot subscribe to approval_request events.', 'Failure toast diagnostic must show up');

  console.log('  testMissingApprovalListenerShowsDiagnostic: PASS');
}

async function main() {
  console.log('agent governance approval contract smoke:');
  await testSetupGovernanceSubscribesToApprovalRequest();
  await testApprovalRequestRendersPremiumOverlay();
  await testApprovalApproveInvokesResolveAgentApprovalTrue();
  await testApprovalRejectInvokesResolveAgentApprovalFalse();
  await testMissingApprovalListenerShowsDiagnostic();
  console.log('agent governance approval contract smoke: ALL PASS');
}

main().catch((err) => {
  console.error('Smoke tests failed:', err);
  process.exit(1);
});
