(function () {
  const HajimiModelPickerController = {
    setupModelPicker(app) {
      const btn = document.getElementById('modelSelectBtn');
      const closeBtn = document.getElementById('modelPickerClose');
      const addBtn = document.getElementById('modelPickerAddBtn');
      const modal = document.getElementById('modelPickerModal');

      if (btn) btn.addEventListener('click', () => this.openModelPicker(app));
      if (closeBtn) closeBtn.addEventListener('click', () => this.closeModelPicker(app));
      if (addBtn) addBtn.addEventListener('click', () => {
        this.closeModelPicker(app);
        if (typeof app.openProviderModal === 'function') {
          app.openProviderModal();
        }
      });
      if (modal) {
        modal.addEventListener('click', (e) => {
          if (e.target === modal) this.closeModelPicker(app);
        });
      }
    },

    openModelPicker(app) {
      if (window.HajimiModelPickerView?.renderModelPicker) {
        window.HajimiModelPickerView.renderModelPicker(app);
      } else {
        app.renderModelPicker();
      }
      document.getElementById('modelPickerModal')?.classList.add('active');
    },

    closeModelPicker(app) {
      document.getElementById('modelPickerModal')?.classList.remove('active');
    }
  };

  window.HajimiModelPickerController = HajimiModelPickerController;
})();
