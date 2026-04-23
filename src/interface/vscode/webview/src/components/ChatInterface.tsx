import React, { useState, useEffect, useCallback } from 'react';
import { MessageList } from './MessageList';
import { InputBox } from './InputBox';
import { useMessages } from '../hooks/useMessages';
import { useIndexedDB } from '../hooks/useIndexedDB';
import { vscodeApi } from '../index';
import type { TraceStep, EditorState, EditState, ContextPreview } from '../types/webview';

/** ChatInterface — Main chat container with trace sync, editor sync, edit state,
 *  ActionButtons (Week 4), and context preview badge (Week 5). */
interface ChatInterfaceProps {
  onSubmit?: (text: string) => void;
  onTraceUpdate?: (trace: TraceStep[]) => void;
  onEditUpdate?: (edit: EditState) => void;
}

export const ChatInterface: React.FC<ChatInterfaceProps> = ({ onSubmit, onTraceUpdate, onEditUpdate }) => {
  const {
    messages, isLoading, addUserMessage, startAssistantStream,
    appendStreamChunk, completeStream, setStreamError, setAllMessages,
  } = useMessages();

  const [traceSteps, setTraceSteps] = useState<TraceStep[]>([]);
  const [editorState, setEditorState] = useState<EditorState | null>(null);
  const [contextPreview, setContextPreview] = useState<ContextPreview | null>(null);
  const [editState, setEditState] = useState<EditState>(() => {
    try {
      const saved = localStorage.getItem('hajimi-edit-mode');
      return { diff: '', isStreaming: false, error: null, mode: saved === 'live' ? 'live' : 'preview' };
    } catch { return { diff: '', isStreaming: false, error: null, mode: 'preview' }; }
  });
  const { saveMessages, loadMessages, clearMessages } = useIndexedDB();

  // Load persisted messages on mount
  useEffect(() => {
    let cancelled = false;
    loadMessages().then((msgs) => { if (!cancelled && msgs.length > 0) setAllMessages(msgs); }).catch(() => {});
    return () => { cancelled = true; };
  }, [loadMessages, setAllMessages]);

  // Persist messages to IndexedDB
  useEffect(() => { if (messages.length > 0) saveMessages(messages).catch(() => {}); }, [messages, saveMessages]);

  // Sync trace/edit upward to SidebarProvider + persist edit mode + reset trace on clear
  useEffect(() => {
    if (onTraceUpdate) onTraceUpdate(traceSteps);
    if (onEditUpdate) onEditUpdate(editState);
    try { localStorage.setItem('hajimi-edit-mode', editState.mode); } catch { /* silent */ }
    if (messages.length === 0) setTraceSteps([]);
  }, [traceSteps, editState, messages.length, onTraceUpdate, onEditUpdate]);

  // Extension message listener: stream, trace, editor, edit events, context preview
  useEffect(() => {
    const handler = (event: MessageEvent) => {
      const msg = event.data as { type: string; payload?: unknown };
      try {
        const lastA = () => [...messages].reverse().find((m) => m.role === 'assistant' && m.status === 'streaming');
        switch (msg.type) {
          case 'streamChunk': {
            const { text } = (msg.payload ?? {}) as { text?: string };
            if (text) { const la = lastA(); if (la) appendStreamChunk(la.id, text); }
            break;
          }
          case 'streamComplete': {
            const { text } = (msg.payload ?? {}) as { text?: string };
            const la = lastA(); if (la) completeStream(la.id, text);
            break;
          }
          case 'streamError': {
            const { error } = (msg.payload ?? {}) as { error?: string };
            const la = lastA(); if (la) setStreamError(la.id, error ?? 'Stream error');
            break;
          }
          case 'traceStep': {
            const p = (msg.payload ?? {}) as Partial<TraceStep>;
            if (p.step) {
              setTraceSteps((prev) => {
                const next = prev.filter((t) => t.step !== p.step);
                next.push({ step: p.step!, details: p.details ?? '', iteration: p.iteration ?? 0, timestamp: p.timestamp ?? Date.now(), status: p.status ?? 'active' });
                return next.sort((a, b) => a.iteration - b.iteration);
              });
            }
            break;
          }
          case 'traceComplete': { setTraceSteps((prev) => prev.map((t) => (t.status === 'active' ? { ...t, status: 'completed' as const } : t))); break; }
          case 'traceError': {
            const { step, error: te } = (msg.payload ?? {}) as { step?: string; error?: string };
            setTraceSteps((prev) => prev.map((t) => (t.step === step ? { ...t, status: 'error' as const, details: te ?? t.details } : t)));
            break;
          }
          case 'editorState': { setEditorState(msg.payload as EditorState); break; }
          case 'editChunk': { setEditState((prev) => ({ ...prev, isStreaming: true })); break; }
          case 'editComplete': { const { diff } = (msg.payload ?? {}) as { diff?: string }; setEditState((prev) => ({ ...prev, isStreaming: false, diff: diff ?? prev.diff })); break; }
          case 'editError': { const { error: ee } = (msg.payload ?? {}) as { error?: string }; setEditState((prev) => ({ ...prev, isStreaming: false, error: ee ?? 'Edit error' })); break; }
          case 'contextPreview': { setContextPreview(msg.payload as ContextPreview); break; }
          default: break;
        }
      } catch (err) { console.error('[ChatInterface] Message handling error:', err); }
    };
    window.addEventListener('message', handler);
    return () => window.removeEventListener('message', handler);
  }, [messages, appendStreamChunk, completeStream, setStreamError]);

  const syncWithEditor = useCallback(() => { vscodeApi.postMessage({ type: 'syncEditor', payload: {} }); }, []);

  useEffect(() => {
    if (traceSteps.length === 1 && traceSteps[0]?.status === 'active') syncWithEditor();
    const act = traceSteps.find((t) => t.step === 'Act');
    if (act?.status === 'active') { setEditState((prev) => ({ ...prev, isStreaming: true })); syncWithEditor(); }
  }, [traceSteps, syncWithEditor]);

  const handleAgentRequest = useCallback((text: string) => {
    try { setTraceSteps([]); setEditState({ diff: '', isStreaming: false, error: null, mode: 'preview' }); addUserMessage(text); startAssistantStream(); if (onSubmit) onSubmit(text); vscodeApi.postMessage({ type: 'sendMessage', payload: { text } }); }
    catch (err) { console.error('[ChatInterface] Request error:', err); }
  }, [addUserMessage, startAssistantStream, onSubmit]);

  const handleClear = useCallback(() => { setAllMessages([]); setTraceSteps([]); setEditState({ diff: '', isStreaming: false, error: null, mode: 'preview' }); setContextPreview(null); clearMessages().catch(() => {}); }, [setAllMessages, clearMessages]);

  // Week 4: ActionButtons callbacks — submit feedback to extension host
  const handleAcceptMessage = useCallback((id: string) => {
    const msg = messages.find((m) => m.id === id);
    vscodeApi.postMessage({ type: 'submitFeedback', payload: { messageId: id, choice: 'accept', context: { query: msg?.content ?? '' } } });
  }, [messages]);

  const handleRejectMessage = useCallback((id: string) => {
    const msg = messages.find((m) => m.id === id);
    vscodeApi.postMessage({ type: 'submitFeedback', payload: { messageId: id, choice: 'reject', context: { query: msg?.content ?? '' } } });
  }, [messages]);

  const handleExplainMessage = useCallback((id: string) => {
    const msg = messages.find((m) => m.id === id);
    vscodeApi.postMessage({ type: 'submitFeedback', payload: { messageId: id, choice: 'explain', context: { query: msg?.content ?? '' } } });
  }, [messages]);

  return (
    <div className="flex h-full flex-col bg-[var(--vscode-editor-background)]">
      <div className="flex items-center justify-between border-b border-[var(--vscode-panel-border)] px-3 py-2">
        <div className="flex items-center gap-2">
          <h2 className="text-xs font-semibold uppercase tracking-wider text-[var(--vscode-descriptionForeground)]">Chat</h2>
          {messages.length > 0 && <span className="text-[10px] text-[var(--vscode-descriptionForeground)] opacity-60">{messages.length} messages</span>}
        </div>
        <div className="flex items-center gap-2">
          {/* Week 5: Context preview badge */}
          {contextPreview && (
            <span
              className="text-[10px] text-[var(--vscode-descriptionForeground)] opacity-60 truncate max-w-[160px]"
              title={`${contextPreview.fileName}${contextPreview.hasSelection ? ' (selection)' : ''}`}
            >
              {contextPreview.language} · {contextPreview.fileName.split('/').pop() ?? contextPreview.fileName} · {contextPreview.lines}L
            </span>
          )}
          {traceSteps.length > 0 && editorState?.uri && <span className="text-[10px] text-[var(--vscode-descriptionForeground)] opacity-60 truncate max-w-[120px]" title={editorState.uri}>{editorState.language}</span>}
          {messages.length > 0 && <button onClick={handleClear} className="text-[10px] text-[var(--vscode-descriptionForeground)] hover:text-[var(--vscode-foreground)] px-1.5 py-0.5 rounded hover:bg-[var(--vscode-list-hoverBackground)]" title="Clear conversation">Clear</button>}
        </div>
      </div>
      <MessageList messages={messages} onAccept={handleAcceptMessage} onReject={handleRejectMessage} onExplain={handleExplainMessage} />
      <InputBox onSubmit={handleAgentRequest} disabled={isLoading} />
    </div>
  );
};
