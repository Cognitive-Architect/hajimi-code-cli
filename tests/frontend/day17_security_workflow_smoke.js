const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const modulePath = path.join(__dirname, '..', '..', 'src', 'interface', 'web', 'modules', 'security-workflow.js');
const source = fs.readFileSync(modulePath, 'utf8');

const invoked = [];
const mockElements = {};

const sandbox = {
  window: {
    __HAJIMI_FLAGS__: {},
  },
  document: {
    createElement(tagName) {
      const el = {
        tagName,
        id: '',
        className: '',
        textContent: '',
        style: {},
        children: [],
        appendChild(child) {
          this.children.push(child);
        },
        removeChild(child) {
          const idx = this.children.indexOf(child);
          if (idx >= 0) this.children.splice(idx, 1);
        },
        addEventListener(event, callback) {
          if (!this._listeners) this._listeners = {};
          this._listeners[event] = callback;
        },
        click() {
          if (this._listeners && this._listeners.click) {
            this._listeners.click();
          }
        },
        get firstChild() {
          return this.children[0] || null;
        }
      };
      return el;
    },
    createTextNode(text) {
      return { textContent: text };
    },
    getElementById(id) {
      if (!mockElements[id]) {
        mockElements[id] = this.createElement('div');
        mockElements[id].id = id;
      }
      return mockElements[id];
    }
  },
};
sandbox.window.document = sandbox.document;

vm.createContext(sandbox);
vm.runInContext(source, sandbox, { filename: modulePath });

const workflow = sandbox.window.HajimiSecurityWorkflow;
assert(workflow, 'HajimiSecurityWorkflow should be exposed');
assert.strictEqual(workflow.UI_FLAG_NAME, 'HAJIMI_SECURITY_UI_ENABLED');
assert.strictEqual(workflow.isUiEnabled(), false, 'security UI should default disabled');

for (const command of ['/security scan', '/security threat-model', '/security validate']) {
  const parsed = workflow.parseSlash(command);
  assert.strictEqual(parsed.ok, true, `${command} should parse`);
}

assert.deepStrictEqual(workflow.parseSlash('/security scan').kind, 'security_scan');
assert.deepStrictEqual(workflow.parseSlash('/security threat-model').kind, 'threat_model');
assert.deepStrictEqual(workflow.parseSlash('/security validate').kind, 'validation');

const fixParsed = workflow.parseSlash('/security fix FIND-123');
assert.strictEqual(fixParsed.ok, true, '/security fix <finding-id> should parse');
assert.strictEqual(fixParsed.kind, 'fix_finding');
assert.strictEqual(fixParsed.findingId, 'FIND-123');

const missingFixId = workflow.parseSlash('/security fix');
assert.strictEqual(missingFixId.ok, false, '/security fix requires an id');
assert.match(missingFixId.error, /requires <finding-id>/);

const unknown = workflow.parseSlash('/security unknown');
assert.strictEqual(unknown.ok, false, 'unknown security subcommand should fail');
assert.match(unknown.error, /unknown|usage|用法/i);

const request = workflow.buildRequest(fixParsed, {
  scope: {
    repository: 'repo',
    branch: 'branch',
    commit: 'commit',
    paths: ['src/interface'],
    out_of_scope: ['webview_smoke'],
  },
});
assert.strictEqual(request.kind, 'fix_finding');
assert.strictEqual(request.dry_run, true, 'fix requests stay dry-run by default');
assert.strictEqual(request.findings.length, 1);
assert.strictEqual(request.findings[0].finding_id, 'FIND-123');
assert.strictEqual(request.findings[0].status, 'unverified');
assert.strictEqual(request.findings[0].human_review_required, true);

