import * as vscode from 'vscode';

/** Single undo entry capturing a document snapshot at a point in time. */
interface UndoEntry {
  uri: string;
  original: string;
  modified: string;
  timestamp: number;
  label: string;
}

/** Undo event payloads for optional listeners. */
interface UndoEvent { uri: string; entry: UndoEntry; remaining: number; }
interface RestoreEvent { uri: string; index: number; entry: UndoEntry; }
interface BoundaryEvent { uri: string; action: 'limit_reached' | 'stack_empty' | 'stack_full'; }

export type UndoListener = (event: UndoEvent) => void;
export type RestoreListener = (event: RestoreEvent) => void;
export type BoundaryListener = (event: BoundaryEvent) => void;

/** Stack-based undo manager for WorkspaceEdit operations.
 *
 *  Integrates with StreamingEditEngine to record snapshots after each
 *  successful apply. Provides undo, restore-by-index, and stack boundary
 *  protection with a configurable limit. Optional event listeners allow
 *  external components (e.g. UI toasts) to react to undo/restore actions.
 *
 *  Each entry stores the original document text captured before the edit
 *  was applied. Undo restores that original text via a full-document
 *  WorkspaceEdit.replace, which is reliable but not line-granular.
 *
 *  Boundary protection:
 *  - limit_reached: oldest entries are evicted when limit exceeded
 *  - stack_empty: undo/restore fails when no history exists
 *  - stack_full: emitted when the stack reaches its configured limit
 */
export class UndoManager {
  private stacks = new Map<string, UndoEntry[]>();
  private readonly limit: number;
  private undoListeners: UndoListener[] = [];
  private redoStacks = new Map<string, UndoEntry[]>();
  private restoreListeners: RestoreListener[] = [];
  private boundaryListeners: BoundaryListener[] = [];
  private redoListeners: UndoListener[] = [];

  constructor(limit = 50) {
    if (limit < 1) { throw new Error('UndoManager limit must be >= 1'); }
    this.limit = limit;
  }

  /** Register a listener called after each successful undo. */
  public onUndo(listener: UndoListener): () => void {
    this.undoListeners.push(listener);
    return () => { this.undoListeners = this.undoListeners.filter((l) => l !== listener); };
  }

  /** Register a listener called after each successful restore. */
  public onRestore(listener: RestoreListener): () => void {
    this.restoreListeners.push(listener);
    return () => { this.restoreListeners = this.restoreListeners.filter((l) => l !== listener); };
  }

  /** Register a listener called after each successful redo. */
  public onRedo(listener: UndoListener): () => void {
    this.redoListeners.push(listener);
    return () => { this.redoListeners = this.redoListeners.filter((l) => l !== listener); };
  }

  /** Register a listener called on boundary events (limit reached, stack empty). */
  public onBoundary(listener: BoundaryListener): () => void {
    this.boundaryListeners.push(listener);
    return () => { this.boundaryListeners = this.boundaryListeners.filter((l) => l !== listener); };
  }

  /** Record a snapshot after a successful edit apply. */
  public push(uri: string, original: string, modified: string, label = 'edit'): void {
    const stack = this.stacks.get(uri) ?? [];
    const wasFull = stack.length >= this.limit;
    stack.push({ uri, original, modified, timestamp: Date.now(), label });
    while (stack.length > this.limit) { stack.shift(); }
    this.stacks.set(uri, stack);
    if (wasFull) { this.emitBoundary({ uri, action: 'limit_reached' }); }
  }

  /** Undo the most recent edit for a URI by restoring its original snapshot. */
  public async undo(uri: string): Promise<boolean> {
    const stack = this.stacks.get(uri);
    if (!stack || stack.length === 0) {
      this.emitBoundary({ uri, action: 'stack_empty' });
      return false;
    }
    const entry = stack.pop()!;
    const ok = await this.restoreSnapshot(entry);
    if (ok) {
      this.emitUndo({ uri, entry, remaining: stack.length });
      // Push to redo stack for potential redo
      const redoStack = this.redoStacks.get(uri) ?? [];
      redoStack.push(entry);
      this.redoStacks.set(uri, redoStack);
    }
    return ok;
  }

