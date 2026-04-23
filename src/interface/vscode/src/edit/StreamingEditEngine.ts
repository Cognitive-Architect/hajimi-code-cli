import * as vscode from 'vscode';
import { unifiedDiff } from './diff-util';

/** Single incremental edit chunk streamed from the agent. */
export interface EditChunk {
  uri: string;
  range: [number, number, number, number]; // [startLine, startChar, endLine, endChar]
  text: string;
  stepIndex: number;
}

/** Per-document edit state. */
interface DocEditState {
  uri: vscode.Uri;
  original: string;
  modified: string;
  pending: EditChunk[];
  abortCtrl: AbortController | null;
  version: number;
}

/** Statistics for a streaming edit session. */
export interface EditStats {
  pendingCount: number;
  insertions: number;
  deletions: number;
  modifiedChars: number;
}

/**
 * StreamingEditEngine — incremental WorkspaceEdit with preview/live dual mode.
 *
 * Core design:
 * - preview mode: accumulates edits into a modified buffer, generates unified diff
 * - live mode: applies each chunk immediately via WorkspaceEdit with version checking
 * - abort: AbortController cancellation + rollback to original
 * - syncWithTrace: triggered when ThinkingTrace reaches the 'Act' step
 *
 * DEBT-W3-EDIT-ENGINE-001: Live mode applies edits directly; rollback uses
 * document.version check for conflict detection. Full undo stack deferred to Week 4.
 * DEBT-W3-DIFF-ALG-001: Uses a simplified line-level diff, not full Myers LCS.
 */
export class StreamingEditEngine {
  private states = new Map<string, DocEditState>();

  /** Begin streaming edits for a document. Captures original text and initializes state. */
  public async start(uri: vscode.Uri): Promise<void> {
    const doc = await vscode.workspace.openTextDocument(uri);
    const state: DocEditState = {
      uri,
      original: doc.getText(),
      modified: doc.getText(),
      pending: [],
      abortCtrl: new AbortController(),
      version: doc.version,
    };
    this.states.set(uri.toString(), state);
  }

  /** Process one edit chunk. Preview mode accumulates; live mode applies immediately. */
  public async onEditChunk(chunk: EditChunk, mode: 'preview' | 'live'): Promise<void> {
    const state = this.states.get(chunk.uri);
    if (!state || state.abortCtrl?.signal.aborted) return;
    this.validateChunk(chunk);
    if (mode === 'live') {
      await this.applyIncremental(chunk, state);
    } else {
      this.accumulatePreview(chunk, state);
    }
  }

  /** Validate that an edit chunk has sane range values. */
  private validateChunk(chunk: EditChunk): void {
    const [sl, sc, el, ec] = chunk.range;
    if (sl < 0 || sc < 0 || el < 0 || ec < 0) {
      throw new Error(`Invalid edit range: [${sl},${sc},${el},${ec}]`);
    }
    if (sl > el || (sl === el && sc > ec)) {
      throw new Error(`Edit range is inverted: [${sl},${sc},${el},${ec}]`);
    }
  }

  /** Live-mode: apply via WorkspaceEdit with range validation and version check. */
  private async applyIncremental(chunk: EditChunk, state: DocEditState): Promise<void> {
    const doc = await vscode.workspace.openTextDocument(state.uri);
    if (doc.version !== state.version) {
      throw new Error(`Document version mismatch: expected ${state.version}, got ${doc.version}. Abort to avoid conflict.`);
    }
    const edit = new vscode.WorkspaceEdit();
    const [sl, sc, el, ec] = chunk.range;
    edit.replace(state.uri, new vscode.Range(sl, sc, el, ec), chunk.text);
    const success = await vscode.workspace.applyEdit(edit);
    if (!success) throw new Error('WorkspaceEdit.applyEdit returned false');
    state.version = doc.version + 1;
  }

  /** Preview-mode: accumulate into modified buffer without touching the real editor. */
  private accumulatePreview(chunk: EditChunk, state: DocEditState): void {
    const lines = state.modified.split('\n');
    const [sl, sc, el, ec] = chunk.range;
    const startIdx = Math.min(sl, Math.max(0, lines.length - 1));
    const endIdx = Math.min(el, Math.max(0, lines.length - 1));
    const before = lines[startIdx].slice(0, sc);
    const after = lines[endIdx].slice(ec);
    const replacement = chunk.text.split('\n');
    replacement[0] = before + replacement[0];
    replacement[replacement.length - 1] = replacement[replacement.length - 1] + after;
    lines.splice(startIdx, endIdx - startIdx + 1, ...replacement);
    state.modified = lines.join('\n');
    state.pending.push(chunk);
  }

