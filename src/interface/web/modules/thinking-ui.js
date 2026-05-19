(function (global) {
  'use strict';

  function getThinkingTag(buffer) {
    const tags = [
      { open: '<thinking>', close: '</thinking>' },
      { open: '<think>', close: '</think>' },
    ];
    return tags
      .map(tag => ({ ...tag, index: buffer.indexOf(tag.open) }))
      .filter(tag => tag.index !== -1)
      .sort((a, b) => a.index - b.index)[0] || null;
  }

  function parseThinkingStream(buffer) {
    const text = String(buffer || '');
    const respOpen = '<response>';
    const respClose = '</response>';
    const tag = getThinkingTag(text);
    if (!tag) {
      return { thinking: null, response: text, state: 'idle' };
    }
    const tStart = tag.index;
    const beforeThinking = text.slice(0, tStart);
    const tEnd = text.indexOf(tag.close, tStart + tag.open.length);
    if (tEnd === -1) {
      const thinking = text.slice(tStart + tag.open.length);
      return { thinking, response: beforeThinking || null, state: 'thinking' };
    }
    const thinking = text.slice(tStart + tag.open.length, tEnd).trim();
    let response = '';
    const rStart = text.indexOf(respOpen, tEnd);
    if (rStart !== -1) {
      const rEnd = text.indexOf(respClose, rStart);
      response = rEnd !== -1
        ? text.slice(rStart + respOpen.length, rEnd)
        : text.slice(rStart + respOpen.length);
    } else {
      response = beforeThinking + text.slice(tEnd + tag.close.length);
    }
    return { thinking, response, state: 'response' };
  }

  function parseStreamEvent(buffer, event = {}) {
    const current = String(buffer || '');
    const chunk = event && Object.prototype.hasOwnProperty.call(event, 'chunk')
      ? String(event.chunk || '')
      : '';
    const nextBuffer = chunk ? current + chunk : current;
    const done = Boolean(event && event.done);

    if (event && event.error) {
      return {
        buffer: nextBuffer,
        type: 'error',
        state: 'error',
        thinking: null,
        response: null,
        error: String(event.chunk || event.error),
        done,
      };
    }

    if (event && Object.prototype.hasOwnProperty.call(event, 'thinking_content')) {
      return {
        buffer: nextBuffer,
        type: 'thinking',
        state: 'thinking',
        thinking: String(event.thinking_content || ''),
        response: null,
        error: null,
        done,
      };
    }

    const parsed = parseThinkingStream(nextBuffer);
    const type = parsed.state === 'thinking' ? 'thinking' : 'response';
    return {
      buffer: nextBuffer,
      type,
      state: parsed.state,
      thinking: parsed.thinking,
      response: parsed.response,
      error: null,
      done,
    };
  }

  function scheduleDomUpdate(app, fn) {
    if (app._pendingRaf) cancelAnimationFrame(app._pendingRaf);
    app._pendingRaf = requestAnimationFrame(() => {
      app._pendingRaf = null;
      fn();
    });
  }

  function startTraceSubscription(app) {
    if (!global.HajimiTauri?.isAvailable?.()) {
      app.traceEvents = [];
      app.renderTraceCards();
      console.warn('Agent trace channel unavailable; no fallback trace data inserted.');
      return;
    }
    try {
      const invoke = global.HajimiTauri.invoke;
      const Channel = global.HajimiTauri.Channel;
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
    const activePanel = document.querySelector('.assistant-turn:last-child .thinking-panel');
    if (activePanel) {
      setThinkingContent(activePanel, content);
      setThinkingState(activePanel, 'thinking');
      return;
    }
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

  function getAssistantTurnContract() {
    return {
      root: 'article.assistant-turn[data-turn-id]',
      body: '.assistant-turn-body',
      thinking: {
        root: '.thinking-panel[data-state][data-collapsed]',
        states: ['thinking', 'done', 'empty', 'error'],
        content: '.thinking-content',
        resizeHandle: '.thinking-resize-handle',
      },
      response: {
        root: '.assistant-response',
        content: '.assistant-response-content',
        states: ['pending', 'streaming', 'done', 'error'],
      },
      stateFields: [
        'id',
        'role',
        'createdAt',
        'updatedAt',
        'thinking.state',
        'thinking.content',
        'thinking.startedAt',
        'thinking.completedAt',
        'thinking.elapsedMs',
        'thinking.collapsed',
        'thinking.height',
        'response.state',
        'response.content',
        'response.error',
      ],
    };
  }

  const THINKING_STATES = new Set(['thinking', 'done', 'empty', 'error']);

  function unwrapThinkingPanel(panel) {
    if (!panel) return null;
    if (panel.root) return panel;
    if (panel._thinkingPanel) return panel._thinkingPanel;
    return { root: panel };
  }

  function setHeaderExpanded(header, expanded) {
    if (!header) return;
    if (typeof header.setAttribute === 'function') {
      header.setAttribute('aria-expanded', expanded ? 'true' : 'false');
    } else {
      header.ariaExpanded = expanded ? 'true' : 'false';
    }
  }

  function createThinkingPanel(app, options = {}) {
    const root = document.createElement('section');
    root.className = 'thinking-panel';
    root.dataset.state = 'empty';
    root.dataset.collapsed = options.collapsed === false ? 'false' : 'true';

    const header = document.createElement('button');
    header.className = 'thinking-panel-header';
    header.type = 'button';
    setHeaderExpanded(header, root.dataset.collapsed === 'false');

    const icon = document.createElement('span');
    icon.className = 'thinking-icon';
    const title = document.createElement('span');
    title.className = 'thinking-title';
    const meta = document.createElement('span');
    meta.className = 'thinking-meta';
    const toggle = document.createElement('span');
    toggle.className = 'thinking-toggle';
    header.appendChild(icon);
    header.appendChild(title);
    header.appendChild(meta);
    header.appendChild(toggle);

    const body = document.createElement('div');
    body.className = 'thinking-panel-body';
    const content = document.createElement('div');
    content.className = 'thinking-content';
    const resizeHandle = document.createElement('div');
    resizeHandle.className = 'thinking-resize-handle';
    body.appendChild(content);
    body.appendChild(resizeHandle);
    root.appendChild(header);
    root.appendChild(body);

    const panel = { root, header, icon, title, meta, toggle, body, content, resizeHandle, app };
    root._thinkingPanel = panel;
    header.addEventListener('click', () => toggleThinkingPanel(panel));
    setThinkingState(panel, options.state || 'empty', options);
    if (options.content) setThinkingContent(panel, options.content);
    bindThinkingResize(panel);
    return panel;
  }

  function setThinkingState(panel, state, patch = {}) {
    const handle = unwrapThinkingPanel(panel);
    if (!handle || !handle.root) return null;
    const nextState = THINKING_STATES.has(state) ? state : 'empty';
    handle.root.dataset.state = nextState;

    const collapsed = handle.root.dataset.collapsed !== 'false';
    setHeaderExpanded(handle.header, !collapsed);
    if (handle.toggle) handle.toggle.textContent = collapsed ? '▾' : '▴';

    const elapsedMs = patch.elapsedMs;
    const elapsed = typeof elapsedMs === 'number' && elapsedMs >= 1000
      ? `用时 ${Math.max(1, Math.round(elapsedMs / 1000))} 秒`
      : '';

    if (handle.icon) {
      handle.icon.textContent = nextState === 'thinking' ? '...' : nextState === 'error' ? '!' : 'i';
    }
    if (handle.title) {
      if (nextState === 'thinking') handle.title.textContent = '正在思考...';
      else if (nextState === 'done') handle.title.textContent = '已思考';
      else if (nextState === 'error') handle.title.textContent = '思考过程出错';
      else handle.title.textContent = '未返回显式思考过程';
    }
    if (handle.meta) {
      if (nextState === 'thinking') handle.meta.textContent = elapsed || '等待模型返回思考内容';
      else if (nextState === 'done') handle.meta.textContent = elapsed || '可展开查看';
      else if (nextState === 'error') handle.meta.textContent = '查看错误详情';
      else handle.meta.textContent = '面板保留为空状态';
    }
    if (handle.content && !handle.content.textContent && !handle.content.innerHTML) {
      if (nextState === 'empty') handle.content.textContent = '未返回显式思考过程';
      else if (nextState === 'thinking') handle.content.textContent = '正在思考...';
      else if (nextState === 'error') handle.content.textContent = '思考过程出错';
    }
    return handle;
  }

  function setThinkingContent(panel, content) {
    const handle = unwrapThinkingPanel(panel);
    if (!handle || !handle.content) return null;
    const safeContent = handle.app && handle.app.safeText
      ? handle.app.safeText(content || '')
      : String(content || '');
    if (!safeContent.trim()) {
      handle.content.textContent = '';
      setThinkingState(handle, 'empty');
      return handle;
    }
    if (handle.app && handle.app.renderMarkdown) {
      handle.content.innerHTML = handle.app.renderMarkdown(safeContent);
    } else {
      handle.content.textContent = safeContent;
    }
    setThinkingState(handle, 'thinking');
    return handle;
  }

  function toggleThinkingPanel(panel) {
    const handle = unwrapThinkingPanel(panel);
    if (!handle || !handle.root) return null;
    const nextCollapsed = handle.root.dataset.collapsed === 'false';
    handle.root.dataset.collapsed = nextCollapsed ? 'true' : 'false';
    setHeaderExpanded(handle.header, !nextCollapsed);
    if (handle.toggle) handle.toggle.textContent = nextCollapsed ? '▾' : '▴';
    return handle;
  }

  function bindThinkingResize(panel) {
    const handle = unwrapThinkingPanel(panel);
    if (!handle || !handle.root) return null;
    handle.root.dataset.resize = 'deferred';
    return handle;
  }

  // Legacy processing card. Keep this for /compact, auto compact, and non-chat
  // task progress only; the normal chat stream must move to assistant-turn v2.
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
          <span class="thinking-status-text">正在处理...</span>
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
    parseStreamEvent,
    scheduleDomUpdate,
    startTraceSubscription,
    renderTraceCards,
    clearTraceCards,
    toggleTracePause,
    getAssistantTurnContract,
    createThinkingPanel,
    setThinkingState,
    setThinkingContent,
    toggleThinkingPanel,
    bindThinkingResize,
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
