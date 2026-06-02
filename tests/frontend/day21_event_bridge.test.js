const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const tauriBridgePath = path.join(repoRoot, 'src/interface/web/modules/tauri-bridge.js');

function createContext(tauriMock = null, internalsMock = null) {
  const context = {
    window: {},
    console,
    globalThis: {},
  };
  if (tauriMock) {
    context.__TAURI__ = tauriMock;
  }
  if (internalsMock) {
    context.__TAURI_INTERNALS__ = internalsMock;
  }
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

function testGlobalTauriListen() {
  let listenCalled = false;
  const tauriMock = {
    event: {
      listen(eventName, handler) {
        listenCalled = true;
        assert.strictEqual(eventName, 'approval_request');
        handler({ payload: { request_id: 'test-req-1' } });
        return Promise.resolve(() => {});
      }
    }
  };

  const ctx = createContext(tauriMock);
  runBridge(ctx);

  assert.ok(ctx.window.HajimiTauri, 'HajimiTauri should exist');
  assert.strictEqual(typeof ctx.window.HajimiTauri.listen, 'function', 'HajimiTauri.listen should be a function');

  ctx.window.HajimiTauri.listen('approval_request', (event) => {
    assert.strictEqual(event.payload.request_id, 'test-req-1');
  }).then(() => {
    assert.ok(listenCalled, 'Global tauri listen should be called');
    console.log('  testGlobalTauriListen: PASS');
  });
}

function testTauriListenUnavailable() {
  const ctx = createContext(null, null);
  runBridge(ctx);

  ctx.window.HajimiTauri.listen('approval_request', () => {})
    .then(() => {
      assert.fail('Should have thrown an error');
    })
    .catch((err) => {
      assert.strictEqual(err.message, 'Tauri event listen unavailable');
      console.log('  testTauriListenUnavailable: PASS');
    });
}

function main() {
  console.log('day21 tauri event bridge tests:');
  testGlobalTauriListen();
  testTauriListenUnavailable();
  console.log('day21 tauri event bridge tests: ALL PASS');
}

main();
