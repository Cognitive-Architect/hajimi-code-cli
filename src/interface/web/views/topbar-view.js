(function () {
  const HajimiTopbarView = {
    renderTopBarWorkspace(app) {
      const projectEl = document.getElementById('topBarProject');
      if (!projectEl) return;
      if (!app.currentWorkspace) {
        projectEl.textContent = 'hajimi-code-cli';
        projectEl.title = '';
        return;
      }
      const normalized = String(app.currentWorkspace).replace(/[\\/]+$/, '');
      projectEl.textContent = normalized.split(/[\\/]/).pop() || normalized || 'workspace';
      projectEl.title = app.currentWorkspace;
    },

    renderLiveShellState(app, statusText) {
      this.renderTopBarWorkspace(app);
      if (typeof app.renderChatShellStatus === 'function') {
        app.renderChatShellStatus(statusText);
      }
      if (typeof app.renderSidebarFileSummary === 'function') {
        app.renderSidebarFileSummary();
      }
      if (typeof app.renderSidebarModelSummary === 'function') {
        app.renderSidebarModelSummary();
      }
      if (typeof app.renderSidebarMcpSummary === 'function') {
        app.renderSidebarMcpSummary();
      }
      if (typeof app.renderInspectorTaskStatus === 'function') {
        app.renderInspectorTaskStatus(statusText);
      }
      if (typeof app.renderInspectorSessionStats === 'function') {
        app.renderInspectorSessionStats();
      }
    },

    updateGitBranch(app, gitStatusOutput) {
      if (!app.isTauriAvailable()) return;
      app.runShellCommand('git', ['branch', '--show-current'])
        .then(result => {
          const branch = (result.stdout || result).trim();
          const statusBranch = document.getElementById('statusBranch');
          if (statusBranch && branch) {
            statusBranch.textContent = `🌿 ${branch}`;
          }
          const topBarBranch = document.getElementById('topBarBranch');
          if (topBarBranch && branch) topBarBranch.textContent = branch;
          const sidebarGitBranch = document.getElementById('sidebarGitBranch');
          if (sidebarGitBranch && branch) sidebarGitBranch.textContent = branch;
          app.renderLiveShellState();
        })
        .catch(() => {
          // Keep existing branch name on error
        });
    }
  };

  window.HajimiTopbarView = HajimiTopbarView;
})();
