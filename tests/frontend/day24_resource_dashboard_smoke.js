const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const repoRoot = path.resolve(__dirname, '..', '..');
const dashboardPath = path.join(repoRoot, 'src/interface/web/modules/resource-dashboard.js');
const dashboardViewPath = path.join(repoRoot, 'src/interface/web/views/dashboard-view.js');
const dashboardControllerPath = path.join(repoRoot, 'src/interface/web/controllers/dashboard-controller.js');

class Element {
  constructor(id) {
    this.id = id;
    this._textContent = '';
  }

  set textContent(value) {
    this._textContent = value == null ? '' : String(value);
  }

  get textContent() {
    return this._textContent;
  }
}

function createDocument(ids = ['metricIterationTab', 'metricBlackboardTab', 'metricEditCountTab']) {
  const elements = new Map();
  ids.forEach(id => elements.set(id, new Element(id)));
  return {
    elements,
    getElementById(id) {
      return elements.get(id) || null;
    },
  };
}

function loadDashboardModule(document, options = {}) {
  const intervals = [];
  const fakeSetInterval = (callback, delay) => {
    const handle = { callback, delay };
    intervals.push(handle);
    return handle;
  };
  const context = {
    window: { document, setInterval: fakeSetInterval },
    document,
    setInterval: fakeSetInterval,
    module: { exports: {} },
    exports: {},
    console,
  };
  context.globalThis = context;
  vm.createContext(context);
  if (options.withViewController) {
    vm.runInContext(fs.readFileSync(dashboardViewPath, 'utf8'), context, { filename: 'dashboard-view.js' });
    vm.runInContext(fs.readFileSync(dashboardControllerPath, 'utf8'), context, { filename: 'dashboard-controller.js' });
    assert.strictEqual(
      typeof context.window.HajimiDashboardView?.renderMetrics,
      'function',
      'dashboard view should expose renderMetrics'
    );
    assert.strictEqual(
      typeof context.window.HajimiDashboardController?.updateMetrics,
      'function',
      'dashboard controller should expose updateMetrics'
    );
  }
  vm.runInContext(fs.readFileSync(dashboardPath, 'utf8'), context, { filename: 'resource-dashboard.js' });
  assert.strictEqual(
    context.window.HajimiResourceDashboard?.setupResourceDashboard,
    context.module.exports.setupResourceDashboard,
    'browser global and module.exports should expose setupResourceDashboard'
  );
  assert.strictEqual(
    context.window.HajimiResourceDashboard?.updateMetrics,
    context.module.exports.updateMetrics,
    'browser global and module.exports should expose updateMetrics'
  );
  return { module: context.module.exports, intervals };
}

function createApp(options = {}) {
  const calls = [];
  const forbidden = () => {
    throw new Error('resource dashboard must not touch checkpoint/provider/agent/shell functions');
  };
  return {
    calls,
    metricsInterval: null,
    isTauriAvailable: () => options.tauri !== false,
    updateMetrics: () => calls.push(['updateMetrics']),
    invokeTauri: async (command) => {
      calls.push(['invokeTauri', command]);
      if (command !== 'get_resource_metrics') {
        throw new Error(`unexpected command ${command}`);
      }
      if (options.throwInvoke) throw new Error('metrics failed');
      return options.metrics || {};
    },
    exportAllCheckpoints: forbidden,
    restoreCheckpoint: forbidden,
    compareCheckpoints: forbidden,
    loadProviders: forbidden,
    runAgentCommand: forbidden,
    sendChatMessage: forbidden,
    executeShellCommand: forbidden,
  };
}

function metricText(document, id) {
  return document.getElementById(id)?.textContent;
}

