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

  let lastReport = null;

  function setSafeText(el, value) {
    if (el) {
      el.textContent = value == null ? '' : String(value);
    }
  }

  function createFindingElement(finding, app) {
    const card = document.createElement('div');
    card.className = `security-finding-card severity-${finding.severity || 'medium'}`;

    const header = document.createElement('div');
    header.className = 'finding-header';

    const title = document.createElement('span');
    title.className = 'finding-title';
    title.textContent = finding.title || finding.finding_id || 'Security Finding';

    const badge = document.createElement('span');
    badge.className = `severity-badge ${finding.severity || 'medium'}`;
    badge.textContent = finding.severity || 'medium';

    header.appendChild(title);
    header.appendChild(badge);
    card.appendChild(header);

    const meta = document.createElement('div');
    meta.className = 'finding-meta';
    meta.textContent = `${finding.finding_id || 'unknown'} · status: ${finding.status || 'unverified'} · conf: ${finding.confidence != null ? finding.confidence : 0.5}`;
    card.appendChild(meta);

    if (finding.file) {
      const loc = document.createElement('div');
      loc.className = 'finding-location';
      loc.textContent = `File: ${finding.file}${finding.line ? `:${finding.line}` : ''}`;
      loc.style.cursor = 'pointer';
      loc.style.textDecoration = 'underline';
      loc.addEventListener('click', () => {
        if (typeof app.openFile === 'function') {
          app.openFile(finding.file);
        }
      });
      card.appendChild(loc);
    }

    if (finding.snippet) {
      const code = document.createElement('pre');
      code.className = 'finding-code';
      const codeChild = document.createElement('code');
      codeChild.textContent = finding.snippet;
      code.appendChild(codeChild);
      card.appendChild(code);
    }

    if (finding.recommendation) {
      const rec = document.createElement('div');
      rec.className = 'finding-recommendation';
      const recLabel = document.createElement('strong');
      recLabel.textContent = 'Recommendation: ';
      const recText = document.createTextNode(finding.recommendation);
      rec.appendChild(recLabel);
      rec.appendChild(recText);
      card.appendChild(rec);
    }

    if (finding.status !== 'fixed' && finding.status !== 'accepted_risk') {
      const fixBtn = document.createElement('button');
      fixBtn.className = 'sidebar-action-btn fix-btn';
      fixBtn.textContent = '生成修复计划 (Dry-run)';
      fixBtn.style.marginTop = '6px';
      fixBtn.addEventListener('click', async () => {
        try {
          fixBtn.disabled = true;
          fixBtn.textContent = '生成中...';
          const commandText = `/security fix ${finding.finding_id}`;
          if (app.addChatMessage) {
            app.addChatMessage('user', commandText);
          }
          const result = await runSlash(app, commandText);
          if (result.ok) {
            if (app.addChatMessage) {
              app.addChatMessage('ai', formatReportText(result.report));
            }
            lastReport = result.report;
            safeRenderSecurityPanel(app);
          } else {
            if (app.addChatMessage) {
              app.addChatMessage('ai', `**修复计划生成失败：** ${result.error}`);
            }
          }
        } catch (e) {
          if (app.addChatMessage) {
            app.addChatMessage('ai', `**错误：** ${e.message || e}`);
          }
        } finally {
          fixBtn.disabled = false;
          fixBtn.textContent = '生成修复计划 (Dry-run)';
        }
      });
      card.appendChild(fixBtn);
    }

    return card;
  }

  function setupSecurityPanel(app) {
    const scanBtn = document.getElementById('runSecurityScanBtn');
    if (scanBtn) {
      scanBtn.addEventListener('click', async () => {
        try {
          scanBtn.disabled = true;
          scanBtn.textContent = '扫描中...';
          const commandText = '/security scan';
          if (app.addChatMessage) {
            app.addChatMessage('user', commandText);
          }
          const result = await runSlash(app, commandText);
          if (result.ok) {
            if (app.addChatMessage) {
              app.addChatMessage('ai', formatReportText(result.report));
            }
            lastReport = result.report;
            safeRenderSecurityPanel(app);
          } else {
            if (app.addChatMessage) {
              app.addChatMessage('ai', `**安全扫描失败：** ${result.error}`);
            }
          }
        } catch (e) {
          if (app.addChatMessage) {
            app.addChatMessage('ai', `**错误：** ${e.message || e}`);
          }
        } finally {
          scanBtn.disabled = false;
          scanBtn.textContent = '扫描';
        }
      });
    }

    safeRenderSecurityPanel(app);
  }

  function safeRenderSecurityPanel(app) {
    const summaryHighEl = document.getElementById('securitySummaryHigh');
    const summaryMediumEl = document.getElementById('securitySummaryMedium');
    const summaryLowEl = document.getElementById('securitySummaryLow');
    const summaryUnverifiedEl = document.getElementById('securitySummaryUnverified');
    const findingsListEl = document.getElementById('securityFindingsList');
    const validationListEl = document.getElementById('securityValidationList');
    const residualListEl = document.getElementById('securityResidualList');

    if (!summaryHighEl || !summaryMediumEl || !summaryLowEl || !summaryUnverifiedEl || !findingsListEl || !validationListEl || !residualListEl) {
      return;
    }

    const report = lastReport;
    if (!report) {
      setSafeText(summaryHighEl, '0');
      setSafeText(summaryMediumEl, '0');
      setSafeText(summaryLowEl, '0');
      setSafeText(summaryUnverifiedEl, '0');

      while (findingsListEl.firstChild) {
        findingsListEl.removeChild(findingsListEl.firstChild);
      }
      const emptyFindings = document.createElement('div');
      emptyFindings.className = 'security-empty-state';
      emptyFindings.textContent = '暂无安全漏洞发现。点击上方“扫描”按钮开始。';
      findingsListEl.appendChild(emptyFindings);

      while (validationListEl.firstChild) {
        validationListEl.removeChild(validationListEl.firstChild);
      }
      const emptyValidation = document.createElement('div');
      emptyValidation.className = 'security-empty-state';
      emptyValidation.textContent = '暂无复测凭证';
      validationListEl.appendChild(emptyValidation);

      while (residualListEl.firstChild) {
        residualListEl.removeChild(residualListEl.firstChild);
      }
      const emptyResidual = document.createElement('li');
      emptyResidual.className = 'security-empty-state';
      emptyResidual.textContent = '暂无残余风险记录';
      residualListEl.appendChild(emptyResidual);
      return;
    }

    const findings = Array.isArray(report.findings) ? report.findings : [];
    let highCount = report.summary?.high ?? 0;
    let mediumCount = report.summary?.medium ?? 0;
    let lowCount = report.summary?.low ?? 0;
    let unverifiedCount = report.summary?.unverified ?? 0;

    if (highCount === 0 && mediumCount === 0 && lowCount === 0 && findings.length > 0) {
      findings.forEach(f => {
        const sev = String(f.severity || '').toLowerCase();
        if (sev === 'high' || sev === 'critical') highCount++;
        else if (sev === 'medium') mediumCount++;
        else if (sev === 'low') lowCount++;

        if (f.status === 'unverified') unverifiedCount++;
      });
    }

    setSafeText(summaryHighEl, String(highCount));
    setSafeText(summaryMediumEl, String(mediumCount));
    setSafeText(summaryLowEl, String(lowCount));
    setSafeText(summaryUnverifiedEl, String(unverifiedCount || report.summary?.unverified || 0));

    while (findingsListEl.firstChild) {
      findingsListEl.removeChild(findingsListEl.firstChild);
    }
    if (findings.length === 0) {
      const empty = document.createElement('div');
      empty.className = 'security-empty-state';
      empty.textContent = '未发现任何安全风险。工作区安全！';
      findingsListEl.appendChild(empty);
    } else {
      findings.forEach(f => {
        findingsListEl.appendChild(createFindingElement(f, app));
      });
    }

    while (validationListEl.firstChild) {
      validationListEl.removeChild(validationListEl.firstChild);
    }
    const receipts = Array.isArray(report.validation_receipts) ? report.validation_receipts : [];
    if (receipts.length === 0) {
      const empty = document.createElement('div');
      empty.className = 'security-empty-state';
      empty.textContent = '暂无复测凭证';
      validationListEl.appendChild(empty);
    } else {
      receipts.forEach(r => {
        const card = document.createElement('div');
        card.className = 'security-validation-card';

        const header = document.createElement('div');
        header.className = 'val-header';

        const cmd = document.createElement('span');
        cmd.className = 'val-cmd';
        cmd.textContent = r.command || 'validation command';

        const status = document.createElement('span');
        const stLower = String(r.status || 'fail').toLowerCase();
        status.className = `val-status ${stLower}`;
        status.textContent = r.status || 'FAIL';

        header.appendChild(cmd);
        header.appendChild(status);
        card.appendChild(header);

        if (r.stdout_summary || r.stderr_summary) {
          const out = document.createElement('pre');
          out.className = 'val-output';
          out.textContent = r.stdout_summary || r.stderr_summary;
          card.appendChild(out);
        }

        validationListEl.appendChild(card);
      });
    }

    while (residualListEl.firstChild) {
      residualListEl.removeChild(residualListEl.firstChild);
    }
    const residuals = Array.isArray(report.residual_risk) ? report.residual_risk : [];
    if (residuals.length === 0) {
      const empty = document.createElement('li');
      empty.className = 'security-empty-state';
      empty.textContent = '暂无残余风险记录';
      residualListEl.appendChild(empty);
    } else {
      residuals.forEach(res => {
        const li = document.createElement('li');
        li.className = 'security-residual-item';
        li.textContent = res;
        residualListEl.appendChild(li);
      });
    }
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
    setupSecurityPanel,
    safeRenderSecurityPanel,
    get lastReport() { return lastReport; },
    set lastReport(val) { lastReport = val; }
  });
})(window);
