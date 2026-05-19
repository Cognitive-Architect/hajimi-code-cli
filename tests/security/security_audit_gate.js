const fs = require('fs');
const path = require('path');

const repoRoot = path.resolve(__dirname, '..', '..');
const tauriConfigPath = 'src/interface/desktop/tauri.conf.json';
const webRoot = 'src/interface/web';
const shellPath = 'src/engine/tool-system/src/shell.rs';
const desktopMainPath = 'src/interface/desktop/src/main.rs';
const allowlistPath = 'tests/security/security_audit_allowlist.json';

const failures = [];
const warnings = [];

function toRepoPath(filePath) {
  return path.relative(repoRoot, filePath).replace(/\\/g, '/');
}

function readText(repoPath) {
  return fs.readFileSync(path.join(repoRoot, repoPath), 'utf8');
}

function addFailure(rule, file, line, message) {
  failures.push({ rule, file, line, message });
}

function addWarning(rule, file, line, message) {
  warnings.push({ rule, file, line, message });
}

function walkFiles(dir, out = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (['dist', 'node_modules'].includes(entry.name)) continue;
      walkFiles(fullPath, out);
    } else if (/\.(html|js|css)$/.test(entry.name)) {
      out.push(fullPath);
    }
  }
  return out;
}

function loadAllowlist() {
  const fullPath = path.join(repoRoot, allowlistPath);
  if (!fs.existsSync(fullPath)) return [];
  const entries = JSON.parse(fs.readFileSync(fullPath, 'utf8'));
  for (const [index, entry] of entries.entries()) {
    if (!entry.path || !entry.pattern || !entry.reason) {
      addFailure('allowlist-reason', allowlistPath, index + 1, 'allowlist entries require path, pattern, and reason');
    }
  }
  return entries;
}

function isAllowed(allowlist, file, text) {
  return allowlist.some(entry => {
    if (entry.path !== file) return false;
    return text.includes(entry.pattern);
  });
}

function scanTauriConfig() {
  const raw = readText(tauriConfigPath);
  const config = JSON.parse(raw);
  const csp = config.app?.security?.csp;
  if (csp === null) {
    addFailure('tauri-csp-null', tauriConfigPath, findLine(raw, '"csp"'), 'Tauri CSP must not be null');
  }
  if (config.app?.withGlobalTauri === true) {
    addFailure('tauri-global-api-fail', tauriConfigPath, findLine(raw, 'withGlobalTauri'), 'withGlobalTauri=true is forbidden after B-18 security closure');
  }
}

function scanInlineHandlers(files) {
  const inlineHandlers = ['onclick', 'onerror', 'onload', 'onmouseover'];
  const inlineHandlerPattern = new RegExp(`\\b(?:${inlineHandlers.join('|')})\\s*=`, 'i');
  for (const fullPath of files) {
    const file = toRepoPath(fullPath);
    const lines = fs.readFileSync(fullPath, 'utf8').split(/\r?\n/);
    lines.forEach((line, index) => {
      if (inlineHandlerPattern.test(line)) {
        addFailure('frontend-inline-handler', file, index + 1, 'inline event handlers are not allowed');
      }
    });
  }
}

function scanDangerousHtmlApi(files, allowlist) {
  const dangerousHtmlPattern = /\b(innerHTML|insertAdjacentHTML)\b/;
  for (const fullPath of files) {
    const file = toRepoPath(fullPath);
    const lines = fs.readFileSync(fullPath, 'utf8').split(/\r?\n/);
    lines.forEach((line, index) => {
      if (!dangerousHtmlPattern.test(line)) return;
      if (file === 'src/interface/web/modules/slash-palette.js') {
        addFailure('slash-palette-dangerous-html', file, index + 1, 'slash palette must use safe DOM rendering only');
        return;
      }
      if (isAllowed(allowlist, file, line)) {
        addWarning('frontend-dangerous-html-allowlisted', file, index + 1, 'known legacy dangerous HTML API allowed with reason');
      } else {
        addFailure('frontend-dangerous-html', file, index + 1, 'dangerous HTML API requires allowlist reason or safe DOM rewrite');
      }
    });
  }
}

