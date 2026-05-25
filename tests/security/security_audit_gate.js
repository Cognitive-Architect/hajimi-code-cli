const fs = require('fs');
const path = require('path');

const repoRoot = path.resolve(__dirname, '..', '..');
const tauriConfigPath = 'src/interface/desktop/tauri.conf.json';
const webRoot = 'src/interface/web';
const shellPath = 'src/engine/tool-system/src/shell.rs';
const desktopMainPath = 'src/interface/desktop/src/main.rs';
const allowlistPath = 'tests/security/security_audit_allowlist.json';
const scanSkipDirs = new Set(['.git', 'target', 'node_modules', 'dist', 'target-ui-refresh']);

const findings = [];
const failures = [];
const warnings = [];
let allowlistedCount = 0;

function toRepoPath(filePath) {
  return path.relative(repoRoot, filePath).replace(/\\/g, '/');
}

function readText(repoPath) {
  return fs.readFileSync(path.join(repoRoot, repoPath), 'utf8');
}

function readSnippet(file, line) {
  if (!file || !line) return '';
  const fullPath = path.join(repoRoot, file);
  if (!fs.existsSync(fullPath)) return '';
  return fs.readFileSync(fullPath, 'utf8').split(/\r?\n/)[line - 1]?.trim() || '';
}

function makeEvidence(file, line, note, snippet) {
  return [{
    kind: 'code',
    file,
    line,
    snippet: snippet || readSnippet(file, line),
    command: null,
    output_hash: null,
    note,
  }];
}

function addFinding(input) {
  const finding = {
    rule_id: input.rule_id,
    severity: input.severity || 'medium',
    status: input.status || 'unverified',
    file: input.file || null,
    line: input.line || null,
    evidence: input.evidence || makeEvidence(input.file, input.line, input.message || input.reason),
    reason: input.reason || input.message || null,
    message: input.message || input.reason || '',
  };
  findings.push(finding);
  return finding;
}

function addFailure(rule, file, line, message, options = {}) {
  const finding = addFinding({
    rule_id: rule,
    severity: options.severity || 'high',
    status: options.status || 'unverified',
    file,
    line,
    message,
    reason: options.reason || message,
    evidence: options.evidence,
  });
  failures.push(finding);
}

function addWarning(rule, file, line, message, options = {}) {
  const finding = addFinding({
    rule_id: rule,
    severity: options.severity || 'low',
    status: options.status || 'unverified',
    file,
    line,
    message,
    reason: options.reason || message,
    evidence: options.evidence,
  });
  warnings.push(finding);
}

function walkFiles(dir, out = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (scanSkipDirs.has(entry.name)) continue;
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
    const pathValue = typeof entry.path === 'string' ? entry.path.trim() : '';
    const patternValue = typeof entry.pattern === 'string' ? entry.pattern.trim() : '';
    const reasonValue = typeof entry.reason === 'string' ? entry.reason.trim() : '';
    if (!pathValue || !patternValue || !reasonValue) {
      addFailure('ALLOWLIST-001', allowlistPath, index + 1, 'allowlist entry missing reason/path/pattern; reason is required for every exception', {
        status: 'confirmed',
      });
    }
  }
  return entries.map(entry => ({
    ...entry,
    rule_id: typeof entry.rule_id === 'string' ? entry.rule_id.trim() : undefined,
    path: typeof entry.path === 'string' ? entry.path.trim() : '',
    pattern: typeof entry.pattern === 'string' ? entry.pattern.trim() : '',
    reason: typeof entry.reason === 'string' ? entry.reason.trim() : '',
  }));
}

function findAllowlistEntry(allowlist, ruleId, file, text) {
  return allowlist.find(entry => {
    if (entry.path !== file) return false;
    if (entry.rule_id && entry.rule_id !== ruleId) return false;
    return text.includes(entry.pattern);
  });
}

