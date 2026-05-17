(function (global) {
  'use strict';

  function parseThinkingStream(buffer) {
    const thinkOpen = '<thinking>';
    const thinkClose = '</thinking>';
    const respOpen = '<response>';
    const respClose = '</response>';
    const tStart = buffer.indexOf(thinkOpen);
    if (tStart === -1) {
      return { thinking: null, response: buffer, state: 'idle' };
    }
    const tEnd = buffer.indexOf(thinkClose, tStart);
    if (tEnd === -1) {
      const thinking = buffer.slice(tStart + thinkOpen.length);
      return { thinking, response: null, state: 'thinking' };
    }
    const thinking = buffer.slice(tStart + thinkOpen.length, tEnd).trim();
    let response = '';
    const rStart = buffer.indexOf(respOpen, tEnd);
    if (rStart !== -1) {
      const rEnd = buffer.indexOf(respClose, rStart);
      response = rEnd !== -1
        ? buffer.slice(rStart + respOpen.length, rEnd)
        : buffer.slice(rStart + respOpen.length);
    } else {
      response = buffer.slice(tEnd + thinkClose.length);
    }
    return { thinking, response, state: 'response' };
  }

  function scheduleDomUpdate(app, fn) {
    if (app._pendingRaf) cancelAnimationFrame(app._pendingRaf);
    app._pendingRaf = requestAnimationFrame(() => {
      app._pendingRaf = null;
      fn();
    });
  }

  function startTraceSubscription(app) {
    const tauri = global.__TAURI__;
    if (!tauri || !tauri.core || !tauri.core.Channel) {
      app.traceEvents = [];
      app.renderTraceCards();
      console.warn('Agent trace channel unavailable; no fallback trace data inserted.');
      return;
    }
    const invoke = tauri.core.invoke;
    try {
      const Channel = tauri.core.Channel;
      const channel = new Channel();
      channel.onmessage = (event) => {
        if (app.tracePaused) return;

        app.traceEvents.push(event);
        if (app.traceEvents.length > 100) app.traceEvents.shift();
        if (app.sidebarView === 'agent-trace') app.renderTraceCards();
        app.safeRenderTraceInspector();
        if (event.step_type === 'EditProposed') app.onEditProposed(event);
        if (event.thinking_content) updateActiveThinking(app, event.thinking_content);
        if (event.operation_summary) app.updateOperationSummary(event.operation_summary, event.tool_name);
        if (event.step_type === 'Act') app.updateOperationProgress(actionStatusText(event.tool_name));
      };
      invoke('subscribe_agent_trace', { onEvent: channel }).catch(() => {
        app.traceEvents = [];
        app.renderTraceCards();
        console.warn('Agent trace subscription failed; no fallback trace data inserted.');
      });
    } catch (e) {
      app.traceEvents = [];
      app.renderTraceCards();
      console.warn('Agent trace setup failed; no fallback trace data inserted.', e);
    }
  }

  function updateActiveThinking(app, content) {
    const activeThinking = document.querySelector('.chat-message.ai .thinking-block');
    if (!activeThinking) return;
    activeThinking.style.display = 'block';
    const md = activeThinking.querySelector('.thinking-block-markdown');
    if (md) md.innerHTML = app.renderMarkdown(content);
  }

  function actionStatusText(toolName) {
    const lower = (toolName || '').toLowerCase();
    if (lower.includes('edit')) return '编辑中...';
    if (lower.includes('delete')) return '删除中...';
    if (lower.includes('create')) return '创建中...';
    return '执行中...';
  }

  function renderTraceCards(app) {
    const panel = document.getElementById('tracePanel');
    if (!panel) return;
    if (app.traceEvents.length === 0) {
      panel.innerHTML = '<div class="trace-empty" style="color:var(--fg-dim);text-align:center;padding:20px;">暂无思考过程</div>';
      return;
    }
    const colors = { Observe: 'var(--fg-green)', Retrieve: 'var(--fg-cyan)', Plan: 'var(--fg-red)', Act: 'var(--fg-dim)', Reflect: 'var(--fg-magenta)', Store: 'var(--fg-dim)', Decide: 'var(--fg-cyan)', Other: 'var(--fg-dim)' };
    panel.innerHTML = app.traceEvents.slice().reverse().map(ev => {
      const color = colors[ev.step_type] || colors.Other;
      const confidence = ev.confidence_score != null ? `<span style="color:var(--fg-red)">(${ev.confidence_score.toFixed(2)})</span>` : '';
      const plan = ev.plan_summary ? `<div style="margin-top:4px;color:var(--fg-dim);font-size:11px;white-space:pre-wrap;">${app.escapeHtml(app.safeText(ev.plan_summary).substring(0, 200))}</div>` : '';
      return `<div class="trace-card" style="border-left:3px solid ${color};padding:6px 8px;margin-bottom:6px;background:var(--bg-hover);border-radius:4px;">
        <div style="display:flex;justify-content:space-between;align-items:center;">
          <span style="font-weight:bold;color:${color};font-size:11px;">${app.escapeHtml(ev.step || ev.step_type || 'Other')} ${confidence}</span>
          <span style="color:var(--fg-dim);font-size:10px;">#${app.escapeHtml(ev.iteration ?? '')}</span>
        </div>
        <div style="color:var(--fg-default);margin-top:2px;font-size:12px;">${app.escapeHtml(ev.details || '')}</div>
        ${plan}
      </div>`;
    }).join('');
  }

  function clearTraceCards(app) {
    if (!confirm('确定要清空 Agent Trace 记录吗？')) return;
    app.traceEvents = [];
    app.renderTraceCards();
  }

  function toggleTracePause(app, btn) {
    app.tracePaused = !app.tracePaused;
    const target = btn || document.getElementById('pauseTraceBtn');
    if (target) {
      target.innerHTML = app.tracePaused ? '▶' : '⏸';
      target.title = app.tracePaused ? '继续' : '暂停';
    }
  }

  function addThinking(app) {
    const id = 't-' + Date.now();
    const container = document.getElementById('aiChatMessages');
    const div = document.createElement('div');
    div.className = 'chat-message ai agent-card';
    div.id = id;
    div.innerHTML = `
      <div class="chat-message-avatar">H</div>
      <div class="chat-message-body message-card">
        <div class="thinking-indicator">
          <div class="thinking-dot"></div>
          <div class="thinking-dot"></div>
          <div class="thinking-dot"></div>
        </div>
      </div>
    `;
    const block = app.createThinkingBlock();
    block.style.display = 'none';
    div.querySelector('.chat-message-body').appendChild(block);
    container.appendChild(div);
    container.scrollTop = container.scrollHeight;
    return id;
  }

  function removeThinking(id) {
    const el = document.getElementById(id);
    if (el) el.remove();
  }

  function createThinkingBlock(app, content) {
    const block = document.createElement('div');
    block.className = 'thinking-block';
    block.innerHTML = `
      <div class="thinking-block-header">
        <span class="thinking-block-icon">🧠</span>
        <span class="thinking-block-title">Thinking</span>
        <button class="thinking-block-toggle" title="Toggle" aria-label="Toggle">▼</button>
      </div>
      <div class="thinking-block-body">
        <div class="thinking-block-markdown">${app.renderMarkdown(content || '')}</div>
      </div>`;
    const btn = block.querySelector('.thinking-block-toggle');
    btn.addEventListener('click', () => app.toggleThinking(block));
    block.querySelector('.thinking-block-header').addEventListener('click', (e) => {
      if (e.target !== btn) app.toggleThinking(block);
    });
    return block;
  }

  function toggleThinking(block) {
    const body = block.querySelector('.thinking-block-body');
    const btn = block.querySelector('.thinking-block-toggle');
    const collapsed = !body.classList.contains('visible');
    if (collapsed) {
      body.classList.add('visible');
      btn.textContent = '▲';
    } else {
      body.classList.remove('visible');
      btn.textContent = '▼';
    }
  }

  function updateThinkingContent(app, id, content) {
    const el = document.getElementById(id);
    if (!el) return;
    const block = el.querySelector('.thinking-block');
    if (!block) return;
    const md = block.querySelector('.thinking-block-markdown');
    if (md) md.innerHTML = app.renderMarkdown(content || '');
    block.style.display = 'block';
  }

  function createOperationSummaryBar(app, summary, toolName) {
    const filesEdited = summary.files_edited || 0;
    const filesCreated = summary.files_created || 0;
    const filesDeleted = summary.files_deleted || 0;
    const commandsRun = summary.commands_run || 0;
    const totalDiffLines = summary.total_diff_lines || 0;
    const totalOps = filesEdited + filesCreated + filesDeleted + commandsRun;
    if (totalOps === 0) return null;

    const bar = document.createElement('div');
    bar.className = 'operation-summary-bar';
    bar._summary = summary;
    const parts = [];
    if (filesEdited > 0) parts.push(`已编辑 ${filesEdited} 个文件`);
    if (filesCreated > 0) parts.push(`已创建 ${filesCreated} 个文件`);
    if (filesDeleted > 0) parts.push(`已删除 ${filesDeleted} 个文件`);
    if (commandsRun > 0) parts.push(`已运行 ${commandsRun} 条命令`);
    const summaryText = parts.join('，');
    const reason = app.generateOperationReason(summary, toolName);

    bar.innerHTML = `
      <div class="operation-summary-header">
        <span class="operation-summary-icon">⚡</span>
        <span class="operation-summary-text">${app.escapeHtml(summaryText)}</span>
        ${reason ? `<span class="operation-summary-reason">${app.escapeHtml(reason)}</span>` : ''}
        <span class="operation-summary-progress"></span>
        <button class="operation-summary-diff-entry" title="在检查器中查看 Diff">Diff 预览</button>
        <button class="operation-summary-toggle" title="展开/折叠">▼</button>
      </div>
      <div class="operation-summary-details" data-lazy="true">
        <div class="operation-summary-stat"><span class="operation-summary-stat-label diff-add">+</span><span>编辑: ${filesEdited}</span></div>
        <div class="operation-summary-stat"><span class="operation-summary-stat-label diff-add">+</span><span>创建: ${filesCreated}</span></div>
        <div class="operation-summary-stat"><span class="operation-summary-stat-label diff-del">-</span><span>删除: ${filesDeleted}</span></div>
        <div class="operation-summary-stat"><span class="operation-summary-stat-label">⌘</span><span>命令: ${commandsRun}</span></div>
        <div class="operation-summary-stat"><span class="operation-summary-stat-label">≡</span><span>Diff 行数: ${totalDiffLines}</span></div>
        <div class="operation-summary-diff-preview"></div>
      </div>`;

    const toggle = bar.querySelector('.operation-summary-toggle');
    const diffEntry = bar.querySelector('.operation-summary-diff-entry');
    toggle.addEventListener('click', () => app.toggleDetails(bar));
    diffEntry.addEventListener('click', (e) => {
      e.stopPropagation();
      app.openDiffPreview(app.currentDiffFile);
    });
    bar.querySelector('.operation-summary-header').addEventListener('click', (e) => {
      if (e.target !== toggle && e.target !== diffEntry) app.toggleDetails(bar);
    });
    return bar;
  }

  function toggleDetails(app, bar) {
    const details = bar.querySelector('.operation-summary-details');
    const toggle = bar.querySelector('.operation-summary-toggle');
    const expanded = details.classList.contains('visible');
    if (expanded) {
      details.classList.remove('visible');
      toggle.textContent = '▼';
      return;
    }
    details.classList.add('visible');
    toggle.textContent = '▲';
    if (details.dataset.lazy === 'true') {
      app.renderOperationDiffPreview(details.querySelector('.operation-summary-diff-preview'), bar._summary);
      details.dataset.lazy = 'false';
    }
  }

  function updateOperationSummary(app, summary, toolName) {
    if (!summary || typeof summary !== 'object') return;
    const container = document.getElementById('aiChatMessages');
    if (!container) return;
    const lastAi = container.querySelector('.chat-message.ai:last-child');
    if (!lastAi) return;
    const body = lastAi.querySelector('.chat-message-body');
    if (!body) return;
    const existing = body.querySelector('.operation-summary-bar');
    if (existing) existing.remove();
    const bar = app.createOperationSummaryBar(summary, toolName);
    if (bar) body.appendChild(bar);
  }

  function generateOperationReason(summary, toolName) {
    const edited = summary.files_edited || 0;
    const created = summary.files_created || 0;
    const deleted = summary.files_deleted || 0;
    const commands = summary.commands_run || 0;
    const parts = [];
    if (edited > 0) parts.push(`编辑 ${edited} 个文件`);
    if (created > 0) parts.push(`创建 ${created} 个新文件`);
    if (deleted > 0) parts.push(`删除 ${deleted} 个旧文件`);
    if (commands > 0) parts.push(`运行 ${commands} 条命令`);
    if (parts.length === 0) return '';
    let reason = '我准备' + parts.join('，');
    if (toolName) {
      const lower = (toolName + '').toLowerCase();
      if (lower.includes('edit')) reason += '以优化代码结构';
      else if (lower.includes('delete')) reason += '以清理冗余代码';
      else if (lower.includes('create')) reason += '以添加新功能';
      else if (lower.includes('test')) reason += '以验证正确性';
      else reason += '以完成任务';
    } else {
      reason += '以完成任务';
    }
    return reason;
  }

  function renderOperationDiffPreview(app, container, summary) {
    if (!container || !summary) return;
    const files = Array.isArray(summary.files) ? summary.files : [];
    const lines = files.map((file) => {
      const path = file.path || file.file_path || file.name || 'unknown';
      const status = file.status || file.change_type || 'modified';
      const prefix = status === 'created' || status === 'added' ? '+' : status === 'deleted' || status === 'removed' ? '-' : '~';
      return `${prefix} ${status}: ${path}`;
    });
    const limit = 50;
    const visible = lines.slice(0, limit);
    let html = visible.map(l => {
      const cls = l.startsWith('+') ? 'diff-add' : l.startsWith('-') ? 'diff-del' : 'diff-hunk';
      return `<div class="diff-preview-line ${cls}">${app.escapeHtml(l)}</div>`;
    }).join('');
    if (!html) {
      const edited = summary.files_edited || 0;
      const created = summary.files_created || 0;
      const deleted = summary.files_deleted || 0;
      const diffLines = summary.total_diff_lines || 0;
      html = `<div class="diff-preview-line diff-hunk">${app.escapeHtml(`无文件级 diff 数据；来源 TraceEvent.operation_summary，edited=${edited}, created=${created}, deleted=${deleted}, diff_lines=${diffLines}`)}</div>`;
    }
    if (lines.length > limit) {
      html += `<div class="diff-preview-more">... 以及 ${lines.length - limit} 行更多</div>`;
    }
    html += '<div class="diff-preview-footer"><span class="diff-preview-link">数据源: TraceEvent.operation_summary</span></div>';
    container.innerHTML = html;
  }

  function updateOperationProgress(text) {
    if (!text) return;
    const container = document.getElementById('aiChatMessages');
    if (!container) return;
    const lastAi = container.querySelector('.chat-message.ai:last-child');
    if (!lastAi) return;
    const bar = lastAi.querySelector('.operation-summary-bar');
    if (!bar) return;
    const progress = bar.querySelector('.operation-summary-progress');
    if (!progress) return;
    progress.textContent = text;
    progress.classList.add('active');
  }

  function startSessionReplay(app, entries, startIndex) {
    app.replayEvents = (entries || []).map(ev => ({
      ...ev,
      source: ev.source || (ev.checkpoint_id ? 'edit_history+checkpoint' : 'edit_history')
    }));
    app.replayIndex = startIndex;
    const bar = document.getElementById('sessionReplayBar');
    if (bar) bar.style.display = 'flex';
    app.updateReplayStatus();
    const traceTab = document.querySelector('.trace-tab[data-tab="trace"]');
    if (traceTab) traceTab.click();
  }

  function replayStep(app, dir) {
    const newIdx = app.replayIndex + dir;
    if (newIdx < 0 || newIdx >= app.replayEvents.length) return;
    app.replayIndex = newIdx;
    app.updateReplayStatus();
    const ev = app.replayEvents[app.replayIndex];
    if (!ev) return;
    const panel = document.getElementById('tracePanel');
    if (!panel) return;
    const entry = document.createElement('div');
    entry.style.cssText = 'padding:4px 8px;margin:4px 0;background:var(--bg-hover);border-radius:4px;font-size:11px;border-left:3px solid var(--fg-red);';
    const checkpointText = ev.checkpoint_id ? ` · checkpoint ${ev.checkpoint_id}` : '';
    const sourceText = ev.source ? ` · source ${ev.source}` : '';
    entry.innerHTML = `<strong>Replay [${app.replayIndex + 1}/${app.replayEvents.length}]</strong> ${app.escapeHtml(ev.step_type || 'Checkpoint')}: ${app.escapeHtml(ev.summary || '').substring(0, 100)}<div style="color:var(--fg-dim);margin-top:2px;">${app.escapeHtml(checkpointText + sourceText)}</div>`;
    panel.insertBefore(entry, panel.firstChild);
    if (ev.thinking_content) app.renderReplayThinking(entry, ev.thinking_content);
    if (ev.operation_summary) {
      const bar = app.createOperationSummaryBar(ev.operation_summary, ev.tool_name);
      if (bar) {
        bar.style.marginTop = '4px';
        entry.appendChild(bar);
      }
    }
  }

  function closeSessionReplay(app) {
    app.replayIndex = -1;
    app.replayEvents = [];
    const bar = document.getElementById('sessionReplayBar');
    if (bar) bar.style.display = 'none';
  }

  function updateReplayStatus(app) {
    const el = document.getElementById('replayStatus');
    if (el) el.textContent = `${app.replayIndex + 1} / ${app.replayEvents.length}`;
  }

  function buildTimelineEvent(type, payload) {
    return {
      id: 'tl-' + Date.now() + '-' + Math.random().toString(36).slice(2, 6),
      timestamp: Date.now(),
      type: type || 'trace_step',
      payload: payload || {},
      source: payload?.agent_id || 'unknown'
    };
  }

  function getTimelineEvents(app, filter) {
    const events = app.traceEvents.map(ev => {
      let type = 'trace_step';
      if (ev.thinking_content) type = 'agent_thinking';
      else if (ev.operation_summary) type = 'agent_action';
      else if (ev.tool_name) type = 'tool_result';
      return app.buildTimelineEvent(type, ev);
    });
    if (filter === 'thinking') return events.filter(e => e.type === 'agent_thinking');
    if (filter === 'action') return events.filter(e => e.type === 'agent_action' || e.type === 'tool_result');
    return events;
  }

  function renderReplayThinking(app, container, thinking) {
    if (!container || !thinking) return;
    const div = document.createElement('div');
    div.style.cssText = 'margin-top:4px;padding:4px;border-left:2px solid var(--fg-cyan);font-size:11px;color:var(--fg-dim);';
    div.innerHTML = `<strong>Thinking:</strong> ${app.renderMarkdown(thinking.substring(0, 200))}`;
    container.appendChild(div);
  }

  global.HajimiThinkingUI = {
    parseThinkingStream,
    scheduleDomUpdate,
    startTraceSubscription,
    renderTraceCards,
    clearTraceCards,
    toggleTracePause,
    addThinking,
    removeThinking,
    createThinkingBlock,
    toggleThinking,
    updateThinkingContent,
    createOperationSummaryBar,
    toggleDetails,
    updateOperationSummary,
    generateOperationReason,
    renderOperationDiffPreview,
    updateOperationProgress,
    startSessionReplay,
    replayStep,
    closeSessionReplay,
    updateReplayStatus,
    buildTimelineEvent,
    getTimelineEvents,
    renderReplayThinking,
  };
})(window);
