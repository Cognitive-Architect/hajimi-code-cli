import { useState, useCallback } from 'react';
import type { ChatMessage } from '../types/webview';

let idCounter = 0;
function genId(): string {
  return `msg-${Date.now()}-${++idCounter}`;
}

export function useMessages() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const addUserMessage = useCallback((content: string): ChatMessage => {
    const msg: ChatMessage = {
      id: genId(),
      role: 'user',
      content,
      timestamp: Date.now(),
      status: 'complete',
    };
    setMessages((prev) => [...prev, msg]);
    return msg;
  }, []);

  const startAssistantStream = useCallback((): string => {
    const id = genId();
    const msg: ChatMessage = {
      id,
      role: 'assistant',
      content: '',
      timestamp: Date.now(),
      status: 'streaming',
    };
    setMessages((prev) => [...prev, msg]);
    setIsLoading(true);
    return id;
  }, []);

  const appendStreamChunk = useCallback((id: string, text: string): void => {
    setMessages((prev) =>
      prev.map((m) => (m.id === id ? { ...m, content: text, status: 'streaming' as const } : m))
    );
  }, []);

  const completeStream = useCallback((id: string, finalText?: string): void => {
    setMessages((prev) =>
      prev.map((m) =>
        m.id === id
          ? { ...m, content: finalText ?? m.content, status: 'complete' as const }
          : m
      )
    );
    setIsLoading(false);
  }, []);

  const setStreamError = useCallback((id: string, errorText: string): void => {
    setMessages((prev) =>
      prev.map((m) =>
        m.id === id
          ? { ...m, content: errorText, status: 'error' as const }
          : m
      )
    );
    setIsLoading(false);
  }, []);

  const setAllMessages = useCallback((msgs: ChatMessage[]): void => {
    setMessages(msgs);
  }, []);

  return {
    messages,
    isLoading,
    addUserMessage,
    startAssistantStream,
    appendStreamChunk,
    completeStream,
    setStreamError,
    setAllMessages,
  };
}
