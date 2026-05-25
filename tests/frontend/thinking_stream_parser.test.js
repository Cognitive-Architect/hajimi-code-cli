const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const securityDomPath = path.join(repoRoot, 'src/interface/web/modules/security-dom.js');
const thinkingUiPath = path.join(repoRoot, 'src/interface/web/modules/thinking-ui.js');

// Helper to mock DOM APIs for thinking-ui.js
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
      return html || text;
    },
  };
}

const context = {
  console,
  window: null,
  document: {
    createElement,
    querySelector() {
      return null;
    },
  },
};
context.window = context;
context.globalThis = context;
vm.createContext(context);

// Load the real modules in the VM context
vm.runInContext(fs.readFileSync(securityDomPath, 'utf8'), context, { filename: securityDomPath });
vm.runInContext(fs.readFileSync(thinkingUiPath, 'utf8'), context, { filename: thinkingUiPath });

const HajimiThinkingUI = context.HajimiThinkingUI;

// Define test cases covering all 5 required scenarios
const testCases = [
  {
    id: 'FUNC-003',
    name: 'Open tag split across chunks',
    chunks: ['Hello ', '<thi', 'nking>plan', ' detail</thinking>done'],
    expected: [
      { step: 0, buffer: 'Hello ', state: 'idle', thinking: null, response: 'Hello ' },
      // Ideally, during a partial open tag '<thi', it should NOT flush it to the response to prevent UI flicker
      { step: 1, buffer: 'Hello <thi', state: 'idle', thinking: null, response: 'Hello ' },
      { step: 2, buffer: 'Hello <thinking>plan', state: 'thinking', thinking: 'plan', response: 'Hello ' },
      { step: 3, buffer: 'Hello <thinking>plan detail</thinking>done', state: 'response', thinking: 'plan detail', response: 'Hello done' }
    ]
  },
  {
    id: 'FUNC-004',
    name: 'Close tag split across chunks',
    chunks: ['Hello <thinking>plan', '</thin', 'king>world'],
    expected: [
      { step: 0, buffer: 'Hello <thinking>plan', state: 'thinking', thinking: 'plan', response: 'Hello ' },
      // Ideally, during a partial close tag '</thin', it should NOT treat '</thin' as thinking content
      { step: 1, buffer: 'Hello <thinking>plan</thin', state: 'thinking', thinking: 'plan', response: 'Hello ' },
      { step: 2, buffer: 'Hello <thinking>plan</thinking>world', state: 'response', thinking: 'plan', response: 'Hello world' }
    ]
  },
  {
    id: 'NEG-004',
    name: 'Mixed text (prefix + thinking + suffix)',
    chunks: ['Prefix ', '<thinking>thoughts</thinking>', ' Suffix'],
    expected: [
      { step: 0, buffer: 'Prefix ', state: 'idle', thinking: null, response: 'Prefix ' },
      { step: 1, buffer: 'Prefix <thinking>thoughts</thinking>', state: 'response', thinking: 'thoughts', response: 'Prefix ' },
      { step: 2, buffer: 'Prefix <thinking>thoughts</thinking> Suffix', state: 'response', thinking: 'thoughts', response: 'Prefix  Suffix' }
    ]
  },
  {
    id: 'NEG-003',
    name: 'Multiple thinking blocks sequentially',
    chunks: ['<thinking>first</thinking>', ' intermediate ', '<thinking>second</thinking>'],
    expected: [
      { step: 0, buffer: '<thinking>first</thinking>', state: 'response', thinking: 'first', response: '' },
      { step: 1, buffer: '<thinking>first</thinking> intermediate ', state: 'response', thinking: 'first', response: ' intermediate ' },
      // Ideally, a second thinking block should be parsed correctly rather than dumped as raw text in the response
      { step: 2, buffer: '<thinking>first</thinking> intermediate <thinking>second</thinking>', state: 'response', thinking: 'first\nsecond', response: ' intermediate ' }
    ]
  },
  {
    id: 'NEG-001',
    name: 'Malformed / unterminated tag',
    chunks: ['Hello <thinking', ' plan without close', ' more text'],
    expected: [
      // Mismatched or partial tags should be degraded once they are clearly not valid tags
      { step: 0, buffer: 'Hello <thinking', state: 'idle', thinking: null, response: 'Hello ' },
      { step: 1, buffer: 'Hello ', state: 'idle', thinking: null, response: 'Hello ' },
      { step: 2, buffer: 'Hello  more text', state: 'idle', thinking: null, response: 'Hello  more text' }
    ]
  },
  {
    id: 'NEG-002',
    name: 'Empty chunk or empty thinking block',
    chunks: ['Hello ', '', '<thinking></thinking>', ''],
    expected: [
      { step: 0, buffer: 'Hello ', state: 'idle', thinking: null, response: 'Hello ' },
      { step: 1, buffer: 'Hello ', state: 'idle', thinking: null, response: 'Hello ' },
      { step: 2, buffer: 'Hello <thinking></thinking>', state: 'empty', thinking: '', response: 'Hello ' },
      { step: 3, buffer: 'Hello <thinking></thinking>', state: 'empty', thinking: '', response: 'Hello ' }
    ]
  }
];

