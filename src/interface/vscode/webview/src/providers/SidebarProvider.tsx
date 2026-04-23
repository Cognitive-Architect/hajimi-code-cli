import React, { useState, useCallback, useEffect } from 'react';
import { ChatInterface } from '../components/ChatInterface';
import { ThinkingTrace } from '../components/ThinkingTrace';
import { DiffPreview } from '../components/DiffPreview';
import { LoadingSpinner, TraceSkeleton } from '../components/LoadingStates';
import { vscodeApi } from '../index';
import type { ChatMessage, TraceStep, EditState, OnboardingState } from '../types/webview';
import { TOOLS } from '../constants/tools';

/** Auto-dismiss hook for transient toast notifications.
 *  Returns the last non-null value until the delay expires,
 *  then returns null. Useful for edit-result and feedback toasts. */
function useAutoDismiss<T>(value: T | null, delay = 3000): T | null {
  const [shown, setShown] = useState<T | null>(value);
  useEffect(() => {
    setShown(value);
    if (!value) return;
    const t = setTimeout(() => setShown(null), delay);
    return () => clearTimeout(t);
  }, [value, delay]);
  return shown;
}

/** SidebarProvider — Root layout container for the Hajimi sidebar webview.
 *  3:2 split: Chat (left) + Trace/Diff/Tools/Onboarding (right).
 *  Bridges extension host messages to the React component tree.
 */
