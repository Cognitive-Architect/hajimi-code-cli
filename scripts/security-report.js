const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const repoRoot = path.resolve(__dirname, '..');
const gateScript = path.join('tests', 'security', 'security_audit_gate.js');
const outputDir = path.join(repoRoot, 'docs', 'security', 'examples');
const markdownOutput = path.join(outputDir, 'SECURITY_REVIEW_SAMPLE.md');
const jsonOutput = path.join(outputDir, 'SECURITY_REVIEW_SAMPLE.json');

function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: 'utf8',
    shell: false,
  });
  return {
    command: [command, ...args].join(' ').replace(/\\/g, '/'),
    exit_code: result.status === null ? 1 : result.status,
    stdout: result.stdout || '',
    stderr: result.stderr || '',
    status: result.status === 0 ? 'pass' : 'fail',
  };
}

function summarizeText(text) {
  const lines = text
    .split(/\r?\n/)
    .map(line => line.trim())
    .filter(Boolean);
  if (!lines.length) return '';
  return lines.slice(0, 6).join(' | ');
}

function parseGateJson(stdout) {
  const marker = 'Security Audit Gate V1 JSON summary';
  const markerIndex = stdout.lastIndexOf(marker);
  const searchStart = markerIndex >= 0 ? markerIndex + marker.length : 0;
  const jsonStart = stdout.indexOf('{', searchStart);
  if (jsonStart < 0) {
    throw new Error('Security gate JSON summary was not found in stdout');
  }
  return JSON.parse(stdout.slice(jsonStart));
}

function countFindings(findings) {
  const counts = {
    severity: {
      critical: 0,
      high: 0,
      medium: 0,
      low: 0,
    },
    status: {
      candidate: 0,
      unverified: 0,
      confirmed: 0,
      fixed: 0,
      accepted_risk: 0,
      false_positive: 0,
    },
  };

  for (const finding of findings) {
    const severity = String(finding.severity || 'medium').toLowerCase();
    const status = String(finding.status || 'unverified').toLowerCase();
    if (Object.prototype.hasOwnProperty.call(counts.severity, severity)) {
      counts.severity[severity] += 1;
    }
    if (Object.prototype.hasOwnProperty.call(counts.status, status)) {
      counts.status[status] += 1;
    }
  }

  return counts;
}

function escapeCell(value) {
  if (value === null || value === undefined || value === '') return '';
  return String(value).replace(/\r?\n/g, ' ').replace(/\|/g, '\\|');
}

function makeFindingId(index) {
  return `FINDING-${String(index + 1).padStart(3, '0')}`;
}

function normalizeFindings(findings) {
  return findings.map((finding, index) => ({
    finding_id: finding.finding_id || makeFindingId(index),
    rule_id: finding.rule_id || 'UNKNOWN',
    title: finding.title || finding.message || finding.reason || 'Security gate finding',
    severity: String(finding.severity || 'medium').toLowerCase(),
    status: String(finding.status || 'unverified').toLowerCase(),
    category: finding.category || 'gate',
    confidence: typeof finding.confidence === 'number' ? finding.confidence : null,
    type: finding.type || finding.type_ || '',
    file: finding.file || '',
    line: finding.line || '',
    snippet: finding.snippet || '',
    evidence: Array.isArray(finding.evidence) ? finding.evidence : [],
    recommendation: finding.recommendation || 'Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.',
    regression_test: finding.regression_test || 'npm run test:security-gate',
    human_review_required: Boolean(finding.human_review_required || finding.severity === 'high' || finding.severity === 'critical'),
    validation_receipts: Array.isArray(finding.validation_receipts) ? finding.validation_receipts : [],
    residual_risk: finding.residual_risk || finding.reason || '',
  }));
}

function renderFindings(findings) {
  if (!findings.length) {
    return 'No gate findings were emitted by the current V1 report input.\n';
  }

  return findings.map(finding => {
    const evidenceRows = finding.evidence.length
      ? finding.evidence.map(evidence => `| ${escapeCell(evidence.kind)} | ${escapeCell(evidence.file)} | ${escapeCell(evidence.line)} | ${escapeCell(evidence.snippet)} | ${escapeCell(evidence.command)} | ${escapeCell(evidence.output_hash)} | ${escapeCell(evidence.note)} |`).join('\n')
      : '|  |  |  |  |  |  | No evidence was supplied; status must remain unverified. |';
    return [
      `### ${finding.finding_id}: ${finding.title}`,
      '',
      `- Rule ID: \`${finding.rule_id}\``,
      `- Severity: \`${finding.severity}\``,
      `- Status: \`${finding.evidence.length ? finding.status : 'unverified'}\``,
      `- Category: \`${finding.category}\``,
      `- Confidence: ${finding.confidence === null ? 'n/a' : finding.confidence}`,
      `- Type: ${finding.type || 'n/a'}`,
      `- File: ${finding.file || 'n/a'}`,
      `- Line: ${finding.line || 'n/a'}`,
      `- Recommendation: ${finding.recommendation}`,
      `- Regression test: \`${finding.regression_test}\``,
      `- Human review required: ${finding.human_review_required}`,
      `- Residual risk: ${finding.residual_risk || 'Tracked by V1 residual-risk section.'}`,
      '',
      '| Kind | File | Line | Snippet | Command | Output Hash | Note |',
      '|---|---|---:|---|---|---|---|',
      evidenceRows,
    ].join('\n');
  }).join('\n\n');
}