function scanTauriConfig() {
  const raw = readText(tauriConfigPath);
  const config = JSON.parse(raw);
  const csp = config.app?.security?.csp;
  if (csp === null) {
    addFailure('TAURI-CSP-001', tauriConfigPath, findLine(raw, '"csp"'), 'Tauri CSP must not be null', {
      severity: 'critical',
      status: 'confirmed',
    });
  }
  if (config.app?.withGlobalTauri === true) {
    addFailure('TAURI-GLOBAL-001', tauriConfigPath, findLine(raw, 'withGlobalTauri'), 'withGlobalTauri=true is forbidden after B-18 security closure', {
      severity: 'critical',
      status: 'confirmed',
    });
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
        addFailure('DOM-INLINE-001', file, index + 1, 'inline event handlers are not allowed');
      }
    });
  }
}

function scanDangerousHtmlApi(files, allowlist) {
  const dangerousHtmlPattern = /\b(innerHTML|outerHTML|insertAdjacentHTML)\b/;
  for (const fullPath of files) {
    const file = toRepoPath(fullPath);
    const lines = fs.readFileSync(fullPath, 'utf8').split(/\r?\n/);
    lines.forEach((line, index) => {
      if (!dangerousHtmlPattern.test(line)) return;
      if (file === 'src/interface/web/modules/slash-palette.js') {
        addFailure('DOM-HTML-001', file, index + 1, 'slash palette must use safe DOM rendering only');
        return;
      }
      const allowlistEntry = findAllowlistEntry(allowlist, 'DOM-HTML-001', file, line);
      if (allowlistEntry) {
        allowlistedCount += 1;
        addWarning('DOM-HTML-001', file, index + 1, 'known legacy dangerous HTML API allowed with reason', {
          status: 'accepted_risk',
          reason: allowlistEntry.reason,
          evidence: makeEvidence(file, index + 1, allowlistEntry.reason, line.trim()),
        });
      } else {
        addFailure('DOM-HTML-001', file, index + 1, 'dangerous HTML API requires allowlist reason or safe DOM rewrite');
      }
    });
  }
}

function scanShellAllowList() {
  const raw = readText(shellPath);
  const block = raw.match(/const\s+ALLOWED_COMMANDS:[\s\S]*?=\s*&\[(?<body>[\s\S]*?)\];/);
  if (!block) {
    addFailure('SHELL-ALLOW-001', shellPath, 21, 'ALLOWED_COMMANDS block not found');
    return;
  }

  const commands = Array.from(block.groups.body.matchAll(/"([^"]+)"/g)).map(match => match[1]);
  const forbiddenShells = ['bash', 'sh', 'pwsh', 'powershell'];
  for (const shell of forbiddenShells) {
    if (commands.includes(shell)) {
      addFailure('SHELL-ALLOW-001', shellPath, findLine(raw, `"${shell}"`), `ALLOWED_COMMANDS must not include ${shell}`, {
        severity: 'critical',
        status: 'confirmed',
      });
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
      addFailure('SHELL-ALLOW-001', desktopMainPath, findLine(raw, `"${command}"`), `legacy run_command must not allow ${command} by default`);
    }
  }
}

