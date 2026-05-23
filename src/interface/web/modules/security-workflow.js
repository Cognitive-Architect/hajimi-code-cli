(function (global) {
  'use strict';

  const UI_FLAG_NAME = 'HAJIMI_SECURITY_UI_ENABLED';
  const DEFAULT_SCOPE = Object.freeze({
    repository: '.',
    branch: 'unknown',
    commit: 'unknown',
    paths: ['.'],
    out_of_scope: ['external_networks', 'real_exploit_execution', 'automatic_fix_application'],
  });

  const COMMANDS = Object.freeze([
    {
      id: 'security-scan',
      trigger: '/security scan',
      title: 'Security scan',
      description: 'Run the report-only security scan contract',
      category: 'security',
      riskLevel: 'medium',
      enabled: true,
      executeMode: 'fill',
      insertText: '/security scan',
      keywords: ['security', 'scan', 'audit'],
    },
    {
      id: 'security-threat-model',
      trigger: '/security threat-model',
      title: 'Security threat model',
      description: 'Build a report-only threat model from supplied scope',
      category: 'security',
      riskLevel: 'medium',
      enabled: true,
      executeMode: 'fill',
      insertText: '/security threat-model',
      keywords: ['security', 'threat', 'model'],
    },
    {
      id: 'security-validate',
      trigger: '/security validate',
      title: 'Security validation',
      description: 'Create validation receipts without local execution',
      category: 'security',
      riskLevel: 'medium',
      enabled: true,
      executeMode: 'fill',
      insertText: '/security validate',
      keywords: ['security', 'validate'],
    },
    {
      id: 'security-fix',
      trigger: '/security fix',
      title: 'Security fix plan',
      description: 'Request a dry-run fix plan for one finding',
      category: 'security',
      riskLevel: 'high',
      enabled: true,
      executeMode: 'fill',
      insertText: '/security fix ',
      keywords: ['security', 'fix', 'dry-run'],
    },
  ]);

  const KIND_BY_SUBCOMMAND = Object.freeze({
    scan: 'security_scan',
    'threat-model': 'threat_model',
    validate: 'validation',
    fix: 'fix_finding',
  });

  function isUiEnabled() {
    return global.__HAJIMI_SECURITY_UI_ENABLED__ === true
      || global.__HAJIMI_FLAGS__?.securityUiEnabled === true;
  }

  function cloneScope(scope = {}) {
    const merged = { ...DEFAULT_SCOPE, ...scope };
    return {
      repository: String(merged.repository || DEFAULT_SCOPE.repository),
      branch: String(merged.branch || DEFAULT_SCOPE.branch),
      commit: String(merged.commit || DEFAULT_SCOPE.commit),
      paths: Array.isArray(merged.paths) && merged.paths.length
        ? merged.paths.map(String)
        : DEFAULT_SCOPE.paths.slice(),
      out_of_scope: Array.isArray(merged.out_of_scope) && merged.out_of_scope.length
        ? merged.out_of_scope.map(String)
        : DEFAULT_SCOPE.out_of_scope.slice(),
    };
  }

  function usage() {
    return '用法: /security scan | /security threat-model | /security validate | /security fix <finding-id>';
  }

  function parseSlash(text) {
    const value = String(text || '').trim();
    const parts = value.split(/\s+/).filter(Boolean);
    if (parts[0] !== '/security') {
      return { ok: false, error: usage() };
    }

    const subcommand = parts[1] || '';
    if (!Object.prototype.hasOwnProperty.call(KIND_BY_SUBCOMMAND, subcommand)) {
      return { ok: false, error: `unknown /security command: ${subcommand || '(empty)'}. ${usage()}` };
    }

    const findingId = parts.slice(2).join(' ').trim();
    if (subcommand === 'fix' && !findingId) {
      return { ok: false, error: 'error: /security fix requires <finding-id>. /security fix <finding-id>' };
    }

    return {
      ok: true,
      subcommand,
      kind: KIND_BY_SUBCOMMAND[subcommand],
      findingId: findingId || null,
    };
  }

  function createFindingStub(findingId) {
    return {
      finding_id: findingId,
      rule_id: 'MANUAL-FIX-REQUEST',
      title: `Dry-run fix request for ${findingId}`,
      severity: 'medium',
      category: 'unknown',
      status: 'unverified',
      confidence: 0.4,
      type: 'manual_fix_request',
      file: null,
      line: null,
      snippet: null,
      evidence: [],
      attack_path: null,
      recommendation: 'Produce a report-only remediation plan; do not modify files.',
      regression_test: null,
      human_review_required: true,
      validation_receipts: [],
      residual_risk: ['Fix requests from slash commands are dry-run only.'],
    };
  }

  function buildRequest(parsed, options = {}) {
    const findings = Array.isArray(options.findings) ? options.findings.slice() : [];
    if (parsed.findingId) {
      findings.push(createFindingStub(parsed.findingId));
    }

    return {
      kind: parsed.kind,
      scope: cloneScope(options.scope),
      dry_run: options.dryRun !== false,
      max_findings: Number.isInteger(options.maxFindings) && options.maxFindings > 0
        ? options.maxFindings
        : 50,
      findings,
    };
  }

  function getScopeFromApp(app) {
    return cloneScope({
      repository: app?.currentWorkspace || '.',
      paths: app?.currentWorkspace ? [app.currentWorkspace] : ['.'],
    });
  }

  async function runSlash(app, text, options = {}) {
    const parsed = parseSlash(text);
    if (!parsed.ok) {
      return { ok: false, error: parsed.error };
    }

    const request = buildRequest(parsed, {
      scope: options.scope || getScopeFromApp(app),
      findings: options.findings,
      dryRun: options.dryRun,
      maxFindings: options.maxFindings,
    });

    if (!app?.isTauriAvailable?.()) {
      return { ok: false, error: 'Tauri 不可用，无法调用 run_security_workflow。' };
    }

    const report = await app.invokeTauri('run_security_workflow', { request });
    return { ok: true, parsed, request, report };
  }

  function formatReportText(report) {
    const summary = report?.summary || {};
    const findings = Array.isArray(report?.findings) ? report.findings : [];
    const notes = Array.isArray(report?.workflow_notes) ? report.workflow_notes : [];
    const receipts = Array.isArray(report?.validation_receipts) ? report.validation_receipts : [];
    const lines = [
      '**Security Workflow Report**',
      '',
      `- kind: \`${report?.kind || 'unknown'}\``,
      `- dry_run: \`${report?.dry_run !== false}\``,
      `- feature_enabled: \`${report?.feature_enabled === true}\``,
      `- findings: \`${summary.total_findings || findings.length || 0}\``,
      `- confirmed: \`${summary.confirmed || 0}\``,
      `- unverified: \`${summary.unverified || 0}\``,
      `- validation_receipts: \`${receipts.length}\``,
    ];

    if (findings.length) {
      lines.push('', '**Findings**');
      findings.slice(0, 10).forEach((finding) => {
        lines.push(`- \`${finding.finding_id}\` ${finding.title || ''} [${finding.status || 'unknown'}]`);
      });
    }

    if (notes.length) {
      lines.push('', '**Workflow Notes**');
      notes.slice(0, 10).forEach((note) => lines.push(`- ${String(note)}`));
    }

    return lines.join('\n');
  }

  function createFindingNode(finding) {
    const node = document.createElement('article');
    node.className = 'security-finding';
    const title = document.createElement('h4');
    title.textContent = finding?.title || finding?.finding_id || 'Security finding';
    const meta = document.createElement('p');
    meta.textContent = `${finding?.finding_id || 'unknown'} · ${finding?.status || 'unknown'}`;
    node.appendChild(title);
    node.appendChild(meta);
    return node;
  }

  global.HajimiSecurityWorkflow = Object.freeze({
    UI_FLAG_NAME,
    commands: COMMANDS,
    isUiEnabled,
    usage,
    parseSlash,
    buildRequest,
    runSlash,
    formatReportText,
    createFindingNode,
  });
})(window);