function renderMarkdown(report) {
  const counts = report.summary.counts;
  const validationRows = report.validation_receipts
    .map(receipt => `| \`${escapeCell(receipt.command)}\` | ${receipt.exit_code} | \`${receipt.status}\` | ${escapeCell(receipt.stdout_summary)} | ${escapeCell(receipt.stderr_summary)} |`)
    .join('\n');

  return `# Security Review Sample Report

> Generated sample from the current repository output. This is a V1 local gate/report rendering receipt, not a complete security audit or full SAST result.

## Scope

- Repository: ${report.scope.repository}
- Branch: \`${report.scope.branch}\`
- Commit: \`${report.scope.commit}\`
- Generated at: ${report.scope.generated_at}
- Input command: \`${report.scope.input_command}\`
- Output JSON: \`${report.scope.output_json}\`
- In scope: Security Gate V1 structured findings and report rendering.
- Out of scope: external network scanning, exploit execution, Agent workflow automation, UI wiring, and real WebView click validation.

## Executive Summary

| Severity | Count |
|---|---:|
| Critical | ${counts.severity.critical} |
| High | ${counts.severity.high} |
| Medium | ${counts.severity.medium} |
| Low | ${counts.severity.low} |

| Status | Count |
|---|---:|
| Candidate | ${counts.status.candidate} |
| Unverified | ${counts.status.unverified} |
| Confirmed | ${counts.status.confirmed} |
| Fixed | ${counts.status.fixed} |
| Accepted Risk | ${counts.status.accepted_risk} |
| False Positive | ${counts.status.false_positive} |

- Gate status: \`${report.gate.status}\`
- Findings: ${report.gate.summary.findings}
- Failures: ${report.gate.summary.failures}
- Warnings: ${report.gate.summary.warnings}
- Allowlisted: ${report.gate.summary.allowlisted}
- V1 closure status: \`PARTIAL/GATED\`

## Threat Model Summary

| Area | Notes |
|---|---|
| Assets | Local source tree, Tauri desktop command surface, frontend DOM rendering surface, and workspace file operations. |
| Entry points | Tauri config, web HTML/JS/CSS, desktop command registration, tool-system shell/file APIs. |
| Trust boundaries | Frontend-to-Tauri bridge, shell command execution boundary, workspace path resolver boundary. |
| High-risk operations | Shell execution, HTML injection sinks, file write/edit/delete tools, global Tauri access. |
| Assumptions | V1 is a local static regression gate; it does not prove absence of vulnerabilities. |

## Findings

${renderFindings(report.findings)}

## Validation Receipts

| Command | Exit Code | Status | Stdout Summary | Stderr Summary |
|---|---:|---|---|---|
${validationRows}

## Residual Risk

- V1 coverage is limited to the current local gate and SecurityAuditTool schema family.
- Accepted-risk DOM warnings remain tracked by allowlist reasons and are not fixed.
- Findings without evidence must remain \`unverified\`.
- Fixes must not be marked \`fixed\` without revalidation receipts.
- Manual WebView / click validation debt remains \`PENDING-WEBVIEW-SMOKE\` per task instruction.
`;
}

function main() {
  fs.mkdirSync(outputDir, { recursive: true });

  const branch = run('git', ['branch', '--show-current']).stdout.trim();
  const commit = run('git', ['rev-parse', 'HEAD']).stdout.trim();
  const gateReceipt = run('node', [gateScript]);
  const gate = parseGateJson(gateReceipt.stdout);
  const findings = normalizeFindings(gate.findings || []);
  const report = {
    schema_version: 'security-review-report/v1',
    scope: {
      repository: path.basename(repoRoot),
      branch,
      commit,
      generated_at: new Date().toISOString(),
      input_command: gateReceipt.command,
      output_markdown: path.relative(repoRoot, markdownOutput).replace(/\\/g, '/'),
      output_json: path.relative(repoRoot, jsonOutput).replace(/\\/g, '/'),
    },
    gate,
    summary: {
      counts: countFindings(findings),
    },
    findings,
    validation_receipts: [{
      command: gateReceipt.command,
      exit_code: gateReceipt.exit_code,
      stdout_summary: summarizeText(gateReceipt.stdout),
      stderr_summary: summarizeText(gateReceipt.stderr),
      status: gateReceipt.status,
    }],
    residual_risk: [
      'V1 is partial/gated and must not be presented as a complete security audit.',
      'Real WebView click validation remains pending by instruction.',
    ],
  };

  fs.writeFileSync(jsonOutput, `${JSON.stringify(report, null, 2)}\n`, 'utf8');
  fs.writeFileSync(markdownOutput, renderMarkdown(report), 'utf8');

  console.log(`Security review Markdown report written to ${path.relative(repoRoot, markdownOutput).replace(/\\/g, '/')}`);
  console.log(`Security review JSON report written to ${path.relative(repoRoot, jsonOutput).replace(/\\/g, '/')}`);
  process.exitCode = gateReceipt.exit_code;
}

main();
