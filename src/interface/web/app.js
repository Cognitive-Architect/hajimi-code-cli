// ============================================================
// Hajimi Code — VSCode-style IDE Frontend
// ============================================================

window.app = {
  // State
  tabs: [],
  activeTab: null,
  sidebarView: 'ai-chat',
  panelView: 'terminal',
  panelCollapsed: false,
  isProcessing: false,
  commands: [],
  providerConfigs: [],
  activeProviderId: null,
  editingProviderId: null,
  currentWorkspace: null,
  fileTree: null,
  commandHistory: [],
  commandHistoryIndex: -1,
  settings: {
    theme: 'dark',
    fontSize: 14,
    wordWrap: true,
    autoSave: 'off',
  },
  chatContextFiles: [],
  chatMessages: [],
  chatSessions: [],
  activeSessionId: null,
  autoCompact: true,
  isAutoCompacting: false,
  tokenStats: { promptTokens: 0, completionTokens: 0, estimatedTokens: 0 },
  cumulativeStats: { promptTokens: 0, completionTokens: 0, requestCount: 0 },
  showCumulative: false,
  mcpServers: [],
  traceEvents: [],
  tracePaused: false,
  traceChannel: null,
  extensions: [
    { id: 'rust', name: 'Rust', desc: 'Rust 语言支持', version: '1.0.0', publisher: 'rust-lang', icon: 'R', iconColor: 'var(--fg-cyan)', installed: true },
    { id: 'hajimi-agent', name: 'Hajimi 智能体', desc: 'AI 助手集成', version: '0.3.0', publisher: 'hajimi', icon: 'H', iconColor: 'var(--fg-magenta)', installed: true },
    { id: 'toml', name: 'TOML', desc: 'TOML 语言支持', version: '0.1.0', publisher: '应用市场', icon: 'T', iconColor: 'var(--fg-green)', installed: false },
    { id: 'python', name: 'Python', desc: 'Python 语言支持', version: '1.2.0', publisher: 'microsoft', icon: 'P', iconColor: 'var(--fg-cyan)', installed: false },
    { id: 'go', name: 'Go', desc: 'Go 语言支持', version: '0.5.0', publisher: 'golang', icon: 'G', iconColor: 'var(--fg-cyan)', installed: false },
    { id: 'docker', name: 'Docker', desc: 'Dockerfile 和 Compose 支持', version: '1.0.0', publisher: 'microsoft', icon: 'D', iconColor: 'var(--fg-cyan)', installed: false },
  ],
  installedExtensions: [],

  init() {
    this.setupActivityBar();
    this.setupChat();
    this.setupCommandPalette();
    this.setupKeyboardShortcuts();
    this.setupStatusBar();
    this.setupTraceTabs();
    this.setupSessionReplay();
    this.setupFileTreeToolbar();
    this.setupAgentTrace();
    this.loadSettings();
    this.setupSystemThemeListener();
    this.loadLayoutSizes();
    this.initWorkspace().then(() => {
      this.loadFileTree();
    });
    this.loadChatSessions();
    this.loadProviders();
    this.setupModelPicker();
    this.setupProviderSettings();
    this.loadProfiles();
    this.setupProfileSettings();
    this.setupAuditLog();
    this.loadCumulativeFromBackend();
    this.setupAgentProvider();
    this.setupMcpSettings();
    this.setupGovernance();
    this.setupSessionBrowser();
    this.setupResourceDashboard();

    // Build command list
    this.commands = [
      { id: 'file.open', label: '文件: 打开文件', key: 'Ctrl+O', action: () => this.openFilePrompt() },
      { id: 'file.openFolder', label: '文件: 打开文件夹', key: 'Ctrl+K Ctrl+O', action: () => this.openFolder() },
      { id: 'view.chat-sessions', label: '视图: 显示会话列表', key: 'Ctrl+Shift+C', action: () => this.showSidebar('chat-sessions') },
      { id: 'view.explorer', label: '视图: 显示文件', key: 'Ctrl+Shift+E', action: () => this.showSidebar('explorer') },
      { id: 'view.models', label: '视图: 显示模型', key: 'Ctrl+Shift+M', action: () => this.showSidebar('models') },
      { id: 'view.system', label: '视图: 显示系统', key: 'Ctrl+Shift+Y', action: () => this.showSidebar('system') },
      { id: 'view.agent-trace', label: '视图: 显示 Agent Trace', key: 'Ctrl+Shift+A', action: () => this.showSidebar('agent-trace') },
      { id: 'view.settings', label: '视图: 显示设置', key: 'Ctrl+Shift+S', action: () => this.showSidebar('settings') },
      { id: 'palette', label: '命令面板', key: 'Ctrl+Shift+P', action: () => this.showCommandPalette() },
      { id: 'chat.new', label: '对话: 新会话', key: '', action: () => this.newChatSession() },
      { id: 'git.commit', label: 'Git: 提交', key: '', action: () => this.gitCommit() },
      { id: 'providers.refresh', label: '模型: 刷新提供商列表', key: '', action: () => this.loadProviders() },
      // Phase 4 Day 5: Agent Command Palette commands
      { id: 'agent.refactor', label: '@agent refactor — 重构选中代码', key: '', action: () => this.runAgentCommand('@agent refactor selection') },
      { id: 'agent.review-pr', label: '@agent review-pr — 审查 PR', key: '', action: () => this.runAgentCommand('@agent review-pr') },
      { id: 'agent.continue', label: '@agent continue-background — 后台继续', key: '', action: () => this.runAgentCommand('@agent continue-background') },
      { id: 'agent.pause', label: '@agent pause — 暂停 Agent', key: '', action: () => this.runAgentCommand('@agent pause') },
      { id: 'agent.status', label: '@agent status — Agent 状态', key: '', action: () => this.runAgentCommand('@agent status') },
      { id: 'edit.history', label: '编辑: 显示编辑历史', key: '', action: () => this.showEditHistoryTab() },
    ];
  },

  // ============================================================
  // Activity Bar
  // ============================================================
  setupActivityBar() {
    document.querySelectorAll('.activity-item').forEach(item => {
      item.addEventListener('click', () => {
        const view = item.dataset.view;
        this.showSidebar(view);
      });
    });
  },

  showSidebar(view) {
    this.sidebarView = view;
    document.querySelectorAll('.activity-item').forEach(el => {
      el.classList.toggle('active', el.dataset.view === view);
    });
    document.querySelectorAll('.sidebar-panel').forEach(el => {
      el.classList.toggle('active', el.dataset.panel === view);
    });
    if (view === 'git') {
      this.loadGitStatus();
    }
    if (view === 'settings') {
      this.loadProviders();
      this.loadAgentProviders();
    }
  },

  toggleSidebar() {
    const sidebar = document.getElementById('sidebar');
    if (!sidebar) return;
    const current = sidebar.style.width || getComputedStyle(sidebar).width;
    if (current === '0px' || current === '0') {
      sidebar.style.width = 'var(--sidebar-width)';
      sidebar.style.display = '';
    } else {
      sidebar.style.width = '0px';
      sidebar.style.display = 'none';
    }
  },

  // ============================================================
  // Search
  // ============================================================
  setupSearch() {
    const searchInput = document.getElementById('searchInput');
    if (!searchInput) return;
    searchInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        this.executeSearch();
      }
    });
  },

  async executeSearch() {
    const searchInput = document.getElementById('searchInput');
    const searchResults = document.getElementById('searchResults');
    const pattern = searchInput.value.trim();
    if (!pattern) return;

    searchResults.innerHTML = '<div style="padding:12px;color:var(--fg-dim);font-size:12px;">搜索中...</div>';

    const tauri = window.__TAURI__;
    if (!tauri) {
      searchResults.innerHTML = '<div style="padding:12px;color:var(--fg-dim);">Tauri 不可用</div>';
      return;
    }
    const invoke = tauri.core?.invoke || tauri.invoke;

    const caseSensitive = document.getElementById('searchCaseSensitive')?.checked || false;
    const regex = document.getElementById('searchRegex')?.checked || false;
    const wholeWord = document.getElementById('searchWholeWord')?.checked || false;

    try {
      const result = await invoke('execute_tool', {
        name: 'grep',
        args: { pattern, path: '.', recursive: true, caseSensitive, regex, wholeWord }
      });
      const output = result.stdout || result.result || '';
      this.renderSearchResults(output);
    } catch (e) {
      searchResults.innerHTML = `<div style="padding:12px;color:var(--fg-red);">搜索失败: ${this.escapeHtml(e.message || e)}</div>`;
    }
  },

  renderSearchResults(output) {
    const searchResults = document.getElementById('searchResults');
    if (!output.trim()) {
      searchResults.innerHTML = '<div style="padding:12px;color:var(--fg-dim);font-size:12px;">未找到匹配</div>';
      return;
    }

    const lines = output.trim().split('\n');
    const results = [];
    for (const line of lines) {
      // Parse: file:line:content or file:line:col:content
      const match = line.match(/^(.+?):(\d+):(?:(\d+):)?(.+)$/);
      if (match) {
        results.push({
          file: match[1],
          line: parseInt(match[2]),
          col: match[3] ? parseInt(match[3]) : null,
          content: match[4]
        });
      }
    }

    if (!results.length) {
      searchResults.innerHTML = '<div style="padding:12px;color:var(--fg-dim);font-size:12px;">未找到匹配</div>';
      return;
    }

    // Group by file
    const byFile = {};
    results.forEach(r => {
      if (!byFile[r.file]) byFile[r.file] = [];
      byFile[r.file].push(r);
    });

    let html = '';
    for (const [file, matches] of Object.entries(byFile)) {
      html += `<div class="search-result-file">${this.escapeHtml(file)}</div>`;
      matches.forEach(m => {
        html += `<div class="search-result-line" data-file="${this.escapeHtml(m.file)}" data-line="${m.line}">
          <span class="search-result-lineno">${m.line}</span>
          <span class="search-result-content">${this.escapeHtml(m.content)}</span>
        </div>`;
      });
    }
    searchResults.innerHTML = html;

    searchResults.querySelectorAll('.search-result-line').forEach(el => {
      el.addEventListener('click', () => {
        const file = el.dataset.file;
        const line = parseInt(el.dataset.line);
        this.openFile(file);
        // TODO: scroll to line when editor supports it
      });
    });
  },

  // ============================================================
  // Git
  // ============================================================
  setupGit() {
    const commitBtn = document.getElementById('gitCommitActionBtn');
    const refreshBtn = document.getElementById('gitRefreshBtn');
    const commitInput = document.getElementById('gitCommitInput');
    const diffClose = document.getElementById('gitDiffClose');

    if (commitBtn) commitBtn.addEventListener('click', () => this.gitCommit());
    if (refreshBtn) refreshBtn.addEventListener('click', () => this.loadGitStatus());
    if (diffClose) diffClose.addEventListener('click', () => this.hideGitDiff());
    if (commitInput) {
      commitInput.addEventListener('keydown', (e) => {
        if (e.ctrlKey && e.key === 'Enter') {
          e.preventDefault();
          this.gitCommit();
        }
      });
    }
  },

  async loadGitStatus() {
    const tauri = window.__TAURI__;
    if (!tauri) return;
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      const result = await invoke('execute_tool', { name: 'git_status', args: {} });
      const output = result.stdout || result.result || '';
      this.renderGitFiles(output);
      this.updateGitBranch(output);
    } catch (e) {
      console.error('loadGitStatus error:', e);
    }
  },

  renderGitFiles(output) {
    const fileList = document.getElementById('gitFileList');
    const badge = document.getElementById('gitBadge');
    if (!fileList) return;

    const lines = output.trim().split('\n').filter(l => l.trim());
    if (!lines.length) {
      fileList.innerHTML = '<div style="padding:12px;color:var(--fg-dim);font-size:12px;">没有更改</div>';
      if (badge) badge.textContent = '0';
      return;
    }

    if (badge) badge.textContent = lines.length;

    let html = '';
    for (const line of lines) {
      // Parse git status --short format: XY filename or XY "filename"
      const match = line.match(/^\s*(\S+)\s+(.+)$/);
      if (!match) continue;
      const status = match[1];
      const file = match[2].replace(/^"|"$/g, '');

      let statusClass = 'modified';
      let statusLabel = 'M';
      if (status.includes('A') || status.startsWith('A')) { statusClass = 'added'; statusLabel = 'A'; }
      else if (status.includes('D') || status.startsWith('D')) { statusClass = 'deleted'; statusLabel = 'D'; }
      else if (status.includes('?')) { statusClass = 'untracked'; statusLabel = '?'; }
      else if (status.includes('M') || status.includes('R')) { statusClass = 'modified'; statusLabel = 'M'; }

      html += `<div class="git-file ${statusClass}" data-file="${this.escapeHtml(file)}">
        <span class="git-file-icon">${statusLabel}</span>
        <span class="git-file-name">${this.escapeHtml(file)}</span>
      </div>`;
    }
    fileList.innerHTML = html;

    fileList.querySelectorAll('.git-file').forEach(el => {
      el.addEventListener('click', () => {
        const file = el.dataset.file;
        this.showGitDiff(file);
      });
    });
  },

  async showGitDiff(file) {
    const tauri = window.__TAURI__;
    if (!tauri) return;
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      const result = await invoke('execute_tool', { name: 'git_diff', args: { file } });
      const diff = result.stdout || result.result || '';
      const diffView = document.getElementById('gitDiffView');
      const diffFileName = document.getElementById('gitDiffFileName');
      const diffContent = document.getElementById('gitDiffContent');
      if (diffView) diffView.style.display = 'block';
      if (diffFileName) diffFileName.textContent = file;
      if (diffContent) {
        // Simple diff coloring
        const colored = diff.split('\n').map(line => {
          if (line.startsWith('+')) return `<span class="diff-add">${this.escapeHtml(line)}</span>`;
          if (line.startsWith('-')) return `<span class="diff-del">${this.escapeHtml(line)}</span>`;
          if (line.startsWith('@@')) return `<span class="diff-hunk">${this.escapeHtml(line)}</span>`;
          return this.escapeHtml(line);
        }).join('\n');
        diffContent.innerHTML = colored;
      }
    } catch (e) {
      this.showErrorToast('获取 diff 失败: ' + (e.message || e));
    }
  },

  hideGitDiff() {
    const diffView = document.getElementById('gitDiffView');
    if (diffView) diffView.style.display = 'none';
  },

  async gitCommit() {
    const commitInput = document.getElementById('gitCommitInput');
    const message = commitInput?.value.trim();
    if (!message) {
      this.showErrorToast('请输入提交信息');
      return;
    }
    const tauri = window.__TAURI__;
    if (!tauri) { this.showErrorToast('Tauri 不可用'); return; }
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      await invoke('execute_tool', { name: 'git_commit', args: { message } });
      commitInput.value = '';
      this.loadGitStatus();
      this.showErrorToast('提交成功');
    } catch (e) {
      this.showErrorToast('提交失败: ' + (e.message || e));
    }
  },

  updateGitBranch(gitStatusOutput) {
    // Try to extract branch name from git status output
    // Format usually includes "On branch xxx" or is part of porcelain output
    // Fallback: try run_command to get current branch
    const tauri = window.__TAURI__;
    if (!tauri) return;
    const invoke = tauri.core?.invoke || tauri.invoke;
    invoke('run_command', { cmd: 'git', args: ['branch', '--show-current'] })
      .then(result => {
        const branch = (result.stdout || result).trim();
        const statusBranch = document.getElementById('statusBranch');
        if (statusBranch && branch) {
          statusBranch.innerHTML = `🌿 ${this.escapeHtml(branch)}`;
        }
      })
      .catch(() => {
        // Keep existing branch name on error
      });
  },

  // ============================================================
  // File Tree
  // ============================================================
  async loadFileTree(path) {
    const tauri = window.__TAURI__;
    if (!tauri) {
      // Fallback: show a placeholder when Tauri is not available
      this.fileTree = { name: 'workspace', type: 'folder', path: '.', expanded: true, children: [] };
      this.renderFileTree();
      return;
    }
    const invoke = tauri.core?.invoke || tauri.invoke;
    const rootPath = path || this.currentWorkspace || '.';
    try {
      const entries = await invoke('list_dir', { path: rootPath });
      this.fileTree = await this.buildTreeFromEntries(rootPath, entries);
      this.renderFileTree();
    } catch (e) {
      console.error('loadFileTree error:', e);
      this.showErrorToast('加载文件树失败: ' + (e.message || e));
    }
  },

  async buildTreeFromEntries(dirPath, entries) {
    const tauri = window.__TAURI__;
    const invoke = tauri ? (tauri.core?.invoke || tauri.invoke) : null;
    const children = [];
    // Sort: folders first, then files, both alphabetically
    const sorted = (entries || []).sort((a, b) => {
      const aIsDir = !a.includes('.');
      const bIsDir = !b.includes('.');
      if (aIsDir && !bIsDir) return -1;
      if (!aIsDir && bIsDir) return 1;
      return a.localeCompare(b);
    });
    for (const name of sorted) {
      const fullPath = dirPath + '/' + name;
      // Heuristic: no dot in name → likely folder
      let isFolder = !name.includes('.');
      // For dot-prefixed names (e.g. .github), we must probe
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
            folderChildren = await this.buildTreeFromEntries(fullPath, subEntries);
          } catch (e) {
            // Permission denied or not a directory — leave empty
          }
        }
        children.push({ name, type: 'folder', path: fullPath, expanded: false, children: folderChildren });
      } else {
        children.push({ name, type: 'file', path: fullPath, lang: this.guessLang(name) });
      }
    }
    return { name: dirPath.split('/').pop() || dirPath, type: 'folder', path: dirPath, expanded: true, children };
  },

  setupFileTreeToolbar() {
    const btns = document.querySelectorAll('.sidebar-view[data-view="explorer"] .sidebar-action-btn');
    if (!btns.length) return;
    // Order: new file, new folder, refresh, collapse all
    btns[0]?.addEventListener('click', () => this.createNewFile());
    btns[1]?.addEventListener('click', () => this.createNewFolder());
    btns[2]?.addEventListener('click', () => this.loadFileTree());
    btns[3]?.addEventListener('click', () => this.collapseAllFolders());
  },

  async createNewFile() {
    const name = prompt('输入新文件名称:');
    if (!name) return;
    const basePath = this.currentWorkspace || '.';
    const path = basePath + '/' + name;
    const tauri = window.__TAURI__;
    if (!tauri) { this.showErrorToast('Tauri 不可用'); return; }
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      await invoke('write_file', { path, content: '' });
      this.loadFileTree();
      this.openFile(path);
    } catch (e) {
      this.showErrorToast('创建文件失败: ' + (e.message || e));
    }
  },

  async createNewFolder() {
    const name = prompt('输入新文件夹名称:');
    if (!name) return;
    const basePath = this.currentWorkspace || '.';
    const path = basePath + '/' + name;
    const tauri = window.__TAURI__;
    if (!tauri) { this.showErrorToast('Tauri 不可用'); return; }
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      await invoke('run_command', { cmd: 'mkdir', args: [path] });
      this.loadFileTree();
    } catch (e) {
      this.showErrorToast('创建文件夹失败: ' + (e.message || e));
    }
  },

  collapseAllFolders() {
    const collapse = (node) => {
      if (node.type === 'folder') {
        node.expanded = false;
        if (node.children) node.children.forEach(collapse);
      }
    };
    if (this.fileTree) collapse(this.fileTree);
    this.renderFileTree();
  },

  renderFileTree() {
    const container = document.getElementById('fileTree');
    container.innerHTML = '';
    if (!this.fileTree) {
      container.innerHTML = '<div style="padding:12px;color:var(--fg-dim);font-size:12px;">加载中...</div>';
      return;
    }
    this.renderTreeNode(this.fileTree, container, 0);
  },

  renderTreeNode(node, container, depth) {
    if (node.type === 'folder') {
      const folderEl = document.createElement('div');
      folderEl.className = `file-tree-item folder${node.expanded ? ' expanded' : ''}`;
      folderEl.style.paddingLeft = `${8 + depth * 16}px`;
      folderEl.innerHTML = `
        <span class="tree-toggle">▶</span>
        <span class="tree-icon">
          📁
        </span>
        <span class="tree-label">${node.name}</span>
      `;
      folderEl.addEventListener('click', (e) => {
        e.stopPropagation();
        node.expanded = !node.expanded;
        this.renderFileTree();
      });
      folderEl.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        e.stopPropagation();
        this.showContextMenu(e, node);
      });
      container.appendChild(folderEl);

      const childrenContainer = document.createElement('div');
      childrenContainer.className = 'file-tree-children';
      if (node.expanded) {
        childrenContainer.style.display = 'block';
      }
      if (node.children) {
        node.children.forEach(child => this.renderTreeNode(child, childrenContainer, depth + 1));
      }
      container.appendChild(childrenContainer);
    } else {
      const fileEl = document.createElement('div');
      fileEl.className = 'file-tree-item';
      fileEl.style.paddingLeft = `${8 + depth * 16}px`;
      const iconColor = this.getFileIconColor(node.name);
      fileEl.innerHTML = `
        <span class="tree-toggle"></span>
        <span class="tree-icon file-icon" style="color:${iconColor}">
          ${this.getFileIconSvg(node.name)}
        </span>
        <span class="tree-label">${node.name}</span>
      `;
      fileEl.addEventListener('click', (e) => {
        e.stopPropagation();
        document.querySelectorAll('.file-tree-item').forEach(el => el.classList.remove('selected'));
        fileEl.classList.add('selected');
        this.openFile(node.path);
      });
      fileEl.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        e.stopPropagation();
        this.showContextMenu(e, node);
      });
      container.appendChild(fileEl);
    }
  },

  getFileIconColor(filename) {
    const ext = filename.split('.').pop();
    const colors = {
      rs: 'var(--fg-dim)', toml: 'var(--fg-dim)', md: 'var(--fg-dim)',
      js: 'var(--fg-dim)', ts: 'var(--fg-dim)', css: 'var(--fg-dim)',
      html: 'var(--fg-dim)', json: 'var(--fg-dim)', py: 'var(--fg-dim)'
    };
    return colors[ext] || 'var(--fg-dim)';
  },

  getFileIconSvg(filename) {
    const ext = filename.split('.').pop();
    const icons = {
      rs: '🦀', toml: '⚙', md: '📝',
      js: '𝕁', ts: '𝕋', css: '🎨',
      html: '𝐇', json: '{ }'
    };
    return icons[ext] || '📄';
  },

  // ============================================================
  // Context Menu
  // ============================================================
  showContextMenu(event, node) {
    this.hideContextMenu();
    const menu = document.createElement('div');
    menu.className = 'context-menu';
    menu.id = 'contextMenu';
    menu.style.left = event.pageX + 'px';
    menu.style.top = event.pageY + 'px';

    const isFile = node.type === 'file';
    const items = [];
    if (isFile) {
      items.push({ label: '打开', action: () => this.openFile(node.path) });
      items.push({ label: '添加到 AI 上下文', action: () => this.addChatContextFile(node.path) });
    }
    items.push({ label: '复制路径', action: () => this.copyPath(node.path) });
    items.push({ label: '重命名', action: () => this.renameFile(node.path) });
    items.push({ label: '删除', action: () => this.deleteFile(node.path), danger: true });
    items.push({ label: '在终端中打开', action: () => this.openInTerminal(node.path) });

    menu.innerHTML = items.map(item =>
      `<div class="context-menu-item${item.danger ? ' danger' : ''}">${item.label}</div>`
    ).join('');

    menu.querySelectorAll('.context-menu-item').forEach((el, i) => {
      el.addEventListener('click', () => {
        items[i].action();
        this.hideContextMenu();
      });
    });

    document.body.appendChild(menu);

    // Close on click outside
    const closeHandler = (e) => {
      if (!menu.contains(e.target)) {
        this.hideContextMenu();
        document.removeEventListener('click', closeHandler);
      }
    };
    setTimeout(() => document.addEventListener('click', closeHandler), 0);
  },

  hideContextMenu() {
    const existing = document.getElementById('contextMenu');
    if (existing) existing.remove();
  },

  async copyPath(path) {
    try {
      await navigator.clipboard.writeText(path);
      this.showErrorToast('路径已复制');
    } catch (e) {
      this.showErrorToast('复制失败');
    }
  },

  async renameFile(oldPath) {
    const oldName = oldPath.split('/').pop();
    const newName = prompt('重命名为:', oldName);
    if (!newName || newName === oldName) return;
    const dir = oldPath.substring(0, oldPath.lastIndexOf('/'));
    const newPath = dir + '/' + newName;
    const tauri = window.__TAURI__;
    if (!tauri) { this.showErrorToast('Tauri 不可用'); return; }
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      await invoke('run_command', { cmd: 'mv', args: [oldPath, newPath] });
      this.loadFileTree();
      // If file is open in a tab, close it
      const tab = this.tabs.find(t => t.id === oldPath);
      if (tab) this._doCloseTab(oldPath);
    } catch (e) {
      this.showErrorToast('重命名失败: ' + (e.message || e));
    }
  },

  async deleteFile(path) {
    const name = path.split('/').pop();
    if (!confirm(`确定要删除 "${name}" 吗？`)) return;
    const tauri = window.__TAURI__;
    if (!tauri) { this.showErrorToast('Tauri 不可用'); return; }
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      await invoke('run_command', { cmd: 'rm', args: ['-rf', path] });
      this.loadFileTree();
      const tab = this.tabs.find(t => t.id === path);
      if (tab) this._doCloseTab(path);
    } catch (e) {
      this.showErrorToast('删除失败: ' + (e.message || e));
    }
  },

  openInTerminal(path) {
    this.showPanel('terminal');
    const isFile = path.includes('.');
    const dir = isFile ? path.substring(0, path.lastIndexOf('/')) : path;
    const cmd = `cd "${dir}"`;
    this.addTerminalOutput(`$ ${cmd}`, 'cmd');
    this.addTerminalOutput(`Working directory: ${dir}`, 'output');
    this.appendTerminalPrompt();
  },

  // ============================================================
  // Tabs & Editor
  // ============================================================
  setupTabs() {
    const tabBar = document.getElementById('tabBar');
    tabBar.addEventListener('click', (e) => {
      const tab = e.target.closest('.tab');
      if (!tab) return;
      const closeBtn = e.target.closest('.tab-close');
      if (closeBtn) {
        this.closeTab(tab.dataset.file);
      } else {
        this.activateTab(tab.dataset.file);
      }
    });
  },

  openTab(id, label, content, type = 'code') {
    const existing = this.tabs.find(t => t.id === id);
    if (existing) {
      this.activateTab(id);
      return;
    }

    const tab = { id, label, content, type, lang: this.guessLang(id), dirty: false, originalContent: content };
    this.tabs.push(tab);
    this.renderTabs();
    this.activateTab(id);
  },

  closeTab(id) {
    const tab = this.tabs.find(t => t.id === id);
    if (!tab) return;
    if (tab.dirty) {
      const choice = confirm(`"${tab.label}" 有未保存的更改。是否保存？\n\n确定 = 保存并关闭\n取消 = 不保存直接关闭`);
      if (choice) {
        this.saveFile().then(() => this._doCloseTab(id));
        return;
      }
    }
    this._doCloseTab(id);
  },

  _doCloseTab(id) {
    const idx = this.tabs.findIndex(t => t.id === id);
    if (idx === -1) return;
    this.tabs.splice(idx, 1);

    if (this.activeTab === id) {
      this.activeTab = this.tabs.length > 0 ? this.tabs[Math.min(idx, this.tabs.length - 1)].id : null;
    }
    this.renderTabs();
    this.renderEditor();
  },

  activateTab(id) {
    this.activeTab = id;
    this.renderTabs();
    this.renderEditor();
    this.updateStatusBar();
  },

  renderTabs() {
    const tabBar = document.getElementById('tabBar');
    tabBar.innerHTML = this.tabs.map(tab => `
      <div class="tab${tab.id === this.activeTab ? ' active' : ''}${tab.dirty ? ' dirty' : ''}" data-file="${tab.id}">
        <span class="tab-icon">${this.getTabIcon(tab.id)}</span>
        <span class="tab-label">${tab.dirty ? '● ' : ''}${tab.label}</span>
        <span class="tab-close">×</span>
      </div>
    `).join('');
  },

  getTabIcon(id) {
    if (id === 'welcome') {
      return '⌂';
    }
    const ext = id.split('.').pop();
    const color = this.getFileIconColor(id);
    return '📄';
  },

  renderEditor() {
    const editorArea = document.getElementById('editorArea');
    editorArea.innerHTML = '';

    this.renderBreadcrumb();

    if (!this.activeTab) return;

    const tab = this.tabs.find(t => t.id === this.activeTab);
    if (!tab) return;

    if (tab.type === 'welcome') {
      this.renderWelcomeView(editorArea);
    } else {
      this.renderCodeView(editorArea, tab);
    }
  },

  renderBreadcrumb() {
    const bar = document.getElementById('breadcrumbBar');
    if (!bar) return;

    if (!this.activeTab || this.activeTab === 'welcome') {
      bar.innerHTML = '';
      bar.style.display = 'none';
      return;
    }

    bar.style.display = 'flex';
    const parts = this.activeTab.split('/');
    let html = '';
    let currentPath = '';
    parts.forEach((part, i) => {
      currentPath = currentPath ? `${currentPath}/${part}` : part;
      const isLast = i === parts.length - 1;
      html += `<span class="breadcrumb-separator">${i > 0 ? '›' : ''}</span>`;
      html += `<span class="breadcrumb-item${isLast ? ' active' : ''}" data-path="${this.escapeHtml(currentPath)}">${this.escapeHtml(part)}</span>`;
    });
    bar.innerHTML = html;

    bar.querySelectorAll('.breadcrumb-item').forEach(el => {
      el.addEventListener('click', () => {
        const path = el.dataset.path;
        const isFile = path.includes('.');
        if (isFile) {
          this.openFile(path);
        } else {
          this.showSidebar('explorer');
          // TODO: expand folder in file tree
        }
      });
    });
  },

  renderWelcomeView(container) {
    container.innerHTML = `
      <div class="editor-view active">
        <div class="welcome-page">
          <div class="welcome-logo">
            <img src="logo.jpg" alt="Hajimi" width="80" height="80" style="border-radius: 12px;">
          </div>
          <h1 class="welcome-title">Hajimi Code</h1>
          <p class="welcome-subtitle">本地优先 AI 智能体 IDE</p>
          <div class="welcome-start">
            <div class="welcome-section">
              <h3>开始</h3>
              <div class="welcome-link" onclick="app.openFile('src/interface/desktop/src/main.rs')">
                📄
                打开 main.rs
              </div>
              <div class="welcome-link" onclick="app.openFolder()">
                📁
                打开文件夹
              </div>
              <div class="welcome-link" onclick="app.cloneRepo()">
                🌿
                克隆仓库
              </div>
            </div>
            <div class="welcome-section">
              <h3>最近</h3>
              <div class="welcome-link" onclick="app.openFile('Cargo.toml')">
                ◷
                hajimi-code-cli
              </div>
            </div>
          </div>
        </div>
      </div>
    `;
  },

  renderCodeView(container, tab) {
    const content = tab.content || `// ${tab.id}\n// （加载中...）`;
    const lines = content.split('\n');
    const highlighted = this.highlightCode(content, tab.lang);

    container.innerHTML = `
      <div class="editor-view active">
        <div class="code-editor">
          <div class="line-numbers">
            ${lines.map((_, i) => `<div>${i + 1}</div>`).join('')}
          </div>
          <div class="editor-content" contenteditable="true" spellcheck="false">${highlighted}</div>
        </div>
      </div>
    `;

    const editorContent = container.querySelector('.editor-content');
    if (editorContent) {
      editorContent.addEventListener('input', () => {
        const currentText = editorContent.innerText;
        tab.dirty = currentText !== (tab.originalContent || '');
        this.renderTabs();
      });
      editorContent.addEventListener('keydown', (e) => {
        if (e.key === 'Tab') {
          e.preventDefault();
          document.execCommand('insertText', false, '  ');
        }
        if (e.ctrlKey && e.key === 's') {
          e.preventDefault();
          this.saveFile();
        }
        if (e.key === 'F12') {
          e.preventDefault();
          this.lspDefinition(tab.id);
        }
        if (e.shiftKey && e.key === 'F12') {
          e.preventDefault();
          this.lspReferences(tab.id);
        }
      });

      // LSP hover tooltip
      let hoverTimeout = null;
      editorContent.addEventListener('mouseover', (e) => {
        if (hoverTimeout) clearTimeout(hoverTimeout);
        hoverTimeout = setTimeout(() => {
          const sel = window.getSelection();
          if (!sel.rangeCount) return;
          const rect = sel.getRangeAt(0).getBoundingClientRect();
          this.lspHover(tab.id, rect);
        }, 800);
      });
      editorContent.addEventListener('mouseout', () => {
        if (hoverTimeout) clearTimeout(hoverTimeout);
        this.hideLspTooltip();
      });
      // Re-highlight on blur (preserve cursor roughly at end for simplicity)
      editorContent.addEventListener('blur', () => {
        if (tab.dirty) {
          const text = editorContent.innerText;
          const newHighlighted = this.highlightCode(text, tab.lang);
          const lines = text.split('\n');
          const lineNumbers = container.querySelector('.line-numbers');
          if (lineNumbers) {
            lineNumbers.innerHTML = lines.map((_, i) => `<div>${i + 1}</div>`).join('');
          }
          editorContent.innerHTML = newHighlighted;
          // Auto save on focus change
          if (this.settings.autoSave === 'onFocusChange') {
            this.saveFile();
          }
        }
      });
    }
  },

  highlightCode(code, lang) {
    const html = code
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');

    if (lang === 'rust') {
      return html
        .replace(/\b(use|fn|let|mut|struct|enum|impl|pub|crate|mod|if|else|match|return|async|await|const|static|type|where|for|in|loop|while|break|continue|trait|unsafe|move|ref|self|Self|super|as|dyn)\b/g, '<span class="syntax-keyword">$1</span>')
        .replace(/\b(String|Vec|Option|Result|Box|Arc|Mutex|HashMap|VecDeque|i32|i64|u32|u64|usize|isize|f32|f64|bool|char|str)\b/g, '<span class="syntax-type">$1</span>')
        .replace(/\b([A-Z][a-zA-Z0-9_]*)\b/g, '<span class="syntax-type">$1</span>')
        .replace(/(".*?")/g, '<span class="syntax-string">$1</span>')
        .replace(/(\/\/.*$)/gm, '<span class="syntax-comment">$1</span>')
        .replace(/\b(\d+)\b/g, '<span class="syntax-number">$1</span>')
        .replace(/\b([a-z_][a-zA-Z0-9_]*)(?=\()/g, '<span class="syntax-function">$1</span>')
        .replace(/(#\[.*?\])/g, '<span class="syntax-macro">$1</span>')
        .replace(/('static|'a|'b|'c)/g, '<span class="syntax-lifetime">$1</span>');
    }
    if (lang === 'toml') {
      return html
        .replace(/^(\[.*?\])$/gm, '<span class="syntax-keyword">$1</span>')
        .replace(/^(\w+)\s*=/gm, '<span class="syntax-function">$1</span> =')
        .replace(/(".*?")/g, '<span class="syntax-string">$1</span>')
        .replace(/(#.*$)/gm, '<span class="syntax-comment">$1</span>');
    }
    if (lang === 'javascript' || lang === 'typescript') {
      return html
        .replace(/\b(const|let|var|function|class|extends|import|export|from|return|if|else|for|while|async|await|new|this|try|catch|throw|typeof|instanceof)\b/g, '<span class="syntax-keyword">$1</span>')
        .replace(/(".*?"|'.*?'|`.*?`)/g, '<span class="syntax-string">$1</span>')
        .replace(/(\/\/.*$|\/\*[\s\S]*?\*\/)/gm, '<span class="syntax-comment">$1</span>')
        .replace(/\b(\d+)\b/g, '<span class="syntax-number">$1</span>')
        .replace(/\b([a-z_][a-zA-Z0-9_]*)(?=\()/g, '<span class="syntax-function">$1</span>');
    }
    if (lang === 'python') {
      return html
        .replace(/\b(def|class|if|elif|else|for|while|return|import|from|as|try|except|finally|with|lambda|yield|async|await|pass|break|continue|raise|assert|del|global|nonlocal|and|or|not|in|is|None|True|False)\b/g, '<span class="syntax-keyword">$1</span>')
        .replace(/(".*?"|'.*?')/g, '<span class="syntax-string">$1</span>')
        .replace(/(#.*$)/gm, '<span class="syntax-comment">$1</span>')
        .replace(/\b(\d+)\b/g, '<span class="syntax-number">$1</span>')
        .replace(/\b([a-z_][a-zA-Z0-9_]*)(?=\()/g, '<span class="syntax-function">$1</span>');
    }
    if (lang === 'go') {
      return html
        .replace(/\b(package|import|func|var|const|type|struct|interface|map|chan|range|if|else|for|switch|case|default|return|defer|go|select|break|continue|goto|fallthrough|nil|true|false)\b/g, '<span class="syntax-keyword">$1</span>')
        .replace(/(".*?")/g, '<span class="syntax-string">$1</span>')
        .replace(/(\/\/.*$)/gm, '<span class="syntax-comment">$1</span>')
        .replace(/\b(\d+)\b/g, '<span class="syntax-number">$1</span>')
        .replace(/\b([a-z_][a-zA-Z0-9_]*)(?=\()/g, '<span class="syntax-function">$1</span>');
    }
    if (lang === 'java') {
      return html
        .replace(/\b(abstract|assert|boolean|break|byte|case|catch|char|class|const|continue|default|do|double|else|enum|extends|final|finally|float|for|if|goto|implements|import|instanceof|int|interface|long|native|new|package|private|protected|public|return|short|static|strictfp|super|switch|synchronized|this|throw|throws|transient|try|void|volatile|while|null|true|false)\b/g, '<span class="syntax-keyword">$1</span>')
        .replace(/\b(String|Integer|Boolean|Double|Float|Long|Object|List|Map|Set|ArrayList|HashMap)\b/g, '<span class="syntax-type">$1</span>')
        .replace(/(".*?")/g, '<span class="syntax-string">$1</span>')
        .replace(/(\/\/.*$|\/\*[\s\S]*?\*\/)/gm, '<span class="syntax-comment">$1</span>')
        .replace(/\b(\d+)\b/g, '<span class="syntax-number">$1</span>')
        .replace(/\b([a-z_][a-zA-Z0-9_]*)(?=\()/g, '<span class="syntax-function">$1</span>');
    }
    if (lang === 'c' || lang === 'cpp') {
      return html
        .replace(/\b(auto|break|case|char|const|continue|default|do|double|else|enum|extern|float|for|goto|if|inline|int|long|register|restrict|return|short|signed|sizeof|static|struct|switch|typedef|union|unsigned|void|volatile|while|class|public|private|protected|virtual|override|namespace|template|typename|new|delete|try|catch|throw|nullptr|true|false|bool)\b/g, '<span class="syntax-keyword">$1</span>')
        .replace(/(".*?")/g, '<span class="syntax-string">$1</span>')
        .replace(/(\/\/.*$|\/\*[\s\S]*?\*\/)/gm, '<span class="syntax-comment">$1</span>')
        .replace(/\b(\d+)\b/g, '<span class="syntax-number">$1</span>')
        .replace(/\b([a-z_][a-zA-Z0-9_]*)(?=\()/g, '<span class="syntax-function">$1</span>');
    }
    if (lang === 'css' || lang === 'scss') {
      return html
        .replace(/([a-z-]+)(?=\s*:)/g, '<span class="syntax-function">$1</span>')
        .replace(/(".*?")/g, '<span class="syntax-string">$1</span>')
        .replace(/(\/\/.*$|\/\*[\s\S]*?\*\/)/gm, '<span class="syntax-comment">$1</span>')
        .replace(/\b(\d+(?:px|em|rem|%|vh|vw|pt|cm|mm|in|ex|ch|vmin|vmax|deg|rad|turn|s|ms|hz|khz)?)\b/g, '<span class="syntax-number">$1</span>')
        .replace(/(#(?:[0-9a-fA-F]{3}){1,2})/g, '<span class="syntax-string">$1</span>');
    }
    if (lang === 'json') {
      return html
        .replace(/(".*?")(?=\s*:)/g, '<span class="syntax-function">$1</span>')
        .replace(/(".*?")/g, '<span class="syntax-string">$1</span>')
        .replace(/\b(true|false|null)\b/g, '<span class="syntax-keyword">$1</span>')
        .replace(/\b(\d+(?:\.\d+)?)\b/g, '<span class="syntax-number">$1</span>');
    }
    if (lang === 'yaml' || lang === 'yml') {
      return html
        .replace(/^(\s*[-]?\s*)(\w+)(?=:)/gm, '$1<span class="syntax-function">$2</span>')
        .replace(/(".*?")/g, '<span class="syntax-string">$1</span>')
        .replace(/(#.*$)/gm, '<span class="syntax-comment">$1</span>')
        .replace(/\b(true|false|null|yes|no|on|off)\b/g, '<span class="syntax-keyword">$1</span>');
    }
    if (lang === 'markdown') {
      return html
        .replace(/^(#{1,6}\s+.+)$/gm, '<span class="syntax-keyword">$1</span>')
        .replace(/(\*\*.*?\*\*)/g, '<span class="syntax-type">$1</span>')
        .replace(/(`.*?`)/g, '<span class="syntax-string">$1</span>')
        .replace(/^(\s*[-*+]\s+.+)$/gm, '<span class="syntax-comment">$1</span>')
        .replace(/^(\s*\d+\.\s+.+)$/gm, '<span class="syntax-comment">$1</span>');
    }
    if (lang === 'sql') {
      return html
        .replace(/\b(SELECT|INSERT|UPDATE|DELETE|FROM|WHERE|JOIN|LEFT|RIGHT|INNER|OUTER|ON|GROUP|BY|ORDER|HAVING|LIMIT|OFFSET|UNION|ALL|DISTINCT|CREATE|TABLE|DROP|ALTER|INDEX|VIEW|TRIGGER|PROCEDURE|FUNCTION|DATABASE|SCHEMA|USE|SHOW|DESCRIBE|EXPLAIN|AND|OR|NOT|IN|BETWEEN|LIKE|IS|NULL|TRUE|FALSE|AS|CASE|WHEN|THEN|ELSE|END|IF|ELSEIF|WHILE|FOR|LOOP|RETURN|BEGIN|COMMIT|ROLLBACK|TRANSACTION|PRIMARY|KEY|FOREIGN|REFERENCES|DEFAULT|AUTO_INCREMENT|UNIQUE|CHECK|CONSTRAINT|CASCADE|SET|VALUES|INTO|VALUES|COUNT|SUM|AVG|MIN|MAX)\b/gi, '<span class="syntax-keyword">$1</span>')
        .replace(/(".*?"|'.*?')/g, '<span class="syntax-string">$1</span>')
        .replace(/(--.*$)/gm, '<span class="syntax-comment">$1</span>')
        .replace(/\b(\d+)\b/g, '<span class="syntax-number">$1</span>');
    }
    if (lang === 'dockerfile') {
      return html
        .replace(/^(FROM|RUN|CMD|LABEL|MAINTAINER|EXPOSE|ENV|ADD|COPY|ENTRYPOINT|VOLUME|USER|WORKDIR|ARG|ONBUILD|STOPSIGNAL|HEALTHCHECK|SHELL)\b/gim, '<span class="syntax-keyword">$1</span>')
        .replace(/(".*?")/g, '<span class="syntax-string">$1</span>')
        .replace(/(#.*$)/gm, '<span class="syntax-comment">$1</span>');
    }
    if (lang === 'bash' || lang === 'powershell') {
      return html
        .replace(/\b(if|then|else|elif|fi|for|while|do|done|case|esac|in|function|return|exit|echo|export|source|alias|unset|read|printf|test|true|false|continue|break|shift|eval|exec|trap|wait)\b/g, '<span class="syntax-keyword">$1</span>')
        .replace(/(".*?")/g, '<span class="syntax-string">$1</span>')
        .replace(/(#.*$)/gm, '<span class="syntax-comment">$1</span>')
        .replace(/\b(\d+)\b/g, '<span class="syntax-number">$1</span>');
    }
    return html;
  },

  guessLang(filename) {
    const ext = filename.split('.').pop().toLowerCase();
    const map = {
      rs: 'rust', toml: 'toml',
      js: 'javascript', ts: 'typescript', jsx: 'javascript', tsx: 'typescript',
      css: 'css', scss: 'scss', sass: 'scss', less: 'scss',
      html: 'html', htm: 'html', xml: 'xml',
      json: 'json', yaml: 'yaml', yml: 'yaml',
      md: 'markdown', markdown: 'markdown',
      py: 'python', pyw: 'python',
      go: 'go',
      java: 'java',
      c: 'c', h: 'c', cpp: 'cpp', cc: 'cpp', hpp: 'cpp', cxx: 'cpp',
      sql: 'sql',
      dockerfile: 'dockerfile',
      sh: 'bash', bash: 'bash', zsh: 'bash', ps1: 'powershell'
    };
    return map[ext] || 'text';
  },

  async openFile(path) {
    const tauri = window.__TAURI__;
    let content = '';
    if (tauri) {
      const invoke = tauri.core?.invoke || tauri.invoke;
      try {
        content = await invoke('read_file', { path });
      } catch (e) {
        this.showErrorToast('读取文件失败: ' + (e.message || e));
        return;
      }
    }
    this.addFilePreviewMessage(path, content);
  },

  addFilePreviewMessage(path, content) {
    const label = path.split('/').pop();
    const messages = document.getElementById('aiChatMessages');
    if (!messages) return;
    const div = document.createElement('div');
    div.className = 'chat-message ai';
    const truncated = content.length > 3000 ? content.slice(0, 3000) + '\n...' : content;
    div.innerHTML = `
      <div class="file-preview-header" style="display:flex;align-items:center;justify-content:space-between;padding:4px 0;margin-bottom:4px;font-size:12px;font-family:var(--font-mono);color:var(--fg-dim);">
        <span>📄 ${this.escapeHtml(label)}</span>
        <button class="copy-file-btn" style="background:transparent;border:none;color:var(--fg-dim);cursor:pointer;font-size:11px;padding:2px 6px;border-radius:3px;">📋 Copy</button>
      </div>
      <pre style="margin:0;background:var(--bg-subtle);border:1px solid var(--border);border-radius:var(--radius-sm);padding:var(--space-2);overflow-x:auto;font-size:12px;line-height:1.5;"><code>${this.escapeHtml(truncated)}</code></pre>
    `;
    const copyBtn = div.querySelector('.copy-file-btn');
    if (copyBtn) {
      copyBtn.addEventListener('click', () => {
        navigator.clipboard?.writeText(content);
        copyBtn.textContent = '✓ Copied';
        setTimeout(() => copyBtn.textContent = '📋 Copy', 2000);
      });
    }
    messages.appendChild(div);
    messages.scrollTop = messages.scrollHeight;
  },

  addDiffMessageCard(filePath, oldLines, newLines) {
    const messages = document.getElementById('aiChatMessages');
    if (!messages) return;
    const div = document.createElement('div');
    div.className = 'chat-message ai diff-card';
    const label = filePath.split('/').pop();
    const cardId = 'diff-' + Date.now();
    div.id = cardId;

    // Build diff lines
    let diffHtml = '';
    const maxLines = Math.max(oldLines.length, newLines.length);
    for (let i = 0; i < maxLines; i++) {
      const oldLine = oldLines[i] !== undefined ? oldLines[i] : null;
      const newLine = newLines[i] !== undefined ? newLines[i] : null;
      if (oldLine === newLine && oldLine !== null) {
        diffHtml += `<div class="diff-line"><span class="diff-line-num ctx">${i + 1}</span><span class="diff-gutter"> </span><span class="diff-line-code">${this.escapeHtml(oldLine)}</span></div>`;
      } else if (oldLine !== null && newLine === null) {
        diffHtml += `<div class="diff-line del"><span class="diff-line-num del">${i + 1}</span><span class="diff-gutter">-</span><span class="diff-line-code">${this.escapeHtml(oldLine)}</span></div>`;
      } else if (newLine !== null && oldLine === null) {
        diffHtml += `<div class="diff-line add"><span class="diff-line-num add">${i + 1}</span><span class="diff-gutter">+</span><span class="diff-line-code">${this.escapeHtml(newLine)}</span></div>`;
      } else if (oldLine !== null && newLine !== null) {
        diffHtml += `<div class="diff-line del"><span class="diff-line-num del">${i + 1}</span><span class="diff-gutter">-</span><span class="diff-line-code">${this.escapeHtml(oldLine)}</span></div>`;
        diffHtml += `<div class="diff-line add"><span class="diff-line-num add">${i + 1}</span><span class="diff-gutter">+</span><span class="diff-line-code">${this.escapeHtml(newLine)}</span></div>`;
      }
    }

    div.innerHTML = `
      <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:8px;padding:4px 0;font-size:12px;font-family:var(--font-mono);">
        <span style="color:var(--fg-default);">📄 ${this.escapeHtml(label)}</span>
        <div style="display:flex;gap:6px;">
          <button class="diff-apply-btn" data-card="${cardId}" style="background:var(--diff-add-bg);border:1px solid var(--fg-green);color:var(--fg-green);cursor:pointer;padding:3px 10px;border-radius:4px;font-size:11px;">✓ Apply</button>
          <button class="diff-reject-btn" data-card="${cardId}" style="background:transparent;border:1px solid var(--fg-dim);color:var(--fg-dim);cursor:pointer;padding:3px 10px;border-radius:4px;font-size:11px;">✗ Reject</button>
        </div>
      </div>
      <div class="diff-view" style="max-height:300px;overflow-y:auto;">${diffHtml}</div>
    `;

    const applyBtn = div.querySelector('.diff-apply-btn');
    const rejectBtn = div.querySelector('.diff-reject-btn');

    applyBtn.addEventListener('click', async () => {
      const tauri = window.__TAURI__;
      if (!tauri) { this.showErrorToast('Tauri 不可用'); return; }
      const invoke = tauri.core?.invoke || tauri.invoke;
      try {
        await invoke('apply_edits', {
          edits: [{
            path: filePath,
            old_string: oldLines.join('\n'),
            new_string: newLines.join('\n'),
          }]
        });
        applyBtn.textContent = '✓ Applied';
        applyBtn.disabled = true;
        applyBtn.style.opacity = '0.6';
        rejectBtn.style.display = 'none';
        this.showErrorToast('修改已应用');
      } catch (e) {
        this.showErrorToast('应用失败: ' + (e.message || e));
      }
    });

    rejectBtn.addEventListener('click', () => {
      div.remove();
    });

    messages.appendChild(div);
    messages.scrollTop = messages.scrollHeight;
  },

  async saveFile() {
    this.showErrorToast('保存功能在当前布局中不可用');
  },

  openFilePrompt() {
    const path = prompt('输入文件路径:');
    if (path) this.openFile(path);
  },

  async openFolder() {
    const path = prompt('输入文件夹路径:', this.currentWorkspace || '.');
    if (!path) return;
    this.currentWorkspace = path;
    await this.loadFileTree(path);
    this.showSidebar('explorer');
  },

  async cloneRepo() {
    const url = prompt('输入仓库 URL:');
    if (!url) return;
    const tauri = window.__TAURI__;
    if (!tauri) { this.showErrorToast('Tauri 不可用'); return; }
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      this.addTerminalOutput(`$ git clone ${url}`, 'cmd');
      const result = await invoke('run_command', { cmd: 'git', args: ['clone', url] });
      this.addTerminalOutput(result, 'output');
      this.loadFileTree();
    } catch (e) {
      this.addTerminalOutput('克隆失败: ' + (e.message || e), 'error');
    }
  },

  // ============================================================
  // Bottom Panel
  // ============================================================
  setupPanel() {
    document.querySelectorAll('.panel-tab').forEach(tab => {
      tab.addEventListener('click', () => {
        this.showPanel(tab.dataset.panel);
      });
    });

    document.getElementById('closePanelBtn').addEventListener('click', () => {
      document.getElementById('bottomPanel').classList.toggle('collapsed');
    });

    document.getElementById('maximizePanelBtn').addEventListener('click', () => {
      const panel = document.getElementById('bottomPanel');
      panel.style.flex = panel.style.flex === '3' ? '' : '3';
    });
  },

  showPanel(view) {
    this.panelView = view;
    document.querySelectorAll('.panel-tab').forEach(el => {
      el.classList.toggle('active', el.dataset.panel === view);
    });
    document.querySelectorAll('.panel-view').forEach(el => {
      el.classList.toggle('active', el.dataset.panel === view);
    });
    document.getElementById('bottomPanel').classList.remove('collapsed');
    if (view === 'problems') {
      this.loadProblems();
    }
  },

  togglePanel(view) {
    const panel = document.getElementById('bottomPanel');
    if (panel.classList.contains('collapsed') || this.panelView !== view) {
      this.showPanel(view);
    } else {
      panel.classList.add('collapsed');
    }
  },

  // ============================================================
  // Terminal
  // ============================================================
  setupTerminal() {
    const terminalContent = document.getElementById('terminalContent');
    if (!terminalContent) return;
    // Clear placeholder content and initialize with a prompt
    terminalContent.innerHTML = '';
    this.appendTerminalPrompt();

    terminalContent.addEventListener('keydown', (e) => {
      const input = terminalContent.querySelector('.terminal-input:focus');
      if (!input) return;

      if (e.key === 'Enter') {
        e.preventDefault();
        const cmd = input.innerText.trim();
        // Remove the input line (will be re-added as output + new prompt)
        const line = input.closest('.terminal-line');
        if (line) line.remove();
        if (cmd) {
          this.commandHistory.push(cmd);
          this.commandHistoryIndex = this.commandHistory.length;
          this.executeTerminalCommand(cmd);
        } else {
          this.appendTerminalPrompt();
        }
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        if (this.commandHistoryIndex > 0) {
          this.commandHistoryIndex--;
          input.innerText = this.commandHistory[this.commandHistoryIndex];
        }
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        if (this.commandHistoryIndex < this.commandHistory.length - 1) {
          this.commandHistoryIndex++;
          input.innerText = this.commandHistory[this.commandHistoryIndex];
        } else {
          this.commandHistoryIndex = this.commandHistory.length;
          input.innerText = '';
        }
      }
    });

    // Focus input when clicking anywhere in terminal
    terminalContent.addEventListener('click', (e) => {
      if (e.target === terminalContent) {
        const input = terminalContent.querySelector('.terminal-input');
        if (input) input.focus();
      }
    });
  },

  appendTerminalPrompt() {
    const terminalContent = document.getElementById('terminalContent');
    if (!terminalContent) return;
    const line = document.createElement('div');
    line.className = 'terminal-line';
    line.innerHTML = '<span class="terminal-prompt">$ </span><span class="terminal-input" contenteditable="true"></span>';
    terminalContent.appendChild(line);
    const input = line.querySelector('.terminal-input');
    if (input) {
      input.focus();
      // Scroll to bottom
      terminalContent.scrollTop = terminalContent.scrollHeight;
    }
  },

  addTerminalOutput(text, type = 'output') {
    const terminalContent = document.getElementById('terminalContent');
    if (!terminalContent) return;
    const lines = text.split('\n');
    lines.forEach(line => {
      const div = document.createElement('div');
      div.className = `terminal-line terminal-${type}`;
      div.textContent = line;
      terminalContent.appendChild(div);
    });
    terminalContent.scrollTop = terminalContent.scrollHeight;
  },

  async executeTerminalCommand(cmd) {
    const terminalContent = document.getElementById('terminalContent');
    if (!terminalContent) return;

    // Show the command
    const cmdLine = document.createElement('div');
    cmdLine.className = 'terminal-line';
    cmdLine.innerHTML = `<span class="terminal-prompt">$ </span><span class="terminal-cmd">${this.escapeHtml(cmd)}</span>`;
    terminalContent.appendChild(cmdLine);

    const tauri = window.__TAURI__;
    if (!tauri) {
      this.addTerminalOutput('Tauri 不可用 — 无法执行命令', 'error');
      this.appendTerminalPrompt();
      return;
    }

    const invoke = tauri.core?.invoke || tauri.invoke;

    // Simple command parsing: first word = cmd, rest = args
    const parts = cmd.split(/\s+/);
    const command = parts[0];
    const args = parts.slice(1);

    try {
      const result = await invoke('run_command', { cmd: command, args });
      if (result) {
        this.addTerminalOutput(result, 'output');
        // Also send to output panel for build/test commands
        if (['cargo', 'npm', 'pnpm', 'yarn', 'make', 'cmake'].includes(command)) {
          this.addOutput(result, command === 'cargo' && args[0] === 'build' ? 'build' : 'info');
        }
      }
    } catch (e) {
      this.addTerminalOutput((e.message || e).toString(), 'error');
      this.addOutput((e.message || e).toString(), 'error');
    }

    this.appendTerminalPrompt();
  },

  escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  },

  // ============================================================
  // Problems Panel
  // ============================================================
  async loadProblems() {
    const problemsContent = document.getElementById('problemsContent');
    if (!problemsContent) return;
    problemsContent.innerHTML = '<div class="problems-empty">扫描中...</div>';

    const tauri = window.__TAURI__;
    if (!tauri) {
      problemsContent.innerHTML = '<div class="problems-empty">Tauri 不可用</div>';
      return;
    }
    const invoke = tauri.core?.invoke || tauri.invoke;

    try {
      // Try cargo check first, fallback to run_command
      let output = '';
      try {
        const result = await invoke('run_command', { cmd: 'cargo', args: ['check'] });
        output = result.stdout || result;
      } catch (e) {
        // If cargo check fails, try a simpler approach
        output = (e.message || e).toString();
      }
      this.renderProblems(output);
    } catch (e) {
      problemsContent.innerHTML = `<div class="problems-empty">扫描失败: ${this.escapeHtml(e.message || e)}</div>`;
    }
  },

  renderProblems(output) {
    const problemsContent = document.getElementById('problemsContent');
    if (!problemsContent) return;

    const problems = [];
    const lines = output.split('\n');

    // Parse cargo check / rustc style errors and warnings
    // Format: error[EXXXX]: message
    //        --> file:line:col
    //        |
    //        | code
    //        |
    //        = help: message
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      // Match error/warning header
      const match = line.match(/^(error|warning)(?:\[\w+\])?:\s*(.+)$/);
      if (match) {
        const level = match[1];
        const message = match[2];
        // Next line usually has location
        let file = '', lineNo = '', col = '';
        if (i + 1 < lines.length) {
          const locMatch = lines[i + 1].match(/-->\s+(.+?):(\d+):?(\d+)?/);
          if (locMatch) {
            file = locMatch[1];
            lineNo = locMatch[2];
            col = locMatch[3] || '';
          }
        }
        problems.push({ level, message, file, line: lineNo, col });
      }
    }

    if (!problems.length) {
      problemsContent.innerHTML = '<div class="problems-empty">工作区中未检测到问题。</div>';
      return;
    }

    let html = '';
    problems.forEach(p => {
      const icon = p.level === 'error' ? '●' : '◐';
      const cls = p.level === 'error' ? 'problem-error' : 'problem-warning';
      html += `<div class="problem-item ${cls}" data-file="${this.escapeHtml(p.file)}" data-line="${p.line}">
        <span class="problem-icon">${icon}</span>
        <div class="problem-info">
          <div class="problem-message">${this.escapeHtml(p.message)}</div>
          <div class="problem-location">${this.escapeHtml(p.file)}${p.line ? ':' + p.line : ''}</div>
        </div>
      </div>`;
    });
    problemsContent.innerHTML = html;

    problemsContent.querySelectorAll('.problem-item').forEach(el => {
      el.addEventListener('click', () => {
        const file = el.dataset.file;
        const line = el.dataset.line;
        if (file) this.openFile(file);
        // TODO: scroll to line when editor supports it
      });
    });
  },

  // ============================================================
  // Output Panel
  // ============================================================
  setupOutputPanel() {
    const panelActions = document.querySelector('.panel-actions');
    if (panelActions) {
      const clearBtn = document.createElement('button');
      clearBtn.className = 'panel-action-btn';
      clearBtn.title = '清空输出';
      clearBtn.innerHTML = '🗑';
      clearBtn.addEventListener('click', () => this.clearOutput());
      panelActions.insertBefore(clearBtn, panelActions.firstChild);
    }
  },

  setupAgentTrace() {
    const clearBtn = document.getElementById('clearTraceBtn');
    const pauseBtn = document.getElementById('pauseTraceBtn');
    if (clearBtn) clearBtn.addEventListener('click', () => this.clearTraceCards());
    if (pauseBtn) pauseBtn.addEventListener('click', () => this.toggleTracePause(pauseBtn));
    this.startTraceSubscription();
  },

  startTraceSubscription() {
    const tauri = window.__TAURI__;
    if (!tauri || !tauri.core || !tauri.core.Channel) {
      this.renderDemoTraceCards();
      return;
    }
    const invoke = tauri.core.invoke;
    try {
      const Channel = tauri.core.Channel;
      const channel = new Channel();
      channel.onmessage = (event) => {
        if (!this.tracePaused) {
          this.traceEvents.push(event);
          if (this.traceEvents.length > 100) this.traceEvents.shift();
          if (this.sidebarView === 'agent-trace') this.renderTraceCards();
          if (event.step_type === 'EditProposed') this.onEditProposed(event);
          if (event.thinking_content) {
            const activeThinking = document.querySelector('.chat-message.ai .thinking-block');
            if (activeThinking) {
              activeThinking.style.display = 'block';
              const md = activeThinking.querySelector('.thinking-block-markdown');
              if (md) md.innerHTML = this.renderMarkdown(event.thinking_content);
            }
          }
          if (event.operation_summary) {
            this.updateOperationSummary(event.operation_summary, event.tool_name);
          }
          if (event.step_type === 'Act') {
            const lower = (event.tool_name || '').toLowerCase();
            let status = '执行中...';
            if (lower.includes('edit')) status = '编辑中...';
            else if (lower.includes('delete')) status = '删除中...';
            else if (lower.includes('create')) status = '创建中...';
            this.updateOperationProgress(status);
          }
        }
      };
      invoke('subscribe_agent_trace', { onEvent: channel }).catch(() => {});
    } catch (e) {
      this.renderDemoTraceCards();
    }
  },

  renderTraceCards() {
    const panel = document.getElementById('tracePanel');
    if (!panel) return;
    if (this.traceEvents.length === 0) {
      panel.innerHTML = '<div class="trace-empty" style="color:var(--fg-dim);text-align:center;padding:20px;">暂无思考过程</div>';
      return;
    }
    const colors = { Observe: 'var(--fg-green)', Retrieve: 'var(--fg-cyan)', Plan: 'var(--fg-red)', Act: 'var(--fg-dim)', Reflect: 'var(--fg-magenta)', Store: 'var(--fg-dim)', Decide: 'var(--fg-cyan)', Other: 'var(--fg-dim)' };
    panel.innerHTML = this.traceEvents.slice().reverse().map(ev => {
      const color = colors[ev.step_type] || colors.Other;
      const confidence = ev.confidence_score != null ? `<span style="color:var(--fg-red)">(${ev.confidence_score.toFixed(2)})</span>` : '';
      const plan = ev.plan_summary ? `<div style="margin-top:4px;color:var(--fg-dim);font-size:11px;white-space:pre-wrap;">${ev.plan_summary.substring(0, 200)}</div>` : '';
      return `<div class="trace-card" style="border-left:3px solid ${color};padding:6px 8px;margin-bottom:6px;background:var(--bg-hover);border-radius:4px;">
        <div style="display:flex;justify-content:space-between;align-items:center;">
          <span style="font-weight:bold;color:${color};font-size:11px;">${ev.step} ${confidence}</span>
          <span style="color:var(--fg-dim);font-size:10px;">#${ev.iteration}</span>
        </div>
        <div style="color:var(--fg-default);margin-top:2px;font-size:12px;">${ev.details}</div>
        ${plan}
      </div>`;
    }).join('');
  },

  renderDemoTraceCards() {
    this.traceEvents = [
      { step: 'Planning', details: 'Planning initial goal: 分析代码结构', iteration: 0, timestamp: new Date().toISOString(), step_type: 'Plan', plan_summary: null, reflection_key_points: [], confidence_score: 0.85 },
      { step: 'Observing', details: 'Observed 12 blackboard keys', iteration: 1, timestamp: new Date().toISOString(), step_type: 'Observe', plan_summary: null, reflection_key_points: [], confidence_score: null },
      { step: 'Retrieving', details: 'Retrieved 3 entries in 2 tiers (120 tokens)', iteration: 1, timestamp: new Date().toISOString(), step_type: 'Retrieve', plan_summary: null, reflection_key_points: [], confidence_score: null },
      { step: 'Acting', details: 'Task t1 completed: success=true', iteration: 1, timestamp: new Date().toISOString(), step_type: 'Act', plan_summary: '执行工具调用', reflection_key_points: [], confidence_score: 0.92 },
    ];
    this.renderTraceCards();
  },

  clearTraceCards() {
    this.traceEvents = [];
    this.renderTraceCards();
  },

  toggleTracePause(btn) {
    this.tracePaused = !this.tracePaused;
    btn.innerHTML = this.tracePaused
      ? '▶'
      : '⏸';
  },

  addOutput(text, type = 'info') {
    const outputContent = document.getElementById('outputContent');
    if (!outputContent) return;
    const lines = text.split('\n');
    lines.forEach(line => {
      let lineType = type;
      if (line.includes('[ERROR]') || line.includes('error:') || line.includes('error[')) lineType = 'error';
      else if (line.includes('[WARN]') || line.includes('warning:')) lineType = 'warn';
      else if (line.includes('[成功]') || line.includes('Finished') || line.includes('Completed')) lineType = 'success';
      const div = document.createElement('div');
      div.className = `output-line ${lineType}`;
      div.textContent = line;
      outputContent.appendChild(div);
    });
    outputContent.scrollTop = outputContent.scrollHeight;
  },

  clearOutput() {
    const outputContent = document.getElementById('outputContent');
    if (outputContent) outputContent.innerHTML = '';
  },

  // ============================================================
  // Resizable Panels
  // ============================================================
  setupResizers() {
    const sidebar = document.querySelector('.sidebar');
    if (!sidebar) return;
    sidebar.addEventListener('mousemove', (e) => {
      const rect = sidebar.getBoundingClientRect();
      const nearRightEdge = rect.right - e.clientX <= 4;
      sidebar.style.cursor = nearRightEdge ? 'col-resize' : '';
    });
    sidebar.addEventListener('mouseleave', () => {
      sidebar.style.cursor = '';
    });
    sidebar.addEventListener('mousedown', (e) => {
      const rect = sidebar.getBoundingClientRect();
      if (rect.right - e.clientX <= 4) {
        e.preventDefault();
        this.startSidebarResize(e);
      }
    });
  },

  startSidebarResize(e) {
    const startX = e.clientX;
    const sidebar = document.querySelector('.sidebar');
    const startWidth = sidebar.getBoundingClientRect().width;

    const onMove = (ev) => {
      const delta = ev.clientX - startX;
      const newWidth = Math.max(180, Math.min(500, startWidth + delta));
      document.documentElement.style.setProperty('--sidebar-width', newWidth + 'px');
    };

    const onUp = () => {
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
      this.saveLayoutSizes();
    };

    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  },

  startPanelResize(e) {
    const startY = e.clientY;
    const bottomPanel = document.getElementById('bottomPanel');
    const startHeight = bottomPanel.getBoundingClientRect().height;
    const mainArea = document.querySelector('.main-area');
    const mainHeight = mainArea.getBoundingClientRect().height;

    const onMove = (ev) => {
      const delta = startY - ev.clientY;
      const newHeight = Math.max(120, Math.min(mainHeight * 0.7, startHeight + delta));
      bottomPanel.style.height = newHeight + 'px';
      bottomPanel.style.flex = 'none';
    };

    const onUp = () => {
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
      this.saveLayoutSizes();
    };

    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  },

  saveLayoutSizes() {
    const sidebarWidth = getComputedStyle(document.documentElement).getPropertyValue('--sidebar-width').trim();
    try {
      localStorage.setItem('hajimi.layout', JSON.stringify({ sidebarWidth }));
    } catch (e) {
      console.error('saveLayoutSizes error:', e);
    }
  },

  loadLayoutSizes() {
    try {
      const raw = localStorage.getItem('hajimi.layout');
      if (raw) {
        const saved = JSON.parse(raw);
        if (saved.sidebarWidth) {
          document.documentElement.style.setProperty('--sidebar-width', saved.sidebarWidth);
        }
        // Panel height removed in chat-first layout
      }
    } catch (e) {
      console.error('loadLayoutSizes error:', e);
    }
  },

  // ============================================================
  // Settings Persistence
  // ============================================================
  loadSettings() {
    try {
      const raw = localStorage.getItem('hajimi.settings');
      if (raw) {
        const saved = JSON.parse(raw);
        this.settings = { ...this.settings, ...saved };
      }
    } catch (e) {
      console.error('loadSettings error:', e);
    }
    this.applySettings();
    this.bindSettingsEvents();
  },

  saveSettings() {
    try {
      localStorage.setItem('hajimi.settings', JSON.stringify(this.settings));
    } catch (e) {
      console.error('saveSettings error:', e);
    }
  },

  applySettings() {
    const s = this.settings;

    // Theme
    const themeSelect = document.getElementById('settingTheme');
    if (themeSelect) themeSelect.value = s.theme;
    this.applyTheme(s.theme);

    // Font size
    const fontSizeInput = document.getElementById('settingFontSize');
    if (fontSizeInput) fontSizeInput.value = s.fontSize;
    document.documentElement.style.setProperty('--editor-font-size', s.fontSize + 'px');

    // Word wrap
    const wordWrapInput = document.getElementById('settingWordWrap');
    if (wordWrapInput) wordWrapInput.checked = s.wordWrap;

    // Auto save
    const autoSaveSelect = document.getElementById('settingAutoSave');
    if (autoSaveSelect) autoSaveSelect.value = s.autoSave;
  },

  applyTheme(theme) {
    const root = document.documentElement;
    let effectiveTheme = theme;
    if (theme === 'system') {
      effectiveTheme = window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
    } else if (theme === 'dark+' || theme === 'high-contrast') {
      effectiveTheme = 'dark';
    }
    root.setAttribute('data-theme', effectiveTheme);
  },

  setupSystemThemeListener() {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: light)');
    mediaQuery.addEventListener('change', () => {
      if (this.settings.theme === 'system') {
        this.applyTheme('system');
      }
    });
  },

  bindSettingsEvents() {
    const themeSelect = document.getElementById('settingTheme');
    const fontSizeInput = document.getElementById('settingFontSize');
    const wordWrapInput = document.getElementById('settingWordWrap');
    const autoSaveSelect = document.getElementById('settingAutoSave');

    if (themeSelect) {
      themeSelect.addEventListener('change', () => {
        this.settings.theme = themeSelect.value;
        this.applyTheme(this.settings.theme);
        this.saveSettings();
      });
    }

    if (fontSizeInput) {
      fontSizeInput.addEventListener('change', () => {
        const val = parseInt(fontSizeInput.value);
        if (val >= 8 && val <= 32) {
          this.settings.fontSize = val;
          document.documentElement.style.setProperty('--editor-font-size', val + 'px');
          this.saveSettings();
        }
      });
    }

    if (wordWrapInput) {
      wordWrapInput.addEventListener('change', () => {
        this.settings.wordWrap = wordWrapInput.checked;
        this.saveSettings();
      });
    }

    if (autoSaveSelect) {
      autoSaveSelect.addEventListener('change', () => {
        this.settings.autoSave = autoSaveSelect.value;
        this.saveSettings();
      });
    }
  },

  // ============================================================
  // AI Chat Context Files
  // ============================================================
  addChatContextFile(path) {
    if (this.chatContextFiles.includes(path)) return;
    this.chatContextFiles.push(path);
    this.renderChatContext();
  },

  removeChatContextFile(path) {
    this.chatContextFiles = this.chatContextFiles.filter(p => p !== path);
    this.renderChatContext();
  },

  clearChatContext() {
    this.chatContextFiles = [];
    this.chatMessages = [];
    this.tokenStats = { promptTokens: 0, completionTokens: 0, estimatedTokens: 0 };
    this.cumulativeStats = { promptTokens: 0, completionTokens: 0, requestCount: 0 };
    this.renderChatContext();
    const chatMsgContainer = document.getElementById('aiChatMessages');
    if (chatMsgContainer) chatMsgContainer.innerHTML = '';
    this.updateTokenDisplay();
  },

  renderChatContext() {
    const container = document.getElementById('aiChatContext');
    const list = document.getElementById('aiChatContextList');
    if (!container || !list) return;

    if (!this.chatContextFiles.length) {
      container.style.display = 'none';
      return;
    }

    container.style.display = 'block';
    list.innerHTML = this.chatContextFiles.map(path => {
      const name = path.split('/').pop();
      return `<div class="ai-context-file">
        <span class="ai-context-file-name">${this.escapeHtml(name)}</span>
        <button class="ai-context-file-remove" data-path="${this.escapeHtml(path)}">×</button>
      </div>`;
    }).join('');

    list.querySelectorAll('.ai-context-file-remove').forEach(btn => {
      btn.addEventListener('click', () => {
        this.removeChatContextFile(btn.dataset.path);
      });
    });
  },

  estimateTokens(text) {
    if (!text) return 0;
    const chineseChars = (text.match(/[\u4e00-\u9fff]/g) || []).length;
    const englishWords = (text.match(/[a-zA-Z]+/g) || []).length;
    return Math.ceil(chineseChars + englishWords * 1.3);
  },

  async loadCumulativeFromBackend() {
    try {
      const tauri = window.__TAURI__;
      const invoke = tauri ? (tauri.core?.invoke || tauri.invoke) : null;
      if (!invoke) {
        this.loadCumulativeFromLocalStorage();
        return;
      }
      const stats = await invoke('get_cumulative_stats');
      if (stats && stats.total) {
        this.cumulativeStats = {
          promptTokens: stats.total.prompt_tokens || 0,
          completionTokens: stats.total.completion_tokens || 0,
          requestCount: stats.total.request_count || 0
        };
        console.log('[Token] Loaded cumulative stats from backend:', this.cumulativeStats);
      }
    } catch (e) {
      console.warn('[Token] Failed to load from backend, fallback to localStorage:', e);
      this.loadCumulativeFromLocalStorage();
    }
    this.updateTokenDisplay();
  },

  loadCumulativeFromLocalStorage() {
    const saved = localStorage.getItem('hajimi_cumulative_stats');
    if (saved) {
      try {
        this.cumulativeStats = JSON.parse(saved);
        console.log('[Token] Loaded cumulative stats from localStorage:', this.cumulativeStats);
      } catch (e) {
        console.warn('[Token] Failed to parse localStorage cumulative stats:', e);
      }
    }
  },

  saveCumulativeToLocalStorage() {
    try {
      localStorage.setItem('hajimi_cumulative_stats', JSON.stringify(this.cumulativeStats));
      console.log('[Token] Saved cumulative stats to localStorage');
    } catch (e) {
      console.warn('[Token] Failed to save cumulative stats:', e);
    }
  },

  updateTokenDisplay() {
    const hintEl = document.querySelector('.composer-hint');
    if (hintEl) {
      hintEl.textContent = 'Enter 发送 · Shift+Enter 换行 · @ 引用文件';
    }
    const statusEl = document.getElementById('statusTokens');
    if (!statusEl) return;
    if (this.chatMessages.length === 0) {
      statusEl.textContent = '';
      return;
    }
    const cfg = this.getActiveProviderConfig();
    const threshold = cfg?.contextThreshold || 6400;
    const estimated = this.chatMessages.reduce((sum, msg) => sum + this.estimateTokens(msg.content), 0);
    let promptTokens = this.tokenStats.promptTokens;
    let completionTokens = this.tokenStats.completionTokens;
    let isPrecise = promptTokens > 0 || completionTokens > 0;
    if (!isPrecise) {
      promptTokens = Math.floor(estimated * 0.35);
      completionTokens = Math.ceil(estimated * 0.65);
      this.tokenStats.estimatedTokens = estimated;
    }
    const totalTokens = promptTokens + completionTokens;
    const percentage = Math.min((totalTokens / threshold) * 100, 99.9).toFixed(1);
    const prefix = isPrecise ? '' : '~';
    let text = `🔄 ${prefix}${percentage}% | ↑ ${prefix}${promptTokens} | ↓ ${prefix}${completionTokens}`;
    if (this.showCumulative && this.cumulativeStats.requestCount > 0) {
      const c = this.cumulativeStats;
      text += ` | 累计: ↑ ${c.promptTokens} ↓ ${c.completionTokens} (${c.requestCount}轮)`;
    }
    statusEl.textContent = text;
  },

  getActiveProviderConfig() {
    const cfg = this.providerConfigs.find(c => c.id === this.activeProviderId);
    if (!cfg) return null;
    return {
      id: cfg.id,
      name: cfg.name,
      providerType: cfg.providerType || 'openai-compatible',
      baseUrl: cfg.baseUrl,
      apiKey: cfg.apiKey,
      model: cfg.model,
      contextThreshold: cfg.contextThreshold || 6400,
    };
  },

  checkAutoCompact() {
    if (!this.autoCompact || this.isAutoCompacting || this.chatMessages.length <= 2) return;
    const totalTokens = this.chatMessages.reduce((sum, msg) => sum + this.estimateTokens(msg.content), 0);
    const cfg = this.getActiveProviderConfig();
    const threshold = cfg?.contextThreshold || 6400;
    if (totalTokens > threshold * 0.8) {
      this.autoCompactContext();
    }
  },

  async autoCompactContext() {
    if (this.isAutoCompacting) return;
    this.isAutoCompacting = true;
    const tauri = window.__TAURI__;
    const invoke = tauri ? (tauri.core?.invoke || tauri.invoke) : null;
    if (!invoke) { this.isAutoCompacting = false; return; }
    const provider = this.activeProviderId;
    const config = this.getActiveProviderConfig();
    const thinkingId = this.addThinking();
    try {
      const summary = await invoke('optimize_context', { messages: this.chatMessages, provider, config });
      this.removeThinking(thinkingId);
      const kept = this.chatMessages.slice(-2);
      this.chatMessages = [
        { role: 'system', content: `[上下文已压缩] ${summary}`, timestamp: Date.now() },
        ...kept
      ];
      this.addChatMessage('ai', `**上下文自动压缩**\n\n摘要：${summary}`);
      this.updateTokenDisplay();
    } catch (e) {
      this.removeThinking(thinkingId);
      console.error('auto compact error:', e);
    } finally {
      this.isAutoCompacting = false;
    }
  },

  async buildContextPrompt() {
    if (!this.chatContextFiles.length) return '';
    const tauri = window.__TAURI__;
    if (!tauri) return '';
    const invoke = tauri.core?.invoke || tauri.invoke;

    const parts = [];
    for (const path of this.chatContextFiles) {
      try {
        const content = await invoke('read_file', { path });
        parts.push(`--- 文件: ${path} ---\n${content}`);
      } catch (e) {
        parts.push(`--- 文件: ${path} ---\n[读取失败: ${e.message || e}]`);
      }
    }
    return parts.join('\n\n') + '\n\n';
  },

  // ============================================================
  // Chat
  // ============================================================
  setupChat() {
    const chatInput = document.getElementById('aiChatInput');
    const chatSendBtn = document.getElementById('aiChatSendBtn');

    chatInput.addEventListener('input', () => {
      chatInput.style.height = 'auto';
      chatInput.style.height = Math.min(chatInput.scrollHeight, 150) + 'px';
    });

    chatInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        this.sendChatMessage();
      }
    });

    chatSendBtn.addEventListener('click', () => this.sendChatMessage());

    const modelSelectBtn = document.getElementById('modelSelectBtn');
    if (modelSelectBtn) {
      modelSelectBtn.addEventListener('click', () => this.openModelPicker());
    }

    document.getElementById('addContextBtn').addEventListener('click', () => {
      this.showSidebar('explorer');
      this.addChatMessage('ai', '**上下文文件：** 在资源管理器中右键点击文件，选择"添加到 AI 上下文"。');
    });

    const clearContextBtn = document.getElementById('clearContextBtn');
    if (clearContextBtn) {
      clearContextBtn.addEventListener('click', () => this.clearChatContext());
    }

    document.getElementById('editModeBtn').addEventListener('click', () => {
      this.addChatMessage('ai', '**编辑模式已激活。** 当您要求修改文件时，我将以 diff 格式建议代码更改。');
    });

    document.getElementById('newChatBtn').addEventListener('click', () => {
      this.newChatSession();
    });

    const newSessionBtn = document.getElementById('newSessionBtn');
    if (newSessionBtn) {
      newSessionBtn.addEventListener('click', () => this.newChatSession());
    }

    this.updateTokenDisplay();
  },

  async sendChatMessage() {
    const chatInput = document.getElementById('aiChatInput');
    const chatSendBtn = document.getElementById('aiChatSendBtn');
    const modelSelect = document.getElementById('aiChatModelSelect');
    let text = chatInput.value.trim();
    if (!text || this.isProcessing) return;

    // Prepend context files if any
    if (this.chatContextFiles.length && !text.startsWith('/')) {
      const contextPrompt = await this.buildContextPrompt();
      if (contextPrompt) {
        text = contextPrompt + text;
      }
    }

    const userContent = chatInput.value.trim();
    this.chatMessages.push({ role: 'user', content: userContent, timestamp: Date.now() });
    this.addChatMessage('user', userContent);
    this.tokenStats = { promptTokens: 0, completionTokens: 0, estimatedTokens: 0 };
    this.updateTokenDisplay();
    chatInput.value = '';
    chatInput.style.height = 'auto';
    this.isProcessing = true;
    chatSendBtn.disabled = true;
    this.showStatusIndicator('working');

    // Handle slash commands
    if (text.startsWith('/')) {
      try {
        await this.handleChatCommand(text);
      } catch (err) {
        this.addChatMessage('ai', `**错误：** ${err.message || err}`);
      } finally {
        this.isProcessing = false;
        chatSendBtn.disabled = false;
        this.hideStatusIndicator();
        this.saveChatSessions();
        this.renderSessionList();
      }
      return;
    }

    const thinkingId = this.addThinking();

    // Check if a provider is selected
    if (!this.activeProviderId) {
      this.removeThinking(thinkingId);
      this.addChatMessage('ai', '**未选择模型。** 请点击右上角「选择模型」按钮配置并选择一个模型。');
      this.isProcessing = false;
      chatSendBtn.disabled = false;
      this.hideStatusIndicator();
      return;
    }

    // Try real backend first
    const tauri = window.__TAURI__;
    if (tauri) {
      const cfg = this.providerConfigs.find(c => c.id === this.activeProviderId);
      const provider = this.activeProviderId;
      const config = cfg ? {
        id: cfg.id,
        name: cfg.name,
        providerType: cfg.providerType || 'openai-compatible',
        baseUrl: cfg.baseUrl,
        apiKey: cfg.apiKey,
        model: cfg.model,
        contextThreshold: cfg.contextThreshold || 6400,
      } : null;

      try {
        const response = await this.streamChat(provider, text, config, this.chatMessages);
        this.removeThinking(thinkingId);
        this.chatMessages.push({ role: 'assistant', content: response, timestamp: Date.now() });
      } catch (err) {
        console.error('stream_chat error:', err);
        this.removeThinking(thinkingId);
        this.addChatMessage('ai', `**错误：** ${err.message || err}\n\n已回退到本地回复。`);
        const demoResponse = this.generateDemoResponse(text);
        this.addChatMessage('ai', demoResponse);
        this.chatMessages.push({ role: 'assistant', content: demoResponse, timestamp: Date.now() });
      } finally {
        this.isProcessing = false;
        chatSendBtn.disabled = false;
        this.hideStatusIndicator();
        this.updateTokenDisplay();
        this.saveCumulativeToLocalStorage();
        this.checkAutoCompact();
        this.saveChatSessions();
        this.renderSessionList();
      }
    } else {
      // Fallback to local demo
      setTimeout(() => {
        this.removeThinking(thinkingId);
        const demoResponse = this.generateDemoResponse(text);
        this.addChatMessage('ai', demoResponse);
        this.chatMessages.push({ role: 'assistant', content: demoResponse, timestamp: Date.now() });
        this.isProcessing = false;
        chatSendBtn.disabled = false;
        this.hideStatusIndicator();
        this.updateTokenDisplay();
        this.saveCumulativeToLocalStorage();
        this.checkAutoCompact();
        this.saveChatSessions();
        this.renderSessionList();
      }, 1200);
    }
  },

  async handleChatCommand(text) {
    const tauri = window.__TAURI__;
    const invoke = tauri ? (tauri.core?.invoke || tauri.invoke) : null;

    if (text === '/tools') {
      if (!invoke) { this.addChatMessage('ai', 'Tauri 不可用'); return; }
      try {
        const tools = await invoke('list_tools');
        const list = tools.map(t => `- \`${t.name}\` — ${t.description || '无描述'}`).join('\n');
        this.addChatMessage('ai', `**可用工具 (${tools.length}个)：**\n\n${list}`);
      } catch (e) {
        this.addChatMessage('ai', `获取工具列表失败: ${e.message || e}`);
      }
      return;
    }

    if (text === '/providers') {
      const fixed = ['ollama', 'anthropic', 'openai'];
      const custom = this.providerConfigs.map(c => c.name);
      const all = [...fixed, ...custom];
      this.addChatMessage('ai', `**可用模型提供商：**\n\n${all.map(p => `- ${p}`).join('\n')}`);
      return;
    }

    if (text.startsWith('/tool ')) {
      if (!invoke) { this.addChatMessage('ai', 'Tauri 不可用'); return; }
      // Parse: /tool <name> <json_args>
      const rest = text.slice(6).trim();
      const spaceIdx = rest.indexOf(' ');
      const toolName = spaceIdx > 0 ? rest.slice(0, spaceIdx) : rest;
      const argsStr = spaceIdx > 0 ? rest.slice(spaceIdx + 1) : '{}';
      let args = {};
      try {
        args = JSON.parse(argsStr);
      } catch (e) {
        this.addChatMessage('ai', `参数 JSON 解析失败: ${e.message}\n\n用法: \`/tool <名称> {\"key\":\"value\"}\``);
        return;
      }
      try {
        const result = await invoke('execute_tool', { name: toolName, args });
        const output = result.stdout || result.result || JSON.stringify(result, null, 2);
        this.addChatMessage('ai', `**工具 \`${toolName}\` 执行结果：**\n\n\`\`\`\n${output}\n\`\`\``);
      } catch (e) {
        this.addChatMessage('ai', `工具执行失败: ${e.message || e}`);
      }
      return;
    }

    if (text.startsWith('/chat ')) {
      if (!invoke) { this.addChatMessage('ai', 'Tauri 不可用'); return; }
      // Parse: /chat <provider> <prompt>
      const rest = text.slice(6).trim();
      const spaceIdx = rest.indexOf(' ');
      if (spaceIdx <= 0) {
        this.addChatMessage('ai', '用法: `/chat <提供商> <提示词>`');
        return;
      }
      const provider = rest.slice(0, spaceIdx);
      const prompt = rest.slice(spaceIdx + 1);

      let config = null;
      const cfg = this.providerConfigs.find(c => c.id === provider || c.name === provider);
      if (cfg) {
        config = {
          id: cfg.id,
          name: cfg.name,
          providerType: cfg.providerType || 'openai-compatible',
          baseUrl: cfg.baseUrl,
          apiKey: cfg.apiKey,
          model: cfg.model,
        };
      }

      const thinkingId = this.addThinking();
      try {
        await this.streamChat(provider, prompt, config);
        this.removeThinking(thinkingId);
      } catch (e) {
        this.removeThinking(thinkingId);
        this.addChatMessage('ai', `**错误：** ${e.message || e}`);
      }
      return;
    }

    if (text.startsWith('/mcp ')) {
      const rest = text.slice(5).trim();
      const parts = rest.split(/\s+/);
      const subCmd = parts[0];

      if (subCmd === 'list') {
        if (!this.mcpServers.length) {
          this.addChatMessage('ai', '暂无已连接的 MCP 服务器。');
          return;
        }
        let msg = '**已连接的 MCP 服务器：**\n\n';
        this.mcpServers.forEach(s => {
          msg += `**${this.escapeHtml(s.url)}** (${s.transport})\n`;
          (s.tools || []).forEach(t => { msg += `- \`${this.escapeHtml(t)}\`\n`; });
          msg += '\n';
        });
        this.addChatMessage('ai', msg);
        return;
      }

      if (subCmd === 'init') {
        const serverUrl = parts.slice(1).join(' ');
        if (!serverUrl) {
          this.addChatMessage('ai', '用法: `/mcp init <server_url>`');
          return;
        }
        try {
          const result = await this.mcpInit(serverUrl, 'stdio');
          this.mcpServers.push({ url: serverUrl, transport: 'stdio', tools: result.tool_names || [] });
          this.saveMcpServers();
          this.renderMcpServers();
          this.addChatMessage('ai', `**MCP 连接成功**\n- 服务器: ${this.escapeHtml(serverUrl)}\n- 工具数: ${result.tools || 0}\n- 工具列表: ${(result.tool_names || []).map(t => `\`${t}\``).join(', ')}`);
        } catch (e) {
          this.addChatMessage('ai', `MCP 连接失败: ${e.message || e}`);
        }
        return;
      }

      if (subCmd === 'invoke') {
        if (parts.length < 3) {
          this.addChatMessage('ai', '用法: `/mcp invoke <server_url> <tool_name> [json_args]`');
          return;
        }
        const serverUrl = parts[1];
        const toolName = parts[2];
        let args = {};
        if (parts.length > 3) {
          try { args = JSON.parse(parts.slice(3).join(' ')); }
          catch (e) { this.addChatMessage('ai', `参数 JSON 解析失败: ${e.message}`); return; }
        }
        try {
          const result = await this.mcpInvoke(serverUrl, toolName, args);
          this.addChatMessage('ai', `**MCP 工具 \`${toolName}\` 执行结果：**\n\n\`\`\`json\n${JSON.stringify(result, null, 2)}\n\`\`\``);
        } catch (e) {
          this.addChatMessage('ai', `MCP 调用失败: ${e.message || e}`);
        }
        return;
      }

      this.addChatMessage('ai', '未知 MCP 命令。用法: `/mcp list`, `/mcp init <url>`, `/mcp invoke <url> <tool> [args]`');
      return;
    }

    if (text === '/search' || text.startsWith('/search ')) {
      if (!invoke) { this.addChatMessage('ai', 'Tauri 不可用'); return; }
      const pattern = text === '/search' ? '' : text.slice(8).trim();
      if (!pattern) { this.addChatMessage('ai', '用法: `/search <pattern>`'); return; }
      try {
        const result = await invoke('execute_tool', {
          name: 'grep',
          args: { pattern, path: '.', recursive: true, caseSensitive: false, regex: false, wholeWord: false }
        });
        const output = result.stdout || result.result || '';
        if (!output.trim()) {
          this.addChatMessage('ai', `**搜索 \`${this.escapeHtml(pattern)}\`**：未找到匹配`);
          return;
        }
        const lines = output.trim().split('\n').slice(0, 30);
        let msg = `**搜索 \`${this.escapeHtml(pattern)}\`**：\n\n`;
        lines.forEach(line => {
          const m = line.match(/^(.+?):(\d+):(?:(\d+):)?(.+)$/);
          if (m) msg += `\`${this.escapeHtml(m[1])}:${m[2]}\` — ${this.escapeHtml(m[4] || '')}\n`;
          else msg += `${this.escapeHtml(line)}\n`;
        });
        if (output.trim().split('\n').length > 30) msg += '\n... (仅显示前 30 条)';
        this.addChatMessage('ai', msg);
      } catch (e) {
        this.addChatMessage('ai', `搜索失败: ${e.message || e}`);
      }
      return;
    }

    if (text === '/git' || text.startsWith('/git ')) {
      const rest = text === '/git' ? '' : text.slice(5).trim();
      const parts = rest.split(/\s+/);
      const subCmd = parts[0] || '';

      if (subCmd === 'status' || subCmd === '') {
        try {
          const result = await invoke('execute_tool', { name: 'git_status', args: {} });
          const output = result.stdout || result.result || '无更改';
          this.addChatMessage('ai', `**Git 状态：**\n\n\`\`\`\n${output}\n\`\`\``);
        } catch (e) {
          this.addChatMessage('ai', `Git 状态获取失败: ${e.message || e}`);
        }
        return;
      }

      if (subCmd === 'diff') {
        const file = parts.slice(1).join(' ');
        try {
          const result = await invoke('execute_tool', { name: 'git_diff', args: file ? { file } : {} });
          const output = result.stdout || result.result || '无 diff';
          this.addChatMessage('ai', `**Git Diff${file ? ' — ' + this.escapeHtml(file) : ''}：**\n\n\`\`\`diff\n${output}\n\`\`\``);
        } catch (e) {
          this.addChatMessage('ai', `Git diff 获取失败: ${e.message || e}`);
        }
        return;
      }

      if (subCmd === 'commit') {
        const message = parts.slice(1).join(' ');
        if (!message) { this.addChatMessage('ai', '用法: `/git commit <message>`'); return; }
        try {
          await invoke('execute_tool', { name: 'git_commit', args: { message } });
          this.addChatMessage('ai', `✅ 已提交: ${this.escapeHtml(message)}`);
        } catch (e) {
          this.addChatMessage('ai', `提交失败: ${e.message || e}`);
        }
        return;
      }

      this.addChatMessage('ai', '未知 Git 命令。用法: `/git status`, `/git diff [file]`, `/git commit <message>`');
      return;
    }

    if (text === '/extensions') {
      if (!this.extensions.length && !this.installedExtensions.length) {
        this.addChatMessage('ai', '**扩展列表：**\n\n暂无扩展。');
        return;
      }
      let msg = '**可用扩展：**\n\n';
      this.extensions.forEach(ext => {
        const installed = this.installedExtensions.includes(ext.id);
        msg += `- **${this.escapeHtml(ext.name)}** v${ext.version} — ${this.escapeHtml(ext.desc)} ${installed ? '✅ 已安装' : ''}\n`;
      });
      this.addChatMessage('ai', msg);
      return;
    }

    if (text === '/compact') {
      if (!invoke) { this.addChatMessage('ai', 'Tauri 不可用'); return; }
      if (this.chatMessages.length <= 2) {
        this.addChatMessage('ai', '对话轮次不足，无需压缩。');
        return;
      }
      const provider = this.activeProviderId;
      const config = this.getActiveProviderConfig();
      const thinkingId = this.addThinking();
      try {
        const summary = await invoke('optimize_context', { messages: this.chatMessages, provider, config });
        this.removeThinking(thinkingId);
        const kept = this.chatMessages.slice(-2);
        this.chatMessages = [
          { role: 'system', content: `[上下文已压缩] ${summary}`, timestamp: Date.now() },
          ...kept
        ];
        this.addChatMessage('ai', `**上下文已压缩**\n\n摘要：${summary}`);
        this.updateTokenDisplay();
      } catch (e) {
        this.removeThinking(thinkingId);
        this.addChatMessage('ai', `压缩失败: ${e.message || e}`);
      }
      return;
    }

    // Unknown command
    this.addChatMessage('ai', `未知命令: \`${text.split(' ')[0]}\`\n\n可用命令: \`/tools\`, \`/providers\`, \`/tool <name> <args>\`, \`/chat <provider> <prompt>\`, \`/mcp <list|init|invoke>\`, \`/search <pattern>\`, \`/git <status|diff|commit>\`, \`/extensions\`, \`/compact\``);
  },

  /// Parse thinking tags from accumulated stream buffer (B-09/12).
  /// Returns { thinking, response, state } where state is 'idle'|'thinking'|'response'.
  parseThinkingStream(buffer) {
    const thinkOpen = '<thinking>';
    const thinkClose = '</thinking>';
    const respOpen = '<response>';
    const respClose = '</response>';
    const tStart = buffer.indexOf(thinkOpen);
    if (tStart === -1) {
      return { thinking: null, response: buffer, state: 'idle' };
    }
    const tEnd = buffer.indexOf(thinkClose, tStart);
    if (tEnd === -1) {
      const thinking = buffer.slice(tStart + thinkOpen.length);
      return { thinking, response: null, state: 'thinking' };
    }
    const thinking = buffer.slice(tStart + thinkOpen.length, tEnd).trim();
    let response = '';
    const rStart = buffer.indexOf(respOpen, tEnd);
    if (rStart !== -1) {
      const rEnd = buffer.indexOf(respClose, rStart);
      response = rEnd !== -1
        ? buffer.slice(rStart + respOpen.length, rEnd)
        : buffer.slice(rStart + respOpen.length);
    } else {
      response = buffer.slice(tEnd + thinkClose.length);
    }
    return { thinking, response, state: 'response' };
  },

  /// Schedule DOM update via requestAnimationFrame for non-blocking rendering (B-09/12).
  scheduleDomUpdate(fn) {
    if (this._pendingRaf) cancelAnimationFrame(this._pendingRaf);
    this._pendingRaf = requestAnimationFrame(() => {
      this._pendingRaf = null;
      fn();
    });
  },

  async streamChat(provider, prompt, config, messages) {
    const tauri = window.__TAURI__;
    if (!tauri) throw new Error('Tauri not available');

    const invoke = tauri.core?.invoke || tauri.invoke;
    const Channel = tauri.core?.Channel;

    const msgContainer = document.getElementById('aiChatMessages');
    const msgDiv = document.createElement('div');
    msgDiv.className = 'chat-message ai';
    msgDiv.innerHTML = '<div class="chat-message-avatar">H</div><div class="chat-message-body"></div>';
    const body = msgDiv.querySelector('.chat-message-body');
    msgContainer.appendChild(msgDiv);
    msgContainer.scrollTop = msgContainer.scrollHeight;

    // Create thinking block and response container for streaming (B-09/12)
    const thinkingBlock = this.createThinkingBlock();
    thinkingBlock.style.display = 'none';
    body.appendChild(thinkingBlock);
    const responseDiv = document.createElement('div');
    responseDiv.className = 'chat-response-body';
    body.appendChild(responseDiv);

    if (!Channel) {
      // Fallback: simulate streaming with local response
      const demoText = this.generateDemoResponse(prompt);
      let fullText = '';
      const chars = demoText.split('');
      for (let i = 0; i < chars.length; i++) {
        fullText += chars[i];
        responseDiv.innerHTML = this.formatText(fullText);
        msgContainer.scrollTop = msgContainer.scrollHeight;
        await new Promise(r => setTimeout(r, 10));
      }
      return fullText;
    }

    const channel = new Channel();
    let buffer = '';

    channel.onmessage = (event) => {
      if (event.chunk) {
        buffer += event.chunk;
        this.scheduleDomUpdate(() => {
          const result = this.parseThinkingStream(buffer);
          if (result.state === 'idle') {
            responseDiv.innerHTML = this.formatText(result.response);
          } else if (result.state === 'thinking') {
            thinkingBlock.style.display = 'block';
            const md = thinkingBlock.querySelector('.thinking-block-markdown');
            if (md) md.innerHTML = this.renderMarkdown(result.thinking);
          } else if (result.state === 'response') {
            thinkingBlock.style.display = 'block';
            const md = thinkingBlock.querySelector('.thinking-block-markdown');
            if (md) md.innerHTML = this.renderMarkdown(result.thinking);
            responseDiv.innerHTML = this.formatText(result.response);
          }
          msgContainer.scrollTop = msgContainer.scrollHeight;
        });
      }
      if (event.error) {
        this.showErrorToast(event.error);
        responseDiv.innerHTML = this.formatText(`**错误：** ${event.error}`);
      }
      if (event.done) {
        // Capture precise token usage from backend
        if (event.promptTokens != null && event.completionTokens != null) {
          this.tokenStats = {
            promptTokens: event.promptTokens,
            completionTokens: event.completionTokens,
            estimatedTokens: 0
          };
          this.cumulativeStats.promptTokens += event.promptTokens;
          this.cumulativeStats.completionTokens += event.completionTokens;
          this.cumulativeStats.requestCount += 1;
        }
        this.updateTokenDisplay();
      }
    };

    await invoke('stream_chat', { provider, prompt, messages, config, onEvent: channel });

    return buffer;
  },

  generateDemoResponse(text) {
    const lower = text.toLowerCase();
    if (lower.includes('help') || lower.startsWith('/help')) {
      return [
        '**可用命令：**',
        '',
        '**文件系统：**',
        '- `read <路径>` — 读取文件内容',
        '- `write <路径> <内容>` — 写入文件',
        '- `ls <路径>` — 列出目录',
        '- `run <命令>` — 运行 shell 命令',
        '',
        '**工具系统：**',
        '- `/tools` — 列出所有已注册工具',
        '- `/tool <名称> <json参数>` — 执行工具',
        '',
        '**LLM：**',
        '- `/providers` — 显示可用 LLM 提供商',
        '- `/chat <提供商> <提示词>` — 流式对话',
        '',
        '或直接输入消息与 AI 对话。'
      ].join('\n');
    }
    if (lower.includes('hello') || lower.includes('hi')) {
      return '您好！我是 Hajimi，您的本地 AI 助手。我可以帮您读取文件、运行命令、搜索代码以及与 LLM 对话。您想做什么？';
    }
    if (lower.includes('rust') || lower.includes('cargo')) {
      return 'Hajimi 是用 Rust 构建的！工作区包含 22 个 crate，包括 `engine-llm-core`、`engine-tool-system` 和 `agent-core`。您可以在编辑器中打开任意 `.rs` 文件来浏览代码库。';
    }
    if (lower.startsWith('/tools')) {
      return '**可用工具 (38个)：**\n\n- `read_file` — 读取文件内容\n- `write_file` — 写入文件\n- `list_dir` — 列出目录内容\n- `run_command` — 执行 shell 命令\n- `search_code` — 搜索代码库\n- `git_status` — Git 状态\n- `git_diff` — Git 差异\n\n... 以及 31 个更多工具。使用 `/tool <名称> <参数>` 来执行。';
    }
    return `我收到了：**"${text}"**\n\n（这是本地回复 — 后端尚未连接。尝试询问 \`help\`、\`rust\`，或使用 \`/tools\`。）`;
  },

  addChatMessage(role, text) {
    const container = document.getElementById('aiChatMessages');
    const div = document.createElement('div');
    div.className = `chat-message ${role}`;
    const avatar = role === 'user' ? 'You' : 'H';
    div.innerHTML = `<div class="chat-message-avatar">${avatar}</div><div class="chat-message-body">${this.formatText(text)}</div>`;
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
  },

  addThinking() {
    const id = 't-' + Date.now();
    const container = document.getElementById('aiChatMessages');
    const div = document.createElement('div');
    div.className = 'chat-message ai';
    div.id = id;
    div.innerHTML = `
      <div class="chat-message-avatar">H</div>
      <div class="chat-message-body">
        <div class="thinking-indicator">
          <div class="thinking-dot"></div>
          <div class="thinking-dot"></div>
          <div class="thinking-dot"></div>
        </div>
      </div>
    `;
    const block = this.createThinkingBlock();
    block.style.display = 'none';
    div.querySelector('.chat-message-body').appendChild(block);
    container.appendChild(div);
    container.scrollTop = container.scrollHeight;
    return id;
  },

  removeThinking(id) {
    const el = document.getElementById(id);
    if (el) el.remove();
  },

  createThinkingBlock(content) {
    const block = document.createElement('div');
    block.className = 'thinking-block';
    block.innerHTML = `
      <div class="thinking-block-header">
        <span class="thinking-block-icon">🧠</span>
        <span class="thinking-block-title">Thinking</span>
        <button class="thinking-block-toggle" title="Toggle" aria-label="Toggle">▼</button>
      </div>
      <div class="thinking-block-body">
        <div class="thinking-block-markdown">${this.renderMarkdown(content || '')}</div>
      </div>`;
    const btn = block.querySelector('.thinking-block-toggle');
    btn.addEventListener('click', () => this.toggleThinking(block));
    block.querySelector('.thinking-block-header').addEventListener('click', (e) => {
      if (e.target !== btn) this.toggleThinking(block);
    });
    return block;
  },

  toggleThinking(block) {
    const body = block.querySelector('.thinking-block-body');
    const btn = block.querySelector('.thinking-block-toggle');
    const collapsed = !body.classList.contains('visible');
    if (collapsed) {
      body.classList.add('visible');
      btn.textContent = '▲';
    } else {
      body.classList.remove('visible');
      btn.textContent = '▼';
    }
  },

  updateThinkingContent(id, content) {
    const el = document.getElementById(id);
    if (!el) return;
    const block = el.querySelector('.thinking-block');
    if (!block) return;
    const md = block.querySelector('.thinking-block-markdown');
    if (md) md.innerHTML = this.renderMarkdown(content || '');
    block.style.display = 'block';
  },

  // ============================================================
  // Operation Summary Bar (B-10/12)
  // ============================================================

  /// Create a Codex-style operation summary bar showing tool execution stats.
  /// Returns null if all stats are zero (hides the bar when no ops performed).
  createOperationSummaryBar(summary, toolName) {
    const filesEdited = summary.files_edited || 0;
    const filesCreated = summary.files_created || 0;
    const filesDeleted = summary.files_deleted || 0;
    const commandsRun = summary.commands_run || 0;
    const totalDiffLines = summary.total_diff_lines || 0;
    const totalOps = filesEdited + filesCreated + filesDeleted + commandsRun;
    if (totalOps === 0) return null;

    const bar = document.createElement('div');
    bar.className = 'operation-summary-bar';
    bar._summary = summary;
    const parts = [];
    if (filesEdited > 0) parts.push(`已编辑 ${filesEdited} 个文件`);
    if (filesCreated > 0) parts.push(`已创建 ${filesCreated} 个文件`);
    if (filesDeleted > 0) parts.push(`已删除 ${filesDeleted} 个文件`);
    if (commandsRun > 0) parts.push(`已运行 ${commandsRun} 条命令`);
    const summaryText = parts.join('，');
    const reason = this.generateOperationReason(summary, toolName);

    bar.innerHTML = `
      <div class="operation-summary-header">
        <span class="operation-summary-icon">⚡</span>
        <span class="operation-summary-text">${this.escapeHtml(summaryText)}</span>
        ${reason ? `<span class="operation-summary-reason">${this.escapeHtml(reason)}</span>` : ''}
        <span class="operation-summary-progress"></span>
        <button class="operation-summary-toggle" title="展开/折叠">▼</button>
      </div>
      <div class="operation-summary-details" data-lazy="true">
        <div class="operation-summary-stat"><span class="operation-summary-stat-label diff-add">+</span><span>编辑: ${filesEdited}</span></div>
        <div class="operation-summary-stat"><span class="operation-summary-stat-label diff-add">+</span><span>创建: ${filesCreated}</span></div>
        <div class="operation-summary-stat"><span class="operation-summary-stat-label diff-del">-</span><span>删除: ${filesDeleted}</span></div>
        <div class="operation-summary-stat"><span class="operation-summary-stat-label">⌘</span><span>命令: ${commandsRun}</span></div>
        <div class="operation-summary-stat"><span class="operation-summary-stat-label">≡</span><span>Diff 行数: ${totalDiffLines}</span></div>
        <div class="operation-summary-diff-preview"></div>
      </div>`;

    const toggle = bar.querySelector('.operation-summary-toggle');
    toggle.addEventListener('click', () => this.toggleDetails(bar));
    bar.querySelector('.operation-summary-header').addEventListener('click', (e) => {
      if (e.target !== toggle) this.toggleDetails(bar);
    });
    return bar;
  },

  /// Toggle expand/collapse of operation summary details panel.
  /// Lazy-loads diff preview on first expand (B-11/12).
  toggleDetails(bar) {
    const details = bar.querySelector('.operation-summary-details');
    const toggle = bar.querySelector('.operation-summary-toggle');
    const expanded = details.classList.contains('visible');
    if (expanded) {
      details.classList.remove('visible');
      toggle.textContent = '▼';
    } else {
      details.classList.add('visible');
      toggle.textContent = '▲';
      if (details.dataset.lazy === 'true') {
        this.renderDiffPreview(details.querySelector('.operation-summary-diff-preview'), bar._summary);
        details.dataset.lazy = 'false';
      }
    }
  },

  /// Update or create the operation summary bar in the most recent AI message.
  /// Removes existing bar before inserting a new one to avoid duplicates.
  updateOperationSummary(summary, toolName) {
    if (!summary || typeof summary !== 'object') return;
    const container = document.getElementById('aiChatMessages');
    if (!container) return;
    const lastAi = container.querySelector('.chat-message.ai:last-child');
    if (!lastAi) return;
    const body = lastAi.querySelector('.chat-message-body');
    if (!body) return;
    const existing = body.querySelector('.operation-summary-bar');
    if (existing) existing.remove();
    const bar = this.createOperationSummaryBar(summary, toolName);
    if (bar) body.appendChild(bar);
  },

  /// Generate natural-language operation reason from stats and tool name (B-11/12).
  generateOperationReason(summary, toolName) {
    const edited = summary.files_edited || 0;
    const created = summary.files_created || 0;
    const deleted = summary.files_deleted || 0;
    const commands = summary.commands_run || 0;
    const parts = [];
    if (edited > 0) parts.push(`编辑 ${edited} 个文件`);
    if (created > 0) parts.push(`创建 ${created} 个新文件`);
    if (deleted > 0) parts.push(`删除 ${deleted} 个旧文件`);
    if (commands > 0) parts.push(`运行 ${commands} 条命令`);
    if (parts.length === 0) return '';
    let reason = '我准备' + parts.join('，');
    if (toolName) {
      const lower = (toolName + '').toLowerCase();
      if (lower.includes('edit')) reason += '以优化代码结构';
      else if (lower.includes('delete')) reason += '以清理冗余代码';
      else if (lower.includes('create')) reason += '以添加新功能';
      else if (lower.includes('test')) reason += '以验证正确性';
      else reason += '以完成任务';
    } else {
      reason += '以完成任务';
    }
    return reason;
  },

  /// Render a virtual diff preview inside the operation summary bar (B-11/12).
  /// Limits output to 50 lines; excess hidden behind a "view full file" link.
  renderDiffPreview(container, summary) {
    if (!container || !summary) return;
    const edited = summary.files_edited || 0;
    const created = summary.files_created || 0;
    const deleted = summary.files_deleted || 0;
    const lines = [];
    for (let i = 0; i < created; i++) lines.push(`+ 新建文件 #${i + 1}`);
    for (let i = 0; i < edited; i++) lines.push(`~ 修改文件 #${i + 1}`);
    for (let i = 0; i < deleted; i++) lines.push(`- 删除文件 #${i + 1}`);
    const limit = 50;
    const visible = lines.slice(0, limit);
    let html = visible.map(l => {
      const cls = l.startsWith('+') ? 'diff-add' : l.startsWith('-') ? 'diff-del' : 'diff-hunk';
      return `<div class="diff-preview-line ${cls}">${this.escapeHtml(l)}</div>`;
    }).join('');
    if (lines.length > limit) {
      html += `<div class="diff-preview-more">... 以及 ${lines.length - limit} 行更多</div>`;
    }
    html += `<div class="diff-preview-footer"><span class="diff-preview-link">查看完整文件</span></div>`;
    container.innerHTML = html;
  },

  /// Update real-time progress text on the active operation summary bar (B-11/12).
  updateOperationProgress(text) {
    if (!text) return;
    const container = document.getElementById('aiChatMessages');
    if (!container) return;
    const lastAi = container.querySelector('.chat-message.ai:last-child');
    if (!lastAi) return;
    const bar = lastAi.querySelector('.operation-summary-bar');
    if (!bar) return;
    const progress = bar.querySelector('.operation-summary-progress');
    if (!progress) return;
    progress.textContent = text;
    progress.classList.add('active');
  },

  async initWorkspace() {
    const tauri = window.__TAURI__;
    if (!tauri) return;
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      this.currentWorkspace = await invoke('get_current_workspace');
    } catch (e) {
      this.currentWorkspace = null;
    }
  },

  newChatSession() {
    // Save current session before creating new one
    if (this.chatMessages.length > 0 && this.activeSessionId) {
      const session = this.chatSessions.find(s => s.id === this.activeSessionId);
      if (session) {
        session.messages = [...this.chatMessages];
        session.updatedAt = Date.now();
      }
    }
    this.activeSessionId = 'session-' + Date.now() + '-' + Math.random().toString(36).slice(2, 7);
    this.chatMessages = [];
    this.tokenStats = { promptTokens: 0, completionTokens: 0, estimatedTokens: 0 };
    this.cumulativeStats = { promptTokens: 0, completionTokens: 0, requestCount: 0 };
    document.getElementById('aiChatMessages').innerHTML = '';
    this.addChatMessage('ai', '新会话已开始。有什么可以帮您的？');
    this.updateTokenDisplay();
    // Add to sessions list
    this.chatSessions.unshift({
      id: this.activeSessionId,
      title: '新会话',
      preview: '有什么可以帮您的？',
      messages: [],
      createdAt: Date.now(),
      updatedAt: Date.now(),
    });
    this.saveChatSessions();
    this.renderSessionList();
  },

  loadChatSessions() {
    try {
      const raw = localStorage.getItem('hajimi_chat_sessions');
      if (raw) {
        this.chatSessions = JSON.parse(raw);
        if (this.chatSessions.length > 0) {
          // Restore the most recent session
          const latest = this.chatSessions[0];
          this.activeSessionId = latest.id;
          this.chatMessages = latest.messages || [];
          this.renderChatMessages();
          this.renderSessionList();
        } else {
          this.newChatSession();
        }
      } else {
        this.newChatSession();
      }
    } catch (e) {
      console.error('loadChatSessions error:', e);
      this.newChatSession();
    }
  },

  saveChatSessions() {
    try {
      // Update current session messages before saving
      if (this.activeSessionId) {
        const session = this.chatSessions.find(s => s.id === this.activeSessionId);
        if (session) {
          session.messages = [...this.chatMessages];
          session.updatedAt = Date.now();
          // Update title/preview from first user message or assistant response
          const firstUser = this.chatMessages.find(m => m.role === 'user');
          const firstAi = this.chatMessages.find(m => m.role === 'assistant');
          if (firstUser) {
            session.title = firstUser.content.slice(0, 30);
            session.preview = firstUser.content.slice(0, 60);
          } else if (firstAi) {
            session.title = firstAi.content.slice(0, 30);
            session.preview = firstAi.content.slice(0, 60);
          }
        }
      }
      localStorage.setItem('hajimi_chat_sessions', JSON.stringify(this.chatSessions));
    } catch (e) {
      console.error('saveChatSessions error:', e);
    }
  },

  switchSession(id) {
    // Save current session first
    if (this.activeSessionId) {
      const current = this.chatSessions.find(s => s.id === this.activeSessionId);
      if (current) {
        current.messages = [...this.chatMessages];
        current.updatedAt = Date.now();
      }
    }
    // Switch to target session
    const target = this.chatSessions.find(s => s.id === id);
    if (target) {
      this.activeSessionId = id;
      this.chatMessages = target.messages || [];
      this.renderChatMessages();
      this.updateTokenDisplay();
      this.renderSessionList();
      this.saveChatSessions();
    }
  },

  renderChatMessages() {
    const container = document.getElementById('aiChatMessages');
    container.innerHTML = '';
    for (const msg of this.chatMessages) {
      this.addChatMessage(msg.role, msg.content, false);
    }
  },

  renderSessionList() {
    const list = document.getElementById('sessionList');
    if (!list) return;
    list.innerHTML = this.chatSessions.map(s => `
      <div class="session-item ${s.id === this.activeSessionId ? 'active' : ''}" data-session="${s.id}">
        <div class="session-title">${this.escapeHtml(s.title || '会话')}</div>
        <div class="session-preview">${this.escapeHtml(s.preview || '')}</div>
      </div>
    `).join('');
    // Re-bind click handlers
    list.querySelectorAll('.session-item').forEach(el => {
      el.addEventListener('click', () => {
        const id = el.dataset.session;
        if (id && id !== this.activeSessionId) {
          this.switchSession(id);
        }
      });
    });
  },

  // ============================================================
  // Provider Management
  // ============================================================
  async loadProviders() {
    const tauri = window.__TAURI__;
    if (!tauri) return;
    const invoke = tauri.core?.invoke || tauri.invoke;

    try {
      const custom = await invoke('get_provider_configs', { workspacePath: this.currentWorkspace });
      this.providerConfigs = custom || [];
      this.renderModelButton();
      this.renderProviderList();
    } catch (e) {
      console.error('loadProviders error:', e);
    }
  },

  renderModelButton() {
    const btn = document.getElementById('modelSelectBtn');
    if (!btn) return;
    const active = this.providerConfigs.find(c => c.id === this.activeProviderId);
    btn.textContent = active ? (active.name || active.model || '选择模型') : '选择模型';
  },

  // ============================================================
  // Model Picker Modal
  // ============================================================
  setupModelPicker() {
    const btn = document.getElementById('modelSelectBtn');
    const closeBtn = document.getElementById('modelPickerClose');
    const addBtn = document.getElementById('modelPickerAddBtn');
    const modal = document.getElementById('modelPickerModal');

    if (btn) btn.addEventListener('click', () => this.openModelPicker());
    if (closeBtn) closeBtn.addEventListener('click', () => this.closeModelPicker());
    if (addBtn) addBtn.addEventListener('click', () => { this.closeModelPicker(); this.openProviderModal(); });
    if (modal) {
      modal.addEventListener('click', (e) => {
        if (e.target === modal) this.closeModelPicker();
      });
    }
  },

  openModelPicker() {
    this.renderModelPicker();
    document.getElementById('modelPickerModal')?.classList.add('active');
  },

  closeModelPicker() {
    document.getElementById('modelPickerModal')?.classList.remove('active');
  },

  renderModelPicker() {
    const body = document.getElementById('modelPickerBody');
    if (!body) return;

    if (!this.providerConfigs.length) {
      body.innerHTML = '<div class="model-picker-empty">暂无配置模型，点击下方按钮添加。</div>';
      return;
    }

    let html = '<div class="model-picker-list">';
    this.providerConfigs.forEach(cfg => {
      const isActive = cfg.id === this.activeProviderId;
      html += `
        <div class="model-picker-item ${isActive ? 'active' : ''}">
          <div class="model-picker-info">
            <div class="model-picker-name">${this.escapeHtml(cfg.name || cfg.id)}</div>
            <div class="model-picker-meta">${this.escapeHtml(cfg.model || '')} · ${this.escapeHtml(cfg.providerType || 'openai-compatible')}</div>
          </div>
          <div class="model-picker-actions">
            <button class="model-picker-btn use" data-id="${this.escapeHtml(cfg.id)}">${isActive ? '当前' : '使用'}</button>
            <button class="model-picker-btn" data-edit="${this.escapeHtml(cfg.id)}">编辑</button>
            <button class="model-picker-btn" data-delete="${this.escapeHtml(cfg.id)}">删除</button>
          </div>
        </div>
      `;
    });
    html += '</div>';
    body.innerHTML = html;

    // Bind actions
    body.querySelectorAll('.model-picker-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const id = btn.dataset.id || btn.dataset.edit || btn.dataset.delete;
        if (btn.dataset.id) {
          this.selectProvider(id);
          this.closeModelPicker();
        } else if (btn.dataset.edit) {
          this.closeModelPicker();
          const cfg = this.providerConfigs.find(c => c.id === id);
          if (cfg) this.openProviderModal(cfg);
        } else if (btn.dataset.delete) {
          if (confirm(`删除模型配置 "${id}"？`)) {
            this.deleteProviderConfig(id);
          }
        }
      });
    });
  },

  selectProvider(id) {
    this.activeProviderId = id;
    this.renderModelButton();
    const cfg = this.providerConfigs.find(c => c.id === id);
    const displayName = cfg ? (cfg.name || cfg.model || id) : id;
    const statusModel = document.getElementById('statusModel');
    if (statusModel) statusModel.textContent = displayName;
    console.log(`Switched to model: ${displayName}`);
  },

  renderProviderList() {
    const list = document.getElementById('providerList');
    if (!list) return;

    const workspaceTag = this.currentWorkspace
      ? `<span class="provider-source-tag workspace" title="${this.currentWorkspace}">workspace</span>`
      : '<span class="provider-source-tag global">global</span>';

    if (!this.providerConfigs.length) {
      list.innerHTML = `<div class="provider-item-empty">暂无自定义模型，点击上方按钮添加。${workspaceTag}</div>`;
      return;
    }

    list.innerHTML = this.providerConfigs.map(cfg => `
      <div class="provider-item">
        <div class="provider-item-info">
          <div class="provider-item-name">${cfg.name}</div>
          <div class="provider-item-meta">${cfg.model} · ${cfg.baseUrl}</div>
        </div>
        <div class="provider-item-actions">
          <button class="provider-item-btn" onclick="app.editProviderConfig('${cfg.id}')">编辑</button>
          <button class="provider-item-btn delete" onclick="app.deleteProviderConfig('${cfg.id}')">删除</button>
        </div>
      </div>
    `).join('') + `<div class="provider-source-hint">来源: ${workspaceTag}</div>`;
  },

  setupProviderSettings() {
    const addBtn = document.getElementById('addProviderBtn');
    const cancelBtn = document.getElementById('cancelProvider');
    const saveBtn = document.getElementById('saveProvider');
    const closeBtn = document.getElementById('providerModalClose');
    const modal = document.getElementById('providerModal');
    const preset = document.getElementById('providerModalBaseUrlPreset');
    const baseUrl = document.getElementById('providerBaseUrl');
    const toggleKey = document.getElementById('providerModalToggleKey');
    const apiKey = document.getElementById('providerApiKey');

    if (addBtn) addBtn.addEventListener('click', () => this.openProviderModal());
    if (cancelBtn) cancelBtn.addEventListener('click', () => this.closeProviderModal());
    if (saveBtn) saveBtn.addEventListener('click', () => this.saveProviderConfig());
    if (closeBtn) closeBtn.addEventListener('click', () => this.closeProviderModal());

    const exportBtn = document.getElementById('exportProviderBtn');
    const importBtn = document.getElementById('importProviderBtn');
    if (exportBtn) exportBtn.addEventListener('click', () => this.openBackupModal('export'));
    if (importBtn) importBtn.addEventListener('click', () => this.openBackupModal('import'));

    if (preset && baseUrl) {
      preset.addEventListener('change', () => {
        if (preset.value !== 'custom') baseUrl.value = preset.value;
      });
    }

    if (toggleKey && apiKey) {
      toggleKey.addEventListener('click', () => {
        apiKey.type = apiKey.type === 'password' ? 'text' : 'password';
      });
    }

    if (modal) {
      modal.addEventListener('click', (e) => {
        if (e.target === modal) this.closeProviderModal();
      });
    }

    const backupModal = document.getElementById('backupModal');
    const backupClose = document.getElementById('backupModalClose');
    const cancelBackup = document.getElementById('cancelBackup');
    const confirmBackup = document.getElementById('confirmBackup');
    const backupToggle = document.getElementById('backupTogglePassword');
    const backupPassword = document.getElementById('backupPassword');
    if (backupClose) backupClose.addEventListener('click', () => this.closeBackupModal());
    if (cancelBackup) cancelBackup.addEventListener('click', () => this.closeBackupModal());
    if (confirmBackup) confirmBackup.addEventListener('click', () => this.confirmBackup());
    if (backupToggle && backupPassword) {
      backupToggle.addEventListener('click', () => {
        backupPassword.type = backupPassword.type === 'password' ? 'text' : 'password';
      });
    }
    if (backupModal) {
      backupModal.addEventListener('click', (e) => {
        if (e.target === backupModal) this.closeBackupModal();
      });
    }
  },

  openProviderModal(config) {
    this.editingProviderId = config ? config.id : null;
    const modal = document.getElementById('providerModal');
    document.getElementById('providerModalTitle').textContent = config ? '编辑模型' : '添加模型';
    document.getElementById('providerId').value = config ? config.id : 'provider-' + Date.now();
    document.getElementById('providerId').disabled = !!config;
    document.getElementById('providerName').value = config ? config.name : '';
    document.getElementById('providerModalType').value = config ? (config.providerType || config.provider_type || 'openai-compatible') : 'openai-compatible';
    document.getElementById('providerBaseUrl').value = config ? config.baseUrl : '';
    document.getElementById('providerModel').value = config ? config.model : '';
    const keyInput = document.getElementById('providerApiKey');
    keyInput.value = config && config.apiKey ? config.apiKey : '';
    // For security, if editing and no key shown, prompt for re-entry or show masked
    if (config && !config.apiKey) {
      keyInput.placeholder = 'sk-•••••••• (re-enter to update)';
    }
    modal.classList.add('active');
  },

  closeProviderModal() {
    document.getElementById('providerModal').classList.remove('active');
    this.editingProviderId = null;
    document.getElementById('providerForm').reset();
  },

  openBackupModal(mode) {
    this.backupMode = mode;
    const modal = document.getElementById('backupModal');
    document.getElementById('backupModalTitle').textContent = mode === 'export' ? '导出加密备份' : '导入备份';
    document.getElementById('backupFileField').style.display = mode === 'import' ? 'block' : 'none';
    document.getElementById('backupPassword').value = '';
    document.getElementById('backupFilePath').value = '';
    modal.classList.add('active');
  },

  closeBackupModal() {
    document.getElementById('backupModal').classList.remove('active');
    this.backupMode = null;
  },

  async confirmBackup() {
    const password = document.getElementById('backupPassword').value;
    if (!password) {
      this.showErrorToast('请输入密码');
      return;
    }
    if (this.backupMode === 'export') {
      await this.exportProviderBackup(password);
    } else {
      const filePath = document.getElementById('backupFilePath').value.trim();
      if (!filePath) {
        this.showErrorToast('请输入备份文件路径');
        return;
      }
      await this.importProviderBackup(password, filePath);
    }
    this.closeBackupModal();
  },

  async exportProviderBackup(password) {
    const tauri = window.__TAURI__;
    if (!tauri) { this.showErrorToast('Tauri 不可用'); return; }
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      const path = await invoke('export_provider_backup', { password, workspacePath: this.currentWorkspace });
      this.showErrorToast('备份已导出: ' + path);
    } catch (e) {
      this.showErrorToast('导出失败: ' + (e.message || e));
    }
  },

  async importProviderBackup(password, filePath) {
    const tauri = window.__TAURI__;
    if (!tauri) { this.showErrorToast('Tauri 不可用'); return; }
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      const count = await invoke('import_provider_backup', { password, filePath });
      this.showErrorToast('成功导入 ' + count + ' 个 Provider');
      await this.loadProviders();
    } catch (e) {
      this.showErrorToast('导入失败: ' + (e.message || e));
    }
  },

  async saveProviderConfig() {
    const id = document.getElementById('providerId').value.trim() || 'provider-' + Date.now();
    const name = document.getElementById('providerName').value.trim();
    const providerType = document.getElementById('providerModalType').value;
    const baseUrl = document.getElementById('providerBaseUrl').value.trim();
    const model = document.getElementById('providerModel').value.trim();
    const apiKey = document.getElementById('providerApiKey').value.trim();

    if (!name || !baseUrl || !model || !apiKey) {
      this.showErrorToast('请填写名称、Base URL、模型名和 API Key');
      return;
    }
    if (!/^[a-z0-9_-]+$/.test(id)) {
      this.showErrorToast('ID 只能包含小写字母、数字、下划线和横线');
      return;
    }

    const config = {
      id,
      name,
      providerType,
      baseUrl,
      apiKey,
      model
    };
    const tauri = window.__TAURI__;
    if (!tauri) { this.showErrorToast('Tauri 不可用'); return; }
    const invoke = tauri.core?.invoke || tauri.invoke;

    try {
      const saveTarget = document.getElementById('providerSaveTarget')?.value || 'global';
      const command = this.editingProviderId ? 'update_provider_config' : 'add_provider_config';
      await invoke(command, { config: config, workspacePath: this.currentWorkspace, saveTarget: saveTarget });
      // Clear sensitive field after save (P0-3)
      document.getElementById('providerApiKey').value = '';
      await this.loadProviders();
      this.closeProviderModal();
      // Optional: show success with masked key
      console.log('Provider saved with key secured in OS keyring');
    } catch (e) {
      this.showErrorToast('保存失败: ' + (e.message || e));
    }
  },

  editProviderConfig(id) {
    const cfg = this.providerConfigs.find(c => c.id === id);
    if (cfg) this.openProviderModal(cfg);
  },

  async deleteProviderConfig(id) {
    if (!confirm('确定要删除此模型配置吗？')) return;
    const tauri = window.__TAURI__;
    if (!tauri) return;
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      await invoke('delete_provider_config', { id: id, workspacePath: this.currentWorkspace, deleteTarget: 'global' });
      await this.loadProviders();
    } catch (e) {
      this.showErrorToast('删除失败: ' + (e.message || e));
    }
  },

  // ============================================================
  // Profile Management (B-05/01)
  // ============================================================
  async loadProfiles() {
    const tauri = window.__TAURI__;
    if (!tauri) return;
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      const profiles = await invoke('list_profiles');
      const active = await invoke('get_active_profile');
      const select = document.getElementById('profileSelect');
      if (!select) return;
      let html = '<option value="">default</option>';
      (profiles || []).forEach(p => {
        html += `<option value="${p}" ${p === active ? 'selected' : ''}>${p}</option>`;
      });
      select.innerHTML = html;
    } catch (e) {
      console.error('loadProfiles error:', e);
    }
  },

  setupProfileSettings() {
    const select = document.getElementById('profileSelect');
    const createBtn = document.getElementById('createProfileBtn');
    const deleteBtn = document.getElementById('deleteProfileBtn');
    if (select) {
      select.addEventListener('change', async () => {
        const tauri = window.__TAURI__;
        if (!tauri) return;
        const invoke = tauri.core?.invoke || tauri.invoke;
        const name = select.value || null;
        try {
          await invoke('set_active_profile', { name });
          await this.loadProviders();
          this.showErrorToast('已切换至 Profile: ' + (name || 'default'));
        } catch (e) {
          this.showErrorToast('切换失败: ' + (e.message || e));
        }
      });
    }
    if (createBtn) {
      createBtn.addEventListener('click', async () => {
        const name = prompt('输入新 Profile 名称:');
        if (!name) return;
        const tauri = window.__TAURI__;
        if (!tauri) return;
        const invoke = tauri.core?.invoke || tauri.invoke;
        try {
          await invoke('create_profile', { name });
          await this.loadProfiles();
          this.showErrorToast('Profile 创建成功: ' + name);
        } catch (e) {
          this.showErrorToast('创建失败: ' + (e.message || e));
        }
      });
    }
    if (deleteBtn) {
      deleteBtn.addEventListener('click', async () => {
        const select = document.getElementById('profileSelect');
        const name = select?.value;
        if (!name) { this.showErrorToast('不能删除 default profile'); return; }
        if (!confirm('确定要删除 Profile "' + name + '" 吗？相关密钥将一并清理。')) return;
        const tauri = window.__TAURI__;
        if (!tauri) return;
        const invoke = tauri.core?.invoke || tauri.invoke;
        try {
          await invoke('delete_profile', { name });
          await this.loadProfiles();
          await this.loadProviders();
          this.showErrorToast('Profile 已删除: ' + name);
        } catch (e) {
          this.showErrorToast('删除失败: ' + (e.message || e));
        }
      });
    }
  },

  // ============================================================
  // Agent Provider Binding (B-05/02)
  // ============================================================
  async loadAgentProviders() {
    const tauri = window.__TAURI__;
    if (!tauri) return;
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      const map = await invoke('get_agent_providers');
      const list = document.getElementById('agentProviderList');
      const select = document.getElementById('agentBindProvider');
      if (!list) return;
      // Update provider dropdown
      let opts = '<option value="">-- 默认 --</option>';
      this.providerConfigs.forEach(c => {
        opts += `<option value="${c.id}">${c.name}</option>`;
      });
      if (select) select.innerHTML = opts;
      // Render bound list
      const entries = Object.entries(map || {});
      if (!entries.length) {
        list.innerHTML = '<div class="agent-provider-empty">暂无绑定</div>';
        return;
      }
      list.innerHTML = entries.map(([agentId, providerId]) => {
        const cfg = this.providerConfigs.find(c => c.id === providerId);
        const name = cfg ? cfg.name : providerId;
        return `<div class="agent-provider-item"><span>${agentId}</span><span>→ ${name}</span><button onclick="app.unbindAgentProvider('${agentId}')">解绑</button></div>`;
      }).join('');
    } catch (e) {
      console.error('loadAgentProviders error:', e);
    }
  },

  setupAgentProvider() {
    const bindBtn = document.getElementById('agentBindBtn');
    if (bindBtn) {
      bindBtn.addEventListener('click', async () => {
        const agentId = document.getElementById('agentBindId')?.value.trim();
        const providerId = document.getElementById('agentBindProvider')?.value || null;
        if (!agentId) { this.showErrorToast('请输入 Agent ID'); return; }
        const tauri = window.__TAURI__;
        if (!tauri) return;
        const invoke = tauri.core?.invoke || tauri.invoke;
        try {
          await invoke('set_agent_provider', { agentId, providerId });
          await this.loadAgentProviders();
          document.getElementById('agentBindId').value = '';
        } catch (e) {
          this.showErrorToast('绑定失败: ' + (e.message || e));
        }
      });
    }
    // Refresh when providers change
    const origLoadProviders = this.loadProviders.bind(this);
    this.loadProviders = async () => { await origLoadProviders(); await this.loadAgentProviders(); };
  },

  async unbindAgentProvider(agentId) {
    const tauri = window.__TAURI__;
    if (!tauri) return;
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      await invoke('set_agent_provider', { agentId, providerId: null });
      await this.loadAgentProviders();
    } catch (e) {
      this.showErrorToast('解绑失败: ' + (e.message || e));
    }
  },

  // ============================================================
  // MCP Settings (P3-18)
  // ============================================================
  setupMcpSettings() {
    this.loadMcpServers();
    const connectBtn = document.getElementById('mcpConnectBtn');
    if (connectBtn) {
      connectBtn.addEventListener('click', () => this.mcpConnectFromInput());
    }
  },

  async mcpConnectFromInput() {
    const urlInput = document.getElementById('mcpServerUrl');
    const transportSelect = document.getElementById('mcpTransport');
    const serverUrl = urlInput?.value.trim();
    const transport = transportSelect?.value || 'stdio';
    if (!serverUrl) { this.showErrorToast('请输入 MCP 服务器命令'); return; }

    this.showErrorToast('正在连接 MCP 服务器...');
    try {
      const result = await this.mcpInit(serverUrl, transport);
      this.mcpServers.push({ url: serverUrl, transport, tools: result.tool_names || [] });
      this.saveMcpServers();
      this.renderMcpServers();
      this.showErrorToast(`MCP 连接成功: ${result.tools || 0} 个工具`);
      if (urlInput) urlInput.value = '';
    } catch (e) {
      this.showErrorToast('MCP 连接失败: ' + (e.message || e));
    }
  },

  async mcpInit(serverUrl, transport) {
    const tauri = window.__TAURI__;
    if (!tauri) throw new Error('Tauri 不可用');
    const invoke = tauri.core?.invoke || tauri.invoke;
    const result = await invoke('execute_tool', {
      name: 'mcp_init',
      args: { server_url: serverUrl, transport }
    });
    const output = result.stdout || result.result || '{}';
    return JSON.parse(output);
  },

  async mcpInvoke(serverUrl, toolName, args) {
    const tauri = window.__TAURI__;
    if (!tauri) throw new Error('Tauri 不可用');
    const invoke = tauri.core?.invoke || tauri.invoke;
    const result = await invoke('execute_tool', {
      name: 'mcp_invoke',
      args: { server_url: serverUrl, tool_name: toolName, arguments: args || {} }
    });
    const output = result.stdout || result.result || '{}';
    return JSON.parse(output);
  },

  renderMcpServers() {
    const list = document.getElementById('mcpServerList');
    if (!list) return;
    if (!this.mcpServers.length) {
      list.innerHTML = '<div class="mcp-empty">暂无 MCP 服务器</div>';
      return;
    }
    list.innerHTML = this.mcpServers.map((s, i) => `
      <div class="mcp-server-item">
        <div class="mcp-server-info">
          <div class="mcp-server-url">${this.escapeHtml(s.url)}</div>
          <div class="mcp-server-meta">${s.transport} · ${(s.tools || []).length} 个工具</div>
        </div>
        <button class="mcp-server-remove" data-index="${i}">断开</button>
      </div>
    `).join('');

    list.querySelectorAll('.mcp-server-remove').forEach(btn => {
      btn.addEventListener('click', () => {
        const idx = parseInt(btn.dataset.index);
        this.mcpServers.splice(idx, 1);
        this.saveMcpServers();
        this.renderMcpServers();
      });
    });
  },

  saveMcpServers() {
    try {
      localStorage.setItem('hajimi.mcpServers', JSON.stringify(this.mcpServers));
    } catch (e) {
      console.error('saveMcpServers error:', e);
    }
  },

  loadMcpServers() {
    try {
      const raw = localStorage.getItem('hajimi.mcpServers');
      if (raw) {
        this.mcpServers = JSON.parse(raw);
        this.renderMcpServers();
      }
    } catch (e) {
      console.error('loadMcpServers error:', e);
    }
  },

  // ============================================================
  // Extensions (P3-16)
  // ============================================================
  setupExtensions() {
    this.loadInstalledExtensions();
    this.renderExtensions();
  },

  renderExtensions() {
    const list = document.getElementById('extensionsList');
    if (!list) return;

    const installedSet = new Set(this.installedExtensions);

    list.innerHTML = this.extensions.map(ext => {
      const isInstalled = ext.installed || installedSet.has(ext.id);
      return `
        <div class="extension-item${isInstalled ? ' installed' : ''}" data-id="${ext.id}">
          <div class="extension-icon" style="background:${ext.iconColor}">${ext.icon}</div>
          <div class="extension-info">
            <div class="extension-name">${ext.name}</div>
            <div class="extension-desc">${ext.desc}</div>
            <div class="extension-meta">${ext.version} • ${ext.publisher}</div>
          </div>
          ${isInstalled
            ? '<span class="extension-status">已安装</span><button class="extension-uninstall-btn" data-id="' + ext.id + '">卸载</button>'
            : '<button class="extension-install-btn" data-id="' + ext.id + '">安装</button>'}
        </div>
      `;
    }).join('');

    list.querySelectorAll('.extension-install-btn').forEach(btn => {
      btn.addEventListener('click', () => this.installExtension(btn.dataset.id));
    });
    list.querySelectorAll('.extension-uninstall-btn').forEach(btn => {
      btn.addEventListener('click', () => this.uninstallExtension(btn.dataset.id));
    });
  },

  installExtension(id) {
    if (!this.installedExtensions.includes(id)) {
      this.installedExtensions.push(id);
    }
    const ext = this.extensions.find(e => e.id === id);
    if (ext) ext.installed = true;
    this.saveInstalledExtensions();
    this.renderExtensions();
    this.showErrorToast(`已安装: ${ext?.name || id}`);
  },

  uninstallExtension(id) {
    this.installedExtensions = this.installedExtensions.filter(x => x !== id);
    const ext = this.extensions.find(e => e.id === id);
    if (ext) ext.installed = false;
    this.saveInstalledExtensions();
    this.renderExtensions();
    this.showErrorToast(`已卸载: ${ext?.name || id}`);
  },

  saveInstalledExtensions() {
    try {
      localStorage.setItem('hajimi.installedExtensions', JSON.stringify(this.installedExtensions));
    } catch (e) {
      console.error('saveInstalledExtensions error:', e);
    }
  },

  loadInstalledExtensions() {
    try {
      const raw = localStorage.getItem('hajimi.installedExtensions');
      if (raw) {
        this.installedExtensions = JSON.parse(raw);
        // Sync with extensions list
        this.extensions.forEach(ext => {
          ext.installed = this.installedExtensions.includes(ext.id);
        });
      }
    } catch (e) {
      console.error('loadInstalledExtensions error:', e);
    }
  },

  // ============================================================
  // LSP Integration (P3-17)
  // ============================================================
  pathToUri(path) {
    // Convert file path to file URI
    // Windows: C:\path → file:///C:/path
    // Unix: /path → file:///path
    const absolute = path.startsWith('/') || /^[A-Za-z]:/.test(path) ? path : (this.currentWorkspace || '.') + '/' + path;
    const normalized = absolute.replace(/\\/g, '/');
    if (normalized.startsWith('/')) {
      return 'file://' + normalized;
    }
    return 'file:///' + normalized;
  },

  async lspDefinition(filePath) {
    const tauri = window.__TAURI__;
    if (!tauri) { this.showErrorToast('Tauri 不可用'); return; }
    const invoke = tauri.core?.invoke || tauri.invoke;
    const sel = window.getSelection();
    if (!sel.rangeCount) return;
    // Simple position estimation: line 0, char 0 for now
    // In a real implementation, we'd map the cursor position to line/char
    const line = 0;
    const character = 0;
    try {
      const result = await invoke('execute_tool', {
        name: 'lsp_definition',
        args: { uri: this.pathToUri(filePath), line, character }
      });
      const output = result.stdout || result.result || '{}';
      const data = JSON.parse(output);
      if (data && data.uri) {
        const targetPath = data.uri.replace('file://', '').replace(/^\//, '');
        this.openFile(targetPath);
      } else {
        this.showErrorToast('未找到定义');
      }
    } catch (e) {
      this.showErrorToast('LSP 定义查找失败: ' + (e.message || e));
    }
  },

  async lspReferences(filePath) {
    const tauri = window.__TAURI__;
    if (!tauri) { this.showErrorToast('Tauri 不可用'); return; }
    const invoke = tauri.core?.invoke || tauri.invoke;
    const line = 0;
    const character = 0;
    try {
      const result = await invoke('execute_tool', {
        name: 'lsp_references',
        args: { uri: this.pathToUri(filePath), line, character }
      });
      const output = result.stdout || result.result || '[]';
      const locations = JSON.parse(output);
      if (!locations || !locations.length) {
        this.showErrorToast('未找到引用');
        return;
      }
      // Show results in output panel
      this.showPanel('output');
      this.clearOutput();
      this.addOutput(`引用 (${locations.length}):`, 'info');
      locations.forEach(loc => {
        const path = loc.uri ? loc.uri.replace('file://', '').replace(/^\//, '') : '?';
        const range = loc.range ? `:${loc.range.start.line + 1}` : '';
        this.addOutput(`  ${path}${range}`, 'info');
      });
    } catch (e) {
      this.showErrorToast('LSP 引用查找失败: ' + (e.message || e));
    }
  },

  async lspHover(filePath, rect) {
    const tauri = window.__TAURI__;
    if (!tauri) return;
    const invoke = tauri.core?.invoke || tauri.invoke;
    const line = 0;
    const character = 0;
    try {
      const result = await invoke('execute_tool', {
        name: 'lsp_hover',
        args: { uri: this.pathToUri(filePath), line, character }
      });
      const output = result.stdout || result.result || '{}';
      const data = JSON.parse(output);
      const contents = data.contents;
      if (!contents) { this.hideLspTooltip(); return; }
      let text = '';
      if (typeof contents === 'string') text = contents;
      else if (contents.value) text = contents.value;
      else if (Array.isArray(contents)) text = contents.map(c => c.value || c).join('\n');
      if (!text) { this.hideLspTooltip(); return; }
      this.showLspTooltip(text, rect);
    } catch (e) {
      this.hideLspTooltip();
    }
  },

  showLspTooltip(text, rect) {
    this.hideLspTooltip();
    const tooltip = document.createElement('div');
    tooltip.id = 'lspTooltip';
    tooltip.className = 'lsp-tooltip';
    tooltip.textContent = text;
    tooltip.style.left = (rect.left + window.scrollX) + 'px';
    tooltip.style.top = (rect.bottom + window.scrollY + 4) + 'px';
    document.body.appendChild(tooltip);
  },

  hideLspTooltip() {
    const tooltip = document.getElementById('lspTooltip');
    if (tooltip) tooltip.remove();
  },

  // ============================================================
  // Audit Log (B-05/03)
  // ============================================================
  async loadAuditLogs() {
    const tauri = window.__TAURI__;
    if (!tauri) return;
    const invoke = tauri.core?.invoke || tauri.invoke;
    try {
      const logs = await invoke('get_audit_logs', { limit: 100, offset: 0 });
      const tbody = document.getElementById('auditLogBody');
      if (!tbody) return;
      if (!logs || !logs.length) {
        tbody.innerHTML = '<tr><td colspan="4" class="audit-empty">暂无记录</td></tr>';
        return;
      }
      tbody.innerHTML = logs.map(r => {
        const time = r.timestamp ? new Date(r.timestamp).toLocaleString() : '-';
        const statusCls = r.status === 'completed' ? 'audit-status-ok' : r.status === 'failed' ? 'audit-status-err' : 'audit-status-start';
        return `<tr><td>${r.providerName || r.provider_name || '-'}</td><td>${r.model || '-'}</td><td>${time}</td><td><span class="audit-status ${statusCls}">${r.status}</span></td></tr>`;
      }).join('');
    } catch (e) {
      console.error('loadAuditLogs error:', e);
    }
  },

  setupAuditLog() {
    const refreshBtn = document.getElementById('refreshAuditBtn');
    if (refreshBtn) {
      refreshBtn.addEventListener('click', () => this.loadAuditLogs());
    }
  },

  setupGovernance() {
    const pauseBtn = document.getElementById('pauseLoopBtn');
    const resumeBtn = document.getElementById('resumeLoopBtn');
    const levelSelect = document.getElementById('approvalLevelSelect');
    const injectBtn = document.getElementById('injectMemoryBtn');
    const updateBtn = document.getElementById('updatePlanBtn');
    if (pauseBtn) pauseBtn.addEventListener('click', () => this.invokeGovernance('pause_loop'));
    if (resumeBtn) resumeBtn.addEventListener('click', () => this.invokeGovernance('resume_loop'));
    if (levelSelect) levelSelect.addEventListener('change', (e) => this.invokeGovernance('set_approval_level', { level: e.target.value }));
    if (injectBtn) injectBtn.addEventListener('click', () => {
      const key = document.getElementById('injectMemoryKey').value.trim();
      const value = document.getElementById('injectMemoryValue').value.trim();
      if (!key || !value) { this.showErrorToast('请输入 key 和 value'); return; }
      this.invokeGovernance('inject_memory', { key, value });
    });
    if (updateBtn) updateBtn.addEventListener('click', () => {
      const plan = document.getElementById('updatePlanInput').value.trim();
      if (!plan) { this.showErrorToast('请输入 plan 描述'); return; }
      this.invokeGovernance('update_plan', { plan });
    });
  },

  async invokeGovernance(cmd, args = {}) {
    const tauri = window.__TAURI__;
    if (!tauri) { this.showErrorToast('Desktop 未连接'); return; }
    try {
      await tauri.core.invoke(cmd, args);
      this.showToast('操作成功');
    } catch (e) {
      this.showErrorToast(`操作失败: ${e}`);
    }
  },

  setupSessionBrowser() {
    const refreshBtn = document.getElementById('refreshCheckpointsBtn');
    const exportAllBtn = document.getElementById('exportAllBtn');
    if (refreshBtn) refreshBtn.addEventListener('click', () => this.loadCheckpoints());
    if (exportAllBtn) exportAllBtn.addEventListener('click', () => this.exportAllCheckpoints());
    this.loadCheckpoints();
  },

  async loadCheckpoints() {
    const list = document.getElementById('checkpointList');
    if (!list) return;
    const tauri = window.__TAURI__;
    if (!tauri) { list.innerHTML = '<div style="color:var(--fg-dim);text-align:center;padding:12px;">Tauri 不可用</div>'; return; }
    try {
      const checkpoints = await tauri.core.invoke('list_checkpoints');
      if (!checkpoints || checkpoints.length === 0) {
        list.innerHTML = '<div style="color:var(--fg-dim);text-align:center;padding:12px;">暂无检查点</div>';
        return;
      }
      list.innerHTML = checkpoints.map((chk, idx) => `
        <div style="border-bottom:1px solid var(--border);padding:6px 0;">
          <div style="display:flex;justify-content:space-between;">
            <span style="font-weight:bold;">${chk.id || 'chk_' + idx}</span>
            <span style="color:var(--fg-dim);">${chk.timestamp || ''}</span>
          </div>
          <div style="display:flex;gap:4px;margin-top:4px;">
            <button class="modal-btn secondary" style="font-size:11px;padding:2px 6px;" onclick="app.restoreCheckpoint('${chk.id}')">恢复</button>
            <button class="modal-btn secondary" style="font-size:11px;padding:2px 6px;" onclick="app.exportCheckpoint('${chk.id}')">导出</button>
          </div>
        </div>
      `).join('');
    } catch (e) {
      list.innerHTML = '<div style="color:var(--fg-dim);text-align:center;padding:12px;">加载失败</div>';
    }
  },

  async restoreCheckpoint(id) {
    if (!confirm('确定要恢复此检查点吗？')) return;
    const tauri = window.__TAURI__;
    if (!tauri) return;
    try { await tauri.core.invoke('restore_checkpoint', { id }); this.showToast('恢复成功'); }
    catch (e) { this.showErrorToast(`恢复失败: ${e}`); }
  },

  async exportCheckpoint(id) {
    const tauri = window.__TAURI__;
    if (!tauri) return;
    try {
      const json = await tauri.core.invoke('export_checkpoint', { id });
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a'); a.href = url; a.download = `checkpoint_${id}.json`; a.click(); URL.revokeObjectURL(url);
    } catch (e) { this.showErrorToast(`导出失败: ${e}`); }
  },

  async exportAllCheckpoints() {
    const tauri = window.__TAURI__;
    if (!tauri) return;
    try {
      const json = await tauri.core.invoke('export_checkpoint', { id: 'all' });
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a'); a.href = url; a.download = 'checkpoints_all.json'; a.click(); URL.revokeObjectURL(url);
    } catch (e) { this.showErrorToast(`导出失败: ${e}`); }
  },

  setupResourceDashboard() {
    this.updateMetrics();
    this.metricsInterval = setInterval(() => this.updateMetrics(), 3000);
  },

  async updateMetrics() {
    const tauri = window.__TAURI__;
    if (!tauri) {
      document.getElementById('metricIteration').textContent = 'N/A';
      return;
    }
    try {
      const m = await tauri.core.invoke('get_resource_metrics');
      document.getElementById('metricIteration').textContent = m.iteration_count != null ? m.iteration_count : 'N/A';
      document.getElementById('metricBlackboard').textContent = m.blackboard_size != null ? m.blackboard_size : 'N/A';
      document.getElementById('metricFailureRate').textContent = m.failure_rate_percent != null ? m.failure_rate_percent.toFixed(1) + '%' : 'N/A';
      document.getElementById('metricLatency').textContent = m.callback_latency_ms != null ? m.callback_latency_ms + 'ms' : 'N/A';
    } catch (e) {}
  },

  showErrorToast(message) {
    let toast = document.getElementById('errorToast');
    if (!toast) {
      toast = document.createElement('div');
      toast.id = 'errorToast';
      toast.className = 'error-toast';
      document.body.appendChild(toast);
    }
    toast.textContent = message;
    toast.classList.add('active');
    setTimeout(() => { toast.classList.remove('active'); }, 4000);
  },

  hideErrorToast() {
    const toast = document.getElementById('errorToast');
    if (toast) toast.classList.remove('active');
  },

  formatText(text) {
    let html = text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
    html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
    html = html.replace(/`(.+?)`/g, '<code>$1</code>');
    html = html.replace(/```([\s\S]*?)```/g, '<pre><code>$1</code></pre>');
    html = html.replace(/\n/g, '<br>');
    return html;
  },

  /// Render Markdown to HTML with XSS-safe URL sanitization (B-08/12).
  renderMarkdown(text) {
    let html = text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
    // Headers
    html = html.replace(/^### (.+)$/gm, '<h4>$1</h4>');
    html = html.replace(/^## (.+)$/gm, '<h4>$1</h4>');
    html = html.replace(/^# (.+)$/gm, '<h3>$1</h3>');
    // Bold
    html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
    // Inline code
    html = html.replace(/`(.+?)`/g, '<code>$1</code>');
    // Code blocks
    html = html.replace(/```([\s\S]*?)```/g, '<pre><code>$1</code></pre>');
    // Unordered lists
    html = html.replace(/(?:^|\n)(?:[-*] (.+)(?:\n|$))+/g, (match) => {
      const items = match.trim().split(/\n/).map(line => {
        const m = line.match(/^[-*] (.+)$/);
        return m ? `<li>${m[1]}</li>` : '';
      }).join('');
      return `<ul>${items}</ul>`;
    });
    // Links with URL sanitization
    html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (match, label, url) => {
      const safe = this.sanitizeUrl(url);
      return safe ? `<a href="${safe}" target="_blank" rel="noopener">${label}</a>` : `<span>${label}</span>`;
    });
    // Line breaks
    html = html.replace(/\n/g, '<br>');
    return html;
  },

  /// Sanitize URL to prevent javascript: XSS (B-08/12).
  sanitizeUrl(url) {
    if (!url) return null;
    const trimmed = url.trim().toLowerCase();
    if (trimmed.startsWith('http://') || trimmed.startsWith('https://') || trimmed.startsWith('mailto:')) {
      return url.trim();
    }
    return null;
  },

  // ============================================================
  // Command Palette
  // ============================================================
  setupCommandPalette() {
    const palette = document.getElementById('commandPalette');
    const input = document.getElementById('commandInput');
    const list = document.getElementById('commandList');

    input.addEventListener('input', () => {
      this.renderCommandList(input.value);
    });

    input.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') this.hideCommandPalette();
      if (e.key === 'Enter') this.executeSelectedCommand();
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
        e.preventDefault();
        this.navigateCommandList(e.key === 'ArrowDown' ? 1 : -1);
      }
    });

    palette.addEventListener('click', (e) => {
      if (e.target === palette) this.hideCommandPalette();
    });
  },

  showCommandPalette() {
    document.getElementById('commandPalette').classList.add('active');
    document.getElementById('commandInput').value = '';
    document.getElementById('commandInput').focus();
    this.renderCommandList('');
  },

  hideCommandPalette() {
    document.getElementById('commandPalette').classList.remove('active');
  },

  renderCommandList(query) {
    const list = document.getElementById('commandList');
    const q = query.toLowerCase();
    const filtered = this.commands.filter(c => c.label.toLowerCase().includes(q));

    list.innerHTML = filtered.map((c, i) => `
      <div class="command-item${i === 0 ? ' selected' : ''}" data-index="${i}" data-id="${c.id}">
        <span>${c.label}</span>
        ${c.key ? `<span class="command-item-key">${c.key}</span>` : ''}
      </div>
    `).join('');

    list.querySelectorAll('.command-item').forEach(el => {
      el.addEventListener('click', () => {
        const cmd = this.commands.find(c => c.id === el.dataset.id);
        if (cmd) { this.hideCommandPalette(); cmd.action(); }
      });
    });
  },

  navigateCommandList(dir) {
    const items = document.querySelectorAll('.command-item');
    if (!items.length) return;
    const current = document.querySelector('.command-item.selected');
    let idx = current ? Array.from(items).indexOf(current) : -1;
    idx = Math.max(0, Math.min(items.length - 1, idx + dir));
    items.forEach(el => el.classList.remove('selected'));
    items[idx].classList.add('selected');
    items[idx].scrollIntoView({ block: 'nearest' });
  },

  executeSelectedCommand() {
    const selected = document.querySelector('.command-item.selected');
    if (!selected) return;
    const cmd = this.commands.find(c => c.id === selected.dataset.id);
    if (cmd) { this.hideCommandPalette(); cmd.action(); }
  },

  // ============================================================
  // Keyboard Shortcuts
  // ============================================================
  setupKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
      // Ctrl+Shift+P — Command Palette
      if (e.ctrlKey && e.shiftKey && e.key === 'P') {
        e.preventDefault();
        this.showCommandPalette();
      }
      // Ctrl+Shift+E — Explorer
      if (e.ctrlKey && e.shiftKey && e.key === 'E') {
        e.preventDefault();
        this.showSidebar('explorer');
      }
      // Ctrl+Shift+F — Search
      if (e.ctrlKey && e.shiftKey && e.key === 'F') {
        e.preventDefault();
        this.showSidebar('search');
      }
      // Ctrl+Shift+G — Git
      if (e.ctrlKey && e.shiftKey && e.key === 'G') {
        e.preventDefault();
        this.showSidebar('git');
      }
      // Ctrl+Shift+A — Agent Trace
      if (e.ctrlKey && e.shiftKey && e.key === 'A') {
        e.preventDefault();
        this.showSidebar('agent-trace');
      }
      // Ctrl+Shift+X — Extensions
      if (e.ctrlKey && e.shiftKey && e.key === 'X') {
        e.preventDefault();
        this.showSidebar('extensions');
      }
      // Ctrl+Shift+S — Settings
      if (e.ctrlKey && e.shiftKey && e.key === 'S') {
        e.preventDefault();
        this.showSidebar('settings');
      }
      // Ctrl+Shift+C — Chat Sessions
      if (e.ctrlKey && e.shiftKey && e.key === 'C') {
        e.preventDefault();
        this.showSidebar('chat-sessions');
      }
      // Ctrl+B — Toggle Sidebar
      if (e.ctrlKey && e.key === 'b') {
        e.preventDefault();
        this.toggleSidebar();
      }
      // Escape — close palette
      if (e.key === 'Escape') {
        this.hideCommandPalette();
      }
    });
  },

  // ============================================================
  // Status Bar
  // ============================================================
  setupStatusBar() {
    this.updateStatusBar();
    const tokensEl = document.getElementById('statusTokens');
    if (tokensEl) {
      tokensEl.style.cursor = 'pointer';
      tokensEl.addEventListener('click', () => {
        this.showCumulative = !this.showCumulative;
        this.updateTokenDisplay();
      });
    }
  },

  updateStatusBar() {
    const lang = document.getElementById('statusLang');
    const cursor = document.getElementById('statusCursor');
    if (lang) lang.textContent = 'Rust';
    if (cursor) cursor.textContent = '';
  },

  // ============================================================
  // Phase 4 Day 3: Inline Edit Panel
  // ============================================================
  setupInlineEditPanel() {
    document.getElementById('acceptAllEditsBtn')?.addEventListener('click', () => this.acceptAllEdits());
    document.getElementById('rejectAllEditsBtn')?.addEventListener('click', () => this.rejectAllEdits());
    document.getElementById('closeEditPanelBtn')?.addEventListener('click', () => this.hideEditPanel());
    this.currentEditPayload = null;
  },

  onEditProposed(event) {
    if (!event.edit_payload) return;
    try {
      const edit = JSON.parse(event.edit_payload);
      this.currentEditPayload = edit;
      this.showEditPanel(edit);
    } catch (e) {
      console.error('Failed to parse edit payload:', e);
    }
  },

  showEditPanel(edit) {
    const panel = document.getElementById('inlineEditPanel');
    const summary = document.getElementById('inlineEditSummary');
    const hunksContainer = document.getElementById('inlineEditHunks');
    if (!panel || !summary || !hunksContainer) return;
    summary.textContent = edit.summary || 'Agent 建议的修改';
    hunksContainer.innerHTML = '';
    const hunks = edit.hunks || [];
    // If hunks is a number (from emit_edit_trace), skip rendering per-hunk diff
    if (typeof hunks === 'number') {
      hunksContainer.innerHTML = `<div style="padding:8px;color:var(--fg-dim);font-size:12px;">${hunks} 个 hunk (详细内容未提供)</div>`;
    } else {
      hunks.forEach((hunk, i) => {
        const hunkEl = document.createElement('div');
        hunkEl.className = 'edit-hunk';
        const filePath = hunk.file_path || 'unknown';
        const startLine = hunk.start_line || 0;
        const oldLines = Array.isArray(hunk.old_lines) ? hunk.old_lines : [];
        const newLines = Array.isArray(hunk.new_lines) ? hunk.new_lines : [];
        hunkEl.innerHTML = `
          <div class="edit-hunk-header">
            <span class="edit-hunk-file">${this.escapeHtml(filePath)}:${startLine}</span>
            <label style="font-size:11px;display:flex;align-items:center;gap:4px;">
              <input type="checkbox" class="hunk-select" data-index="${i}" checked> Accept
            </label>
          </div>
          <div class="edit-hunk-diff">
            ${oldLines.map(l => `<div class="diff-del">-${this.escapeHtml(l)}</div>`).join('')}
            ${newLines.map(l => `<div class="diff-add">+${this.escapeHtml(l)}</div>`).join('')}
          </div>
        `;
        hunksContainer.appendChild(hunkEl);
      });
    }
    panel.style.display = 'flex';
  },

  hideEditPanel() {
    const panel = document.getElementById('inlineEditPanel');
    if (panel) panel.style.display = 'none';
    this.currentEditPayload = null;
  },

  async acceptAllEdits() {
    const tauri = window.__TAURI__;
    if (!tauri) { this.showErrorToast('Tauri 不可用'); return; }
    const invoke = tauri.core?.invoke || tauri.invoke;
    const checked = document.querySelectorAll('.hunk-select:checked');
    if (!checked.length || !this.currentEditPayload || typeof this.currentEditPayload.hunks === 'number') {
      this.hideEditPanel();
      return;
    }
    const edits = Array.from(checked).map(cb => {
      const idx = parseInt(cb.dataset.index);
      const hunk = this.currentEditPayload.hunks[idx];
      return {
        path: hunk.file_path,
        old_string: (hunk.old_lines || []).join('\n'),
        new_string: (hunk.new_lines || []).join('\n'),
      };
    });
    try {
      await invoke('apply_edits', { edits });
      this.showErrorToast('修改已应用');
      this.hideEditPanel();
      // Refresh open file if affected
      if (this.activeTab && this.activeTab !== 'welcome') {
        this.openFile(this.activeTab);
      }
    } catch (e) {
      this.showErrorToast('应用失败: ' + (e.message || e));
    }
  },

  rejectAllEdits() {
    this.hideEditPanel();
  },

  // ============================================================
  // Phase 4 Day 5: Command Palette & Advanced Observability
  // ============================================================
  setupTraceTabs() {
    document.querySelectorAll('.trace-tab').forEach(tab => {
      tab.addEventListener('click', () => {
        document.querySelectorAll('.trace-tab').forEach(t => t.classList.remove('active'));
        tab.classList.add('active');
        const name = tab.dataset.tab;
        document.getElementById('tracePanel').style.display = name === 'trace' ? 'block' : 'none';
        document.getElementById('editHistoryPanel').style.display = name === 'edit-history' ? 'block' : 'none';
        if (name === 'edit-history') this.loadEditHistory();
      });
    });
  },

  setupSessionReplay() {
    document.getElementById('replayPrevBtn')?.addEventListener('click', () => this.replayStep(-1));
    document.getElementById('replayNextBtn')?.addEventListener('click', () => this.replayStep(1));
    document.getElementById('replayCloseBtn')?.addEventListener('click', () => this.closeSessionReplay());
    this.replayIndex = -1;
    this.replayEvents = [];
  },

  async runAgentCommand(cmd) {
    const tauri = window.__TAURI__;
    if (!tauri) { this.showErrorToast('Tauri 不可用'); return; }
    try {
      const result = await tauri.core.invoke('run_agent_command', { cmd });
      this.showErrorToast(result);
    } catch (e) {
      this.showErrorToast('命令失败: ' + (e.message || e));
    }
  },

  showEditHistoryTab() {
    this.showSidebar('agent-trace');
    const tab = document.querySelector('.trace-tab[data-tab="edit-history"]');
    if (tab) tab.click();
  },

  async loadEditHistory() {
    const panel = document.getElementById('editHistoryPanel');
    if (!panel) return;
    const tauri = window.__TAURI__;
    if (!tauri) {
      panel.innerHTML = '<div style="color:var(--fg-dim);text-align:center;padding:20px;">Tauri 不可用</div>';
      return;
    }
    try {
      const entries = await tauri.core.invoke('get_edit_history');
      this.renderEditHistory(entries);
    } catch (e) {
      panel.innerHTML = '<div style="color:var(--fg-dim);text-align:center;padding:20px;">加载失败</div>';
    }
  },

  renderEditHistory(entries) {
    const panel = document.getElementById('editHistoryPanel');
    if (!panel) return;
    if (!entries || entries.length === 0) {
      panel.innerHTML = '<div class="edit-history-empty" style="color:var(--fg-dim);text-align:center;padding:20px;">暂无编辑历史</div>';
      return;
    }
    const colors = { EditProposed: 'var(--fg-red)', EditApplied: 'var(--fg-green)', EditRejected: 'var(--fg-red)' };
    panel.innerHTML = entries.slice().reverse().map((e, i) => {
      const color = colors[e.step_type] || 'var(--fg-dim)';
      const time = e.timestamp ? new Date(e.timestamp).toLocaleTimeString() : '';
      return `<div class="edit-history-item" style="border-left:3px solid ${color};padding:6px 8px;margin-bottom:6px;background:var(--bg-hover);border-radius:4px;cursor:pointer;" data-index="${entries.length - 1 - i}">
        <div style="display:flex;justify-content:space-between;align-items:center;">
          <span style="font-weight:bold;font-size:11px;color:${color};">${e.step_type}</span>
          <span style="font-size:10px;color:var(--fg-dim);">${time}</span>
        </div>
        <div style="font-size:11px;color:var(--fg-dim);margin-top:2px;">${this.escapeHtml(e.summary || '').substring(0, 120)}</div>
        ${e.confidence != null ? `<div style="font-size:10px;color:var(--fg-dim);margin-top:2px;">confidence: ${e.confidence.toFixed(2)}</div>` : ''}
      </div>`;
    }).join('');

    panel.querySelectorAll('.edit-history-item').forEach(el => {
      el.addEventListener('click', () => {
        const idx = parseInt(el.dataset.index);
        this.startSessionReplay(entries, idx);
      });
    });
  },

  startSessionReplay(entries, startIndex) {
    this.replayEvents = entries;
    this.replayIndex = startIndex;
    const bar = document.getElementById('sessionReplayBar');
    if (bar) bar.style.display = 'flex';
    this.updateReplayStatus();
    // Switch to trace tab for replay context
    const traceTab = document.querySelector('.trace-tab[data-tab="trace"]');
    if (traceTab) traceTab.click();
  },

  replayStep(dir) {
    const newIdx = this.replayIndex + dir;
    if (newIdx < 0 || newIdx >= this.replayEvents.length) return;
    this.replayIndex = newIdx;
    this.updateReplayStatus();
    const ev = this.replayEvents[this.replayIndex];
    if (ev) {
      const panel = document.getElementById('tracePanel');
      if (panel) {
        const entry = document.createElement('div');
        entry.style.cssText = 'padding:4px 8px;margin:4px 0;background:var(--bg-hover);border-radius:4px;font-size:11px;border-left:3px solid var(--fg-red);';
        entry.innerHTML = `<strong>Replay [${this.replayIndex + 1}/${this.replayEvents.length}]</strong> ${this.escapeHtml(ev.step_type)}: ${this.escapeHtml(ev.summary || '').substring(0, 100)}`;
        panel.insertBefore(entry, panel.firstChild);
        if (ev.thinking_content) this.renderReplayThinking(entry, ev.thinking_content);
        if (ev.operation_summary) {
          const bar = this.createOperationSummaryBar(ev.operation_summary, ev.tool_name);
          if (bar) { bar.style.marginTop = '4px'; entry.appendChild(bar); }
        }
      }
    }
  },

  closeSessionReplay() {
    this.replayIndex = -1;
    this.replayEvents = [];
    const bar = document.getElementById('sessionReplayBar');
    if (bar) bar.style.display = 'none';
  },

  updateReplayStatus() {
    const el = document.getElementById('replayStatus');
    if (el) el.textContent = `${this.replayIndex + 1} / ${this.replayEvents.length}`;
  },

  // ============================================================
  // Timeline Event Model (B-12/12)
  // ============================================================

  /// Build a unified TimelineEvent from type and payload.
  /// Types: user_message, agent_thinking, agent_action, trace_step, tool_result.
  buildTimelineEvent(type, payload) {
    return {
      id: 'tl-' + Date.now() + '-' + Math.random().toString(36).slice(2, 6),
      timestamp: Date.now(),
      type: type || 'trace_step',
      payload: payload || {},
      source: payload?.agent_id || 'unknown'
    };
  },

  /// Convert traceEvents into a unified timeline, optionally filtered.
  /// filter: 'all' | 'thinking' | 'action'.
  getTimelineEvents(filter) {
    const events = this.traceEvents.map(ev => {
      let type = 'trace_step';
      if (ev.thinking_content) type = 'agent_thinking';
      else if (ev.operation_summary) type = 'agent_action';
      else if (ev.tool_name) type = 'tool_result';
      return this.buildTimelineEvent(type, ev);
    });
    if (filter === 'thinking') return events.filter(e => e.type === 'agent_thinking');
    if (filter === 'action') return events.filter(e => e.type === 'agent_action' || e.type === 'tool_result');
    return events;
  },

  /// Render thinking content inside a Replay entry (B-12/12).
  renderReplayThinking(container, thinking) {
    if (!container || !thinking) return;
    const div = document.createElement('div');
    div.style.cssText = 'margin-top:4px;padding:4px;border-left:2px solid var(--fg-cyan);font-size:11px;color:var(--fg-dim);';
    div.innerHTML = `<strong>Thinking:</strong> ${this.renderMarkdown(thinking.substring(0, 200))}`;
    container.appendChild(div);
  },

  async updateMetrics() {
    const tauri = window.__TAURI__;
    if (!tauri) {
      document.getElementById('metricIteration').textContent = 'N/A';
      return;
    }
    try {
      const m = await tauri.core.invoke('get_resource_metrics');
      document.getElementById('metricIteration').textContent = m.iteration_count != null ? m.iteration_count : 'N/A';
      document.getElementById('metricBlackboard').textContent = m.blackboard_size != null ? m.blackboard_size : 'N/A';
      document.getElementById('metricFailureRate').textContent = m.failure_rate_percent != null ? m.failure_rate_percent.toFixed(1) + '%' : 'N/A';
      document.getElementById('metricLatency').textContent = m.callback_latency_ms != null ? m.callback_latency_ms + 'ms' : 'N/A';
      // Phase 4 Day 5: Edit metrics
      const editCount = document.getElementById('metricEditCount');
      if (editCount) editCount.textContent = m.edit_count != null ? m.edit_count : '0';
      const appliedCount = document.getElementById('metricAppliedCount');
      if (appliedCount) appliedCount.textContent = m.applied_count != null ? m.applied_count : '0';
    } catch (e) {}
  },

  // ── Codex-style utilities ──
  fmtElapsedCompact(seconds) {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) {
      const m = Math.floor(seconds / 60);
      const s = seconds % 60;
      return `${m}m ${s.toString().padStart(2, '0')}s`;
    }
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    return `${h}h ${m.toString().padStart(2, '0')}m ${s.toString().padStart(2, '0')}s`;
  },

  truncatePathMiddle(path, maxLen = 40) {
    if (path.length <= maxLen) return path;
    const half = Math.floor((maxLen - 1) / 2);
    return path.slice(0, half) + '…' + path.slice(-half);
  },

  _spinnerInterval: null,
  _spinnerEl: null,
  startSpinner(el) {
    const frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let idx = 0;
    this.stopSpinner();
    this._spinnerEl = el;
    el.textContent = frames[0];
    this._spinnerInterval = setInterval(() => {
      idx = (idx + 1) % frames.length;
      if (this._spinnerEl) this._spinnerEl.textContent = frames[idx];
    }, 80);
  },

  stopSpinner() {
    if (this._spinnerInterval) {
      clearInterval(this._spinnerInterval);
      this._spinnerInterval = null;
    }
    this._spinnerEl = null;
  },

  setShimmer(el, enabled) {
    if (!el) return;
    if (enabled) {
      el.classList.add('shimmer-text');
    } else {
      el.classList.remove('shimmer-text');
    }
  },

  _statusIndicatorTimer: null,
  _statusIndicatorStart: null,

  showStatusIndicator(header = 'working') {
    const indicator = document.getElementById('statusIndicatorRow');
    const spinner = document.getElementById('statusSpinner');
    const headerEl = document.getElementById('statusHeader');
    if (!indicator) return;
    indicator.classList.remove('hidden');
    if (headerEl) {
      headerEl.textContent = header;
      this.setShimmer(headerEl, true);
    }
    if (spinner) this.startSpinner(spinner);
    this._statusIndicatorStart = Date.now();
    this._statusIndicatorTimer = setInterval(() => {
      const elapsed = document.getElementById('statusElapsed');
      if (elapsed && this._statusIndicatorStart) {
        const secs = Math.floor((Date.now() - this._statusIndicatorStart) / 1000);
        elapsed.textContent = `(${this.fmtElapsedCompact(secs)} · esc to interrupt)`;
      }
    }, 1000);
  },

  hideStatusIndicator() {
    const indicator = document.getElementById('statusIndicatorRow');
    const headerEl = document.getElementById('statusHeader');
    if (indicator) indicator.classList.add('hidden');
    if (headerEl) this.setShimmer(headerEl, false);
    this.stopSpinner();
    if (this._statusIndicatorTimer) {
      clearInterval(this._statusIndicatorTimer);
      this._statusIndicatorTimer = null;
    }
    this._statusIndicatorStart = null;
  },
};