  /** Restore a specific historical snapshot by index (0 = oldest). */
  public async restore(uri: string, index: number): Promise<boolean> {
    const stack = this.stacks.get(uri);
    if (!stack || index < 0 || index >= stack.length) {
      this.emitBoundary({ uri, action: 'stack_empty' });
      return false;
    }
    const entry = stack[index];
    // Truncate stack to the restored point
    stack.splice(index + 1);
    this.stacks.set(uri, stack);
    const ok = await this.restoreSnapshot(entry);
    if (ok) { this.emitRestore({ uri, index, entry }); }
    return ok;
  }

  /** Get the undo history for a URI as a read-only view. */
  public getHistory(uri: string): ReadonlyArray<Pick<UndoEntry, 'timestamp' | 'label'>> {
    return this.stacks.get(uri)?.map((e) => ({ timestamp: e.timestamp, label: e.label })) ?? [];
  }

  /** Redo the most recently undone edit for a URI. */
  public async redo(uri: string): Promise<boolean> {
    const redoStack = this.redoStacks.get(uri);
    if (!redoStack || redoStack.length === 0) {
      this.emitBoundary({ uri, action: 'stack_empty' });
      return false;
    }
    const entry = redoStack.pop()!;
    const ok = await this.restoreSnapshot({ ...entry, original: entry.modified, modified: entry.original });
    if (ok) {
      this.emitRedo({ uri, entry, remaining: redoStack.length });
      // Push back to undo stack
      const stack = this.stacks.get(uri) ?? [];
      stack.push(entry);
      this.stacks.set(uri, stack);
    }
    return ok;
  }

  /** Check if redo is available for a URI. */
  public canRedo(uri: string): boolean {
    return (this.redoStacks.get(uri)?.length ?? 0) > 0;
  }

  /** Check if undo is available for a URI. */
  public canUndo(uri: string): boolean {
    return (this.stacks.get(uri)?.length ?? 0) > 0;
  }

  /** Drop all history for a URI. */
  public clear(uri: string): void {
    this.stacks.delete(uri);
    this.redoStacks.delete(uri);
  }

  /** Current stack size for a URI (for boundary monitoring). */
  public stackSize(uri: string): number {
    return this.stacks.get(uri)?.length ?? 0;
  }

  /** Maximum allowed stack size. */
  public getLimit(): number { return this.limit; }

  /** Serialize the undo history for a URI to a JSON string (for diagnostics). */
  public serialize(uri: string): string {
    const stack = this.stacks.get(uri);
    if (!stack) return '[]';
    return JSON.stringify(stack.map((e) => ({ ...e, original: e.original.slice(0, 80) + (e.original.length > 80 ? '…' : '') })));
  }

  /** Total entries across all URIs. */
  public totalEntries(): number {
    let total = 0;
    for (const stack of this.stacks.values()) { total += stack.length; }
    return total;
  }

  /** Restore a snapshot via WorkspaceEdit.replace + applyEdit. */
  private async restoreSnapshot(entry: UndoEntry): Promise<boolean> {
    try {
      const uri = vscode.Uri.parse(entry.uri);
      const doc = await vscode.workspace.openTextDocument(uri);
      const edit = new vscode.WorkspaceEdit();
      const fullRange = new vscode.Range(0, 0, doc.lineCount, 0);
      edit.replace(uri, fullRange, entry.original);
      return await vscode.workspace.applyEdit(edit);
    } catch (err) {
      console.error('[UndoManager] restoreSnapshot failed:', err);
      return false;
    }
  }

  private emitUndo(event: UndoEvent): void { this.undoListeners.forEach((l) => { try { l(event); } catch {} }); }
  private emitRedo(event: UndoEvent): void { this.redoListeners.forEach((l) => { try { l(event); } catch {} }); }
  private emitRestore(event: RestoreEvent): void { this.restoreListeners.forEach((l) => { try { l(event); } catch {} }); }
  private emitBoundary(event: BoundaryEvent): void { this.boundaryListeners.forEach((l) => { try { l(event); } catch {} }); }
}
