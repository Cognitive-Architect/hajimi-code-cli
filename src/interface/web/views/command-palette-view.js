'use strict';

// V3X Day 3-B: Command Palette View
// Responsible for DOM query, open/close, filter render, active class, list item render.
// All DOM ids and class selectors remain unchanged from the original app.js implementation.

/**
 * @param {object} app - The window.app object (or compatible interface).
 *   Required properties/methods: commands, escapeHtml, escapeAttr, hideCommandPalette (for click-to-execute).
 */
function createCommandPaletteView(app) {
  function show() {
    var palette = document.getElementById('commandPalette');
    if (palette) palette.classList.add('active');
    var input = document.getElementById('commandInput');
    if (input) {
      input.value = '';
      input.focus();
    }
    renderList('');
  }

  function hide() {
    var palette = document.getElementById('commandPalette');
    if (palette) palette.classList.remove('active');
  }

  function renderList(query) {
    var list = document.getElementById('commandList');
    if (!list) return;
    var q = query.toLowerCase();
    var filtered = (app.commands || []).filter(function (c) {
      return c.label.toLowerCase().includes(q);
    });

    list.innerHTML = filtered.map(function (c, i) {
      return '<div class="command-item' + (i === 0 ? ' selected' : '') + '" data-index="' + i + '" data-id="' + app.escapeAttr(c.id) + '">' +
        '<span>' + app.escapeHtml(c.label) + '</span>' +
        (c.key ? '<span class="command-item-key">' + app.escapeHtml(c.key) + '</span>' : '') +
        '</div>';
    }).join('');

    list.querySelectorAll('.command-item').forEach(function (el) {
      el.addEventListener('click', function () {
        var cmd = (app.commands || []).find(function (c) { return c.id === el.dataset.id; });
        if (cmd) { hide(); cmd.action(); }
      });
    });
  }

  function navigate(dir) {
    var items = document.querySelectorAll('.command-item');
    if (!items.length) return;
    var current = document.querySelector('.command-item.selected');
    var idx = current ? Array.from(items).indexOf(current) : -1;
    idx = Math.max(0, Math.min(items.length - 1, idx + dir));
    items.forEach(function (el) { el.classList.remove('selected'); });
    items[idx].classList.add('selected');
    items[idx].scrollIntoView({ block: 'nearest' });
  }

  function executeSelected() {
    var selected = document.querySelector('.command-item.selected');
    if (!selected) return;
    var cmd = (app.commands || []).find(function (c) { return c.id === selected.dataset.id; });
    if (cmd) { hide(); cmd.action(); }
  }

  return {
    show: show,
    hide: hide,
    renderList: renderList,
    navigate: navigate,
    executeSelected: executeSelected,
  };
}

if (typeof window !== 'undefined') {
  window.HajimiCommandPaletteView = { createCommandPaletteView: createCommandPaletteView };
}
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { createCommandPaletteView: createCommandPaletteView };
}
