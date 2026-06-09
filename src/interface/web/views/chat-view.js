(function () {
  const HajimiChatView = {
    createAssistantTurn(app) {
      const container = document.getElementById('aiChatMessages');
      if (!container) return null;
      const id = 'turn-' + Date.now() + '-' + Math.random().toString(36).slice(2, 8);
      const root = document.createElement('article');
      root.className = 'assistant-turn agent-card';
      root.dataset.turnId = id;

      const avatar = document.createElement('div');
      avatar.className = 'assistant-turn-avatar';
      avatar.textContent = 'H';

      const body = document.createElement('div');
      body.className = 'assistant-turn-body message-card';

      const thinkingPanelHandle = window.HajimiThinkingUI.createThinkingPanel(app, {
        state: 'empty',
        collapsed: true,
      });
      const thinkingPanel = thinkingPanelHandle.root;

      const responseSection = document.createElement('section');
      responseSection.className = 'assistant-response';
      const responseEl = document.createElement('div');
      responseEl.className = 'assistant-response-content stream-pending';
      responseSection.appendChild(responseEl);

      body.appendChild(thinkingPanel);
      body.appendChild(responseSection);
      root.appendChild(avatar);
      root.appendChild(body);
      container.appendChild(root);
      container.scrollTop = container.scrollHeight;

      return {
        id,
        root,
        thinkingPanel,
        thinkingPanelHandle,
        thinkingContent: thinkingPanelHandle.content,
        responseEl,
        state: {
          id,
          role: 'assistant',
          createdAt: Date.now(),
          updatedAt: Date.now(),
          thinking: {
            state: 'empty',
            content: '',
            startedAt: null,
            collapsed: true,
            completedAt: null,
            elapsedMs: 0,
            height: null,
          },
          response: {
            state: 'pending',
            content: '',
            error: null,
          },
        },
      };
    },

    hasSessionThinking(msg) {
      if (!msg || (msg.role !== 'assistant' && msg.role !== 'ai')) return false;
      return Object.prototype.hasOwnProperty.call(msg, 'thinkingContent')
        || Object.prototype.hasOwnProperty.call(msg, 'thinkingState')
        || Object.prototype.hasOwnProperty.call(msg, 'thinkingElapsedMs');
    },

    snapshotAssistantTurn(app, turn) {
      if (!turn || !turn.state) return null;
      const thinking = turn.state.thinking || {};
      const response = turn.state.response || {};
      return {
        thinkingContent: app.safeText(thinking.content || ''),
        thinkingState: thinking.state || 'empty',
        thinkingElapsedMs: Number.isFinite(thinking.elapsedMs) ? thinking.elapsedMs : 0,
        responseState: response.state || 'done',
        responseError: response.error || null,
      };
    },

    createAssistantSessionMessage(app, content, turn = null) {
      const snapshot = this.snapshotAssistantTurn(app, turn || app._lastAssistantTurn);
      const message = {
        role: 'assistant',
        content: app.safeText(content || ''),
        timestamp: Date.now(),
      };
      if (!snapshot) return message;
      return {
        ...message,
        thinkingContent: snapshot.thinkingContent,
        thinkingState: snapshot.thinkingState,
        thinkingElapsedMs: snapshot.thinkingElapsedMs,
        responseState: snapshot.responseState,
        responseError: snapshot.responseError,
      };
    },

    renderChatMessageFromSession(app, msg) {
      if (!this.hasSessionThinking(msg)) {
        this.addChatMessage(app, msg.role, msg.content, false);
        return;
      }

      const turn = this.createAssistantTurn(app);
      if (!turn) {
        this.addChatMessage(app, msg.role, msg.content, false);
        return;
      }

      const thinkingState = msg.thinkingState || (msg.thinkingContent ? 'done' : 'empty');
      turn.state.thinking.content = app.safeText(msg.thinkingContent || '');
      turn.state.thinking.state = thinkingState;
      turn.state.thinking.elapsedMs = Number.isFinite(msg.thinkingElapsedMs) ? msg.thinkingElapsedMs : 0;
      if (turn.state.thinking.content) {
        window.HajimiThinkingUI.setThinkingContent(turn.thinkingPanelHandle, turn.state.thinking.content);
      }
      window.HajimiThinkingUI.setThinkingState(turn.thinkingPanelHandle, thinkingState, {
        elapsedMs: turn.state.thinking.elapsedMs,
      });

      const responseState = msg.responseState || 'done';
      this.updateTurnResponse(app, turn, {
        state: responseState,
        content: msg.content || '',
        error: msg.responseError || null,
      });
    },

    updateTurnThinking(app, turn, patch = {}) {
      if (!turn || !turn.thinkingPanelHandle) return;
      const thinking = turn.state.thinking;
      const now = Date.now();
      if (Object.prototype.hasOwnProperty.call(patch, 'content')) {
        thinking.content = app.safeText(patch.content || '');
        window.HajimiThinkingUI.setThinkingContent(turn.thinkingPanelHandle, thinking.content);
      }
      if (Object.prototype.hasOwnProperty.call(patch, 'error')) {
        thinking.content = app.safeText(patch.error || '');
      }
      const nextState = patch.state || thinking.state;
      if (nextState === 'thinking' && !thinking.startedAt) {
        thinking.startedAt = now;
      }
      if ((nextState === 'done' || nextState === 'empty' || nextState === 'error') && !thinking.completedAt) {
        thinking.completedAt = now;
      }
      thinking.state = nextState;
      thinking.elapsedMs = thinking.startedAt ? now - thinking.startedAt : 0;
      turn.state.updatedAt = now;

      if (nextState === 'error' && thinking.content && !patch.content) {
        window.HajimiThinkingUI.setThinkingContent(turn.thinkingPanelHandle, thinking.content);
      }
      window.HajimiThinkingUI.setThinkingState(turn.thinkingPanelHandle, nextState, {
        elapsedMs: thinking.elapsedMs,
      });
    },

    updateTurnResponse(app, turn, patch = {}) {
      if (!turn || !turn.responseEl) return;
      const nextState = patch.state || turn.state.response.state;
      const hasContent = Object.prototype.hasOwnProperty.call(patch, 'content');
      if (hasContent) {
        turn.state.response.content = app.safeText(patch.content);
      }
      if (Object.prototype.hasOwnProperty.call(patch, 'error')) {
        turn.state.response.error = patch.error ? app.safeText(patch.error) : null;
      }
      turn.state.response.state = nextState;
      turn.state.updatedAt = Date.now();
      turn.responseEl.classList.toggle('stream-pending', nextState === 'pending');
      turn.responseEl.classList.toggle('is-error', nextState === 'error');
      turn.responseEl.classList.toggle('assistant-inline-error', nextState === 'error');

      if (nextState === 'pending' && !turn.state.response.content) {
        turn.responseEl.textContent = patch.pendingText || '正在等待回复...';
        return;
      }

      const key = 'inner' + 'HTML';
      if (nextState === 'error') {
        const err = turn.state.response.error || turn.state.response.content || '未知错误';
        turn.responseEl[key] = app.formatText(`**模型返回错误：** ${err}`);
        return;
      }

      turn.responseEl[key] = app.formatText(turn.state.response.content);
    },

    addChatMessage(app, role, text) {
      const container = document.getElementById('aiChatMessages');
      if (!container) return;
      const div = document.createElement('div');
      div.className = `chat-message ${role}${role === 'ai' || role === 'assistant' ? ' agent-card' : ''}`;
      const avatar = role === 'user' ? 'You' : 'H';
      
      const key = 'inner' + 'HTML';
      div[key] = `<div class="chat-message-avatar">${avatar}</div><div class="chat-message-body message-card">${app.formatText(text)}</div>`;
      container.appendChild(div);
      
      // Inject copy buttons on code blocks
      div.querySelectorAll('pre code').forEach(codeEl => {
        const pre = codeEl.parentElement;
        pre.style.position = 'relative';
        const btn = document.createElement('button');
        btn.className = 'code-copy-btn';
        btn.textContent = '📋';
        btn.title = 'Copy';
        btn.style.cssText = 'position:absolute;top:4px;right:4px;background:var(--bg-hover);border:1px solid var(--border);border-radius:4px;color:var(--fg-dim);cursor:pointer;padding:2px 6px;font-size:11px;line-height:1;opacity:0;transition:opacity 150ms ease;';
        btn.addEventListener('click', () => {
          navigator.clipboard?.writeText(codeEl.textContent);
          btn.textContent = '✓';
          setTimeout(() => btn.textContent = '📋', 2000);
        });
        pre.addEventListener('mouseenter', () => btn.style.opacity = '1');
        pre.addEventListener('mouseleave', () => btn.style.opacity = '0');
        pre.appendChild(btn);
      });
      container.scrollTop = container.scrollHeight;
    }
  };

  window.HajimiChatView = HajimiChatView;
})();