function scanShellAllowList() {
  const raw = readText(shellPath);
  const block = raw.match(/const\s+ALLOWED_COMMANDS:[\s\S]*?=\s*&\[(?<body>[\s\S]*?)\];/);
  if (!block) {
    addFailure('shell-allow-list-missing', shellPath, 21, 'ALLOWED_COMMANDS block not found');
    return;
  }

  const commands = Array.from(block.groups.body.matchAll(/"([^"]+)"/g)).map(match => match[1]);
  const forbiddenShells = ['bash', 'sh', 'pwsh', 'powershell'];
  for (const shell of forbiddenShells) {
    if (commands.includes(shell)) {
      addFailure('shell-complex-shell-allowlist', shellPath, findLine(raw, `"${shell}"`), `ALLOWED_COMMANDS must not include ${shell}`);
    }
  }
}

function scanDesktopCommandAllowList() {
  const raw = readText(desktopMainPath);
  const block = raw.match(/const\s+ALLOWED_COMMANDS:[\s\S]*?=\s*&\[(?<body>[\s\S]*?)\];/);
  if (!block) {
    return;
  }

  const commands = Array.from(block.groups.body.matchAll(/"([^"]+)"/g)).map(match => match[1]);
  const highCapabilityCommands = ['npx', 'pnpm', 'pip', 'pip3', 'code', 'cursor'];
  for (const command of highCapabilityCommands) {
    if (commands.includes(command)) {
      addFailure('desktop-run-command-high-capability', desktopMainPath, findLine(raw, `"${command}"`), `legacy run_command must not allow ${command} by default`);
    }
  }
}

