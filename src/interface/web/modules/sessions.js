(function (global) {
  'use strict';

  const STORAGE_KEY = 'hajimi_chat_sessions';

  function makeSessionId() {
    return 'session-' + Date.now() + '-' + Math.random().toString(36).slice(2, 7);
  }

  function syncActiveSession(app) {
    if (!app.activeSessionId) return;
    const session = app.chatSessions.find(s => s.id === app.activeSessionId);
    if (!session) return;
    session.messages = [...app.chatMessages];
    session.updatedAt = Date.now();

    const firstUser = app.chatMessages.find(m => m.role === 'user');
    const firstAi = app.chatMessages.find(m => m.role === 'assistant');
    if (firstUser) {
      session.title = firstUser.content.slice(0, 30);
      session.preview = firstUser.content.slice(0, 60);
    } else if (firstAi) {
      session.title = firstAi.content.slice(0, 30);
      session.preview = firstAi.content.slice(0, 60);
    }
  }

  function newChatSession(app) {
    if (app.chatMessages.length > 0) {
      syncActiveSession(app);
    }

    app.activeSessionId = makeSessionId();
    app.chatMessages = [];
    app.tokenStats = { promptTokens: 0, completionTokens: 0, estimatedTokens: 0 };
    app.cumulativeStats = { promptTokens: 0, completionTokens: 0, requestCount: 0 };

    const messages = document.getElementById('aiChatMessages');
    if (messages) messages.innerHTML = '';

    app.addChatMessage('ai', '新会话已开始。有什么可以帮您的？');
    app.updateTokenDisplay();
    app.chatSessions.unshift({
      id: app.activeSessionId,
      title: '新会话',
      preview: '有什么可以帮您的？',
      messages: [],
      createdAt: Date.now(),
      updatedAt: Date.now(),
    });
    app.saveChatSessions();
    app.renderSessionList();
  }

  function loadChatSessions(app) {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) {
        app.newChatSession();
        return;
      }
      app.chatSessions = JSON.parse(raw);
      if (app.chatSessions.length === 0) {
        app.newChatSession();
        return;
      }

      const latest = app.chatSessions[0];
      app.activeSessionId = latest.id;
      app.chatMessages = latest.messages || [];
      app.renderChatMessages();
      app.renderSessionList();
    } catch (e) {
      console.error('loadChatSessions error:', e);
      app.newChatSession();
    }
  }

  function saveChatSessions(app) {
    try {
      syncActiveSession(app);
      localStorage.setItem(STORAGE_KEY, JSON.stringify(app.chatSessions));
    } catch (e) {
      console.error('saveChatSessions error:', e);
    }
  }

  function switchSession(app, id) {
    syncActiveSession(app);
    const target = app.chatSessions.find(s => s.id === id);
    if (!target) return;

    app.activeSessionId = id;
    app.chatMessages = target.messages || [];
    app.renderChatMessages();
    app.updateTokenDisplay();
    app.renderSessionList();
    app.saveChatSessions();
  }

  function renderChatMessages(app) {
    const container = document.getElementById('aiChatMessages');
    if (!container) return;
    container.innerHTML = '';
    for (const msg of app.chatMessages) {
      app.addChatMessage(msg.role, msg.content, false);
    }
  }

  function renderSessionList(app) {
    const list = document.getElementById('sessionList');
    if (!list) return;
    list.innerHTML = app.chatSessions.map(s => `
      <div class="session-item ${s.id === app.activeSessionId ? 'active' : ''}" data-session="${app.escapeAttr(s.id)}">
        <div class="session-title">${app.escapeHtml(s.title || '会话')}</div>
        <div class="session-preview">${app.escapeHtml(s.preview || '')}</div>
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
  }

  global.HajimiSessions = {
    storageKey: STORAGE_KEY,
    newChatSession,
    loadChatSessions,
    saveChatSessions,
    switchSession,
    renderChatMessages,
    renderSessionList,
  };
})(window);
