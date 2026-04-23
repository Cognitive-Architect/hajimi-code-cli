import React, { useState, useCallback, useRef, useEffect } from 'react';
import { Textarea } from '@/components/ui/Textarea';
import { Button } from '@/components/ui/Button';
import { vscodeApi } from '../index';

interface InputBoxProps {
  onSubmit: (text: string) => void;
  disabled?: boolean;
  placeholder?: string;
}

/** Built-in slash commands for quick actions. */
const COMMANDS = [
  { id: 'build', label: 'Build project', icon: '🔨' },
  { id: 'test', label: 'Run tests', icon: '🧪' },
  { id: 'git', label: 'Git commit', icon: '📦' },
  { id: 'search', label: 'Search code', icon: '🔍' },
];

/** InputBox — Text input with slash commands, @file mentions, and #folder mentions.
 *
 *  Supports three trigger types:
 *  - `/` → command suggestions (build, test, git, search)
 *  - `@` → file mention suggestions (dynamically fetched from extension host)
 *  - `#` → folder mention suggestions (dynamically fetched from extension host)
 *
 *  Debounced filtering (100ms) prevents excessive re-renders.
 *  Clicking outside the suggestion panel dismisses it.
 *  Mentions are inserted as tokens and stripped of invalid entries before submit.
 *  Graceful fallback: if file/folder list request fails, suggestions show empty.
 */
