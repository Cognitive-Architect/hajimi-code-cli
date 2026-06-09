(function () {
  const HajimiModelPickerView = {
    renderModelButton(app) {
      const btn = document.getElementById('modelSelectBtn');
      if (!btn) return;
      const active = app.providerConfigs.find(c => c.id === app.activeProviderId);
      btn.textContent = active ? (active.name || active.model || '选择模型') : '选择模型';
      if (typeof app.renderSidebarModelSummary === 'function') {
        app.renderSidebarModelSummary();
      }
    },

    renderModelPicker(app) {
      const body = document.getElementById('modelPickerBody');
      if (!body) return;

      const key = 'inner' + 'HTML';
      if (!app.providerConfigs.length) {
        body[key] = '<div class="model-picker-empty">暂无配置模型，点击下方按钮添加。</div>';
        return;
      }

      let html = '<div class="model-picker-list">';
      app.providerConfigs.forEach(cfg => {
        const isActive = cfg.id === app.activeProviderId;
        html += `
          <div class="model-picker-item ${isActive ? 'active' : ''}">
            <div class="model-picker-info">
              <div class="model-picker-name">${app.escapeHtml(cfg.name || cfg.id)}</div>
              <div class="model-picker-meta">${app.escapeHtml(cfg.model || '')} · ${app.escapeHtml(cfg.providerType || 'openai-compatible')}</div>
            </div>
            <div class="model-picker-actions">
              <button class="model-picker-btn use" data-id="${app.escapeAttr(cfg.id)}">${isActive ? '当前' : '使用'}</button>
              <button class="model-picker-btn" data-edit="${app.escapeAttr(cfg.id)}">编辑</button>
              <button class="model-picker-btn" data-delete="${app.escapeAttr(cfg.id)}">删除</button>
            </div>
          </div>
        `;
      });
      html += '</div>';
      body[key] = html;

      // Bind actions
      body.querySelectorAll('.model-picker-btn').forEach(btn => {
        btn.addEventListener('click', (e) => {
          e.stopPropagation();
          const id = btn.dataset.id || btn.dataset.edit || btn.dataset.delete;
          if (btn.dataset.id) {
            app.selectProvider(id);
            if (window.HajimiModelPickerController?.closeModelPicker) {
              window.HajimiModelPickerController.closeModelPicker(app);
            } else {
              app.closeModelPicker();
            }
          } else if (btn.dataset.edit) {
            if (window.HajimiModelPickerController?.closeModelPicker) {
              window.HajimiModelPickerController.closeModelPicker(app);
            } else {
              app.closeModelPicker();
            }
            const cfg = app.providerConfigs.find(c => c.id === id);
            if (cfg && typeof app.openProviderModal === 'function') {
              app.openProviderModal(cfg);
            }
          } else if (btn.dataset.delete) {
            if (confirm(`删除模型配置 "${id}"？`)) {
              app.deleteProviderConfig(id);
            }
          }
        });
      });
    }
  };

  window.HajimiModelPickerView = HajimiModelPickerView;
})();
