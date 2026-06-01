/**
 * DEBT-AGENT-LLM-NATIVE-THINKING-LEAK smoke test
 *
 * Validates that:
 * 1. parseThinkingStream correctly separates <thinking> content from response
 * 2. The handleAgentEvent result branch uses parseThinkingStream for outcome text
 * 3. Final result body contains the clean response (no <thinking> tags)
 * 4. Thinking panel receives the thinking content
 * 5. Success / Aborted / BudgetExceeded / ActFailed branches remain intact
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');

// ---------- Part 1: parseThinkingStream unit tests ----------

// Load the thinking-ui module in a minimal global scope
const thinkingUiPath = path.resolve(__dirname, '..', '..', 'src/interface/web/modules/thinking-ui.js');
const thinkingUiCode = fs.readFileSync(thinkingUiPath, 'utf8');

// Create a minimal `window` global for the IIFE
const fakeWindow = {};
const wrappedCode = thinkingUiCode.replace(
  /\}\)\(window\);\s*$/,
  '})(fakeWindow);'
);
// eslint-disable-next-line no-eval
const fn = new Function('fakeWindow', wrappedCode);
fn(fakeWindow);

const { parseThinkingStream } = fakeWindow.HajimiThinkingUI;
assert(typeof parseThinkingStream === 'function', 'parseThinkingStream must be exported');

// Test case: thinking + response mixed input
const testInput = '<thinking>选择前10个</thinking>1. .cargo-lock\n2. .codex';
const parsed = parseThinkingStream(testInput);

// Thinking panel must contain the reasoning
assert(
  parsed.thinking && parsed.thinking.includes('选择前10个'),
  `Thinking panel should contain '选择前10个', got: ${JSON.stringify(parsed.thinking)}`
);

// Final result body must contain the actual answer
assert(
  parsed.response && parsed.response.includes('.cargo-lock'),
  `Final result body should contain '.cargo-lock', got: ${JSON.stringify(parsed.response)}`
);

// Final result body must NOT contain <thinking> or </thinking>
assert(
  !parsed.response.includes('<thinking>'),
  `Final result body must not contain '<thinking>', got: ${JSON.stringify(parsed.response)}`
);
assert(
  !parsed.response.includes('</thinking>'),
  `Final result body must not contain '</thinking>', got: ${JSON.stringify(parsed.response)}`
);

// State should be 'response' (thinking block is closed)
assert.strictEqual(parsed.state, 'response', `State should be 'response', got: ${parsed.state}`);

console.log('Part 1 (parseThinkingStream unit tests): PASS');

// ---------- Part 2: app.js code-level contract validation ----------

const repoRoot = path.resolve(__dirname, '..', '..');
const appJsPath = path.join(repoRoot, 'src/interface/web/app.js');
const appJs = fs.readFileSync(appJsPath, 'utf8');

// Extract the result branch for inspection
const resultBranch = appJs.match(
  /} else if \(event\.type === 'result'\) \{[\s\S]*?\n    } else if \(event\.type === 'error'\)/
);
assert(resultBranch, 'agent result branch should be discoverable');
const branchCode = resultBranch[0];

// The outcome.trim() branch must call parseThinkingStream
assert(
  branchCode.includes('this.parseThinkingStream(outcome)'),
  'outcome.trim() branch must call parseThinkingStream to strip thinking tags'
);

// The outcome.trim() branch must call updateTurnThinking for thinking content
assert(
  branchCode.includes('this.updateTurnThinking(turn,'),
  'outcome.trim() branch must route thinking content to updateTurnThinking'
);

// The outcome.trim() branch must use cleanResponse (not raw outcome) for display
assert(
  branchCode.includes('displayBody') && branchCode.includes('cleanResponse'),
  'outcome.trim() branch must use cleanResponse for the display body'
);

// Legacy branches must remain intact
assert(
  branchCode.includes("outcome === 'Success'"),
  'Success branch must remain intact'
);
assert(
  branchCode.includes("outcome === 'Aborted'"),
  'Aborted branch must remain intact'
);
assert(
  branchCode.includes("outcome === 'BudgetExceeded'"),
  'BudgetExceeded branch must remain intact'
);
assert(
  branchCode.includes("outcome && outcome.startsWith('ActFailed')"),
  'ActFailed branch must remain intact'
);
assert(
  branchCode.includes('智能体在执行动作时失败'),
  'ActFailed should still render as a failure message'
);
assert(
  branchCode.includes('智能体任务已成功完成，但没有返回可展示内容'),
  'Empty outcome branch must remain intact'
);

console.log('Part 2 (app.js contract validation): PASS');

// ---------- Part 3: Additional parser edge cases ----------

// Pure text with no thinking tags should pass through unchanged
const noTags = parseThinkingStream('Hello world, no tags here');
assert.strictEqual(noTags.thinking, null, 'No tags input: thinking should be null');
assert.strictEqual(noTags.response, 'Hello world, no tags here', 'No tags input: response unchanged');
assert(!noTags.response.includes('<thinking>'), 'No tags input: no thinking tags in response');

// Empty input
const empty = parseThinkingStream('');
assert.strictEqual(empty.response, '', 'Empty input: response should be empty');

// Thinking only, no response body
const thinkingOnly = parseThinkingStream('<thinking>just reasoning</thinking>');
assert(thinkingOnly.thinking.includes('just reasoning'), 'Thinking-only: thinking extracted');
assert.strictEqual(thinkingOnly.response.trim(), '', 'Thinking-only: response should be empty');

// <think> short form tag
const shortForm = parseThinkingStream('<think>short form</think>actual answer');
assert(shortForm.thinking.includes('short form'), 'Short form: thinking extracted');
assert(shortForm.response.includes('actual answer'), 'Short form: response extracted');
assert(!shortForm.response.includes('<think>'), 'Short form: no think tags in response');

console.log('Part 3 (parser edge cases): PASS');
console.log('');
console.log('agent_thinking_leak_smoke: ALL PASS');
