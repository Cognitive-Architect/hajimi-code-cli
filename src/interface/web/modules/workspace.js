(function (global) {
  'use strict';

  function getInvoke() {
    const tauri = global.__TAURI__;
    if (!tauri) return null;
    return tauri.core?.invoke || tauri.invoke;
  }

  async function initWorkspace(app) {
    const invoke = getInvoke();
    if (!invoke) return;
    try {
      app.currentWorkspace = await invoke('get_current_workspace');
    } catch (e) {
      app.currentWorkspace = null;
    }
  }

  async function loadFileTree(app, path) {
    const invoke = getInvoke();
    if (!invoke) {
      app.fileTree = { name: 'workspace', type: 'folder', path: '.', expanded: true, children: [] };
      app.renderFileTree();
      return;
    }
    const rootPath = path || app.currentWorkspace || '.';
    try {
      const entries = await invoke('list_dir', { path: rootPath });
      app.fileTree = await buildTreeFromEntries(app, rootPath, entries);
      app.renderFileTree();
    } catch (e) {
      console.error('loadFileTree error:', e);
      app.showErrorToast('加载文件树失败: ' + (e.message || e));
    }
  }

  async function buildTreeFromEntries(app, dirPath, entries) {
    const invoke = getInvoke();
    const children = [];
    const sorted = (entries || []).sort((a, b) => {
      const aIsDir = !a.includes('.');
      const bIsDir = !b.includes('.');
      if (aIsDir && !bIsDir) return -1;
      if (!aIsDir && bIsDir) return 1;
      return a.localeCompare(b);
    });

    for (const name of sorted) {
      const fullPath = dirPath + '/' + name;
      let isFolder = !name.includes('.');
      if (name.startsWith('.') && invoke) {
        try {
          await invoke('list_dir', { path: fullPath });
          isFolder = true;
        } catch (e) {
          isFolder = false;
        }
      }

      if (isFolder) {
        let folderChildren = [];
        if (invoke) {
          try {
            const subEntries = await invoke('list_dir', { path: fullPath });
            const subTree = await buildTreeFromEntries(app, fullPath, subEntries);
            folderChildren = subTree.children || [];
          } catch (e) {
            // Permission denied or not a directory: keep the node expandable but empty.
          }
        }
        children.push({ name, type: 'folder', path: fullPath, expanded: false, children: folderChildren });
      } else {
        children.push({ name, type: 'file', path: fullPath, lang: app.guessLang(name) });
      }
    }

    return { name: dirPath.split('/').pop() || dirPath, type: 'folder', path: dirPath, expanded: true, children };
  }

  async function createNewFolder(app) {
    const name = prompt('输入新文件夹名称:');
    if (!name) return;
    const path = (app.currentWorkspace || '.') + '/' + name;
    const invoke = getInvoke();
    if (!invoke) {
      app.showErrorToast('Tauri 不可用');
      return;
    }
    try {
      await invoke('create_dir', { path });
      app.loadFileTree();
    } catch (e) {
      app.showErrorToast('创建文件夹失败: ' + (e.message || e));
    }
  }

  function collapseAllFolders(app) {
    const collapse = (node) => {
      if (node.type === 'folder') {
        node.expanded = false;
        if (node.children) node.children.forEach(collapse);
      }
    };
    if (app.fileTree) collapse(app.fileTree);
    app.renderFileTree();
  }

  function renderFileTree(app) {
    const container = document.getElementById('fileTree');
    if (!container) return;
    container.innerHTML = '';
    if (!app.fileTree) {
      container.innerHTML = '<div style="padding:12px;color:var(--fg-dim);font-size:12px;">加载中...</div>';
      return;
    }
    renderTreeNode(app, app.fileTree, container, 0);
  }

  function renderTreeNode(app, node, container, depth) {
    if (node.type === 'folder') {
      const folderEl = document.createElement('div');
      folderEl.className = `file-tree-item folder${node.expanded ? ' expanded' : ''}`;
      folderEl.style.paddingLeft = `${8 + depth * 16}px`;
      folderEl.innerHTML = `
        <span class="tree-toggle">▶</span>
        <span class="tree-icon">
          📁
        </span>
        <span class="tree-label">${app.escapeHtml(node.name)}</span>
      `;
      folderEl.addEventListener('click', (e) => {
        e.stopPropagation();
        node.expanded = !node.expanded;
        app.renderFileTree();
      });
      folderEl.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        e.stopPropagation();
        app.showContextMenu(e, node);
      });
      container.appendChild(folderEl);

      const childrenContainer = document.createElement('div');
      childrenContainer.className = 'file-tree-children';
      if (node.expanded) {
        childrenContainer.style.display = 'block';
      }
      if (node.children) {
        node.children.forEach(child => renderTreeNode(app, child, childrenContainer, depth + 1));
      }
      container.appendChild(childrenContainer);
      return;
    }

    const fileEl = document.createElement('div');
    fileEl.className = 'file-tree-item';
    fileEl.style.paddingLeft = `${8 + depth * 16}px`;
    const iconColor = app.getFileIconColor(node.name);
    fileEl.innerHTML = `
      <span class="tree-toggle"></span>
      <span class="tree-icon file-icon" style="color:${iconColor}">
        ${app.getFileIconSvg(node.name)}
      </span>
      <span class="tree-label">${app.escapeHtml(node.name)}</span>
    `;
    fileEl.addEventListener('click', (e) => {
      e.stopPropagation();
      document.querySelectorAll('.file-tree-item').forEach(el => el.classList.remove('selected'));
      fileEl.classList.add('selected');
      app.openFile(node.path);
    });
    fileEl.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      e.stopPropagation();
      app.showContextMenu(e, node);
    });
    container.appendChild(fileEl);
  }

  async function renameFile(app, oldPath) {
    const oldName = oldPath.split('/').pop();
    const newName = prompt('重命名为:', oldName);
    if (!newName || newName === oldName) return;
    const dir = oldPath.substring(0, oldPath.lastIndexOf('/'));
    const newPath = dir + '/' + newName;
    const invoke = getInvoke();
    if (!invoke) {
      app.showErrorToast('Tauri 不可用');
      return;
    }
    try {
      await invoke('rename_path', { oldPath, newPath });
      app.loadFileTree();
      const tab = app.tabs.find(t => t.id === oldPath);
      if (tab) app._doCloseTab(oldPath);
    } catch (e) {
      app.showErrorToast('重命名失败: ' + (e.message || e));
    }
  }

  async function deleteFile(app, path) {
    const name = path.split('/').pop();
    if (!confirm(`确定要删除 "${name}" 吗？`)) return;
    const invoke = getInvoke();
    if (!invoke) {
      app.showErrorToast('Tauri 不可用');
      return;
    }
    try {
      await invoke('delete_path', { path, recursive: true });
      app.loadFileTree();
      const tab = app.tabs.find(t => t.id === path);
      if (tab) app._doCloseTab(path);
    } catch (e) {
      app.showErrorToast('删除失败: ' + (e.message || e));
    }
  }

  global.HajimiWorkspace = {
    initWorkspace,
    loadFileTree,
    buildTreeFromEntries,
    createNewFolder,
    collapseAllFolders,
    renderFileTree,
    renderTreeNode,
    renameFile,
    deleteFile,
  };
})(window);
