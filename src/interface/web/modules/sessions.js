(function (global) {
  'use strict';

  const STORAGE_KEY = 'hajimi_chat_sessions';
  let warnedMissingSessionListView = false;

  function ensureSessionListView() {
    if (global.HajimiSessionListView?.renderSessionList) {
      return global.HajimiSessionListView;
    }

    return {
      renderSessionList() {
        if (!warnedMissingSessionListView) {
          warnedMissingSessionListView = true;
          console.warn('HajimiSessionListView is required before HajimiSessions.renderSessionList; skipping session list render.');
        }
      },
    };
  }

  function makeSessionId() {
    return 'session-' + Date.now() + '-' + Math.random().toString(36).slice(2, 7);
  }

  function syncActiveSession(app) {
    if (!app.activeSessionId) return;
    const session = app.chatSessions.find(s => s.id === app.activeSessionId);
    if (!session) return;
    session.messages = app.chatMessages.map(msg => ({ ...msg }));
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
    app.renderLiveShellState?.('就绪');
  }

  function ensureStorageService() {
    if (global.HajimiStorageService) {
      return global.HajimiStorageService;
    }
    return null;
  }

  function loadChatSessions(app) {
    try {
      var store = ensureStorageService();
      var sessions = null;
      if (store) {
        sessions = store.getSessions();
      } else {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) {
          sessions = JSON.parse(raw);
        }
      }

      if (!sessions || sessions.length === 0) {
        app.newChatSession();
        return;
      }
      app.chatSessions = sessions;

      const latest = app.chatSessions[0];
      app.activeSessionId = latest.id;
      app.chatMessages = latest.messages || [];
      app.renderChatMessages();
      app.renderSessionList();
      app.renderLiveShellState?.('就绪');
    } catch (e) {
      console.error('loadChatSessions error:', e);
      app.newChatSession();
    }
  }

  function saveChatSessions(app) {
    try {
      syncActiveSession(app);
      var store = ensureStorageService();
      if (store) {
        store.setSessions(app.chatSessions);
      } else {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(app.chatSessions));
      }
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
    app.renderLiveShellState?.('就绪');
  }

  function renderChatMessages(app) {
    const container = document.getElementById('aiChatMessages');
    if (!container) return;
    container.innerHTML = '';
    for (const msg of app.chatMessages) {
      if (app.renderChatMessageFromSession) {
        app.renderChatMessageFromSession(msg);
      } else {
        app.addChatMessage(msg.role, msg.content, false);
      }
    }
  }

  function renderSessionList(app) {
    return ensureSessionListView().renderSessionList(app);
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