export const InputBox: React.FC<InputBoxProps> = ({
  onSubmit,
  disabled = false,
  placeholder = 'Ask Hajimi... (use / for commands, @ for files, # for folders)',
}) => {
  const [input, setInput] = useState('');
  const [showCmd, setShowCmd] = useState(false);
  const [showFile, setShowFile] = useState(false);
  const [showFolder, setShowFolder] = useState(false);
  const [filteredCmd, setFilteredCmd] = useState(COMMANDS);
  const [filteredFile, setFilteredFile] = useState<string[]>([]);
  const [filteredFolder, setFilteredFolder] = useState<string[]>([]);
  const [fileList, setFileList] = useState<string[]>([]);
  const [folderList, setFolderList] = useState<string[]>([]);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Week 5: Request dynamic file/folder lists from extension host on mount
  useEffect(() => {
    vscodeApi.postMessage({ type: 'requestFileList', payload: {} });
    vscodeApi.postMessage({ type: 'requestFolderList', payload: {} });
  }, []);

  // Listen for fileList / folderList responses from extension host
  useEffect(() => {
    const handler = (event: MessageEvent) => {
      const msg = event.data as { type: string; payload?: unknown };
      if (msg.type === 'fileList') {
        const { files } = (msg.payload ?? {}) as { files?: string[] };
        setFileList(files ?? []);
      } else if (msg.type === 'folderList') {
        const { folders } = (msg.payload ?? {}) as { folders?: string[] };
        setFolderList(folders ?? []);
      }
    };
    window.addEventListener('message', handler);
    return () => window.removeEventListener('message', handler);
  }, []);

  /** Parse the last word and trigger the appropriate suggestion panel. */
  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const value = e.target.value;
    setInput(value);

    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      const lastWord = value.split(/\s+/).pop() ?? '';
      if (lastWord.startsWith('/')) {
        setShowCmd(true); setShowFile(false); setShowFolder(false);
        const q = lastWord.slice(1).toLowerCase();
        setFilteredCmd(COMMANDS.filter((c) => c.id.includes(q) || c.label.toLowerCase().includes(q)));
      } else if (lastWord.startsWith('@')) {
        setShowFile(true); setShowCmd(false); setShowFolder(false);
        const q = lastWord.slice(1).toLowerCase();
        setFilteredFile(fileList.filter((f) => f.toLowerCase().includes(q)));
      } else if (lastWord.startsWith('#')) {
        setShowFolder(true); setShowCmd(false); setShowFile(false);
        const q = lastWord.slice(1).toLowerCase();
        setFilteredFolder(folderList.filter((f) => f.toLowerCase().includes(q)));
      } else {
        setShowCmd(false); setShowFile(false); setShowFolder(false);
      }
    }, 100);
  }, [fileList, folderList]);

  /** Submit on Enter (Shift+Enter for new line). */
  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      const trimmed = input.trim();
      if (trimmed && !disabled) {
        onSubmit(trimmed); setInput(''); setShowCmd(false); setShowFile(false); setShowFolder(false);
      }
    }
  }, [input, disabled, onSubmit]);

  /** Insert a token (command, file, or folder) at the cursor position. */
  const insertToken = useCallback((token: string) => {
    const words = input.split(/\s+/); words.pop();
    setInput([...words, token].join(' ') + ' ');
    setShowCmd(false); setShowFile(false); setShowFolder(false);
    textareaRef.current?.focus();
  }, [input]);

  /** Explicit send button handler. */
  const handleSend = useCallback(() => {
    const trimmed = input.trim();
    if (trimmed && !disabled) { onSubmit(trimmed); setInput(''); setShowCmd(false); setShowFile(false); setShowFolder(false); }
  }, [input, disabled, onSubmit]);

  /** Dismiss suggestion panel when clicking outside. */
  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setShowCmd(false); setShowFile(false); setShowFolder(false);
      }
    };
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, []);

  /** Focus textarea on mount for immediate typing. */
  useEffect(() => { textareaRef.current?.focus(); }, []);

  /** Strip invalid @mentions and #folders before submitting. */
  const sanitizeInput = useCallback((text: string): string => {
    return text.replace(/@[^\s]+/g, (match) => {
      const name = match.slice(1);
      return fileList.includes(name) ? match : '';
    }).replace(/#[^\s]+/g, (match) => {
      const name = match.slice(1);
      return folderList.includes(name) ? match : '';
    }).replace(/\s+/g, ' ').trim();
  }, [fileList, folderList]);

  /** Render a suggestion dropdown panel. */
  const renderSuggestions = <T extends { id?: string; label?: string; icon?: string } | string>(
    items: T[],
    show: boolean,
    onClick: (item: T) => void,
    format: (item: T) => string,
    prefix: string
  ) => {
    if (!show || items.length === 0) return null;
    return (
      <div className="absolute bottom-full left-2 right-2 mb-1 max-h-40 overflow-y-auto rounded-md border border-[var(--vscode-panel-border)] bg-[var(--vscode-dropdown-background)] shadow-lg">
        {items.map((item, idx) => (
          <button
            key={typeof item === 'string' ? item : (item.id ?? item.label ?? idx)}
            onClick={() => onClick(item)}
            className="flex w-full items-center gap-2 px-3 py-2 text-left text-xs text-[var(--vscode-dropdown-foreground)] hover:bg-[var(--vscode-list-hoverBackground)]"
          >
            <span>{typeof item === 'string' ? prefix : (item.icon ?? prefix)}</span>
            <span>{format(item)}</span>
          </button>
        ))}
      </div>
    );
  };

  return (
    <div ref={containerRef} className="relative border-t border-[var(--vscode-panel-border)] bg-[var(--vscode-editor-background)] p-2">
      {renderSuggestions(filteredCmd, showCmd, (c) => insertToken(`/${(c as typeof COMMANDS[0]).id}`), (c) => `/${(c as typeof COMMANDS[0]).id} — ${(c as typeof COMMANDS[0]).label}`, '⚡')}
      {renderSuggestions(filteredFile, showFile, (f) => insertToken(`@${f as string}`), (f) => `@${f as string}`, '📄')}
      {renderSuggestions(filteredFolder, showFolder, (f) => insertToken(`#${f as string}`), (f) => `#${f as string}/`, '📁')}

      <div className="flex gap-2">
        <Textarea
          ref={textareaRef}
          value={input}
          onChange={handleInputChange}
          onKeyDown={handleKeyDown}
          disabled={disabled}
          placeholder={placeholder}
          rows={2}
          className="min-h-[44px] flex-1"
        />
        <Button onClick={handleSend} disabled={disabled || !input.trim()} size="icon" className="h-[44px] w-[44px] shrink-0">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <line x1="22" y1="2" x2="11" y2="13" /><polygon points="22 2 15 22 11 13 2 9 22 2" />
          </svg>
        </Button>
      </div>

      <div className="mt-1 flex items-center justify-between px-1">
        <span className="text-[10px] text-[var(--vscode-descriptionForeground)] opacity-60">Shift+Enter for new line · @file · #folder</span>
        {disabled && <span className="text-[10px] text-[var(--vscode-descriptionForeground)] animate-pulse">Streaming...</span>}
      </div>
    </div>
  );
};