const app = {
  currentWorkspace: 'F:/hajimi-code-cli',
  isTauriAvailable() {
    return true;
  },
  async invokeTauri(command, args) {
    invoked.push({ command, args });
    const findings = args.request.findings.map(f => {
      return {
        ...f,
        evidence: [
          "Safe evidence text",
          "<script>alert(1)</script><div id='bad'>html injected</div>",
          "Another piece of evidence"
        ]
      };
    });
    return {
      kind: args.request.kind,
      scope: args.request.scope,
      dry_run: args.request.dry_run,
      feature_enabled: false,
      summary: {
        total_findings: findings.length,
        confirmed: 0,
        unverified: findings.length,
      },
      findings: findings,
      validation_receipts: [],
      workflow_notes: ['smoke contract exercised'],
    };
  },
};

// Fix 3 test: HAJIMI_SECURITY_UI_ENABLED defaults to false in sandbox, check behavior
sandbox.window.__HAJIMI_FLAGS__ = { securityUiEnabled: false };
assert.strictEqual(workflow.isUiEnabled(), false);
assert.strictEqual(workflow.commands.length, 0); // commands must return empty array

// Exercise setupSecurityPanel when UI is disabled
const tabEl = sandbox.document.getElementById('securityPanelTab');
tabEl.style.display = 'block'; // set initial visible display
workflow.setupSecurityPanel(app);
assert.strictEqual(tabEl.style.display, 'none'); // must be hidden

// Re-enable UI flags to run full active rendering test
sandbox.window.__HAJIMI_FLAGS__ = { securityUiEnabled: true };
assert.strictEqual(workflow.isUiEnabled(), true);

workflow.runSlash(app, '/security scan').then((result) => {
  assert.strictEqual(result.ok, true);
  assert.strictEqual(invoked.length, 1);
  assert.strictEqual(invoked[0].command, 'run_security_workflow');
  assert.strictEqual(invoked[0].args.request.kind, 'security_scan');
  assert.strictEqual(invoked[0].args.request.dry_run, true);
  assert.match(workflow.formatReportText(result.report), /Security Workflow Report/);

  // Assert that new rendering and setup functions are exposed
  assert.strictEqual(typeof workflow.setupSecurityPanel, 'function');
  assert.strictEqual(typeof workflow.safeRenderSecurityPanel, 'function');

  // Exercise setupSecurityPanel and safeRenderSecurityPanel with a custom report containing HTML injection payload in evidence
  workflow.setupSecurityPanel(app);

  const mockReport = {
    summary: { high: 0, medium: 1, low: 0, unverified: 1 },
    findings: [{
      finding_id: 'FIND-TEST',
      title: 'XSS in evidence test',
      severity: 'medium',
      status: 'unverified',
      confidence: 0.8,
      snippet: 'const x = 1;',
      evidence: [
        "Safe evidence text",
        "<script>alert(1)</script><div id='bad'>html injected</div>",
        "Another piece of evidence"
      ]
    }],
    validation_receipts: [],
    residual_risks: []
  };

  workflow.lastReport = mockReport;
  workflow.safeRenderSecurityPanel(app);

  // Verify elements are updated
  const mediumCounter = sandbox.document.getElementById('securitySummaryMedium');
  assert.strictEqual(mediumCounter.textContent, '1');

  // Verify evidence rendering limits & safe text content
  const findingsList = sandbox.document.getElementById('securityFindingsList');
  assert.strictEqual(findingsList.children.length, 1);
  const card = findingsList.children[0];

  const evidenceContainers = card.children.filter(child => child.className === 'finding-evidence-container');
  assert.strictEqual(evidenceContainers.length, 1);
  const evList = evidenceContainers[0].children.find(child => child.className === 'finding-evidence-list');
  assert(evList);
  assert.strictEqual(evList.children.length, 3);

  // Assert the HTML payload is rendered strictly as plain text (escaped/stored in textContent)
  assert.strictEqual(evList.children[1].textContent, "<script>alert(1)</script><div id='bad'>html injected</div>");

  // Strict check: verify no innerHTML/insertAdjacentHTML are used in the source code
  const hasInnerHTML = source.includes('innerHTML') || source.includes('insertAdjacentHTML');
  assert.strictEqual(hasInnerHTML, false, 'Do NOT use innerHTML or insertAdjacentHTML in security-workflow.js!');

  console.log('day17_security_workflow_smoke: ok');
}).catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