function scanRunCommandExposure() {
  const raw = readText(desktopMainPath);
  if (/fn\s+run_command\s*\(/.test(raw)) {
    addFailure('SHELL-ALLOW-001', desktopMainPath, findLine(raw, 'fn run_command'), 'legacy run_command must not be exposed as a naked Tauri command', {
      severity: 'critical',
    });
  }
  if (/generate_handler!\[[\s\S]*\brun_command\s*,/.test(raw)) {
    addFailure('SHELL-ALLOW-001', desktopMainPath, findLine(raw, 'run_command,'), 'run_command must not appear in the Tauri invoke_handler', {
      severity: 'critical',
    });
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
        addFailure('TAURI-GLOBAL-001', file, index + 1, 'frontend code must use HajimiTauri adapter instead of direct global Tauri access');
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
      addFailure('FILE-OPS-001', desktopMainPath, findLine(raw, 'fn build_registry'), `${pattern} must be used in desktop registry`);
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
      addFailure('FILE-OPS-001', desktopMainPath, findLine(raw, command), `${command} must resolve paths through workspace resolver`);
    }
  }
}

function scanFileOpsBypass(files) {
  const fileOpsBypassPattern = /run_command[\s\S]{0,160}\b(mkdir|mv|rm|rmdir|del|delete|rename|write|create)\b/i;
  for (const fullPath of files) {
    const file = toRepoPath(fullPath);
    const lines = fs.readFileSync(fullPath, 'utf8').split(/\r?\n/);
    lines.forEach((line, index) => {
      if (fileOpsBypassPattern.test(line)) {
        addFailure('FILE-OPS-001', file, index + 1, 'file operations must use dedicated Tauri commands, not shell run_command');
      }
    });
  }
}

function scanProviderWorkspaceConfigSecurity() {
  const raw = readText(desktopMainPath);
  const commands = [
    'fn get_provider_configs',
    'fn add_provider_config',
    'fn update_provider_config',
    'fn delete_provider_config',
    'fn get_providers',
  ];
  const workspaceSecurityPatterns = [
    'trusted_workspace_path',
    'trusted_workspace_path_for_current',
    'trusted_workspace_config_path_for_current',
    'delete_workspace_provider_config_for_current',
  ];
  for (const command of commands) {
    const index = raw.indexOf(command);
    if (index < 0) {
      addFailure('desktop-provider-command-missing', desktopMainPath, 1, `${command} not found`);
      continue;
    }
    let nextIndex = raw.indexOf('#[tauri::command]', index + command.length);
    if (nextIndex < 0) {
      nextIndex = raw.indexOf('fn ', index + command.length);
    }
    if (nextIndex < 0 || nextIndex > index + 4000) {
      nextIndex = index + 4000;
    }
    const body = raw.slice(index, nextIndex);
    const hasSecurityCheck = workspaceSecurityPatterns.some(pat => body.includes(pat));
    if (!hasSecurityCheck) {
      addFailure('desktop-provider-workspace-security', desktopMainPath, findLine(raw, command), `${command} must validate workspace path through workspace safety functions`);
    }
  }
}

function findLine(text, needle) {
  const index = text.split(/\r?\n/).findIndex(line => line.includes(needle));
  return index >= 0 ? index + 1 : 1;
}

function printSummary() {
  const report = {
    status: failures.length ? 'fail' : 'pass',
    summary: {
      findings: findings.length,
      failures: failures.length,
      warnings: warnings.length,
      allowlisted: allowlistedCount,
      allowlist: {
        path: allowlistPath,
      },
    },
    findings,
  };

  console.log('Security Audit Gate V1 summary');
  console.log(`findings: ${findings.length}`);
  console.log(`failures: ${failures.length}`);
  console.log(`warnings: ${warnings.length}`);
  console.log(`allowlisted: ${allowlistedCount}`);

  if (warnings.length) {
    console.log('\nwarnings:');
    for (const warning of warnings) {
      console.log(`- [${warning.rule_id}] ${warning.file}:${warning.line} ${warning.message}`);
    }
  }

  if (failures.length) {
    console.error('\nfailures:');
    for (const failure of failures) {
      console.error(`- [${failure.rule_id}] ${failure.file}:${failure.line} ${failure.message}`);
    }
    console.error('\nSecurity Audit Gate V1: FAIL');
    console.log('\nSecurity Audit Gate V1 JSON summary');
    console.log(JSON.stringify(report, null, 2));
    process.exitCode = 1;
    return;
  }

  console.log('\nSecurity Audit Gate V1: PASS');
  console.log('\nSecurity Audit Gate V1 JSON summary');
  console.log(JSON.stringify(report, null, 2));
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
  scanProviderWorkspaceConfigSecurity();
  printSummary();
}

main();

