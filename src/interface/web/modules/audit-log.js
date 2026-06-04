// ============================================================
// Hajimi Audit Log Readonly Module
// ============================================================

(function (global) {
  function getDocument() {
    return global.document || (typeof document !== 'undefined' ? document : null);
  }

  function createCell(doc, text, className) {
    const cell = doc.createElement('td');
    if (className) cell.className = className;
    cell.textContent = text == null || text === '' ? '-' : String(text);
    return cell;
  }

  function clearChildren(element) {
    while (element.firstChild) {
      element.removeChild(element.firstChild);
    }
  }

  function appendEmptyRow(doc, tbody) {
    const row = doc.createElement('tr');
    const cell = createCell(doc, '暂无记录', 'audit-empty');
    cell.setAttribute('colspan', '4');
    row.appendChild(cell);
    tbody.appendChild(row);
  }

  function statusClass(status) {
    if (status === 'completed') return 'audit-status-ok';
    if (status === 'failed') return 'audit-status-err';
    return 'audit-status-start';
  }

  async function loadAuditLogs(app) {
    if (!app?.isTauriAvailable?.()) return;
    try {
      const logs = await app.invokeTauri('get_audit_logs', { limit: 100, offset: 0 });
      const doc = getDocument();
      const tbody = doc?.getElementById('auditLogBodyTab');
      if (!tbody) return;

      clearChildren(tbody);
      if (!logs || !logs.length) {
        appendEmptyRow(doc, tbody);
        return;
      }

      logs.forEach((record) => {
        const row = doc.createElement('tr');
        const time = record.timestamp ? new Date(record.timestamp).toLocaleString() : '-';
        const provider = record.providerName || record.provider_name || '-';
        const model = record.model || '-';
        const status = record.status || '';
        const statusCell = doc.createElement('td');
        const statusBadge = doc.createElement('span');
        statusBadge.className = `audit-status ${statusClass(record.status)}`;
        statusBadge.textContent = status;

        row.appendChild(createCell(doc, provider));
        row.appendChild(createCell(doc, model));
        row.appendChild(createCell(doc, time));
        statusCell.appendChild(statusBadge);
        row.appendChild(statusCell);
        tbody.appendChild(row);
      });
    } catch (error) {
      console.error('loadAuditLogs error:', error);
    }
  }

  function setupAuditLog(app) {
    const doc = getDocument();
    const refreshBtn = doc?.getElementById('refreshAuditBtnTab');
    if (refreshBtn) {
      refreshBtn.addEventListener('click', () => app.loadAuditLogs());
    }
  }

  global.HajimiAuditLog = {
    setupAuditLog,
    loadAuditLogs,
  };

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
      setupAuditLog,
      loadAuditLogs,
    };
  }
})(typeof window !== 'undefined' ? window : globalThis);
