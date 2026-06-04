const assert = require('assert');
const fs = require('fs');
const path = require('path');

const root = path.resolve(__dirname, '..', '..');

const paths = {
  html: 'src/interface/web/index.html',
  app: 'src/interface/web/app.js',
  modulesDir: 'src/interface/web/modules',
  css: 'src/interface/web/style.css',
  testsDir: 'tests/frontend',
};

const knownMissingIds = new Set([
  'acceptAllEditsBtn',
  'aiChatModelSelect',
  'bottomPanel',
  'breadcrumbBar',
  'checkpointCompareResultTab',
  'closeEditPanelBtn',
  'closePanelBtn',
  'contextMenu',
  'editHistoryPanel',
  'editorArea',
  'extensionsList',
  'gitBadge',
  'gitCommitActionBtn',
  'gitCommitBtn',
  'gitCommitInput',
  'gitDiffClose',
  'gitDiffContent',
  'gitDiffFileName',
  'gitDiffView',
  'gitFileList',
  'gitRefreshBtn',
  'inlineEditHunks',
  'inlineEditPanel',
  'inlineEditSummary',
  'inspectorOldDiffBtn',
  'lspTooltip',
  'maximizePanelBtn',
  'outputContent',
  'pauseTraceBtn',
  'problemsContent',
  'rejectAllEditsBtn',
  'replayCloseBtn',
  'replayNextBtn',
  'replayPrevBtn',
  'replayStatus',
  'searchCaseSensitive',
  'searchInput',
  'searchRegex',
  'searchResults',
  'searchWholeWord',
  'sessionReplayBar',
  'settingAutoSave',
  'statusCursor',
  'tabBar',
  'terminalContent',
  'tracePanel',
]);

const coreDomIds = [
  'aiChatInput',
  'aiChatSendBtn',
  'auditLogBodyTab',
  'metricIterationTab',
  'metricBlackboardTab',
  'metricEditCountTab',
  'inspectorTraceContent',
  'contextReceiptBody',
  'fileTree',
  'modelSelectBtn',
];

const expectedUncoveredIds = new Set([
  'commandPalette',
  'commandInput',
  'commandList',
  'sessionList',
]);

function read(relPath) {
  return fs.readFileSync(path.join(root, relPath), 'utf8');
}

function lineOf(text, index) {
  return text.slice(0, index).split(/\r?\n/).length;
}

function listJsFiles() {
  const moduleFiles = fs
    .readdirSync(path.join(root, paths.modulesDir))
    .filter((name) => name.endsWith('.js'))
    .sort()
    .map((name) => path.join(paths.modulesDir, name).replace(/\\/g, '/'));
  return [paths.app, ...moduleFiles];
}

function listTestFiles() {
  return fs
    .readdirSync(path.join(root, paths.testsDir))
    .filter((name) => name.endsWith('.js'))
    .sort()
    .map((name) => path.join(paths.testsDir, name).replace(/\\/g, '/'));
}

function collectHtmlDom(html) {
  const ids = new Map();
  const classes = new Map();
  const dataAttrs = new Map();
  let match;

  const idRe = /\bid\s*=\s*["']([^"']+)["']/g;
  while ((match = idRe.exec(html))) {
    ids.set(match[1], lineOf(html, match.index));
  }

  const classRe = /\bclass\s*=\s*["']([^"']+)["']/g;
  while ((match = classRe.exec(html))) {
    for (const cls of match[1].split(/\s+/).filter(Boolean)) {
      if (!classes.has(cls)) classes.set(cls, lineOf(html, match.index));
    }
  }

  const dataRe = /\b(data-[a-zA-Z0-9_-]+)\s*=/g;
  while ((match = dataRe.exec(html))) {
    if (!dataAttrs.has(match[1])) dataAttrs.set(match[1], lineOf(html, match.index));
  }

  return { ids, classes, dataAttrs };
}