// D3-MINIMAL-FIX (redteam): bind key zombie buttons with real handlers + loading states.
// replayPrevBtn/replayNextBtn already bound in setupSessionReplay(); acceptAllEditsBtn in setupInlineEditPanel().
  const bindZombieBtns = () => {
    const setLoading = (id, loading) => {
      const el = document.getElementById(id);
      if (!el) return;
      el.disabled = loading;
      el.classList.toggle('loading', loading);
    };
    document.getElementById('testProviderBtn')?.addEventListener('click', async () => {
      try {
        setLoading('testProviderBtn', true);
        const id = document.getElementById('providerId').value.trim() || 'provider-' + Date.now();
        const name = document.getElementById('providerName').value.trim();
        const providerType = document.getElementById('providerModalType').value;
        const baseUrl = document.getElementById('providerBaseUrl').value.trim();
        const model = document.getElementById('providerModel').value.trim();
        const apiKey = document.getElementById('providerApiKey').value.trim();
        if (!name) { if (app.showErrorToast) app.showErrorToast('Provider name required'); return; }
        const config = { id, name, providerType, baseUrl, apiKey, model };
        const tauri = window.__TAURI__;
        if (!tauri) { if (app.showErrorToast) app.showErrorToast('Tauri not available'); return; }
        const invoke = tauri.core?.invoke || tauri.invoke;
        const result = await invoke('validate_provider', { config });
        if (app.showErrorToast) app.showErrorToast(result);
      } catch (e) {
        if (app.showErrorToast) app.showErrorToast('Provider validation failed: ' + (e.message || e));
      } finally {
        setLoading('testProviderBtn', false);
      }
    });
    document.getElementById('gitCommitBtn')?.addEventListener('click', () => app.gitCommit());
  };
  bindZombieBtns(); // one-time bind post-init
  app.init(); // Initialize the app
