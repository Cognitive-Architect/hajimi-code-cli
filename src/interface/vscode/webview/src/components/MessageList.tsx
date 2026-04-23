import React, { useEffect, useRef, useCallback } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { ActionButtons } from './ActionButtons';
import { EmptyState, Skeleton } from './LoadingStates';
import type { ChatMessage } from '../types/webview';
// Message history persisted to IndexedDB via useIndexedDB hook

interface MessageListProps {
  messages: ChatMessage[];
  onAccept?: (id: string) => void;
  onReject?: (id: string) => void;
  onExplain?: (id: string) => void;
}

function formatTime(timestamp: number): string {
  const d = new Date(timestamp);
  return d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
}

export const MessageList: React.FC<MessageListProps> = ({ messages, onAccept, onReject, onExplain }) => {
  const endRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleCopy = useCallback((text: string) => {
    if (navigator.clipboard) {
      navigator.clipboard.writeText(text).catch(() => {});
    }
  }, []);

  const renderMarkdown = (content: string) => (
    <ReactMarkdown
      remarkPlugins={[remarkGfm]}
      components={{
        code({ className, children, ...props }) {
          const isInline = !className;
          return (
            <code
              className={
                isInline
                  ? 'rounded bg-[var(--vscode-textCodeBlock-background)] px-1 py-0.5 text-xs font-mono'
                  : 'block rounded bg-[var(--vscode-textCodeBlock-background)] p-2 text-xs font-mono overflow-x-auto'
              }
              {...props}
            >
              {children}
            </code>
          );
        },
        pre({ children }) {
          return (
            <pre className="rounded bg-[var(--vscode-textCodeBlock-background)] p-2 overflow-x-auto">
              {children}
            </pre>
          );
        },
        p({ children }) {
          return <p className="mb-1 last:mb-0">{children}</p>;
        },
      }}
    >
      {content || ' '}
    </ReactMarkdown>
  );

  return (
    <div ref={containerRef} className="flex-1 overflow-y-auto space-y-3 px-3 py-2">
      {/* Week 6: Empty state with friendly illustration */}
      {messages.length === 0 && <EmptyState />}

      {messages.map((message) => {
        const isUser = message.role === 'user';
        const isAssistant = message.role === 'assistant';
        const isSystem = message.role === 'system';

        return (
          <div
            key={message.id}
            className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}
          >
            <div
              className={`max-w-[90%] rounded-lg px-3 py-2 text-sm ${
                isUser
                  ? 'bg-[var(--vscode-button-background)] text-[var(--vscode-button-foreground)]'
                  : isSystem
                  ? 'bg-[var(--vscode-editorWarning-background)] text-[var(--vscode-editorWarning-foreground)] text-xs'
                  : 'bg-[var(--vscode-editor-inactiveSelectionBackground)] text-[var(--vscode-foreground)]'
              }`}
            >
              {/* Role header */}
              <div className="flex items-center gap-1.5 mb-1 opacity-70">
                <span className="text-[10px] font-semibold uppercase tracking-wide">
                  {isUser ? 'You' : isAssistant ? 'Hajimi' : 'System'}
                </span>
                <span className="text-[10px] opacity-50">{formatTime(message.timestamp)}</span>
              </div>

              {/* Content */}
              {isUser || isSystem ? (
                <p className="whitespace-pre-wrap break-words">{message.content}</p>
              ) : (
                <div className="prose prose-invert prose-sm max-w-none">
                  {renderMarkdown(message.content)}
                </div>
              )}

              {/* Week 6: Skeleton loading animation for streaming assistant messages */}
              {message.status === 'streaming' && isAssistant && (
                <div className="mt-2">
                  <Skeleton lines={2} />
                  <div className="mt-1 flex items-center gap-1">
                    <span className="inline-block h-2 w-2 animate-pulse rounded-full bg-[var(--vscode-button-background)]" />
                    <span className="text-[10px] opacity-60">Thinking...</span>
                  </div>
                </div>
              )}

              {message.status === 'error' && (
                <p className="text-[10px] text-red-400 mt-1">An error occurred while generating the response</p>
              )}

              {/* Action buttons for completed assistant messages */}
              {isAssistant && message.status === 'complete' && message.content && (
                <ActionButtons
                  messageId={message.id}
                  onAccept={onAccept ?? (() => {})}
                  onReject={onReject ?? (() => {})}
                  onExplain={onExplain ?? (() => {})}
                />
              )}
            </div>
          </div>
        );
      })}

      <div ref={endRef} />
    </div>
  );
};
