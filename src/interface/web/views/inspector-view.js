// ============================================================
// Hajimi Inspector Shell View
// ============================================================

(function (global) {
  'use strict';

  function getDocument() {
    return global.document || (typeof document !== 'undefined' ? document : null);
  }

  function bindTabs(onTabSelected) {
    const doc = getDocument();
    if (!doc) return;
    doc.querySelectorAll('.inspector-tab').forEach(tab => {
      tab.addEventListener('click', () => {
        onTabSelected(tab.dataset.inspectorTab);
      });
    });
  }

  function bindClose(onClose) {
    const closeBtn = getDocument()?.getElementById('inspectorCloseBtn');
    if (closeBtn) closeBtn.addEventListener('click', onClose);
  }

  function setVisible(visible) {
    const inspector = getDocument()?.getElementById('rightInspector');
    if (inspector) inspector.style.display = visible ? '' : 'none';
  }

  function setActiveTab(tabId, onActivePanel) {
    const doc = getDocument();
    if (!doc) return;

    doc.querySelectorAll('.inspector-tab').forEach(el => {
      el.classList.toggle('active', el.dataset.inspectorTab === tabId);
    });

    doc.querySelectorAll('.inspector-panel').forEach(el => {
      const isActive = el.dataset.inspectorPanel === tabId;
      el.classList.toggle('active', isActive);
      if (isActive && typeof onActivePanel === 'function') {
        onActivePanel(tabId);
      }
    });
  }

  const api = {
    bindTabs,
    bindClose,
    setVisible,
    setActiveTab,
  };

  global.HajimiInspectorView = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
