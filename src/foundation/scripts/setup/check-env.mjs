#!/usr/bin/env node
/**
 * EVM Environment Checker
 * Verifies Foundry, Slither, Anvil installation
 * Exit code 0 = all checks passed
 */

import { execSync } from 'child_process';
import { createServer } from 'net';
import os from 'os';

const CHECKS = { passed: [], failed: [], warnings: [] };

function check(name, cmd, pattern, versionCheck) {
  try {
    const out = execSync(cmd, { encoding: 'utf8', timeout: 5000 }).trim();
    const firstLine = out.split('\n')[0];
    if (pattern && !firstLine.match(pattern)) throw new Error('not detected');
    if (versionCheck) {
      const ver = versionCheck(out);
      if (!ver.ok) throw new Error(ver.msg);
    }
    CHECKS.passed.push({ name, result: firstLine });
    return true;
  } catch (err) {
    CHECKS.failed.push({ name, error: err.message });
    return false;
  }
}

function warn(name, msg) {
  CHECKS.warnings.push({ name, message: msg });
}

console.log('==========================================');
console.log('  EVM Environment Checker');
console.log('==========================================\n');

// Check 1-3: Foundry components
check('Foundry (forge)', 'forge --version', /forge/);
check('Foundry (cast)', 'cast --version', /cast/);
check('Foundry (anvil)', 'anvil --version', /anvil/);

// Check 4: Slither
check('Slither', 'slither --version', /\d+\.\d+/);

// Check 5: Python >= 3.8
try {
  const pyCmd = os.platform() === 'win32' ? 'python' : 'python3';
  const out = execSync(`${pyCmd} --version`, { encoding: 'utf8', timeout: 5000 }).trim();
  const m = out.match(/(\d+)\.(\d+)/);
  if (!m) throw new Error('version not detected');
  const [maj, min] = [parseInt(m[1]), parseInt(m[2])];
  if (maj < 3 || (maj === 3 && min < 8)) throw new Error(`Python ${maj}.${min} < 3.8 required`);
  CHECKS.passed.push({ name: 'Python >= 3.8', result: `${maj}.${min}` });
} catch (e) {
  CHECKS.failed.push({ name: 'Python >= 3.8', error: e.message });
}

// Check 6: Node.js >= 18
try {
  const m = process.version.match(/v(\d+)/);
  if (!m) throw new Error('version not detected');
  const maj = parseInt(m[1]);
  if (maj < 18) throw new Error(`Node ${maj} < 18 required`);
  CHECKS.passed.push({ name: 'Node.js >= 18', result: process.version });
} catch (e) {
  CHECKS.failed.push({ name: 'Node.js >= 18', error: e.message });
}

// Check 7: Port 8545 availability
try {
  const port = 8545;
  const srv = createServer();
  const status = await new Promise((resolve, reject) => {
    srv.once('error', (err) => {
      if (err.code === 'EADDRINUSE') {
        warn('Port 8545', 'Port already in use (anvil may be running)');
        resolve('IN_USE');
      } else {
        reject(err);
      }
    });
    srv.once('listening', () => {
      srv.close(() => resolve('AVAILABLE'));
    });
    srv.listen(port);
  });
  if (status === 'AVAILABLE') {
    CHECKS.passed.push({ name: 'Port 8545', result: 'Available' });
  }
} catch (e) {
  CHECKS.failed.push({ name: 'Port 8545', error: e.message });
}

// Display results
console.log('Results:\n');
for (const c of CHECKS.passed) console.log(`  ✓ ${c.name}: ${c.result}`);
for (const c of CHECKS.warnings) console.log(`  ⚠ ${c.name}: ${c.message}`);
for (const c of CHECKS.failed) console.log(`  ✗ ${c.name}: ${c.error}`);

console.log('\n==========================================');

// Summary table
const table = [
  { Tool: 'Foundry (forge)', Status: CHECKS.passed.find(c => c.name === 'Foundry (forge)') ? '✓' : '✗' },
  { Tool: 'Foundry (cast)', Status: CHECKS.passed.find(c => c.name === 'Foundry (cast)') ? '✓' : '✗' },
  { Tool: 'Foundry (anvil)', Status: CHECKS.passed.find(c => c.name === 'Foundry (anvil)') ? '✓' : '✗' },
  { Tool: 'Slither', Status: CHECKS.passed.find(c => c.name === 'Slither') ? '✓' : '✗' },
  { Tool: 'Python >= 3.8', Status: CHECKS.passed.find(c => c.name === 'Python >= 3.8') ? '✓' : '✗' },
  { Tool: 'Node.js >= 18', Status: CHECKS.passed.find(c => c.name === 'Node.js >= 18') ? '✓' : '✗' },
  { Tool: 'Port 8545', Status: CHECKS.passed.find(c => c.name === 'Port 8545') || CHECKS.warnings.find(c => c.name === 'Port 8545') ? '✓/⚠' : '✗' }
];
console.table(table);

// Final status
console.log('\n==========================================');
if (CHECKS.failed.length === 0) {
  console.log('All checks passed!');
  process.exit(0);
} else {
  console.log(`Failed checks: ${CHECKS.failed.length}`);
  process.exit(1);
}