function scanRunCommandExposure() {
  const raw = readText(desktopMainPath);
  if (/fn\s+run_command\s*\(/.test(raw)) {
    addFailure('run-command-not-naked', desktopMainPath, findLine(raw, 'fn run_command'), 'legacy run_command must not be exposed as a naked Tauri command');
  }
  if (/generate_handler!\[[\s\S]*\brun_command\s*,/.test(raw)) {
    addFailure('run-command-not-naked', desktopMainPath, findLine(raw, 'run_command,'), 'run_command must not appear in the Tauri invoke_handler');
  }
}

function scanConfirmationTokenNotPublicMint() {
  const desktopRaw = readText(desktopMainPath);
  if (desktopRaw.includes('create_tool_confirmation_token')) {
    addFailure('confirmation-token-not-public-mint', desktopMainPath, findLine(desktopRaw, 'create_tool_confirmation_token'), 'frontend-mintable confirmation token command must not exist');
  }

  const webFiles = walkFiles(path.join(repoRoot, webRoot));
  for (const fullPath of webFiles) {
    const file = toRepoPath(fullPath);
    const raw = fs.readFileSync(fullPath, 'utf8');
    if (raw.includes('create_tool_confirmation_token')) {
      addFailure('confirmation-token-not-public-mint', file, findLine(raw, 'create_tool_confirmation_token'), 'frontend must not request executable confirmation tokens');
    }
  }
}

function scanTauriGlobalApiUsage(files) {
  const directGlobalPattern = /(window\.__TAURI__|tauri\.core|tauri\.invoke)/;
  for (const fullPath of files) {
    const file = toRepoPath(fullPath);
    if (file === 'src/interface/web/modules/tauri-bridge.js') continue;
    const lines = fs.readFileSync(fullPath, 'utf8').split(/\r?\n/);
    lines.forEach((line, index) => {
      if (directGlobalPattern.test(line)) {
        addFailure('tauri-global-api-usage', file, index + 1, 'frontend code must use HajimiTauri adapter instead of direct global Tauri access');
      }
    });
  }
}

function scanDesktopToolGate() {
  const raw = readText(desktopMainPath);
  const executeToolIndex = raw.indexOf('async fn execute_tool');
  if (executeToolIndex < 0) {
    addFailure('desktop-execute-tool-missing', desktopMainPath, 1, 'execute_tool command not found');
    return;
  }
  const executeToolBody = raw.slice(executeToolIndex, executeToolIndex + 900);
  if (!executeToolBody.includes('enforce_tool_permissions')) {
    addFailure('desktop-execute-tool-permission-gate', desktopMainPath, findLine(raw, 'async fn execute_tool'), 'execute_tool must enforce ToolPermissions before tool.execute');
  }
  const executePosition = executeToolBody.indexOf('tool.execute');
  const gatePosition = executeToolBody.indexOf('enforce_tool_permissions');
  if (executePosition >= 0 && (gatePosition < 0 || gatePosition > executePosition)) {
    addFailure('desktop-execute-tool-gate-order', desktopMainPath, findLine(raw, 'tool.execute(args)'), 'permission gate must run before tool.execute');
  }
}

function scanWorkspaceBoundFileTools() {
  const raw = readText(desktopMainPath);
  const required = [
    'ReadFileTool::with_allowed_paths',
    'WriteFileTool::with_allowed_paths',
    'DeleteFileTool::with_allowed_paths',
    'EditFileTool::with_allowed_paths',
  ];
  for (const pattern of required) {
    if (!raw.includes(pattern)) {
      addFailure('desktop-file-tools-workspace-bound', desktopMainPath, findLine(raw, 'fn build_registry'), `${pattern} must be used in desktop registry`);
    }
  }
}

function scanInlineEditWorkspaceResolver() {
  const raw = readText(desktopMainPath);
  for (const command of ['async fn apply_edits', 'fn preview_edit']) {
    const index = raw.indexOf(command);
    if (index < 0) {
      addFailure('desktop-inline-edit-command-missing', desktopMainPath, 1, `${command} not found`);
      continue;
    }
    const body = raw.slice(index, index + 900);
    if (!body.includes('resolve_workspace_path')) {
      addFailure('desktop-inline-edit-workspace-resolver', desktopMainPath, findLine(raw, command), `${command} must resolve paths through workspace resolver`);
    }
  }
}

function scanFileOpsBypass(files) {
  const fileOpsBypassPattern = /run_command[\s\S]{0,120}\b(mkdir|mv|rm|rmdir|del)\b/i;
  for (const fullPath of files) {
    const file = toRepoPath(fullPath);
    const lines = fs.readFileSync(fullPath, 'utf8').split(/\r?\n/);
    lines.forEach((line, index) => {
      if (fileOpsBypassPattern.test(line)) {
        addFailure('frontend-file-ops-shell-bypass', file, index + 1, 'file operations must use dedicated Tauri commands, not shell run_command');
      }
    });
  }
}

function findLine(text, needle) {
  const index = text.split(/\r?\n/).findIndex(line => line.includes(needle));
  return index >= 0 ? index + 1 : 1;
}

function printSummary() {
  console.log('Security Audit Gate V1 summary');
  console.log(`failures: ${failures.length}`);
  console.log(`warnings: ${warnings.length}`);

  if (warnings.length) {
    console.log('\nwarnings:');
    for (const warning of warnings) {
      console.log(`- [${warning.rule}] ${warning.file}:${warning.line} ${warning.message}`);
    }
  }

  if (failures.length) {
    console.error('\nfailures:');
    for (const failure of failures) {
      console.error(`- [${failure.rule}] ${failure.file}:${failure.line} ${failure.message}`);
    }
    console.error('\nSecurity Audit Gate V1: FAIL');
    process.exitCode = 1;
    return;
  }

  console.log('\nSecurity Audit Gate V1: PASS');
}

function main() {
  const allowlist = loadAllowlist();
  const webFiles = walkFiles(path.join(repoRoot, webRoot));
  scanTauriConfig();
  scanInlineHandlers(webFiles);
  scanDangerousHtmlApi(webFiles, allowlist);
  scanTauriGlobalApiUsage(webFiles);
  scanShellAllowList();
  scanDesktopCommandAllowList();
  scanRunCommandExposure();
  scanConfirmationTokenNotPublicMint();
  scanDesktopToolGate();
  scanWorkspaceBoundFileTools();
  scanInlineEditWorkspaceResolver();
  scanFileOpsBypass(webFiles);
  printSummary();
}

main();
