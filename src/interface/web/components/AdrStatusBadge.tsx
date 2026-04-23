import React from 'react';

type AdrStatus = 'Accepted' | 'Proposed' | 'Deprecated' | string;

interface AdrStatusBadgeProps {
  status: AdrStatus;
  debt_id?: string;
  adr_id?: string;
  url?: string;
  onClick?: (url?: string) => void;
}

// Maps ADR status values to their display colors.
const STATUS_COLORS: Record<string, string> = {
  Accepted: '#22c55e',
  Proposed: '#eab308',
  Deprecated: '#ef4444',
};

export const AdrStatusBadge: React.FC<AdrStatusBadgeProps> = ({
  status,
  debt_id,
  adr_id,
  url,
  onClick,
}) => {
  const normalized = (status || '').trim();
  // Fallback to gray when the status is unknown or missing.
  const color = STATUS_COLORS[normalized] || '#6b7280';
  const label = normalized || 'Unknown';
  const tooltip = `ADR ${adr_id || ''} — ${label}${debt_id ? ` (debt: ${debt_id})` : ''}${url ? ` — ${url}` : ''}`;

  const handleClick = () => {
    if (url) {
      onClick ? onClick(url) : window.open(url, '_blank');
    } else if (debt_id && debt_id.startsWith('DEBT-')) {
      const debtUrl = `/debt/${debt_id}`;
      onClick ? onClick(debtUrl) : window.open(debtUrl, '_blank');
    } else if (adr_id) {
      const adrUrl = `/adr/${adr_id}`;
      onClick ? onClick(adrUrl) : window.open(adrUrl, '_blank');
    }
  };

  return (
    <span
      title={tooltip}
      onClick={handleClick}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '4px',
        padding: '2px 8px',
        borderRadius: '9999px',
        backgroundColor: color,
        color: '#fff',
        fontSize: '12px',
        fontWeight: 600,
        cursor: onClick || url || debt_id || adr_id ? 'pointer' : 'default',
      }}
    >
      <span style={{ fontSize: '10px' }}>●</span>
      {label}
    </span>
  );
};