export const SidebarProvider: React.FC = () => {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [currentTrace, setCurrentTrace] = useState<TraceStep[]>([]);
  const [streamingId, setStreamingId] = useState<string | null>(null);
  const [editState, setEditState] = useState<EditState>({ diff: '', isStreaming: false, error: null, mode: 'preview' });
  const [editResult, setEditResult] = useState<{ success: boolean; message: string } | null>(null);
  const [feedbackToast, setFeedbackToast] = useState<string | null>(null);
  const [canUndo, setCanUndo] = useState(false);
  const [onboarding, setOnboarding] = useState<OnboardingState | null>(null);
  const shownEditResult = useAutoDismiss(editResult);
  const shownFeedback = useAutoDismiss(feedbackToast);

  // Extension message listener
  useEffect(() => {
    const handler = (event: MessageEvent) => {
      const msg = event.data as { type: string; payload?: unknown };
      switch (msg.type) {
        case 'streamChunk': {
          const { text } = (msg.payload ?? {}) as { text?: string };
          if (text && streamingId) { setMessages((prev) => prev.map((m) => (m.id === streamingId ? { ...m, content: text, status: 'streaming' as const } : m))); }
          break;
        }
        case 'streamComplete': {
          const { text } = (msg.payload ?? {}) as { text?: string };
          if (streamingId) { setMessages((prev) => prev.map((m) => (m.id === streamingId ? { ...m, content: text ?? m.content, status: 'complete' as const } : m))); setIsLoading(false); setStreamingId(null); }
          break;
        }
        case 'streamError': {
          const { error } = (msg.payload ?? {}) as { error?: string };
          if (streamingId) { setMessages((prev) => prev.map((m) => (m.id === streamingId ? { ...m, content: error ?? 'Error', status: 'error' as const } : m))); setIsLoading(false); setStreamingId(null); }
          break;
        }
        case 'toolResult': {
          const { success, toolId } = (msg.payload ?? {}) as { success?: boolean; toolId?: string };
          if (toolId) { const rt = success ? `Tool **${toolId}** completed successfully.` : `Tool **${toolId}** failed.`; setMessages((prev) => [...prev, { id: `tool-${Date.now()}`, role: 'system', content: rt, timestamp: Date.now(), status: 'complete' }]); }
          break;
        }
        case 'editResult': {
          const { success } = (msg.payload ?? {}) as { success?: boolean };
          setEditResult({ success: !!success, message: success ? 'Edits applied successfully' : 'Failed to apply edits' });
          if (success) setEditState({ diff: '', isStreaming: false, error: null, mode: editState.mode });
          break;
        }
        case 'feedbackResult': {
          const { success, storedCount } = (msg.payload ?? {}) as { success?: boolean; storedCount?: number };
          setFeedbackToast(success ? `Feedback recorded (${storedCount ?? 0} items)` : 'Feedback failed');
          break;
        }
        case 'undoResult': {
          const { success } = (msg.payload ?? {}) as { success?: boolean };
          setFeedbackToast(success ? 'Undo successful' : 'Nothing to undo');
          setCanUndo(success ?? false);
          break;
        }
        case 'onboardingState': { setOnboarding(msg.payload as OnboardingState); break; }
        case 'traceError': { break; }
        default: break;
      }
    };
    window.addEventListener('message', handler);
    return () => window.removeEventListener('message', handler);
  }, [streamingId, editState.mode]);

  const handleAgentRequest = useCallback((text: string) => {
    const um: ChatMessage = { id: `user-${Date.now()}`, role: 'user', content: text, timestamp: Date.now(), status: 'complete' };
    setMessages((prev) => [...prev, um]);
    setIsLoading(true);
    const aid = `assistant-${Date.now()}`;
    setStreamingId(aid);
    setMessages((prev) => [...prev, { id: aid, role: 'assistant', content: '', timestamp: Date.now(), status: 'streaming' }]);
    setEditState({ diff: '', isStreaming: false, error: null, mode: 'preview' });
    setEditResult(null);
    vscodeApi.postMessage({ type: 'sendMessage', payload: { text } });
  }, []);

  const handleTraceUpdate = useCallback((trace: TraceStep[]) => { setCurrentTrace(trace); }, []);
  const handleEditUpdate = useCallback((edit: EditState) => { setEditState((prev) => ({ ...prev, ...edit })); }, []);

  // Edit action handlers — bridge to extension host StreamingEditEngine
  const handleAccept = useCallback(() => { vscodeApi.postMessage({ type: 'applyEdits', payload: {} }); }, []);
  const handleReject = useCallback(() => { vscodeApi.postMessage({ type: 'rejectEdits', payload: {} }); setEditState({ diff: '', isStreaming: false, error: null, mode: 'preview' }); setEditResult(null); }, []);
  const handleCancel = useCallback(() => { vscodeApi.postMessage({ type: 'cancelEdit', payload: {} }); setEditState((prev) => ({ ...prev, isStreaming: false })); }, []);
  const handleUndo = useCallback(() => { vscodeApi.postMessage({ type: 'requestUndo', payload: {} }); }, []);
  const handleModeChange = useCallback((mode: 'preview' | 'live') => { setEditState((prev) => ({ ...prev, mode })); vscodeApi.postMessage({ type: 'setEditMode', payload: { mode } }); }, []);
  const handleToolExecute = useCallback((toolId: string) => { vscodeApi.postMessage({ type: 'executeTool', payload: { toolId } }); }, []);
  const dismissOnboarding = useCallback(() => { setOnboarding(null); vscodeApi.postMessage({ type: 'dismissOnboarding', payload: {} }); }, []);

  // Merge edit result and feedback toasts into a single display slot
  // Merge edit result and feedback toasts into a single display slot.
  // Priority: editResult > feedback > none.
  const toast = shownEditResult
    ? { text: shownEditResult.message, cls: shownEditResult.success ? 'bg-green-600/20 text-green-400' : 'bg-red-600/20 text-red-400' }
    : shownFeedback
    ? { text: shownFeedback, cls: 'bg-blue-600/20 text-blue-400' }
    : null;

  return (
    <div className="flex h-full w-full overflow-hidden">
      <div className="flex w-3/5 flex-col border-r border-[var(--vscode-panel-border)]">
        <ChatInterface onSubmit={handleAgentRequest} onTraceUpdate={handleTraceUpdate} onEditUpdate={handleEditUpdate} />
      </div>
      <div className="flex w-2/5 flex-col overflow-y-auto bg-[var(--vscode-sidebar-background)] p-2">
        {/* Onboarding welcome panel — Week 5 */}
        {onboarding && (
          <div className="mb-2 rounded bg-[var(--vscode-list-hoverBackground)] p-2">
            <div className="flex items-center justify-between mb-1">
              <h3 className="text-[10px] font-semibold text-[var(--vscode-foreground)]">{onboarding.welcome.emoji} {onboarding.welcome.title}</h3>
              <button onClick={dismissOnboarding} className="text-[10px] text-[var(--vscode-descriptionForeground)] hover:text-[var(--vscode-foreground)] px-1" title="Dismiss">✕</button>
            </div>
            <p className="text-[9px] text-[var(--vscode-descriptionForeground)] mb-1.5">{onboarding.welcome.body}</p>
            <div className="flex flex-col gap-1">
              {onboarding.examples.map((ex) => (
                <button key={ex.id} onClick={() => handleAgentRequest(ex.text)} className="flex items-center gap-1.5 rounded px-1.5 py-1 text-[9px] text-left text-[var(--vscode-foreground)] hover:bg-[var(--vscode-list-activeSelectionBackground)]" title={ex.text}><span>{ex.icon}</span><span className="truncate">{ex.label}</span></button>
              ))}
            </div>
          </div>
        )}
        {/* Quick tools grid */}
        <div className="mb-2 grid grid-cols-2 gap-1">
          {TOOLS.map((tool) => (
            <button key={tool.id} onClick={() => handleToolExecute(tool.id)} className="flex items-center gap-1.5 rounded px-2 py-1.5 text-[10px] text-[var(--vscode-foreground)] hover:bg-[var(--vscode-list-hoverBackground)]" title={tool.name}><span className={`codicon codicon-${tool.icon}`} style={{ fontFamily: 'codicon', fontSize: 12 }}>◆</span><span className="truncate">{tool.name}</span></button>
          ))}
        </div>
        {/* Thinking trace — Week 6: show skeleton when loading and trace is empty */}
        <div className="flex-1 min-h-0">
          {currentTrace.length === 0 && isLoading ? (
            <div className="flex flex-col items-center justify-center h-full gap-2">
              <LoadingSpinner size={20} />
              <span className="text-[10px]" style={{ color: 'var(--vscode-descriptionForeground)' }}>Agent is thinking...</span>
              <TraceSkeleton />
            </div>
          ) : (
            <ThinkingTrace trace={currentTrace} />
          )}
        </div>
        {/* Diff preview */}
        <div className="mt-2 flex-1 min-h-0"><DiffPreview diff={editState.diff} mode={editState.mode} isStreaming={editState.isStreaming} error={editState.error} onAccept={handleAccept} onReject={handleReject} onCancel={handleCancel} onModeChange={handleModeChange} /></div>
        {/* Unified toast */}
        {toast && (<div className={`mt-2 px-2 py-1 rounded text-[10px] text-center ${toast.cls}`}>{toast.text}</div>)}
        {/* Undo button */}
        {canUndo && (<button onClick={handleUndo} className="mt-2 w-full text-[10px] px-2 py-1 rounded bg-[var(--vscode-button-secondaryBackground)] text-[var(--vscode-button-secondaryForeground)] hover:opacity-80 transition-opacity" title="Undo last edit">↩ Undo Last Edit</button>)}
      </div>
    </div>
  );
};
