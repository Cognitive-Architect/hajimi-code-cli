(function (global) {
  'use strict';

  // V3X Day4-H: Session Button Wiring Minimal Controller
  // Binds session buttons to their respective actions in the app context.

  function setupSessionButtons(app) {
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

  const api = {
    setupSessionButtons,
  };

  global.HajimiSessionController = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
