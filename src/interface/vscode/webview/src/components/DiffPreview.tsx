import React, { useRef, useEffect, useState, useMemo, useCallback } from 'react';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/Card';
import { html as diff2html } from 'diff2html';
import 'diff2html/bundles/css/diff2html.min.css';
import type { DiffStats } from '../types/webview';

/**
 * DiffPreview — colored diff preview with diff2html side-by-side rendering.
 *
 * Features:
 * - Side-by-side or line-by-line diff output format toggle
 * - Real-time streaming indicator with cancel button
 * - Accept / Reject / Mode switch controls
 * - Auto-computed insertion/deletion statistics
 * - Error boundary with dismiss action
 *
 * DEBT-W3-MONACO-001: Using diff2html instead of Monaco Diff Editor to avoid
 * CSP complexity and bundle bloat. Monaco integration deferred to Week 6 evaluation.
 */
interface DiffPreviewProps {
  diff?: string;
  mode?: 'preview' | 'live';
  isStreaming?: boolean;
  stats?: DiffStats;
  error?: string | null;
  onAccept?: () => void;
  onReject?: () => void;
  onCancel?: () => void;
  onModeChange?: (mode: 'preview' | 'live') => void;
}

export const DiffPreview: React.FC<DiffPreviewProps> = ({
  diff,
  mode = 'preview',
  isStreaming,
  stats,
  error,
  onAccept,
  onReject,
  onCancel,
  onModeChange,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [parseError, setParseError] = useState<string | null>(null);
  const [outputFormat, setOutputFormat] = useState<'side-by-side' | 'line-by-line'>('side-by-side');

  const renderDiff = useCallback(() => {
    if (!diff || !containerRef.current) return;
    try {
      setParseError(null);
      const output = diff2html(diff, {
        drawFileList: false,
        matching: 'lines',
        outputFormat,
        colorScheme: 'dark',
      });
      containerRef.current.innerHTML = output;
    } catch (err) {
      setParseError(err instanceof Error ? err.message : 'Diff render failed');
    }
  }, [diff, outputFormat]);

  useEffect(() => {
    renderDiff();
  }, [renderDiff]);

  // Clean up diff2html DOM on unmount to prevent memory leaks
  useEffect(() => {
    return () => {
      if (containerRef.current) containerRef.current.innerHTML = '';
    };
  }, []);

  // Compute diff highlight stats from the unified diff string
  const highlightStats = useMemo(() => {
    if (!diff) return { insertions: 0, deletions: 0 };
    return {
      insertions: (diff.match(/^\+[^+]/gm) ?? []).length,
      deletions: (diff.match(/^-[^-]/gm) ?? []).length,
    };
  }, [diff]);

  const hasChanges = useMemo(() => !!diff && diff.length > 0 && diff.includes('@@'), [diff]);

  const computedStats = useMemo(() => {
    if (!diff) return { insertions: 0, deletions: 0 };
    const ins = (diff.match(/^\+[^+]/gm) ?? []).length;
    const del = (diff.match(/^-[^-]/gm) ?? []).length;
    return { insertions: ins, deletions: del };
  }, [diff]);

  const displayStats = stats ?? computedStats;
  const displayError = error || parseError;

  return (
    <Card className="mt-2 flex-1 min-h-0 flex flex-col">
      <CardHeader className="py-2 px-3 flex flex-row items-center justify-between">
        <div className="flex items-center gap-2">
          <CardTitle className="text-xs uppercase tracking-wider text-[var(--vscode-descriptionForeground)]">
            Diff Preview
          </CardTitle>
          {isStreaming && (
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-blue-600 text-white animate-pulse">
              Streaming
            </span>
          )}
          {mode === 'preview' && !isStreaming && (
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-amber-600 text-white">
              Preview
            </span>
          )}
          {mode === 'live' && !isStreaming && (
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-green-600 text-white">
              Live
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          {hasChanges && !isStreaming && (
            <button
              onClick={() => setOutputFormat((f) => (f === 'side-by-side' ? 'line-by-line' : 'side-by-side'))}
              className="text-[10px] px-1.5 py-0.5 rounded bg-[var(--vscode-badge-background)] text-[var(--vscode-badge-foreground)] hover:opacity-80 transition-opacity"
              title="Toggle diff layout"
            >
              {outputFormat === 'side-by-side' ? 'Side' : 'Line'}
            </button>
          )}
          <span className="text-[10px] text-green-400" title="Lines inserted">+{displayStats.insertions}</span>
          <span className="text-[10px] text-red-400" title="Lines deleted">-{displayStats.deletions}</span>
        </div>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto px-0 py-0">
        {displayError ? (
          <div className="p-4 text-center">
            <p className="text-xs text-red-400">Error: {displayError}</p>
            <button
              onClick={onReject}
              className="mt-2 text-[10px] px-2 py-1 rounded bg-[var(--vscode-button-secondaryBackground)] text-[var(--vscode-button-secondaryForeground)]"
            >
              Dismiss
            </button>
          </div>
        ) : !hasChanges ? (
          <div className="rounded border border-dashed border-[var(--vscode-panel-border)] p-4 m-4 text-center">
            <p className="text-xs text-[var(--vscode-descriptionForeground)]">
              Changes will appear here during streaming edits.
            </p>
            <p className="mt-1 text-[10px] text-[var(--vscode-descriptionForeground)] opacity-50">
              Side-by-side diff with Accept / Reject controls.
            </p>
          </div>
        ) : (
          <div ref={containerRef} className="diff2html" />
        )}
      </CardContent>
      {hasChanges && !isStreaming && !displayError && (
        <div className="px-3 py-2 border-t border-[var(--vscode-panel-border)] flex items-center gap-2">
          <button
            onClick={onAccept}
            className="text-[10px] px-2 py-1 rounded bg-green-600 text-white hover:bg-green-500 transition-colors"
            title="Apply all pending edits to the workspace"
          >
            Accept
          </button>
          <button
            onClick={onReject}
            className="text-[10px] px-2 py-1 rounded bg-red-600 text-white hover:bg-red-500 transition-colors"
            title="Discard all pending edits"
          >
            Reject
          </button>
          <div className="flex-1" />
          <button
            onClick={() => onModeChange?.(mode === 'preview' ? 'live' : 'preview')}
            className="text-[10px] px-2 py-1 rounded bg-[var(--vscode-button-secondaryBackground)] text-[var(--vscode-button-secondaryForeground)] hover:opacity-80 transition-opacity"
            title={mode === 'preview' ? 'Switch to live incremental editing' : 'Switch to preview mode'}
          >
            {mode === 'preview' ? 'Switch to Live' : 'Switch to Preview'}
          </button>
        </div>
      )}
      {isStreaming && (
        <div className="px-3 py-2 border-t border-[var(--vscode-panel-border)] flex items-center gap-2">
          <button
            onClick={onCancel}
            className="text-[10px] px-2 py-1 rounded bg-red-600 text-white hover:bg-red-500 transition-colors"
            title="Cancel current streaming edit"
          >
            Cancel
          </button>
          <span className="text-[10px] text-[var(--vscode-descriptionForeground)] animate-pulse">
            Applying edits in real-time...
          </span>
        </div>
      )}
    </Card>
  );
};
