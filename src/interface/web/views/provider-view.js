'use strict';

(function (global) {
  function getDocument() {
    return global.document || (typeof document !== 'undefined' ? document : null);
  }

  function clearElement(el) {
    if (!el) return;
    if (typeof el.replaceChildren === 'function') {
      el.replaceChildren();
      return;
    }
    while (el.firstChild) el.removeChild(el.firstChild);
  }

  function createElement(doc, tag, className, text) {
    const el = doc.createElement(tag);
    if (className) el.className = className;
    if (text != null) el.textContent = text;
    return el;
  }

  function appendSourceTag(doc, parent, source) {
    const tag = createElement(doc, 'span', `provider-source-tag ${source.type}`, source.label);
    if (source.title) tag.title = source.title;
    parent.appendChild(tag);
  }

  function renderEmptyState(doc, list, source) {
    const empty = createElement(doc, 'div', 'provider-item-empty', '暂无自定义模型，点击上方按钮添加。');
    appendSourceTag(doc, empty, source);
    list.appendChild(empty);
  }

  function renderSourceHint(doc, list, source) {
    const hint = createElement(doc, 'div', 'provider-source-hint', '来源: ');
    appendSourceTag(doc, hint, source);
    list.appendChild(hint);
  }

  function renderProviderItem(doc, list, provider, service) {
    const item = createElement(doc, 'div', 'provider-item');
    if (provider.id) item.dataset.providerReadonlyId = provider.id;

    const info = createElement(doc, 'div', 'provider-item-info');
    const name = createElement(doc, 'div', 'provider-item-name', provider.name || provider.id);
    const meta = createElement(doc, 'div', 'provider-item-meta', service.formatProviderMeta(provider));

    info.appendChild(name);
    info.appendChild(meta);
    item.appendChild(info);
    list.appendChild(item);
  }

  function renderProviderListReadOnly(app, options) {
    const doc = getDocument();
    const service = global.HajimiProviderService;
    const targetId = options?.targetId || 'providerListTab';
    const list = options?.target || doc?.getElementById(targetId);

    if (!doc || !list || !service) {
      return { rendered: false, count: 0, readonly: true };
    }

    const providers = service.normalizeProviderConfigs(options?.providers || app?.providerConfigs || []);
    const source = service.getProviderSource(app?.currentWorkspace || '');

    clearElement(list);
    if (!providers.length) {
      renderEmptyState(doc, list, source);
    } else {
      providers.forEach((provider) => renderProviderItem(doc, list, provider, service));
      renderSourceHint(doc, list, source);
    }

    return { rendered: true, count: providers.length, readonly: true };
  }

  const api = {
    renderProviderListReadOnly,
  };

  global.HajimiProviderView = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
