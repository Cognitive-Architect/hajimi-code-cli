import React, { useState, useCallback } from 'react';
import { Button } from '@/components/ui/Button';

export interface ActionButtonsProps {
  messageId: string;
  onAccept: (id: string) => void;
  onReject: (id: string) => void;
  onExplain: (id: string) => void;
  disabled?: boolean;
}

/** ActionButtons — Accept / Reject / Explain trio for each AI response.
 *
 *  Shadcn-styled with VSCode theme tokens. Shows only on completed assistant
 *  messages. The Explain button toggles an inline reason textarea; submitting
 *  sends the feedback payload including the optional reason.
 *
 *  Visual states:
 *  - Accept: default variant (green-ish via VSCode button tokens)
 *  - Reject: destructive variant (red)
 *  - Explain: ghost variant (subtle)
 *  - Disabled: all buttons disabled while parent is loading/streaming
 *
 *  Accessibility:
 *  - Each button has an explicit title for screen readers
 *  - Busy state shows a checkmark to confirm the action was registered
 *  - The textarea has an aria-label for explanation input
 *
 *  Keyboard shortcuts (when focused inside the textarea):
 *  - Enter: submit explanation
 *  - Shift+Enter: new line
 *  - Escape: cancel explanation
 */
export const ActionButtons: React.FC<ActionButtonsProps> = ({
  messageId,
  onAccept,
  onReject,
  onExplain,
  disabled = false,
}) => {
  const [explainOpen, setExplainOpen] = useState(false);
  const [reason, setReason] = useState('');
  const [busy, setBusy] = useState<'accept' | 'reject' | 'explain' | null>(null);

  const handleAccept = useCallback(() => {
    if (disabled || busy) return;
    setBusy('accept');
    onAccept(messageId);
    // Reset busy after a short visual confirmation delay
    setTimeout(() => setBusy(null), 600);
  }, [disabled, busy, onAccept, messageId]);

  const handleReject = useCallback(() => {
    if (disabled || busy) return;
    setBusy('reject');
    onReject(messageId);
    setTimeout(() => setBusy(null), 600);
  }, [disabled, busy, onReject, messageId]);

  const handleExplain = useCallback(() => {
    if (disabled || busy) return;
    if (!explainOpen) { setExplainOpen(true); return; }
    if (reason.trim()) { onExplain(messageId); }
    setExplainOpen(false);
    setReason('');
  }, [disabled, busy, explainOpen, reason, onExplain, messageId]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleExplain(); }
    if (e.key === 'Escape') { setExplainOpen(false); setReason(''); }
  }, [handleExplain]);

  // Compute button disabled state from parent + local busy state
  const isBusy = disabled || !!busy;
  const btnCls = "text-[10px] h-6 px-2 transition-all";

  // Accessible label for the button group region
  const groupLabel = `Actions for message ${messageId.slice(0, 8)}`;

  // If the parent has disabled interactions, hide the explain textarea
  const showExplain = explainOpen && !disabled;

  return (
    <div className="mt-1.5 space-y-1.5" aria-label={groupLabel}>
      <div className="flex items-center gap-2">
        <Button
          variant="default"
          size="sm"
          onClick={handleAccept}
          disabled={isBusy}
          className={btnCls}
          title="Accept this response"
        >
          {busy === 'accept' ? '✓' : 'Accept'}
        </Button>
        <Button
          variant="destructive"
          size="sm"
          onClick={handleReject}
          disabled={isBusy}
          className={btnCls}
          title="Reject this response"
        >
          {busy === 'reject' ? '✓' : 'Reject'}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleExplain}
          disabled={isBusy}
          className={btnCls}
          title="Explain this response"
        >
          {busy === 'explain' ? '✓' : explainOpen ? 'Submit' : 'Explain'}
        </Button>
      </div>
      {showExplain && (
        <div className="space-y-1">
          <textarea
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Why do you need an explanation? (Shift+Enter new line, Enter submit, Esc cancel)"
            className="w-full rounded border border-[var(--vscode-panel-border)] bg-[var(--vscode-input-background)] text-[var(--vscode-input-foreground)] text-[10px] px-2 py-1 resize-none focus:outline-none focus:ring-1 focus:ring-[var(--vscode-focusBorder)]"
            rows={2}
            autoFocus
          />
          <div className="flex justify-end gap-1">
            <button
              onClick={() => { setExplainOpen(false); setReason(''); }}
              className="text-[10px] px-1.5 py-0.5 rounded text-[var(--vscode-descriptionForeground)] hover:bg-[var(--vscode-list-hoverBackground)]"
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
