(function () {
  'use strict';

  const HajimiSettingsPanel = {
    init(app) {
      this.setupSettingsTabs(app);
    },

    setupSettingsTabs(app) {
      const tabs = document.querySelectorAll('.settings-tab');
      tabs.forEach(tab => {
        tab.addEventListener('click', () => {
          this.switchSettingsTab(app, tab.dataset.tab);
        });
      });
    },

    showSidebar(app, view) {
      // Redirect old views to settings tabs
      if (view === 'models' || view === 'system') {
        this.showSidebar(app, 'settings');
        this.switchSettingsTab(app, view === 'models' ? 'providers' : 'governance');
        return;
      }

      app.sidebarView = view;
      document.querySelectorAll('.activity-item').forEach(el => {
        el.classList.toggle('active', el.dataset.view === view);
      });
      document.querySelectorAll('.sidebar-panel').forEach(el => {
        el.classList.toggle('active', el.dataset.panel === view);
      });
      if (view === 'git') {
        app.loadGitStatus();
      }
      if (view === 'settings') {
        app.loadProviders();
        app.loadAgentProviders();
        app.loadMcpServers();
      }
    },

    switchSettingsTab(app, tabId) {
      document.querySelectorAll('.settings-tab').forEach(el => {
        el.classList.toggle('active', el.dataset.tab === tabId);
      });
      document.querySelectorAll('.settings-tab-panel').forEach(el => {
        const isActive = el.dataset.settingsPanel === tabId;
        el.classList.toggle('active', isActive);
        el.style.display = isActive ? 'block' : 'none';
      });

      if (tabId === 'providers') {
        app.loadProviders();
        app.loadAgentProviders();
      } else if (tabId === 'mcp') {
        app.loadMcpServers();
      } else if (tabId === 'governance') {
        // Governance logic if needed
      } else if (tabId === 'audit') {
        app.loadCheckpoints();
        app.loadAuditLogs();
      }
    }
  };

  // Mount to global window namespace
  window.HajimiSettingsPanel = HajimiSettingsPanel;
})();