function collectJsRefs(files) {
  const idRefs = [];
  const classRefs = [];
  const dataRefs = [];

  for (const relPath of files) {
    const text = read(relPath);
    let match;
    const domIdWrapperNames = new Set();

    const wrapperRe = /function\s+([A-Za-z_$][\w$]*)\s*\(\s*([A-Za-z_$][\w$]*)[^)]*\)\s*\{([\s\S]*?)\n\s*\}/g;
    while ((match = wrapperRe.exec(text))) {
      const [, name, firstParam, body] = match;
      const escapedParam = firstParam.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      const paramRefRe = new RegExp(`getElementById\\(\\s*${escapedParam}\\s*\\)`);
      if (paramRefRe.test(body)) domIdWrapperNames.add(name);
    }

    const idRe = /getElementById\(\s*["']([^"']+)["']\s*\)/g;
    while ((match = idRe.exec(text))) {
      idRefs.push({ value: match[1], file: relPath, line: lineOf(text, match.index), kind: 'getElementById' });
    }

    for (const wrapperName of domIdWrapperNames) {
      const wrapperCallRe = new RegExp(`${wrapperName}\\(\\s*["']([^"']+)["']`, 'g');
      while ((match = wrapperCallRe.exec(text))) {
        idRefs.push({ value: match[1], file: relPath, line: lineOf(text, match.index), kind: `${wrapperName}->getElementById` });
      }
    }

    const qsRe = /querySelector(?:All)?\(\s*["']([^"']*)["']/g;
    while ((match = qsRe.exec(text))) {
      const selector = match[1];
      for (const id of selector.matchAll(/#([A-Za-z_][A-Za-z0-9_-]*)/g)) {
        idRefs.push({ value: id[1], file: relPath, line: lineOf(text, match.index), kind: 'querySelector' });
      }
      for (const cls of selector.matchAll(/\.([A-Za-z_][A-Za-z0-9_-]*)/g)) {
        classRefs.push({ value: cls[1], file: relPath, line: lineOf(text, match.index), kind: 'querySelector' });
      }
      for (const data of selector.matchAll(/\[(data-[a-zA-Z0-9_-]+)/g)) {
        dataRefs.push({ value: data[1], file: relPath, line: lineOf(text, match.index), kind: 'querySelector' });
      }
    }

    const classListRe = /classList\.(?:add|remove|toggle|contains)\(\s*["']([^"']+)["']/g;
    while ((match = classListRe.exec(text))) {
      classRefs.push({ value: match[1], file: relPath, line: lineOf(text, match.index), kind: 'classList' });
    }

    const datasetRe = /\.dataset\.([A-Za-z_][A-Za-z0-9_]*)/g;
    while ((match = datasetRe.exec(text))) {
      const attr = 'data-' + match[1].replace(/[A-Z]/g, (c) => '-' + c.toLowerCase());
      dataRefs.push({ value: attr, file: relPath, line: lineOf(text, match.index), kind: 'dataset' });
    }
  }

  return { idRefs, classRefs, dataRefs };
}

function isHexColorToken(token) {
  return /^[0-9A-Fa-f]{3,8}$/.test(token);
}

function collectCssRefs(css) {
  const idRefs = new Set();
  const classRefs = new Set();
  const dataRefs = new Set();
  let match;

  const idRe = /#([A-Za-z_][A-Za-z0-9_-]*)/g;
  while ((match = idRe.exec(css))) {
    if (!isHexColorToken(match[1])) idRefs.add(match[1]);
  }

  const classRe = /\.([A-Za-z_][A-Za-z0-9_-]*)/g;
  while ((match = classRe.exec(css))) {
    classRefs.add(match[1]);
  }

  const dataRe = /\[(data-[a-zA-Z0-9_-]+)/g;
  while ((match = dataRe.exec(css))) {
    dataRefs.add(match[1]);
  }

  return { idRefs, classRefs, dataRefs };
}

function collectSmokeCoverage(testFiles) {
  const texts = new Map(
    testFiles
      .filter((file) => path.basename(file) !== 'day26_dom_contract_smoke.js')
      .map((file) => [file, read(file)])
  );
  return function coverageFor(name) {
    return [...texts.entries()]
      .filter(([, text]) => text.includes(name))
      .map(([file]) => path.basename(file));
  };
}

function groupRefs(refs) {
  const grouped = new Map();
  for (const ref of refs) {
    if (!grouped.has(ref.value)) grouped.set(ref.value, []);
    grouped.get(ref.value).push(ref);
  }
  return grouped;
}

function firstRef(grouped, value) {
  const ref = grouped.get(value)?.[0];
  return ref ? `${ref.file}:${ref.line}` : '';
}

function buildIdRows(htmlIds, jsIdRefs, cssIdRefs, coverageFor) {
  const grouped = groupRefs(jsIdRefs);
  const names = new Set([...htmlIds.keys(), ...grouped.keys()]);
  const rows = [];

  for (const id of [...names].sort()) {
    const htmlExists = htmlIds.has(id);
    const jsRefs = grouped.get(id) || [];
    const cssExists = cssIdRefs.has(id);
    const coverage = coverageFor(id);
    let status = 'UNKNOWN';

    if (jsRefs.length > 0 && !htmlExists) status = 'MISSING';
    else if (htmlExists && jsRefs.length === 0) status = 'ORPHAN';
    else if (htmlExists && jsRefs.length > 0 && coverage.length === 0) status = 'UNCOVERED';
    else if (htmlExists && jsRefs.length > 0 && coverage.length > 0) status = 'PASS';

    rows.push({
      id,
      status,
      htmlLine: htmlExists ? htmlIds.get(id) : null,
      jsRef: firstRef(grouped, id),
      cssExists,
      coverage,
    });
  }

  return rows;
}

function main() {
  const html = read(paths.html);
  const css = read(paths.css);
  const jsFiles = listJsFiles();
  const testFiles = listTestFiles();

  const htmlDom = collectHtmlDom(html);
  const jsRefs = collectJsRefs(jsFiles);
  const cssRefs = collectCssRefs(css);
  const coverageFor = collectSmokeCoverage(testFiles);
  const idRows = buildIdRows(htmlDom.ids, jsRefs.idRefs, cssRefs.idRefs, coverageFor);

  const missing = idRows.filter((row) => row.status === 'MISSING');
  const unexpectedMissing = missing.filter((row) => !knownMissingIds.has(row.id));
  const orphan = idRows.filter((row) => row.status === 'ORPHAN');
  const uncovered = idRows.filter((row) => row.status === 'UNCOVERED');
  const pass = idRows.filter((row) => row.status === 'PASS');

  assert.strictEqual(unexpectedMissing.length, 0, `unexpected missing DOM IDs: ${unexpectedMissing.map((row) => row.id).join(', ')}`);

  for (const id of coreDomIds) {
    const row = idRows.find((candidate) => candidate.id === id);
    assert(row, `core DOM id not scanned: ${id}`);
    assert.strictEqual(row.status, 'PASS', `core DOM id should be PASS: ${id} -> ${row.status}`);
  }

  for (const id of expectedUncoveredIds) {
    const row = idRows.find((candidate) => candidate.id === id);
    assert(row, `expected uncovered DOM id not scanned: ${id}`);
    assert.strictEqual(row.status, 'UNCOVERED', `expected UNCOVERED DOM id changed: ${id} -> ${row.status}`);
  }

  const dataRefNames = new Set(jsRefs.dataRefs.map((ref) => ref.value));
  for (const attr of ['data-view', 'data-panel', 'data-tab', 'data-settings-panel', 'data-inspector-tab']) {
    assert(htmlDom.dataAttrs.has(attr), `HTML data attribute missing: ${attr}`);
    assert(dataRefNames.has(attr), `JS data attribute reference missing: ${attr}`);
  }

  const classRefNames = new Set(jsRefs.classRefs.map((ref) => ref.value));
  for (const cls of ['activity-item', 'settings-tab', 'trace-tab', 'inspector-tab', 'chat-message']) {
    assert(htmlDom.classes.has(cls) || cssRefs.classRefs.has(cls), `class not found in HTML/CSS contract: ${cls}`);
    assert(classRefNames.has(cls) || coverageFor(cls).length > 0, `class has no JS/test contract reference: ${cls}`);
  }

  console.log('day26 dom contract smoke: PASS');
  console.log(`html ids: ${htmlDom.ids.size}`);
  console.log(`js id refs: ${new Set(jsRefs.idRefs.map((ref) => ref.value)).size}`);
  console.log(`css id refs: ${cssRefs.idRefs.size}`);
  console.log(`PASS ids: ${pass.length}`);
  console.log(`MISSING ids: ${missing.length} (known: ${missing.filter((row) => knownMissingIds.has(row.id)).length}, unexpected: ${unexpectedMissing.length})`);
  console.log(`ORPHAN ids: ${orphan.length}`);
  console.log(`UNCOVERED ids: ${uncovered.length}`);
  console.log(`HTML classes: ${htmlDom.classes.size}`);
  console.log(`JS class refs: ${new Set(jsRefs.classRefs.map((ref) => ref.value)).size}`);
  console.log(`CSS class refs: ${cssRefs.classRefs.size}`);
  console.log(`HTML data attrs: ${htmlDom.dataAttrs.size}`);
  console.log(`JS data refs: ${dataRefNames.size}`);
}

main();
