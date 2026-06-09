(function () {
  const HajimiFeedbackView = {
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
    }
  };

  window.HajimiFeedbackView = HajimiFeedbackView;
})();