const lifecycleCases = [
  {
    id: 'NEG-005',
    name: 'Done event resets retained parser buffer',
    run() {
      const res = HajimiThinkingUI.parseStreamEvent('Hello <thinking>plan</thinking>done', { done: true });
      assert.strictEqual(res.done, true);
      assert.strictEqual(res.buffer, '');
      assert.strictEqual(res.state, 'response');
      assert.strictEqual(res.thinking, 'plan');
      assert.strictEqual(res.response, 'Hello done');
    }
  },
  {
    id: 'NEG-006',
    name: 'Error event resets retained parser buffer',
    run() {
      const res = HajimiThinkingUI.parseStreamEvent('partial <thinking data', { error: 'boom' });
      assert.strictEqual(res.buffer, '');
      assert.strictEqual(res.state, 'error');
      assert.strictEqual(res.error, 'boom');
    }
  },
  {
    id: 'NEG-007',
    name: 'Long unterminated thinking block is capped',
    run() {
      const longThinking = 'x'.repeat(9000);
      const res = HajimiThinkingUI.parseStreamEvent('Prefix <thinking>' + longThinking, {});
      assert.strictEqual(res.truncated, true);
      assert.strictEqual(res.buffer, 'Prefix ');
      assert.strictEqual(res.state, 'idle');
      assert.strictEqual(res.thinking, null);
      assert.strictEqual(res.response, 'Prefix ');
    }
  },
  {
    id: 'NEG-008',
    name: 'Cancel event resets retained parser buffer',
    run() {
      const res = HajimiThinkingUI.parseStreamEvent('partial <thinking data', { cancelled: true });
      assert.strictEqual(res.cancelled, true);
      assert.strictEqual(res.buffer, '');
      assert.strictEqual(res.state, 'idle');
      assert.strictEqual(res.response, 'partial ');
    }
  }
];

function runSuite() {
  console.log('========================================================');
  console.log('STARTING THINKING STREAM PARSER FIXTURE TESTS (TDD MODE)');
  console.log('========================================================\n');

  let passedSuite = true;
  const reports = [];

  for (const tc of testCases) {
    console.log(`[CASE] ${tc.id}: ${tc.name}`);
    let buffer = '';
    let stepFailed = false;
    const stepReports = [];

    for (let i = 0; i < tc.chunks.length; i++) {
      const chunk = tc.chunks[i];
      const expected = tc.expected[i];

      // Simulate streaming by accumulating buffer
      const res = HajimiThinkingUI.parseStreamEvent(buffer, { chunk });
      buffer = res.buffer;

      // Verify the parsed properties
      const checks = [];
      let passedStep = true;
      try {
        assert.strictEqual(res.buffer, expected.buffer, `Buffer mismatch: expected "${expected.buffer}", got "${res.buffer}"`);
        assert.strictEqual(res.state, expected.state, `State mismatch: expected "${expected.state}", got "${res.state}"`);
        assert.strictEqual(res.thinking, expected.thinking, `Thinking content mismatch: expected "${expected.thinking}", got "${res.thinking}"`);
        assert.strictEqual(res.response, expected.response, `Response mismatch: expected "${expected.response}", got "${res.response}"`);
        checks.push('PASS');
      } catch (err) {
        checks.push(`FAIL: ${err.message}`);
        passedStep = false;
        stepFailed = true;
      }

      stepReports.push({
        step: i,
        chunk: JSON.stringify(chunk),
        expected: { state: expected.state, thinking: expected.thinking, response: expected.response },
        actual: { state: res.state, thinking: res.thinking, response: res.response },
        status: passedStep ? 'PASS' : 'FAIL',
        details: checks.filter(c => c.startsWith('FAIL'))
      });
    }

    if (stepFailed) {
      passedSuite = false;
      console.log(`  => Status: 🔴 RED LIGHT (Expected bug/gap identified)\n`);
      for (const r of stepReports) {
        console.log(`     Step ${r.step} (chunk ${r.chunk}): ${r.status === 'PASS' ? '🟢 PASS' : '🔴 FAIL'}`);
        if (r.status === 'FAIL') {
          console.log(`       Expected: state=${r.expected.state}, thinking=${JSON.stringify(r.expected.thinking)}, response=${JSON.stringify(r.expected.response)}`);
          console.log(`       Actual:   state=${r.actual.state}, thinking=${JSON.stringify(r.actual.thinking)}, response=${JSON.stringify(r.actual.response)}`);
          for (const d of r.details) {
            console.log(`       Error:    ${d}`);
          }
        }
      }
      console.log('');
    } else {
      console.log(`  => Status: 🟢 GREEN LIGHT (Fully covered & functioning correctly)\n`);
    }

    reports.push({
      id: tc.id,
      name: tc.name,
      passed: !stepFailed
    });
  }

  for (const tc of lifecycleCases) {
    console.log(`[CASE] ${tc.id}: ${tc.name}`);
    try {
      tc.run();
      console.log(`  => Status: 🟢 GREEN LIGHT (Lifecycle invariant covered)\n`);
      reports.push({ id: tc.id, name: tc.name, passed: true });
    } catch (err) {
      passedSuite = false;
      console.log(`  => Status: 🔴 RED LIGHT (Lifecycle invariant failed)`);
      console.log(`     Error: ${err.message}\n`);
      reports.push({ id: tc.id, name: tc.name, passed: false });
    }
  }

  console.log('========================================================');
  console.log('TEST SUITE SUMMARY');
  console.log('========================================================');
  for (const r of reports) {
    console.log(`- [${r.id}] ${r.name}: ${r.passed ? '🟢 PASS' : '🔴 RED LIGHT (Expected Day 4 Bug)'}`);
  }
  console.log('\nResult: ' + (passedSuite ? '🟢 ALL GREEN' : '⚠️ RED LIGHTS PRESENT (Perfect for Day 4 TDD Target)'));
  console.log('========================================================\n');

  process.exit(passedSuite ? 0 : 1);
}

runSuite();
