#!/usr/bin/env node
'use strict';

const fs = require('fs');
const path = require('path');
const assert = require('assert');

const repoRoot = path.resolve(__dirname, '..', '..');
const appJsPath = path.join(repoRoot, 'src', 'interface', 'web', 'app.js');
const appJs = fs.readFileSync(appJsPath, 'utf8');

function lineOf(offset) {
  return appJs.slice(0, offset).split(/\r?\n/).length;
}

function extractMethod(source, methodName) {
  const methodPattern = new RegExp(`(?:async\\s+)?${methodName}\\s*\\([^)]*\\)\\s*\\{`, 'm');
  const match = methodPattern.exec(source);
  assert(match, `Unable to locate ${methodName} method`);

  const openBrace = source.indexOf('{', match.index);
  const nextMethod = source.indexOf('\n  parseThinkingStream(buffer)', openBrace);
  assert(nextMethod > openBrace, `Unable to locate method boundary after ${methodName}`);
  const end = source.lastIndexOf('}', nextMethod);
  assert(end > openBrace, `Unable to find closing brace for ${methodName}`);

  return {
    start: match.index,
    bodyStart: openBrace + 1,
    end,
    text: source.slice(openBrace + 1, end),
    startLine: lineOf(match.index),
    endLine: lineOf(end),
  };
}

function extractBranch(methodText, branchName, marker) {
  const start = methodText.indexOf(marker);
  assert(start >= 0, `Unable to locate ${branchName} branch`);

  const rest = methodText.slice(start);
  const nextBranch = /\n\s{4}if\s*\(/g;
  nextBranch.lastIndex = 1;
  const next = nextBranch.exec(rest);
  const end = next ? start + next.index : methodText.length;

  return {
    name: branchName,
    text: methodText.slice(start, end),
    relativeStart: start,
  };
}

function hasInvokeUse(text) {
  return /\binvoke\s*(?:\(|[=!]=|[=!]==|[?]|\)|,|;)/.test(text) ||
    /!\s*invoke\b/.test(text);
}

function hasInvokeSource(text) {
  return /\b(?:const|let|var)\s+invoke\b/.test(text) ||
    /\binvoke\s*=\s*this\.getTauriInvoke\s*\(/.test(text) ||
    /\bthis\.getTauriInvoke\s*\(/.test(text) ||
    /\bgetTauriInvoke\s*\(/.test(text);
}

function branchStatus(branch, methodHasSource, methodHasParamSource) {
  const usesInvoke = hasInvokeUse(branch.text);
  if (!usesInvoke) {
    return {
      status: 'PASS',
      reason: 'branch does not directly use invoke',
      usesInvoke,
    };
  }

  if (methodHasSource || methodHasParamSource) {
    return {
      status: 'PASS',
      reason: 'branch uses invoke and handleChatCommand has an explicit invoke source',
      usesInvoke,
    };
  }

  return {
    status: 'FAIL',
    reason: 'branch uses invoke but handleChatCommand has no local or parameter invoke source',
    usesInvoke,
  };
}

const method = extractMethod(appJs, 'handleChatCommand');
const signature = appJs.slice(method.start, method.bodyStart);
const methodHasSource = hasInvokeSource(method.text);
const methodHasParamSource = /\(([^)]*\binvoke\b[^)]*)\)/.test(signature);
const appObjectStart = appJs.indexOf('window.app = {');
const prefixBeforeAppObject = appObjectStart >= 0 ? appJs.slice(0, appObjectStart) : '';
const explicitClosureInvokeSource = /(?:^|\n)\s*(?:const|let|var)\s+invoke\b/.test(prefixBeforeAppObject);

const branches = [
  extractBranch(method.text, '/chat', "if (text.startsWith('/chat '))"),
  extractBranch(method.text, '/search', "if (text === '/search' || text.startsWith('/search '))"),
  extractBranch(method.text, '/compact', "if (text === '/compact')"),
];

const results = branches.map((branch) => ({
  branch: branch.name,
  line: method.startLine + method.text.slice(0, branch.relativeStart).split(/\r?\n/).length - 1,
  ...branchStatus(branch, methodHasSource, methodHasParamSource),
}));

const overall = results.some((r) => r.status === 'FAIL')
  ? 'FAIL'
  : results.some((r) => r.status === 'UNKNOWN')
    ? 'UNKNOWN'
    : 'PASS';

console.log('day27 handleChatCommand invoke smoke');
console.log(`method: handleChatCommand(text) lines ${method.startLine}-${method.endLine}`);
console.log(`method has local invoke source: ${methodHasSource ? 'YES' : 'NO'}`);
console.log(`method has invoke parameter source: ${methodHasParamSource ? 'YES' : 'NO'}`);
console.log(`explicit closure invoke source before window.app object: ${explicitClosureInvokeSource ? 'YES' : 'NO'}`);
for (const result of results) {
  console.log(`${result.branch}: ${result.status} line=${result.line} usesInvoke=${result.usesInvoke ? 'YES' : 'NO'} reason=${result.reason}`);
}
console.log(`overall: ${overall}`);

assert(results.length === 3, 'expected exactly three branch results');

// This is a fact-finding smoke. A detected FAIL is reported as data, not used as
// a process failure, because this task explicitly forbids fixing production code.
process.exitCode = 0;