async function main() {
  {
    const document = createDocument();
    const { module, intervals } = loadDashboardModule(document, { withViewController: true });
    const app = createApp();
    module.setupResourceDashboard(app);
    assert.deepStrictEqual(app.calls, [['updateMetrics']], 'setupResourceDashboard should refresh immediately');
    assert.strictEqual(intervals.length, 1, 'setupResourceDashboard should register one interval');
    assert.strictEqual(intervals[0].delay, 3000, 'setupResourceDashboard should use 3000ms interval');
    assert.strictEqual(app.metricsInterval, intervals[0], 'setupResourceDashboard should store interval handle on app');
    intervals[0].callback();
    assert.strictEqual(app.calls.length, 2, 'interval callback should refresh metrics');
  }

  {
    const document = createDocument();
    const { module } = loadDashboardModule(document, { withViewController: true });
    const app = createApp({ tauri: false });
    await module.updateMetrics(app);
    assert.deepStrictEqual(app.calls, [], 'Tauri unavailable should not invoke backend');
    assert.strictEqual(metricText(document, 'metricIterationTab'), 'N/A', 'iteration should be N/A without Tauri');
    assert.strictEqual(metricText(document, 'metricBlackboardTab'), 'N/A', 'blackboard should be N/A without Tauri');
    assert.strictEqual(metricText(document, 'metricEditCountTab'), 'N/A', 'edit count should be N/A without Tauri');
  }

  {
    const document = createDocument();
    const { module } = loadDashboardModule(document, { withViewController: true });
    const app = createApp({
      metrics: { iteration_count: 7, blackboard_size: 12, edit_count: 3 },
    });
    await module.updateMetrics(app);
    assert.strictEqual(app.calls[0][0], 'invokeTauri', 'should invoke Tauri for metrics');
    assert.strictEqual(app.calls[0][1], 'get_resource_metrics', 'should only call get_resource_metrics');
    assert.strictEqual(metricText(document, 'metricIterationTab'), '7', 'iteration metric should update');
    assert.strictEqual(metricText(document, 'metricBlackboardTab'), '12', 'blackboard metric should update');
    assert.strictEqual(metricText(document, 'metricEditCountTab'), '3', 'edit count metric should update');
  }

  {
    const document = createDocument(['metricIterationTab']);
    const { module } = loadDashboardModule(document, { withViewController: true });
    await assert.doesNotReject(
      () => module.updateMetrics(createApp({ metrics: { iteration_count: 4, blackboard_size: 2, edit_count: 1 } })),
      'missing metric DOM nodes should not throw'
    );
    assert.strictEqual(metricText(document, 'metricIterationTab'), '4', 'present metric should still update');
  }

  {
    const document = createDocument();
    const { module } = loadDashboardModule(document, { withViewController: true });
    await assert.doesNotReject(
      () => module.updateMetrics(createApp({ throwInvoke: true })),
      'get_resource_metrics failure should not throw'
    );
  }

  {
    const document = createDocument();
    const { module } = loadDashboardModule(document, { withViewController: true });
    const app = createApp({ metrics: { iteration_count: 1, blackboard_size: 2, edit_count: 3 } });
    await module.updateMetrics(app);
    assert.strictEqual(app.calls.length, 1, 'resource dashboard should only perform one readonly metrics call');
    assert.strictEqual(app.calls[0][1], 'get_resource_metrics', 'resource dashboard must not touch checkpoint/provider/agent/shell commands');
  }

  {
    const document = createDocument();
    const { module } = loadDashboardModule(document);
    const app = createApp({
      metrics: { iteration_count: 9, blackboard_size: 8, edit_count: 7 },
    });
    await assert.doesNotReject(
      () => module.updateMetrics(app),
      'missing dashboard controller should no-op without throwing'
    );
    module.setupResourceDashboard(app);
    assert.deepStrictEqual(app.calls, [], 'missing dashboard controller should not invoke backend');
    assert.strictEqual(metricText(document, 'metricIterationTab'), '', 'missing controller should not render old fallback metric');
    assert.strictEqual(metricText(document, 'metricBlackboardTab'), '', 'missing controller should not render old fallback metric');
    assert.strictEqual(metricText(document, 'metricEditCountTab'), '', 'missing controller should not render old fallback metric');
  }

  console.log('day24 resource dashboard smoke: PASS (controller path + missing-controller guard)');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
