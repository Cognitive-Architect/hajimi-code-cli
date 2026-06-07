(function (global) {
  'use strict';

  function formatSessionTime(value) {
    const time = value ? new Date(value) : new Date();
    const now = new Date();
    if (time.toDateString() === now.toDateString()) {
      return time.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', hour12: false });
    }
    const yesterday = new Date(now);
    yesterday.setDate(now.getDate() - 1);
    if (time.toDateString() === yesterday.toDateString()) {
      return '昨天';
    }
    return `${time.getMonth() + 1}月${time.getDate()}日`;
  }

  function renderSessionList(app) {
    const list = document.getElementById('sessionList');
    if (!list) return;
    if (!app.chatSessions.length) {
      list.innerHTML = '<div class="session-empty">暂无会话</div>';
      app.renderLiveShellState?.();
      return;
    }
    list.innerHTML = app.chatSessions.map(s => `
      <div class="session-item ${s.id === app.activeSessionId ? 'active' : ''}" data-session="${app.escapeAttr(s.id)}">
        <div class="session-item-main">
          <div class="session-title">${app.escapeHtml(s.title || '会话')}</div>
          <div class="session-preview">${app.escapeHtml(s.preview || '')}</div>
        </div>
        <span class="session-time">${app.escapeHtml(formatSessionTime(s.updatedAt || s.createdAt))}</span>
      </div>
    `).join('');

    list.querySelectorAll('.session-item').forEach(el => {
      el.addEventListener('click', () => {
        const id = el.dataset.session;
        if (id && id !== app.activeSessionId) {
          app.switchSession(id);
        }
      });
    });
    app.renderLiveShellState?.();
  }

  const api = {
    formatSessionTime,
    renderSessionList,
  };

  global.HajimiSessionListView = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
