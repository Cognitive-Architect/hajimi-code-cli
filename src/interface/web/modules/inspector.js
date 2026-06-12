(function (global) {
  'use strict';

  function getDocument() {
    return global.document || (typeof document !== 'undefined' ? document : null);
  }

  const compatInspectorView = {
    bindTabs(onTabSelected) {
      const doc = getDocument();
      if (!doc) return;
      doc.querySelectorAll('.inspector-tab').forEach(tab => {
        tab.addEventListener('click', () => {
          onTabSelected(tab.dataset.inspectorTab);
        });
      });
    },
    bindClose(onClose) {
      const closeBtn = getDocument()?.getElementById('inspectorCloseBtn');
      if (closeBtn) closeBtn.addEventListener('click', onClose);
    },
    setVisible(visible) {
      const inspector = getDocument()?.getElementById('rightInspector');
      if (inspector) inspector.style.display = visible ? '' : 'none';
    },
    setActiveTab(tabId, onActivePanel) {
      const doc = getDocument();
      if (!doc) return;

      doc.querySelectorAll('.inspector-tab').forEach(el => {
        el.classList.toggle('active', el.dataset.inspectorTab === tabId);
      });

      doc.querySelectorAll('.inspector-panel').forEach(el => {
        const isActive = el.dataset.inspectorPanel === tabId;
        el.classList.toggle('active', isActive);
        if (isActive && typeof onActivePanel === 'function') {
          onActivePanel(tabId);
        }
      });
    },
  };

  const compatInspectorController = {
    init(app) {
      compatInspectorView.bindTabs((tabId) => app.showInspectorTab(tabId));
      compatInspectorView.bindClose(() => compatInspectorView.setVisible(false));
    },
    showInspectorTab(app, tabId) {
      compatInspectorView.setActiveTab(tabId, (activeTabId) => {
        if (activeTabId === 'diff-preview') app.safeRenderInspectorDiffPreview();
        if (activeTabId === 'agent-trace') app.safeRenderTraceInspector();
      });
    },
    withInspectorGuard(app, label, renderFn) {
      try {
        renderFn();
      } catch (e) {
        console.warn(`Inspector render skipped (${label}):`, e);
      }
    },
    safeUpdateTaskDetails(app, statusText) {
      app.withInspectorGuard('task details', () => app.updateTaskDetails(statusText));
    },
    safeRenderContextFiles(app) {
      app.withInspectorGuard('context files', () => app.renderContextFiles());
    },
    safeRenderModelInfo(app) {
      app.withInspectorGuard('model info', () => app.renderModelInfo());
    },
    safeRenderInspectorDiffPreview(app) {
      app.withInspectorGuard('diff preview', () => app.renderInspectorDiffPreview());
    },
    safeRenderTraceInspector(app) {
      app.withInspectorGuard('trace summary', () => app.renderTraceInspector());
    },
    openDiffPreview(app, file = null) {
      if (file) app.currentDiffFile = file;
      compatInspectorView.setVisible(true);
      app.showInspectorTab('diff-preview');
    },
    updateTaskDetails(app, statusText) {
      const text = statusText || (app.isProcessing ? '处理中...' : '就绪');
      app.renderInspectorTaskStatus(text);
      app.renderChatShellStatus(text);
      app.renderInspectorSessionStats();
      if (typeof app.renderInspectorOperationSummary === 'function') {
        app.renderInspectorOperationSummary();
      }
    },
    renderTaskSteps(app) {
      app.renderInspectorSessionStats();
    },
  };

  function getInspectorController() {
    return global.HajimiInspectorController || compatInspectorController;
  }

  function init(app) {
    return getInspectorController().init(app);
  }

  function showInspectorTab(app, tabId) {
    return getInspectorController().showInspectorTab(app, tabId);
  }

  function withInspectorGuard(app, label, renderFn) {
    return getInspectorController().withInspectorGuard(app, label, renderFn);
  }

  function safeUpdateTaskDetails(app, statusText) {
    return getInspectorController().safeUpdateTaskDetails(app, statusText);
  }

  function safeRenderContextFiles(app) {
    return getInspectorController().safeRenderContextFiles(app);
  }

  function safeRenderModelInfo(app) {
    return getInspectorController().safeRenderModelInfo(app);
  }

  function safeRenderInspectorDiffPreview(app) {
    return getInspectorController().safeRenderInspectorDiffPreview(app);
  }

  function safeRenderTraceInspector(app) {
    return getInspectorController().safeRenderTraceInspector(app);
  }

  function openDiffPreview(app, file = null) {
    return getInspectorController().openDiffPreview(app, file);
  }

  function updateTaskDetails(app, statusText) {
    return getInspectorController().updateTaskDetails(app, statusText);
  }

  function renderTaskSteps(app) {
    return getInspectorController().renderTaskSteps(app);
  }

  function renderEditSummary(app) {
    const el = document.getElementById('inspectorEditSummary');
    if (!el) return;
    if (!app.currentEditPayload) {
      el.innerHTML = '<span style="color:var(--fg-dim);">无待处理修改</span>';
      return;
    }
    const hunks = app.currentEditPayload.hunks;
    const count = typeof hunks === 'number' ? hunks : (hunks ? hunks.length : 0);
    el.innerHTML = `
      <div style="font-size:11px;">
        <div style="font-weight:bold;color:var(--fg-magenta);">${count} 个待处理修改</div>
        <div style="color:var(--fg-dim);margin-top:2px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">${app.escapeHtml(app.currentEditPayload.summary || '无')}</div>
      </div>
    `;
  }

  function renderContextFiles(app) {
    const contextEl = document.getElementById('inspectorContextFiles');
    if (contextEl) {
      if (!app.chatContextFiles || app.chatContextFiles.length === 0) {
        contextEl.innerHTML = '<span style="color:var(--fg-dim);">暂无上下文文件</span>';
      } else {
        contextEl.innerHTML = app.chatContextFiles.map(path => {
          const name = String(path).split(/[\\/]/).pop();
          return `<div style="font-size:12px;margin-bottom:4px;color:var(--fg-default);">${app.escapeHtml(name)}</div>`;
        }).join('');
      }
    }
  }

  function renderModelInfo(app) {
    const modelEl = document.getElementById('inspectorModelInfo');
    if (modelEl) {
      if (!app.activeProviderId) {
        modelEl.innerHTML = '<span style="color:var(--fg-dim);">未选择模型</span>';
      } else {
        const cfg = app.providerConfigs.find(c => c.id === app.activeProviderId);
        const name = cfg ? (cfg.name || cfg.id) : app.activeProviderId;
        const model = cfg ? cfg.model : '';
        modelEl.innerHTML = `<div style="font-size:12px;color:var(--fg-default);">
          <div style="font-weight:bold;">${app.escapeHtml(name)}</div>
          <div style="color:var(--fg-dim);margin-top:2px;">${app.escapeHtml(model || '')}</div>
        </div>`;
      }
    }
  }

  function renderInspectorDiffPreview(app) {
    const container = document.getElementById('inspectorDiffContent');
    if (!container) return;

    if (!app.currentEditPayload || !app.currentEditPayload.hunks) {
      const fallbackText = app.currentDiffFile
        ? `可通过旧 Diff 入口查看 ${app.escapeHtml(app.currentDiffFile)}`
        : '选择文件或等待 Agent 建议修改后显示 Diff';
      container.innerHTML = `<div class="inspector-empty-state">
        <span>${fallbackText}</span>
        ${app.currentDiffFile ? '<button class="modal-btn secondary btn-secondary" id="inspectorOldDiffBtn" style="margin-top:8px;">打开旧 Diff 入口</button>' : ''}
      </div>`;
      const fallbackBtn = document.getElementById('inspectorOldDiffBtn');
      if (fallbackBtn) fallbackBtn.addEventListener('click', () => app.showGitDiff(app.currentDiffFile));
      return;
    }

    let html = `<div class="inspector-card" style="padding:0; overflow:hidden;">
      <div class="inspector-card-title" style="padding:12px 12px 8px;">${app.escapeHtml(app.currentEditPayload.summary || '修改建议')}</div>
      <div class="inspector-card-body" id="inspectorDiffList" style="padding:0;">`;

    const hunks = app.currentEditPayload.hunks;
    if (typeof hunks === 'number') {
      html += `<div style="padding:12px;color:var(--fg-dim);font-size:12px;">${hunks} 个 hunk (详细内容见主编辑器)</div>`;
    } else {
      const displayHunks = Array.isArray(hunks) ? hunks : [];
      if (displayHunks.length === 0) {
        html += `<div style="padding:12px;color:var(--fg-dim);font-size:12px;">无可用修改详情</div>`;
      } else {
        displayHunks.forEach((hunk, i) => {
          const oldLines = Array.isArray(hunk.old_lines) ? hunk.old_lines : [];
          const newLines = Array.isArray(hunk.new_lines) ? hunk.new_lines : [];
          const filePath = hunk.file_path || app.currentDiffFile || 'unknown';
          const startLine = hunk.start_line || 0;
          html += `
            <div class="inspector-diff-hunk" style="border-top:1px solid var(--border);padding:8px;">
              <div style="font-size:10px;color:var(--fg-dim);margin-bottom:4px;font-family:var(--font-mono);">${app.escapeHtml(filePath)}:${startLine}</div>
              <div style="font-family:var(--font-mono);font-size:11px;background:var(--bg-subtle);border-radius:4px;padding:6px;overflow-x:auto;line-height:1.4;">
                ${oldLines.slice(0, 5).map(l => `<div style="color:var(--fg-red);white-space:pre;">- ${app.escapeHtml(l)}</div>`).join('')}
                ${oldLines.length > 5 ? '<div style="color:var(--fg-dim);font-size:9px;">...</div>' : ''}
                ${newLines.slice(0, 5).map(l => `<div style="color:var(--fg-green);white-space:pre;">+ ${app.escapeHtml(l)}</div>`).join('')}
                ${newLines.length > 5 ? '<div style="color:var(--fg-dim);font-size:9px;">...</div>' : ''}
              </div>
            </div>
          `;
        });
      }
    }

    html += `</div></div>`;
    container.innerHTML = html;
  }

  function renderDiffPreview(app) {
    app.renderInspectorDiffPreview();
  }

  function renderTraceInspector(app) {
    const container = document.getElementById('inspectorTraceContent');
    if (!container) return;

    if (!app.traceEvents || app.traceEvents.length === 0) {
      container.innerHTML = '<div class="inspector-empty-state"><span>任务执行后显示 Trace</span></div>';
      return;
    }

    const recentEvents = app.traceEvents.slice(-15).reverse();
    const colors = { Observe: 'var(--fg-green)', Retrieve: 'var(--fg-cyan)', Plan: 'var(--fg-red)', Act: 'var(--fg-magenta)', Reflect: 'var(--fg-magenta)', Store: 'var(--fg-dim)', Decide: 'var(--fg-cyan)', Other: 'var(--fg-dim)' };

    container.innerHTML = `
      <div class="inspector-card" style="padding:8px;">
        <div class="inspector-card-title">最近执行步骤</div>
        <div class="inspector-card-body" style="padding:0;">
          ${recentEvents.map(ev => {
            const color = colors[ev.step_type] || colors.Other;
            const step = app.escapeHtml(ev.step || ev.step_type || 'Other');
            const iteration = app.escapeHtml(String(ev.iteration ?? '-'));
            const details = app.escapeHtml(ev.details || '');

            const isStoreCheckpoint = ev.step_type === 'Store' && (ev.details && ev.details.toLowerCase().includes('checkpoint'));
            const isEditStep = ev.step_type === 'EditProposed' || ev.step_type === 'EditApplied' || ev.step_type === 'EditRejected';
            const isCheckpoint = isStoreCheckpoint || isEditStep;

            let checkpointHtml = '';
            if (isCheckpoint) {
              const stepTypeLower = (ev.step_type || 'other').toLowerCase();
              const ts = new Date(ev.timestamp).getTime();
              const chkId = `chk_trace_${ev.iteration}_${stepTypeLower}_${ts}`;
              checkpointHtml = `
                <div class="trace-checkpoint-badge" style="margin-top:6px; padding:6px; background:var(--bg-subtle); border-radius:4px; font-size:10px; display:flex; flex-direction:column; gap:4px;">
                  <div style="display:flex; justify-content:space-between; align-items:center;">
                    <span style="color:var(--fg-magenta); font-weight:bold;">🔒 检查点已保存</span>
                    <span style="color:var(--fg-dim); font-family:var(--font-mono); font-size:9px;">${chkId}</span>
                  </div>
                  <div style="display:flex; gap:6px; margin-top:2px;">
                    <button class="trace-chk-btn restore" data-id="${chkId}">恢复</button>
                    <button class="trace-chk-btn compare" data-id="${chkId}">对比</button>
                  </div>
                </div>
              `;
            }

            return `
              <div style="border-left:3px solid ${color};padding:6px 8px;margin-bottom:6px;background:var(--bg-hover);border-radius:4px;font-size:11px;line-height:1.4;">
                <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:2px;">
                  <span style="font-weight:bold;color:${color};text-transform:uppercase;">${step}</span>
                  <span style="color:var(--fg-dim);font-size:10px;">#${iteration}</span>
                </div>
                <div style="color:var(--fg-default);">${details}</div>
                ${checkpointHtml}
              </div>
            `;
          }).join('')}
        </div>
      </div>
    `;

    container.querySelectorAll('.trace-chk-btn.restore').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        app.restoreCheckpoint(btn.dataset.id);
      });
    });

    container.querySelectorAll('.trace-chk-btn.compare').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        alert(`对比检查点 [${btn.dataset.id}]\n\n架构设计约束：\n本地 Trace 检查点由运行时流式生成，当前底层未包含历史完整快照。请前往编辑器“Git”面板查看实时工作区与 HEAD 的 Git Diff 差异。`);
      });
    });
  }

  async function loadLatestReceipt(app) {
    if (!app.isTauriAvailable()) return;
    try {
      const receipt = await app.invokeTauri('get_latest_receipt');
      app.renderContextReceiptPanel(receipt);
    } catch (e) {
      app.renderContextReceiptPanel(null);
    }
  }

  function renderContextReceiptPanel(app, receipt) {
    const panel = document.getElementById('contextReceiptBody');
    if (!panel) return;

    if (!receipt) {
      panel.innerHTML = `
        <div class="receipt-empty">
          <span class="receipt-empty-icon">📋</span>
          <span>暂无上下文小票（Context Receipt）</span>
          <span class="receipt-hint">每次 Agent LLM 请求后自动记录。</span>
        </div>`;
      return;
    }

    const mode = receipt.mode || 'Unknown';
    const maxCtx = (receipt.maxContextTokens || 0).toLocaleString();
    const inputBudget = (receipt.inputBudget || 0).toLocaleString();
    const estimatedInputTokens = (receipt.estimatedInputTokens || 0).toLocaleString();
    const longCtx = receipt.longContextMode ? 'Long Context ✓' : 'Fast / Standard';
    const includedCount = (receipt.includedBlocks || []).length;
    const omittedCount = (receipt.omittedBlocks || []).length;
    const bridgeRole = receipt.bridgeRole || '-';
    const model = receipt.model || '-';
    const provider = receipt.providerId || '-';
    const ts = receipt.timestamp
      ? new Date(receipt.timestamp * 1000).toLocaleTimeString()
      : '-';

    const omittedItems = (receipt.omittedBlocks || []).slice(0, 5).map(b =>
      `<li class="receipt-omit-item"><span class="omit-name">${app.escapeHtml(b.name)}</span><span class="omit-reason">${app.escapeHtml(b.reason)}</span><span class="omit-tokens">${(b.tokenEstimate ?? b.token_estimate ?? 0).toLocaleString()} tokens</span></li>`
    ).join('');
    const moreOmitted = omittedCount > 5 ? `<li class="receipt-omit-more">…还有 ${omittedCount - 5} 个省略块</li>` : '';

    panel.innerHTML = `
      <div class="receipt-header">
        <span class="receipt-title">Context Receipt</span>
        <span class="receipt-time">${app.escapeHtml(ts)}</span>
      </div>
      <div class="receipt-grid">
        <div class="receipt-row"><span class="receipt-label">Provider</span><span class="receipt-value">${app.escapeHtml(provider)}</span></div>
        <div class="receipt-row"><span class="receipt-label">Model</span><span class="receipt-value">${app.escapeHtml(model)}</span></div>
        <div class="receipt-row"><span class="receipt-label">Bridge Role</span><span class="receipt-value">${app.escapeHtml(bridgeRole)}</span></div>
        <div class="receipt-row"><span class="receipt-label">Mode</span><span class="receipt-value receipt-mode">${app.escapeHtml(mode)} · ${app.escapeHtml(longCtx)}</span></div>
        <div class="receipt-row"><span class="receipt-label">Max Context</span><span class="receipt-value">${maxCtx} tokens</span></div>
        <div class="receipt-row"><span class="receipt-label">Input Budget</span><span class="receipt-value receipt-budget">${inputBudget} tokens</span></div>
        <div class="receipt-row receipt-highlight"><span class="receipt-label">Estimated Input</span><span class="receipt-value receipt-estimate">${estimatedInputTokens} tokens</span></div>
        <div class="receipt-row"><span class="receipt-label">Included Blocks</span><span class="receipt-value receipt-included">${includedCount}</span></div>
        <div class="receipt-row"><span class="receipt-label">Omitted Blocks</span><span class="receipt-value receipt-omitted">${omittedCount}</span></div>
      </div>
      ${omittedCount > 0 ? `
      <div class="receipt-omitted-section">
        <div class="receipt-omitted-title">省略块 (Omitted Blocks)</div>
        <ul class="receipt-omit-list">${omittedItems}${moreOmitted}</ul>
      </div>` : ''}
      <div class="receipt-disclaimer">⚠️ 以上 Token 数为估算值，非 Provider 实际计费用量。</div>
    `;
  }

  function setupReceiptPanel(app) {
    loadLatestReceipt(app);
    const refreshBtn = document.getElementById('refreshReceiptBtn');
    if (refreshBtn) {
      refreshBtn.addEventListener('click', () => loadLatestReceipt(app));
    }
  }

  global.HajimiInspector = {
    init,
    showInspectorTab,
    withInspectorGuard,
    safeUpdateTaskDetails,
    safeRenderContextFiles,
    safeRenderModelInfo,
    safeRenderInspectorDiffPreview,
    safeRenderTraceInspector,
    openDiffPreview,
    updateTaskDetails,
    renderTaskSteps,
    renderEditSummary,
    renderContextFiles,
    renderModelInfo,
    renderInspectorDiffPreview,
    renderDiffPreview,
    renderTraceInspector,
    loadLatestReceipt,
    renderContextReceiptPanel,
    setupReceiptPanel,
  };
})(window);
