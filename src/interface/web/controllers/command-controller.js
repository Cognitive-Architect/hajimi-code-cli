'use strict';

// V3X Day 3-B: Command Controller
// Responsible for initialization (binding DOM events), keyboard navigation,
// executing selected command, and Ctrl+Shift+P / Escape shortcut integration.
// Delegates DOM rendering to command-palette-view.js.

/**
 * @param {object} app - The window.app object (or compatible interface).
 * @param {object} view - The CommandPaletteView instance (from createCommandPaletteView).
 */
function createCommandController(app, view) {
  function setup() {
    var palette = document.getElementById('commandPalette');
    var input = document.getElementById('commandInput');

    if (input) {
      input.addEventListener('input', function () {
        view.renderList(input.value);
      });

      input.addEventListener('keydown', function (e) {
        if (e.key === 'Escape') view.hide();
        if (e.key === 'Enter') view.executeSelected();
        if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
          e.preventDefault();
          view.navigate(e.key === 'ArrowDown' ? 1 : -1);
        }
      });
    }

    if (palette) {
      palette.addEventListener('click', function (e) {
        if (e.target === palette) view.hide();
      });
    }
  }

  function setupKeyboardShortcuts() {
    document.addEventListener('keydown', function (e) {
      // Ctrl+Shift+P — Command Palette
      if (e.ctrlKey && e.shiftKey && e.key === 'P') {
        e.preventDefault();
        view.show();
      }
      // Ctrl+Shift+E — Explorer
      if (e.ctrlKey && e.shiftKey && e.key === 'E') {
        e.preventDefault();
        app.showSidebar('explorer');
      }
      // Ctrl+Shift+F — Search
      if (e.ctrlKey && e.shiftKey && e.key === 'F') {
        e.preventDefault();
        app.showSidebar('search');
      }
      // Ctrl+Shift+G — Git
      if (e.ctrlKey && e.shiftKey && e.key === 'G') {
        e.preventDefault();
        app.showSidebar('git');
      }
      // Ctrl+Shift+A — Agent Trace
      if (e.ctrlKey && e.shiftKey && e.key === 'A') {
        e.preventDefault();
        app.showSidebar('agent-trace');
      }
      // Ctrl+Shift+X — Extensions
      if (e.ctrlKey && e.shiftKey && e.key === 'X') {
        e.preventDefault();
        app.showSidebar('extensions');
      }
      // Ctrl+Shift+S — Settings
      if (e.ctrlKey && e.shiftKey && e.key === 'S') {
        e.preventDefault();
        app.showSidebar('settings');
      }
      // Ctrl+Shift+C — Chat Sessions
      if (e.ctrlKey && e.shiftKey && e.key === 'C') {
        e.preventDefault();
        app.showSidebar('chat-sessions');
      }
      // Ctrl+B — Toggle Sidebar
      if (e.ctrlKey && e.key === 'b') {
        e.preventDefault();
        app.toggleSidebar();
      }
      // Escape — close palette
      if (e.key === 'Escape') {
        view.hide();
      }
    });
  }

  return {
    setup: setup,
    setupKeyboardShortcuts: setupKeyboardShortcuts,
  };
}

if (typeof window !== 'undefined') {
  window.HajimiCommandController = { createCommandController: createCommandController };
}
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { createCommandController: createCommandController };
}
