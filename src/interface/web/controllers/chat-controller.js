(function () {
  const HajimiChatController = {
    init(app) {
      const chatInput = document.getElementById('aiChatInput');
      const chatSendBtn = document.getElementById('aiChatSendBtn');
      const slashPaletteContainer = document.getElementById('slashPalette');
      const slashPaletteEnabled = window.__HAJIMI_FLAGS__?.slashPaletteEnabled !== false;

      if (slashPaletteEnabled && window.HajimiSlashPalette && chatInput && slashPaletteContainer) {
        app.slashPalette = window.HajimiSlashPalette.createSlashPalette({
          inputEl: chatInput,
          containerEl: slashPaletteContainer,
          getCommands: () => app.getSlashCommands(),
          onSelect: (item) => {
            if (item.disabled || item.enabled === false) return;
            chatInput.value = item.insertText || item.trigger || '';
            chatInput.focus();
            chatInput.dispatchEvent(new Event('input', { bubbles: true }));
            if (item.executeMode === 'direct' && item.riskLevel === 'low') {
              app.sendChatMessage();
            }
          },
        });
      }

      if (chatInput) {
        chatInput.addEventListener('input', () => {
          chatInput.style.height = 'auto';
          chatInput.style.height = Math.min(chatInput.scrollHeight, 150) + 'px';
          if (app.slashPalette) {
            app.slashPalette.handleInput();
          }
        });

        chatInput.addEventListener('keydown', (e) => {
          if (app.slashPalette?.isOpen() && app.slashPalette.handleKeyDown(e)) {
            return;
          }

          if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            app.sendChatMessage();
          }
        });

        chatInput.addEventListener('blur', () => {
          if (app.slashPalette?.isOpen()) {
            app.slashPalette.close('blur');
          }
        });
      }

      if (chatSendBtn) {
        chatSendBtn.addEventListener('click', () => app.sendChatMessage());
      }

      const modelSelectBtn = document.getElementById('modelSelectBtn');
      if (modelSelectBtn) {
        modelSelectBtn.addEventListener('click', () => app.openModelPicker());
      }

      const addContextBtn = document.getElementById('addContextBtn');
      if (addContextBtn) {
        addContextBtn.addEventListener('click', () => {
          app.showSidebar('explorer');
          app.addChatMessage('ai', '**上下文文件：** 在资源管理器中右键点击文件，选择"添加到 AI 上下文"。');
        });
      }

      const clearContextBtn = document.getElementById('clearContextBtn');
      if (clearContextBtn) {
        clearContextBtn.addEventListener('click', () => app.clearChatContext());
      }

      const editModeBtn = document.getElementById('editModeBtn');
      if (editModeBtn) {
        editModeBtn.addEventListener('click', () => {
          app.addChatMessage('ai', '**编辑模式已激活。** 当您要求修改文件时，我将以 diff 格式建议代码更改。');
        });
      }

      if (window.HajimiSessionController?.setupSessionButtons) {
        window.HajimiSessionController.setupSessionButtons(app);
      } else {
        const newChatBtn = document.getElementById('newChatBtn');
        if (newChatBtn) {
          newChatBtn.addEventListener('click', () => {
            app.newChatSession();
          });
        }

        const newSessionBtn = document.getElementById('newSessionBtn');
        if (newSessionBtn) {
          newSessionBtn.addEventListener('click', () => {
            app.newChatSession();
          });
        }
      }

      if (typeof app.updateTokenDisplay === 'function') {
        app.updateTokenDisplay();
      }
    }
  };

  window.HajimiChatController = HajimiChatController;
})();
