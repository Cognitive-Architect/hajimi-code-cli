(function (global) {
  'use strict';

  function safeText(value) {
    return value == null ? '' : String(value);
  }

  function escapeHtml(value) {
    const div = document.createElement('div');
    div.textContent = safeText(value);
    return div.innerHTML;
  }

  function escapeAttr(value) {
    return safeText(value)
      .replace(/&/g, '&amp;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
  }

  function setSafeHtml(element, value) {
    if (!element) return;
    element.innerHTML = escapeHtml(value);
  }

  global.HajimiSecurityDom = {
    safeText,
    escapeHtml,
    escapeAttr,
    setSafeHtml,
  };
})(window);
