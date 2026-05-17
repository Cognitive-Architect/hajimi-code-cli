const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const root = path.resolve(__dirname, '../..');

function escapeText(value) {
  return String(value ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

class Element {
  constructor(tagName) {
    this.tagName = tagName;
    this.children = [];
    this.listeners = {};
    this.style = {};
    this.dataset = {};
    this.className = '';
    this._text = '';
    this._html = '';
  }

  set textContent(value) {
    this._text = String(value ?? '');
    this._html = '';
  }

  get innerHTML() {
    return this._html || escapeText(this._text);
  }

  set innerHTML(value) {
    this._html = String(value ?? '');
    this._text = '';
  }

  appendChild(child) {
    this.children.push(child);
    return child;
  }

  addEventListener(type, listener) {
    this.listeners[type] = listener;
  }

  querySelectorAll() {
    return [];
  }
}

function makeDocument() {
  const nodes = new Map();
  return {
    createElement(tagName) {
      return new Element(tagName);
    },
    getElementById(id) {
      if (!nodes.has(id)) nodes.set(id, new Element('div'));
      return nodes.get(id);
    },
    querySelectorAll() {
      return [];
    },
  };
}

async function main() {
  const calls = [];
  const prompts = ['new-folder', 'renamed.txt'];
  const context = {
    window: {},
    document: makeDocument(),
    console,
    prompt: () => prompts.shift(),
    confirm: () => true,
  };

  context.window.__TAURI__ = {
    core: {
      invoke: async (cmd, args = {}) => {
        calls.push({ cmd, args });
        if (cmd === 'get_current_workspace') return 'workspace-root';
        if (cmd === 'list_dir') {
          return args.path === 'workspace-root'
            ? ['src', 'README.md', '<bad>.txt']
            : [];
        }
        if (cmd === 'create_dir' || cmd === 'rename_path' || cmd === 'delete_path') {
          return null;
        }
        throw new Error(`unexpected invoke: ${cmd}`);
      },
    },
  };

  vm.createContext(context);
  vm.runInContext(
    fs.readFileSync(path.join(root, 'src/interface/web/modules/security-dom.js'), 'utf8'),
    context,
    { filename: 'security-dom.js' },
  );
  vm.runInContext(
    fs.readFileSync(path.join(root, 'src/interface/web/modules/workspace.js'), 'utf8'),
    context,
    { filename: 'workspace.js' },
  );

  const security = context.window.HajimiSecurityDom;
  const workspace = context.window.HajimiWorkspace;
  assert.ok(security, 'HajimiSecurityDom should be mounted');
  assert.ok(workspace, 'HajimiWorkspace should be mounted');
  assert.strictEqual(security.safeText(null), '');
  assert.strictEqual(security.escapeHtml('<img src=x onerror=1>'), '&lt;img src=x onerror=1&gt;');
  assert.strictEqual(security.escapeAttr('"\'<>&'), '&quot;&#39;&lt;&gt;&amp;');

  let reloadCount = 0;
  const app = {
    currentWorkspace: null,
    fileTree: null,
    tabs: [{ id: 'workspace-root/old.txt' }],
    escapeHtml: security.escapeHtml,
    guessLang: () => 'text',
    getFileIconColor: () => 'var(--fg-default)',
    getFileIconSvg: () => '<svg></svg>',
    renderFileTree() {
      workspace.renderFileTree(this);
    },
    loadFileTree() {
      reloadCount += 1;
    },
    showErrorToast(message) {
      throw new Error(message);
    },
    showContextMenu() {},
    openFile(filePath) {
      this.openedFile = filePath;
    },
    _doCloseTab(tabId) {
      this.closedTab = tabId;
    },
  };

  await workspace.initWorkspace(app);
  assert.strictEqual(app.currentWorkspace, 'workspace-root');

  await workspace.loadFileTree(app);
  assert.strictEqual(app.fileTree.path, 'workspace-root');
  assert.strictEqual(
    JSON.stringify(app.fileTree.children.map((child) => child.name)),
    JSON.stringify(['src', '<bad>.txt', 'README.md']),
  );
  assert.ok(context.document.getElementById('fileTree').children.length > 0, 'file tree should render nodes');

  await workspace.createNewFolder(app);
  await workspace.renameFile(app, 'workspace-root/old.txt');
  await workspace.deleteFile(app, 'workspace-root/renamed.txt');

  const commandNames = calls.map((call) => call.cmd);
  for (const expected of ['get_current_workspace', 'list_dir', 'create_dir', 'rename_path', 'delete_path']) {
    assert.ok(commandNames.includes(expected), `${expected} should be invoked`);
  }
  assert.ok(!commandNames.includes('run_command'), 'workspace file ops must not fall back to run_command');
  assert.strictEqual(reloadCount, 3, 'create/rename/delete should request file tree reload');
  assert.strictEqual(app.closedTab, 'workspace-root/old.txt');

  console.log('day13 workspace/security modules smoke: PASS');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
