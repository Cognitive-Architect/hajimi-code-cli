const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const tauriBridgePath = path.join(repoRoot, 'src/interface/web/modules/tauri-bridge.js');

/**
 * Regression test for DEBT-TAURI-CHANNEL-ENVELOPE-REGRESSION.
 *
 * Tauri v2 wraps Channel callbacks in an envelope:
 *   { message: <payload>, id: n }
 *
 * The bridge must unwrap this envelope before passing to the business handler.
 * This test verifies both envelope and legacy/direct payload paths.
 */

function createContext() {
  const received = [];
  const context = {
    window: {},
    console,
    __TAURI_INTERNALS__: {
      transformCallback(cb) {
        // Store callback so we can invoke it manually with either envelope or direct payload
        context._lastCallback = cb;
        return 42;
      },
    },
    _received: received,
  };
  context.window = context;
  context.globalThis = context;
  return context;
}

function runBridge(context) {
  vm.createContext(context);
  vm.runInContext(fs.readFileSync(tauriBridgePath, 'utf8'), context, {
    filename: 'tauri-bridge.js',
  });
}

function testEnvelopePayload() {
  const ctx = createContext();
  runBridge(ctx);

  const channel = new ctx.window.HajimiTauri.Channel();
  const payloads = [];
  channel.onmessage = (payload) => {
    payloads.push(payload);
  };

  // Simulate Tauri v2 envelope delivery
  ctx._lastCallback({
    message: { chunk: 'pong', done: false, promptTokens: 3, completionTokens: 1 },
    id: 0,
  });

  assert.strictEqual(payloads.length, 1, 'handler should be called once');
  assert.strictEqual(payloads[0].chunk, 'pong', 'envelope should be unwrapped');
  assert.strictEqual(payloads[0].done, false, 'done flag should be preserved');
  assert.strictEqual(payloads[0].promptTokens, 3, 'promptTokens should be preserved');
  assert.strictEqual(payloads[0].completionTokens, 1, 'completionTokens should be preserved');
  assert.strictEqual(payloads[0].id, undefined, 'envelope id should NOT leak to business layer');
  assert.strictEqual(payloads[0].message, undefined, 'envelope wrapper should NOT leak');

  console.log('  testEnvelopePayload: PASS');
}

function testLegacyDirectPayload() {
  const ctx = createContext();
  runBridge(ctx);

  const channel = new ctx.window.HajimiTauri.Channel();
  const payloads = [];
  channel.onmessage = (payload) => {
    payloads.push(payload);
  };

  // Simulate legacy direct delivery (no envelope)
  ctx._lastCallback({ chunk: 'direct', done: true });

  assert.strictEqual(payloads.length, 1, 'handler should be called once');
  assert.strictEqual(payloads[0].chunk, 'direct', 'direct payload should pass through');
  assert.strictEqual(payloads[0].done, true, 'done flag should be preserved');

  console.log('  testLegacyDirectPayload: PASS');
}

function testPrimitivePayload() {
  const ctx = createContext();
  runBridge(ctx);

  const channel = new ctx.window.HajimiTauri.Channel();
  const payloads = [];
  channel.onmessage = (payload) => {
    payloads.push(payload);
  };

  // Simulate primitive payload (string)
  ctx._lastCallback('raw-string');

  assert.strictEqual(payloads.length, 1, 'handler should be called once');
  assert.strictEqual(payloads[0], 'raw-string', 'primitive payload should pass through');

  console.log('  testPrimitivePayload: PASS');
}

function testNullPayload() {
  const ctx = createContext();
  runBridge(ctx);

  const channel = new ctx.window.HajimiTauri.Channel();
  const payloads = [];
  channel.onmessage = (payload) => {
    payloads.push(payload);
  };

  ctx._lastCallback(null);

  assert.strictEqual(payloads.length, 1, 'handler should be called once');
  assert.strictEqual(payloads[0], null, 'null payload should pass through');

  console.log('  testNullPayload: PASS');
}

function testChannelToJson() {
  const ctx = createContext();
  runBridge(ctx);

  const channel = new ctx.window.HajimiTauri.Channel();
  assert.strictEqual(channel.toJSON(), '__CHANNEL__:42', 'toJSON should serialize channel id');

  console.log('  testChannelToJson: PASS');
}

function main() {
  console.log('day20 tauri channel envelope regression:');
  testEnvelopePayload();
  testLegacyDirectPayload();
  testPrimitivePayload();
  testNullPayload();
  testChannelToJson();
  console.log('day20 tauri channel envelope regression: PASS');
}

main();
