(function (global) {
  'use strict';

  const EMPTY_LABEL = '无匹配命令';

  function normalizeItem(item) {
    const riskLevel = item.riskLevel || 'low';
    const enabled = item.enabled !== false;
    const executeMode = item.executeMode || 'fill';
    return {
      id: item.id || item.trigger,
      trigger: item.trigger || '',
      title: item.title || item.trigger || '',
      description: item.description || '',
      category: item.category || 'general',
      riskLevel,
      enabled,
      disabled: !enabled || executeMode === 'disabled',
      insertText: item.insertText || item.trigger || '',
      executeMode,
      keywords: Array.isArray(item.keywords) ? item.keywords : [],
    };
  }

  function itemMatches(item, query) {
    const needle = String(query || '').toLowerCase();
    if (!needle || needle === '/') return true;
    const fields = [
      item.trigger,
      item.title,
      item.description,
      item.category,
      item.riskLevel,
      ...item.keywords,
    ];
    return fields.some(field => String(field || '').toLowerCase().includes(needle));
  }

  function clearNode(node) {
    while (node.firstChild) {
      node.removeChild(node.firstChild);
    }
  }

  function createTextNode(className, text) {
    const el = document.createElement('span');
    el.className = className;
    el.textContent = text;
    return el;
  }

  function createSlashPalette(options) {
    const inputEl = options.inputEl;
    const containerEl = options.containerEl;
    const getCommands = options.getCommands || (() => []);
    const onSelect = options.onSelect || (() => {});
    const onOpen = options.onOpen || (() => {});
    const onClose = options.onClose || (() => {});

    const state = {
      isOpen: false,
      query: '',
      items: [],
      filteredItems: [],
      activeIndex: 0,
    };

    if (!inputEl || !containerEl) {
      throw new Error('createSlashPalette requires inputEl and containerEl');
    }

    containerEl.classList.add('slash-palette', 'hidden');
    containerEl.setAttribute('role', 'listbox');
    containerEl.setAttribute('aria-label', 'Slash commands');

    function loadItems() {
      state.items = getCommands().map(normalizeItem).filter(item => item.trigger.startsWith('/'));
    }

    function renderEmpty() {
      clearNode(containerEl);
      const empty = document.createElement('div');
      empty.className = 'slash-palette-empty';
      empty.textContent = EMPTY_LABEL;
      containerEl.appendChild(empty);
    }

    function renderItem(item, index) {
      const row = document.createElement('button');
      row.type = 'button';
      row.className = 'slash-palette-item';
      row.dataset.commandId = item.id;
      row.setAttribute('role', 'option');
      row.setAttribute('aria-selected', index === state.activeIndex ? 'true' : 'false');

      if (index === state.activeIndex) {
        row.classList.add('active');
      }

      if (item.disabled) {
        row.classList.add('disabled');
        row.disabled = true;
        row.setAttribute('aria-disabled', 'true');
      }

      const main = document.createElement('span');
      main.className = 'slash-palette-main';
      main.appendChild(createTextNode('slash-palette-trigger', item.trigger));
      main.appendChild(createTextNode('slash-palette-title', item.title));

      const meta = document.createElement('span');
      meta.className = 'slash-palette-meta';
      meta.appendChild(createTextNode('slash-palette-category', item.category));
      meta.appendChild(createTextNode('slash-palette-risk', item.riskLevel));

      const description = createTextNode('slash-palette-description', item.description);

      row.appendChild(main);
      row.appendChild(description);
      row.appendChild(meta);

      row.addEventListener('click', () => {
        if (item.disabled) return;
        selectItem(item);
      });

      row.addEventListener('mousedown', (event) => {
        event.preventDefault();
      });

      return row;
    }

    function render() {
      clearNode(containerEl);
      if (!state.filteredItems.length) {
        renderEmpty();
        return;
      }

      const list = document.createElement('div');
      list.className = 'slash-palette-list';
      state.filteredItems.forEach((item, index) => {
        list.appendChild(renderItem(item, index));
      });
      containerEl.appendChild(list);
    }

    function filterItems(query) {
      state.query = query || '/';
      state.filteredItems = state.items.filter(item => itemMatches(item, state.query));
      state.activeIndex = state.filteredItems.findIndex(item => !item.disabled);
      if (state.activeIndex < 0) state.activeIndex = 0;
      render();
    }

    function getEnabledIndexes() {
      return state.filteredItems
        .map((item, index) => item.disabled ? -1 : index)
        .filter(index => index >= 0);
    }

    function moveActive(delta) {
      const indexes = getEnabledIndexes();
      if (!state.isOpen || !indexes.length) return false;

      const currentPosition = indexes.indexOf(state.activeIndex);
      const startPosition = currentPosition >= 0 ? currentPosition : 0;
      const nextPosition = (startPosition + delta + indexes.length) % indexes.length;
      state.activeIndex = indexes[nextPosition];
      render();
      return true;
    }

    function selectItem(item) {
      if (!item || item.disabled || item.enabled === false) return false;
      onSelect(item);
      close('select');
      return true;
    }

    function selectActive() {
      if (!state.isOpen || !state.filteredItems.length) return false;
      const item = state.filteredItems[state.activeIndex];
      return selectItem(item);
    }

    function handleKeyDown(event) {
      if (!state.isOpen) return false;

      if (event.key === 'ArrowDown') {
        event.preventDefault();
        moveActive(1);
        return true;
      }

      if (event.key === 'ArrowUp') {
        event.preventDefault();
        moveActive(-1);
        return true;
      }

      if (event.key === 'Escape') {
        event.preventDefault();
        close('escape');
        return true;
      }

      if (event.key === 'Enter') {
        if (!selectActive()) return false;
        event.preventDefault();
        return true;
      }

      return false;
    }

    function open(query) {
      loadItems();
      state.isOpen = true;
      containerEl.classList.remove('hidden');
      filterItems(query || '/');
      onOpen();
    }

    function close(reason) {
      if (!state.isOpen) return;
      state.isOpen = false;
      state.query = '';
      state.filteredItems = [];
      state.activeIndex = 0;
      clearNode(containerEl);
      containerEl.classList.add('hidden');
      onClose(reason || 'close');
    }

    function updateQuery(query) {
      if (!state.isOpen) {
        open(query);
        return;
      }
      filterItems(query);
    }

    function handleInput() {
      const value = inputEl.value || '';
      const caret = typeof inputEl.selectionStart === 'number' ? inputEl.selectionStart : value.length;
      const beforeCaret = value.slice(0, caret);
      const token = beforeCaret.split(/\s/).pop();
      if (token && token.startsWith('/')) {
        updateQuery(token);
      } else {
        close('input');
      }
    }

    function isOpen() {
      return state.isOpen;
    }

    function destroy() {
      close('destroy');
    }

    return {
      open,
      close,
      updateQuery,
      handleInput,
      handleKeyDown,
      moveActive,
      selectActive,
      isOpen,
      destroy,
    };
  }

  global.HajimiSlashPalette = {
    createSlashPalette,
  };

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = { createSlashPalette };
  }
})(window);
