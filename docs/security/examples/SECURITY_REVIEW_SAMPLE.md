# Security Review Sample Report

> Generated sample from the current repository output. This is a V1 local gate/report rendering receipt, not a complete security audit or full SAST result.

## Scope

- Repository: hajimi-code-cli
- Branch: `codex/security-workflow-day01`
- Commit: `e541aee5e9f1e71d51bdb61b93ed0a8c3c7eec6e`
- Generated at: 2026-05-23T12:39:09.457Z
- Input command: `node tests/security/security_audit_gate.js`
- Output JSON: `docs/security/examples/SECURITY_REVIEW_SAMPLE.json`
- In scope: Security Gate V1 structured findings and report rendering.
- Out of scope: external network scanning, exploit execution, Agent workflow automation, UI wiring, and real WebView click validation.

## Executive Summary

| Severity | Count |
|---|---:|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 108 |

| Status | Count |
|---|---:|
| Candidate | 0 |
| Unverified | 0 |
| Confirmed | 0 |
| Fixed | 0 |
| Accepted Risk | 108 |
| False Positive | 0 |

- Gate status: `pass`
- Findings: 108
- Failures: 0
- Warnings: 108
- Allowlisted: 108
- V1 closure status: `PARTIAL/GATED`

## Threat Model Summary

| Area | Notes |
|---|---|
| Assets | Local source tree, Tauri desktop command surface, frontend DOM rendering surface, and workspace file operations. |
| Entry points | Tauri config, web HTML/JS/CSS, desktop command registration, tool-system shell/file APIs. |
| Trust boundaries | Frontend-to-Tauri bridge, shell command execution boundary, workspace path resolver boundary. |
| High-risk operations | Shell execution, HTML injection sinks, file write/edit/delete tools, global Tauri access. |
| Assumptions | V1 is a local static regression gate; it does not prove absence of vulnerabilities. |

## Findings