  /** Generate unified diff between original and modified for diff2html rendering. */
  public generateDiff(uriStr: string): string {
    const state = this.states.get(uriStr);
    if (!state) return '';
    return unifiedDiff(state.original, state.modified, uriStr);
  }

  /** Get the current modified text for a URI (preview mode buffer). */
  public getModifiedText(uriStr: string): string | null {
    return this.states.get(uriStr)?.modified ?? null;
  }

  /** Get the original text captured at session start. */
  public getOriginalText(uriStr: string): string | null {
    return this.states.get(uriStr)?.original ?? null;
  }

/** Apply all pending edits via a single batched WorkspaceEdit (preview → commit). */
  public async applyPending(uriStr: string): Promise<boolean> {
    const state = this.states.get(uriStr);
    if (!state || state.pending.length === 0) return false;
    const edit = new vscode.WorkspaceEdit();
    for (const chunk of state.pending) {
      const [sl, sc, el, ec] = chunk.range;
      edit.replace(state.uri, new vscode.Range(sl, sc, el, ec), chunk.text);
    }
    const ok = await vscode.workspace.applyEdit(edit);
    if (ok) {
      state.pending = [];
      state.original = state.modified;
    }
    return ok;
  }

  /** Apply pending edits in small batches to avoid blocking the extension host on large files. */
  public async applyPendingChunked(uriStr: string, batchSize = 10): Promise<boolean> {
    const state = this.states.get(uriStr);
    if (!state || state.pending.length === 0) return false;
    let ok = true;
    while (state.pending.length > 0 && ok) {
      if (state.abortCtrl?.signal.aborted) return false;
      const batch = state.pending.splice(0, batchSize);
      const edit = new vscode.WorkspaceEdit();
      for (const chunk of batch) {
        const [sl, sc, el, ec] = chunk.range;
        edit.replace(state.uri, new vscode.Range(sl, sc, el, ec), chunk.text);
      }
      ok = await vscode.workspace.applyEdit(edit);
    }
    if (ok) {
      state.original = state.modified;
    }
    return ok;
  }

  /** Abort current stream (onCancel / abortEdit), discard pending edits, and roll back modified buffer. */
  public abort(uriStr: string): void {
    const state = this.states.get(uriStr);
    if (state?.abortCtrl) {
      state.abortCtrl.abort();
      state.pending = [];
      state.modified = state.original;
    }
  }

  /** Undo the last applied edit chunk (placeholder for Week 4 full Undo stack). */
  undoLast(uriStr: string): void {
    // DEBT-W3-UNDO-001: Full undo stack deferred to Week 4.
    // Currently this is a no-op; Week 4 will integrate vscode.commands.executeCommand('undo').
    void uriStr;
  }

  /** Check if a URI's stream has been aborted. */
  public isAborted(uriStr: string): boolean {
    return this.states.get(uriStr)?.abortCtrl?.signal.aborted ?? false;
  }

  /** Check if a URI has pending edits waiting to be applied. */
  public hasPending(uriStr: string): boolean {
    return (this.states.get(uriStr)?.pending.length ?? 0) > 0;
  }

  /** Compute statistics for the current edit session. */
  public getStats(uriStr: string): EditStats | null {
    const state = this.states.get(uriStr);
    if (!state) return null;
    const origLen = state.original.length;
    const modLen = state.modified.length;
    const pendingCount = state.pending.length;
    const insertions = state.modified.split('\n').length - state.original.split('\n').length;
    return {
      pendingCount,
      insertions: Math.max(0, insertions),
      deletions: Math.max(0, -insertions),
      modifiedChars: Math.abs(modLen - origLen),
    };
  }

  /** Sync with ThinkingTrace timeline: start editing when Act step begins. */
  public syncWithTrace(step: string, uri: vscode.Uri): void {
    if (step === 'Act') {
      void this.start(uri);
    }
  }

  /** Cleanup state for a URI. */
  public dispose(uriStr: string): void {
    this.states.delete(uriStr);
  }
}
