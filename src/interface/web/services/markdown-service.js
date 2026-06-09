(function () {
  const HajimiMarkdownService = {
    safeText(text) {
      if (window.HajimiSecurityDom && typeof window.HajimiSecurityDom.safeText === 'function') {
        return window.HajimiSecurityDom.safeText(text);
      }
      // Fallback
      return String(text);
    },

    escapeAttr(text) {
      if (window.HajimiSecurityDom && typeof window.HajimiSecurityDom.escapeAttr === 'function') {
        return window.HajimiSecurityDom.escapeAttr(text);
      }
      // Fallback
      return String(text).replace(/"/g, '&quot;');
    },

    formatText(text) {
      let html = this.safeText(text)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');
      html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
      html = html.replace(/`(.+?)`/g, '<code>$1</code>');
      html = html.replace(/```([\s\S]*?)```/g, '<pre><code>$1</code></pre>');
      html = html.replace(/\n/g, '<br>');
      return html;
    },

    renderMarkdown(text) {
      let html = this.safeText(text)
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
        return safe ? `<a href="${this.escapeAttr(safe)}" target="_blank" rel="noopener">${label}</a>` : `<span>${label}</span>`;
      });
      // Line breaks
      html = html.replace(/\n/g, '<br>');
      return html;
    },

    sanitizeUrl(url) {
      if (!url) return null;
      const trimmed = url.trim().toLowerCase();
      if (trimmed.startsWith('http://') || trimmed.startsWith('https://') || trimmed.startsWith('mailto:')) {
        return url.trim();
      }
      return null;
    }
  };

  window.HajimiMarkdownService = HajimiMarkdownService;
})();