### FINDING-001: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 265
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 265 | statusEl.innerHTML = ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-002: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 282
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 282 | container.innerHTML = '<div class="sidebar-live-empty">加载文件中...</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-003: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 298
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 298 | container.innerHTML = '<div class="sidebar-live-empty">当前工作区暂无可显示文件</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-004: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 311
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 311 | container.innerHTML = rows.map(({ node, depth }) => { |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-005: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 335
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 335 | providerMetaEl.innerHTML = `${this.escapeHtml(meta)} <span class="online-dot"></span>`; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-006: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 352
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 352 | summaryEl.innerHTML = `${this.mcpServers.length} 个服务已记录 · ${toolCount} 个工具 <span class="online-dot"></span>`; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-007: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 360
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 360 | el.innerHTML = '<span style="color:var(--fg-dim);">暂无会话统计</span>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-008: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 369
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 369 | el.innerHTML = ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-009: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 551
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 551 | el.innerHTML = '<span style="color:var(--fg-dim);">无待处理修改</span>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-010: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 556
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 556 | el.innerHTML = ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-011: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 568
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 568 | contextEl.innerHTML = '<span style="color:var(--fg-dim);">暂无上下文文件</span>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-012: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 570
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 570 | contextEl.innerHTML = this.chatContextFiles.map(path => { |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-013: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 582
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 582 | modelEl.innerHTML = '<span style="color:var(--fg-dim);">未选择模型</span>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-014: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 587
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 587 | modelEl.innerHTML = `<div style="font-size:12px;color:var(--fg-default);"> |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-015: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 603
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 603 | container.innerHTML = `<div class="inspector-empty-state"> |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-016: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 645
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 645 | container.innerHTML = html; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-017: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 657
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 657 | container.innerHTML = '<div class="inspector-empty-state"><span>任务执行后显示 Trace</span></div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-018: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 664
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 664 | container.innerHTML = ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-019: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 708
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 708 | searchResults.innerHTML = '<div style="padding:12px;color:var(--fg-dim);font-size:12px;">搜索中...</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-020: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 711
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 711 | searchResults.innerHTML = '<div style="padding:12px;color:var(--fg-dim);">Tauri 不可用</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-021: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 724
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 724 | searchResults.innerHTML = `<div style="padding:12px;color:var(--fg-red);">搜索失败: ${this.escapeHtml(e.message \|\| e)}</div>`; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-022: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 731
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 731 | searchResults.innerHTML = '<div style="padding:12px;color:var(--fg-dim);font-size:12px;">未找到匹配</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-023: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 751
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 751 | searchResults.innerHTML = '<div style="padding:12px;color:var(--fg-dim);font-size:12px;">未找到匹配</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-024: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 772
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 772 | searchResults.innerHTML = html; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-025: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 825
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 825 | fileList.innerHTML = '<div style="padding:12px;color:var(--fg-dim);font-size:12px;">没有更改</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-026: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 852
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 852 | fileList.innerHTML = html; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-027: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 880
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 880 | diffContent.innerHTML = colored; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-028: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 920
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 920 | statusBranch.innerHTML = `🌿 ${this.escapeHtml(branch)}`; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-029: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1023
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1023 | menu.innerHTML = items.map((item, i) => |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-030: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1090
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1090 | menu.innerHTML = items.map(item => |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-031: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1209
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1209 | tabBar.innerHTML = this.tabs.map(tab => ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-032: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1229
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1229 | editorArea.innerHTML = ''; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-033: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1250
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1250 | bar.innerHTML = ''; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-034: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1265
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1265 | bar.innerHTML = html; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-035: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1282
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1282 | container.innerHTML = ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-036: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1337
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1337 | container.innerHTML = ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-037: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1397
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1397 | lineNumbers.innerHTML = lines.map((_, i) => `<div>${i + 1}</div>`).join(''); |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-038: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1399
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1399 | editorContent.innerHTML = newHighlighted; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-039: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1568
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1568 | div.innerHTML = ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-040: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1614
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1614 | div.innerHTML = ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-041: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1663
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1663 | div.innerHTML = ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-042: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1686
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1686 | div.innerHTML = ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-043: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1782
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1782 | terminalContent.innerHTML = ''; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-044: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1834
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1834 | line.innerHTML = '<span class="terminal-prompt">$ </span><span class="terminal-input" contenteditable="true"></span>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-045: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1864
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1864 | cmdLine.innerHTML = `<span class="terminal-prompt">$ </span><span class="terminal-cmd">${this.escapeHtml(cmd)}</span>`; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-046: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1913
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1913 | problemsContent.innerHTML = '<div class="problems-empty">扫描中...</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-047: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1916
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1916 | problemsContent.innerHTML = '<div class="problems-empty">Tauri 不可用</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-048: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1931
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1931 | problemsContent.innerHTML = `<div class="problems-empty">扫描失败: ${this.escapeHtml(e.message \|\| e)}</div>`; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-049: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1971
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1971 | problemsContent.innerHTML = '<div class="problems-empty">工作区中未检测到问题。</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-050: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 1987
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 1987 | problemsContent.innerHTML = html; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-051: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 2008
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 2008 | clearBtn.innerHTML = '🗑'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-052: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 2058
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 2058 | if (outputContent) outputContent.innerHTML = ''; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-053: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 2287
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 2287 | if (chatMsgContainer) chatMsgContainer.innerHTML = ''; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-054: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 2303
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 2303 | list.innerHTML = this.chatContextFiles.map(path => { |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-055: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 3158
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 3158 | turn.responseEl.innerHTML = this.formatText(`**模型返回错误：** ${err}`); |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-056: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 3162
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 3162 | turn.responseEl.innerHTML = this.formatText(turn.state.response.content); |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-057: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 3326
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 3326 | div.innerHTML = `<div class="chat-message-avatar">${avatar}</div><div class="chat-message-body message-card">${this.formatText(text)}</div>`; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-058: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 3494
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 3494 | body.innerHTML = '<div class="model-picker-empty">暂无配置模型，点击下方按钮添加。</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-059: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 3516
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 3516 | body.innerHTML = html; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-060: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 3560
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 3560 | list.innerHTML = `<div class="provider-item-empty">暂无自定义模型，点击上方按钮添加。${workspaceTag}</div>`; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-061: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 3564
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 3564 | list.innerHTML = this.providerConfigs.map(cfg => ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-062: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 3984
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 3984 | panel.innerHTML = ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-063: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4013
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4013 | panel.innerHTML = ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-064: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4197
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4197 | select.innerHTML = html; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-065: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4268
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4268 | if (select) select.innerHTML = opts; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-066: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4272
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4272 | list.innerHTML = '<div class="agent-provider-empty">暂无绑定</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-067: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4275
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4275 | list.innerHTML = entries.map(([agentId, providerId]) => { |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-068: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4372
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4372 | list.innerHTML = '<div class="mcp-empty">暂无 MCP 服务器</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-069: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4376
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4376 | list.innerHTML = this.mcpServers.map((s, i) => ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-070: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4433
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4433 | list.innerHTML = this.extensions.map(ext => { |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-071: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4612
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4612 | tbody.innerHTML = '<tr><td colspan="4" class="audit-empty">暂无记录</td></tr>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-072: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4615
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4615 | tbody.innerHTML = logs.map(r => { |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-073: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4675
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4675 | if (!this.isTauriAvailable()) { list.innerHTML = '<div style="color:var(--fg-dim);text-align:center;padding:12px;">Tauri 不可用</div>'; return; } |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-074: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4680
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4680 | list.innerHTML = '<div style="color:var(--fg-dim);text-align:center;padding:12px;">暂无检查点</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-075: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4683
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4683 | list.innerHTML = checkpoints.map((chk, idx) => { |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-076: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4719
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4719 | list.innerHTML = '<div style="color:var(--fg-dim);text-align:center;padding:12px;">加载失败</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-077: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4764
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4764 | target.innerHTML = ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-078: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 4936
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 4936 | list.innerHTML = filtered.map((c, i) => ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-079: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 5077
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 5077 | hunksContainer.innerHTML = ''; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-080: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 5081
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 5081 | hunksContainer.innerHTML = `<div style="padding:8px;color:var(--fg-dim);font-size:12px;">${hunks} 个 hunk (详细内容未提供)</div>`; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-081: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 5090
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 5090 | hunkEl.innerHTML = ` |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-082: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 5194
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 5194 | panel.innerHTML = '<div style="color:var(--fg-dim);text-align:center;padding:20px;">Tauri 不可用</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-083: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 5201
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 5201 | panel.innerHTML = '<div style="color:var(--fg-dim);text-align:center;padding:20px;">加载失败</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-084: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 5209
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 5209 | panel.innerHTML = '<div class="edit-history-empty" style="color:var(--fg-dim);text-align:center;padding:20px;">暂无编辑历史</div>'; |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-085: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/app.js
- Line: 5213
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/app.js | 5213 | panel.innerHTML = entries.slice().reverse().map((e, i) => { |  |  | Legacy monolithic frontend rendering debt. B16 gate tracks this as warning while new slash-palette module must stay safe-DOM only. |

### FINDING-086: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/security-dom.js
- Line: 11
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Security helper intentionally uses a detached element to escape text; tracked as known implementation detail.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/security-dom.js | 11 | return div.innerHTML; |  |  | Security helper intentionally uses a detached element to escape text; tracked as known implementation detail. |

### FINDING-087: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/security-dom.js
- Line: 25
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Security helper intentionally uses a detached element to escape text; tracked as known implementation detail.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/security-dom.js | 25 | element.innerHTML = escapeHtml(value); |  |  | Security helper intentionally uses a detached element to escape text; tracked as known implementation detail. |

### FINDING-088: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/sessions.js
- Line: 39
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy session rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/sessions.js | 39 | if (messages) messages.innerHTML = ''; |  |  | Legacy session rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-089: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/sessions.js
- Line: 107
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy session rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/sessions.js | 107 | container.innerHTML = ''; |  |  | Legacy session rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-090: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/sessions.js
- Line: 135
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy session rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/sessions.js | 135 | list.innerHTML = '<div class="session-empty">暂无会话</div>'; |  |  | Legacy session rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-091: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/sessions.js
- Line: 139
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy session rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/sessions.js | 139 | list.innerHTML = app.chatSessions.map(s => ` |  |  | Legacy session rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-092: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/thinking-ui.js
- Line: 143
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/thinking-ui.js | 143 | if (md) md.innerHTML = app.renderMarkdown(content); |  |  | Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-093: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/thinking-ui.js
- Line: 158
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/thinking-ui.js | 158 | panel.innerHTML = '<div class="trace-empty" style="color:var(--fg-dim);text-align:center;padding:20px;">暂无思考过程</div>'; |  |  | Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-094: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/thinking-ui.js
- Line: 162
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/thinking-ui.js | 162 | panel.innerHTML = app.traceEvents.slice().reverse().map(ev => { |  |  | Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-095: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/thinking-ui.js
- Line: 187
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/thinking-ui.js | 187 | target.innerHTML = app.tracePaused ? '▶' : '⏸'; |  |  | Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-096: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/thinking-ui.js
- Line: 318
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/thinking-ui.js | 318 | if (handle.content && !handle.content.textContent && !handle.content.innerHTML) { |  |  | Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-097: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/thinking-ui.js
- Line: 338
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/thinking-ui.js | 338 | handle.content.innerHTML = handle.app.renderMarkdown(safeContent); |  |  | Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-098: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/thinking-ui.js
- Line: 371
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/thinking-ui.js | 371 | div.innerHTML = ` |  |  | Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-099: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/thinking-ui.js
- Line: 398
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/thinking-ui.js | 398 | block.innerHTML = ` |  |  | Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-100: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/thinking-ui.js
- Line: 434
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/thinking-ui.js | 434 | if (md) md.innerHTML = app.renderMarkdown(content \|\| ''); |  |  | Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-101: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/thinking-ui.js
- Line: 458
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/thinking-ui.js | 458 | bar.innerHTML = ` |  |  | Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-102: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/thinking-ui.js
- Line: 571
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/thinking-ui.js | 571 | container.innerHTML = html; |  |  | Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-103: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/thinking-ui.js
- Line: 614
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/thinking-ui.js | 614 | entry.innerHTML = `<strong>Replay [${app.replayIndex + 1}/${app.replayEvents.length}]</strong> ${app.escapeHtml(ev.step_type \|\| 'Checkpoint')}: ${app.escapeHtml(ev.summary \|\| '').substring(0, 100)}<div style="color:var(--fg-dim);margin-top:2px;">${app.escapeHtml(checkpointText + sourceText)}</div>`; |  |  | Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-104: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/thinking-ui.js
- Line: 665
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/thinking-ui.js | 665 | div.innerHTML = `<strong>Thinking:</strong> ${app.renderMarkdown(thinking.substring(0, 200))}`; |  |  | Legacy thinking/checkpoint rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-105: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/workspace.js
- Line: 111
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy workspace tree rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/workspace.js | 111 | container.innerHTML = ''; |  |  | Legacy workspace tree rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-106: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/workspace.js
- Line: 113
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy workspace tree rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/workspace.js | 113 | container.innerHTML = '<div style="padding:12px;color:var(--fg-dim);font-size:12px;">加载中...</div>'; |  |  | Legacy workspace tree rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-107: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/workspace.js
- Line: 124
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy workspace tree rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/workspace.js | 124 | folderEl.innerHTML = ` |  |  | Legacy workspace tree rendering remains out of B16 Day 5 scope and is tracked as warning. |

### FINDING-108: known legacy dangerous HTML API allowed with reason

- Rule ID: `DOM-HTML-001`
- Severity: `low`
- Status: `accepted_risk`
- Category: `gate`
- Confidence: n/a
- Type: n/a
- File: src/interface/web/modules/workspace.js
- Line: 159
- Recommendation: Review the rule-specific guidance and either fix the issue or record an explicit accepted-risk decision.
- Regression test: `npm run test:security-gate`
- Human review required: false
- Residual risk: Legacy workspace tree rendering remains out of B16 Day 5 scope and is tracked as warning.

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | src/interface/web/modules/workspace.js | 159 | fileEl.innerHTML = ` |  |  | Legacy workspace tree rendering remains out of B16 Day 5 scope and is tracked as warning. |

## Validation Receipts

| Command | Exit Code | Status | Stdout Summary | Stderr Summary |
|---|---:|---|---|---|
| `node tests/security/security_audit_gate.js` | 0 | `pass` | Security Audit Gate V1 summary \| findings: 108 \| failures: 0 \| warnings: 108 \| allowlisted: 108 \| warnings: |  |

## Residual Risk

- V1 coverage is limited to the current local gate and SecurityAuditTool schema family.
- Accepted-risk DOM warnings remain tracked by allowlist reasons and are not fixed.
- Findings without evidence must remain `unverified`.
- Fixes must not be marked `fixed` without revalidation receipts.
- Manual WebView / click validation debt remains `PENDING-WEBVIEW-SMOKE` per task instruction.
