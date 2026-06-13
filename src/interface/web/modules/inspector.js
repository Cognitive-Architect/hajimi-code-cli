(function (global) {
  'use strict';

  function getDocument() {
    return global.document || (typeof document !== 'undefined' ? document : null);
  }

  function createNode(tagName, options = {}) {
    const doc = getDocument();
    if (!doc || typeof doc.createElement !== 'function') return null;

    const node = doc.createElement(tagName);
    if (options.id) node.id = options.id;
    if (options.className) node.className = options.className;
    if (options.text !== undefined) node.textContent = String(options.text);
    if (options.style && node.style) node.style.cssText = options.style;
    if (options.dataset) {
      Object.keys(options.dataset).forEach(key => {
        node.dataset[key] = String(options.dataset[key]);
      });
    }
    return node;
  }

  function replaceNodeChildren(node, children = []) {
    if (!node) return;
    const safeChildren = children.filter(Boolean);
    if (typeof node.replaceChildren === 'function') {
      node.replaceChildren(...safeChildren);
      return;
    }

    while (node.firstChild) {
      node.removeChild(node.firstChild);
    }
    safeChildren.forEach(child => node.appendChild(child));
  }

  function appendNode(parent, child) {
    if (parent && child) parent.appendChild(child);
    return child;
  }

  function appendTextNode(parent, tagName, text, options = {}) {
    return appendNode(parent, createNode(tagName, Object.assign({}, options, { text })));
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
      replaceNodeChildren(el, [
        createNode('span', {
          text: '无待处理修改',
          style: 'color:var(--fg-dim);',
        }),
      ]);
      return;
    }
    const hunks = app.currentEditPayload.hunks;
    const count = typeof hunks === 'number' ? hunks : (hunks ? hunks.length : 0);
    const wrapper = createNode('div', { style: 'font-size:11px;' });
    appendTextNode(wrapper, 'div', `${count} 个待处理修改`, {
      style: 'font-weight:bold;color:var(--fg-magenta);',
    });
    appendTextNode(wrapper, 'div', app.currentEditPayload.summary || '无', {
      style: 'color:var(--fg-dim);margin-top:2px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;',
    });
    replaceNodeChildren(el, [wrapper]);
  }

  function renderContextFiles(app) {
    const contextEl = document.getElementById('inspectorContextFiles');
    if (contextEl) {
      if (!app.chatContextFiles || app.chatContextFiles.length === 0) {
        replaceNodeChildren(contextEl, [
          createNode('span', {
            text: '暂无上下文文件',
            style: 'color:var(--fg-dim);',
          }),
        ]);
      } else {
        const nodes = app.chatContextFiles.map(path => {
          const name = String(path).split(/[\\/]/).pop();
          return createNode('div', {
            text: name,
            style: 'font-size:12px;margin-bottom:4px;color:var(--fg-default);',
          });
        });
        replaceNodeChildren(contextEl, nodes);
      }
    }
  }

  function renderModelInfo(app) {
    const modelEl = document.getElementById('inspectorModelInfo');
    if (modelEl) {
      if (!app.activeProviderId) {
        replaceNodeChildren(modelEl, [
          createNode('span', {
            text: '未选择模型',
            style: 'color:var(--fg-dim);',
          }),
        ]);
      } else {
        const cfg = app.providerConfigs.find(c => c.id === app.activeProviderId);
        const name = cfg ? (cfg.name || cfg.id) : app.activeProviderId;
        const model = cfg ? cfg.model : '';
        const wrapper = createNode('div', { style: 'font-size:12px;color:var(--fg-default);' });
        appendTextNode(wrapper, 'div', name, { style: 'font-weight:bold;' });
        appendTextNode(wrapper, 'div', model || '', { style: 'color:var(--fg-dim);margin-top:2px;' });
        replaceNodeChildren(modelEl, [wrapper]);
      }
    }
  }

  function renderInspectorDiffPreview(app) {
    const container = document.getElementById('inspectorDiffContent');
    if (!container) return;

    if (!app.currentEditPayload || !app.currentEditPayload.hunks) {
      const fallbackText = app.currentDiffFile
        ? `可通过旧 Diff 入口查看 ${app.currentDiffFile}`
        : '选择文件或等待 Agent 建议修改后显示 Diff';
      const empty = createNode('div', { className: 'inspector-empty-state' });
      appendTextNode(empty, 'span', fallbackText);
      const fallbackBtn = app.currentDiffFile
        ? appendNode(empty, createNode('button', {
          id: 'inspectorOldDiffBtn',
          className: 'modal-btn secondary btn-secondary',
          text: '打开旧 Diff 入口',
          style: 'margin-top:8px;',
        }))
        : null;
      if (fallbackBtn) fallbackBtn.addEventListener('click', () => app.showGitDiff(app.currentDiffFile));
      replaceNodeChildren(container, [empty]);
      return;
    }

    const card = createNode('div', {
      className: 'inspector-card',
      style: 'padding:0; overflow:hidden;',
    });
    appendTextNode(card, 'div', app.currentEditPayload.summary || '修改建议', {
      className: 'inspector-card-title',
      style: 'padding:12px 12px 8px;',
    });
    const body = appendNode(card, createNode('div', {
      id: 'inspectorDiffList',
      className: 'inspector-card-body',
      style: 'padding:0;',
    }));

    const hunks = app.currentEditPayload.hunks;
    if (typeof hunks === 'number') {
      appendTextNode(body, 'div', `${hunks} 个 hunk (详细内容见主编辑器)`, {
        style: 'padding:12px;color:var(--fg-dim);font-size:12px;',
      });
    } else {
      const displayHunks = Array.isArray(hunks) ? hunks : [];
      if (displayHunks.length === 0) {
        appendTextNode(body, 'div', '无可用修改详情', {
          style: 'padding:12px;color:var(--fg-dim);font-size:12px;',
        });
      } else {
        displayHunks.forEach((hunk) => {
          const oldLines = Array.isArray(hunk.old_lines) ? hunk.old_lines : [];
          const newLines = Array.isArray(hunk.new_lines) ? hunk.new_lines : [];
          const filePath = hunk.file_path || app.currentDiffFile || 'unknown';
          const startLine = hunk.start_line || 0;
          const hunkEl = appendNode(body, createNode('div', {
            className: 'inspector-diff-hunk',
            style: 'border-top:1px solid var(--border);padding:8px;',
          }));
          appendTextNode(hunkEl, 'div', `${filePath}:${startLine}`, {
            style: 'font-size:10px;color:var(--fg-dim);margin-bottom:4px;font-family:var(--font-mono);',
          });
          const linesEl = appendNode(hunkEl, createNode('div', {
            style: 'font-family:var(--font-mono);font-size:11px;background:var(--bg-subtle);border-radius:4px;padding:6px;overflow-x:auto;line-height:1.4;',
          }));
          oldLines.slice(0, 5).forEach(line => {
            appendTextNode(linesEl, 'div', `- ${line}`, {
              style: 'color:var(--fg-red);white-space:pre;',
            });
          });
          if (oldLines.length > 5) {
            appendTextNode(linesEl, 'div', '...', { style: 'color:var(--fg-dim);font-size:9px;' });
          }
          newLines.slice(0, 5).forEach(line => {
            appendTextNode(linesEl, 'div', `+ ${line}`, {
              style: 'color:var(--fg-green);white-space:pre;',
            });
          });
          if (newLines.length > 5) {
            appendTextNode(linesEl, 'div', '...', { style: 'color:var(--fg-dim);font-size:9px;' });
          }
        });
      }
    }

    replaceNodeChildren(container, [card]);
  }

  function renderDiffPreview(app) {
    app.renderInspectorDiffPreview();
  }

  function renderTraceInspector(app) {
    const container = document.getElementById('inspectorTraceContent');
    if (!container) return;

    if (!app.traceEvents || app.traceEvents.length === 0) {
      const empty = createNode('div', { className: 'inspector-empty-state' });
      appendTextNode(empty, 'span', '任务执行后显示 Trace');
      replaceNodeChildren(container, [empty]);
      return;
    }

    const recentEvents = app.traceEvents.slice(-15).reverse();
    const colors = { Observe: 'var(--fg-green)', Retrieve: 'var(--fg-cyan)', Plan: 'var(--fg-red)', Act: 'var(--fg-magenta)', Reflect: 'var(--fg-magenta)', Store: 'var(--fg-dim)', Decide: 'var(--fg-cyan)', Other: 'var(--fg-dim)' };

    const card = createNode('div', {
      className: 'inspector-card',
      style: 'padding:8px;',
    });
    appendTextNode(card, 'div', '最近执行步骤', { className: 'inspector-card-title' });
    const body = appendNode(card, createNode('div', {
      className: 'inspector-card-body',
      style: 'padding:0;',
    }));

    recentEvents.forEach(ev => {
      const color = colors[ev.step_type] || colors.Other;
      const step = ev.step || ev.step_type || 'Other';
      const iteration = String(ev.iteration ?? '-');
      const details = ev.details || '';

      const item = appendNode(body, createNode('div', {
        style: `border-left:3px solid ${color};padding:6px 8px;margin-bottom:6px;background:var(--bg-hover);border-radius:4px;font-size:11px;line-height:1.4;`,
      }));
      const header = appendNode(item, createNode('div', {
        style: 'display:flex;justify-content:space-between;align-items:center;margin-bottom:2px;',
      }));
      appendTextNode(header, 'span', step, {
        style: `font-weight:bold;color:${color};text-transform:uppercase;`,
      });
      appendTextNode(header, 'span', `#${iteration}`, {
        style: 'color:var(--fg-dim);font-size:10px;',
      });
      appendTextNode(item, 'div', details, { style: 'color:var(--fg-default);' });

      const isStoreCheckpoint = ev.step_type === 'Store' && (ev.details && ev.details.toLowerCase().includes('checkpoint'));
      const isEditStep = ev.step_type === 'EditProposed' || ev.step_type === 'EditApplied' || ev.step_type === 'EditRejected';
      const isCheckpoint = isStoreCheckpoint || isEditStep;
      if (isCheckpoint) {
        const stepTypeLower = (ev.step_type || 'other').toLowerCase();
        const ts = new Date(ev.timestamp).getTime();
        const chkId = `chk_trace_${ev.iteration}_${stepTypeLower}_${ts}`;
        const checkpoint = appendNode(item, createNode('div', {
          className: 'trace-checkpoint-badge',
          style: 'margin-top:6px; padding:6px; background:var(--bg-subtle); border-radius:4px; font-size:10px; display:flex; flex-direction:column; gap:4px;',
        }));
        const checkpointHeader = appendNode(checkpoint, createNode('div', {
          style: 'display:flex; justify-content:space-between; align-items:center;',
        }));
        appendTextNode(checkpointHeader, 'span', '🔒 检查点已保存', {
          style: 'color:var(--fg-magenta); font-weight:bold;',
        });
        appendTextNode(checkpointHeader, 'span', chkId, {
          style: 'color:var(--fg-dim); font-family:var(--font-mono); font-size:9px;',
        });
        const actions = appendNode(checkpoint, createNode('div', {
          style: 'display:flex; gap:6px; margin-top:2px;',
        }));
        appendNode(actions, createNode('button', {
          className: 'trace-chk-btn restore',
          text: '恢复',
          dataset: { id: chkId },
        }));
        appendNode(actions, createNode('button', {
          className: 'trace-chk-btn compare',
          text: '对比',
          dataset: { id: chkId },
        }));
      }
    });

    replaceNodeChildren(container, [card]);

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
      const empty = createNode('div', { className: 'receipt-empty' });
      appendTextNode(empty, 'span', '📋', { className: 'receipt-empty-icon' });
      appendTextNode(empty, 'span', '暂无上下文小票（Context Receipt）');
      appendTextNode(empty, 'span', '每次 Agent LLM 请求后自动记录。', { className: 'receipt-hint' });
      replaceNodeChildren(panel, [empty]);
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

    const nodes = [];
    const header = createNode('div', { className: 'receipt-header' });
    appendTextNode(header, 'span', 'Context Receipt', { className: 'receipt-title' });
    appendTextNode(header, 'span', ts, { className: 'receipt-time' });
    nodes.push(header);

    const grid = createNode('div', { className: 'receipt-grid' });
    appendReceiptRow(grid, 'Provider', provider, 'receipt-value');
    appendReceiptRow(grid, 'Model', model, 'receipt-value');
    appendReceiptRow(grid, 'Bridge Role', bridgeRole, 'receipt-value');
    appendReceiptRow(grid, 'Mode', `${mode} · ${longCtx}`, 'receipt-value receipt-mode');
    appendReceiptRow(grid, 'Max Context', `${maxCtx} tokens`, 'receipt-value');
    appendReceiptRow(grid, 'Input Budget', `${inputBudget} tokens`, 'receipt-value receipt-budget');
    appendReceiptRow(grid, 'Estimated Input', `${estimatedInputTokens} tokens`, 'receipt-value receipt-estimate', 'receipt-row receipt-highlight');
    appendReceiptRow(grid, 'Included Blocks', includedCount, 'receipt-value receipt-included');
    appendReceiptRow(grid, 'Omitted Blocks', omittedCount, 'receipt-value receipt-omitted');
    nodes.push(grid);

    if (omittedCount > 0) {
      const omittedSection = createNode('div', { className: 'receipt-omitted-section' });
      appendTextNode(omittedSection, 'div', '省略块 (Omitted Blocks)', { className: 'receipt-omitted-title' });
      const list = appendNode(omittedSection, createNode('ul', { className: 'receipt-omit-list' }));
      (receipt.omittedBlocks || []).slice(0, 5).forEach(block => {
        const item = appendNode(list, createNode('li', { className: 'receipt-omit-item' }));
        appendTextNode(item, 'span', block.name, { className: 'omit-name' });
        appendTextNode(item, 'span', block.reason, { className: 'omit-reason' });
        appendTextNode(item, 'span', `${(block.tokenEstimate ?? block.token_estimate ?? 0).toLocaleString()} tokens`, {
          className: 'omit-tokens',
        });
      });
      if (omittedCount > 5) {
        appendTextNode(list, 'li', `…还有 ${omittedCount - 5} 个省略块`, {
          className: 'receipt-omit-more',
        });
      }
      nodes.push(omittedSection);
    }

    nodes.push(createNode('div', {
      className: 'receipt-disclaimer',
      text: '⚠️ 以上 Token 数为估算值，非 Provider 实际计费用量。',
    }));
    replaceNodeChildren(panel, nodes);
  }

  function appendReceiptRow(parent, label, value, valueClassName, rowClassName = 'receipt-row') {
    const row = appendNode(parent, createNode('div', { className: rowClassName }));
    appendTextNode(row, 'span', label, { className: 'receipt-label' });
    appendTextNode(row, 'span', value, { className: valueClassName });
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
