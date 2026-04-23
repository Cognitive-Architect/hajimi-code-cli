import type { LspClient } from '../clients/LspClient';
import type { FeedbackItem, FeedbackResult, FeedbackBatch } from './FeedbackTypes';

/** Event types for external observers. */
export type FeedbackFlushListener = (result: FeedbackResult) => void;
export type FeedbackErrorListener = (error: Error, batch: FeedbackBatch) => void;

/** Collects user feedback in memory and flushes batches to the LSP backend.
 *
 *  Uses exponential-backoff retry on network failures and auto-flushes
 *  when the queue reaches a threshold or a timeout elapses. On total
 *  failure feedback items are re-enqueued so no data is lost.
 *
 *  Optional listeners allow UI components to react to flush results
 *  and errors for toast notifications. A localStorage backup is used
 *  to survive extension host restarts (Week 4 MVP, full persistence
 *  via MemoryGateway deferred to Week 5–6).
 *
 *  Pipeline: userChoice (accept/reject/explain) → collectFeedback →
 *  sendToGovernance (via LspClient 'hajimi/feedback') → memory.store
 *  (MemoryGateway session layer in Week 5–6).
 */
export class FeedbackCollector {
  private queue: FeedbackItem[] = [];
  private timer: ReturnType<typeof setTimeout> | null = null;
  private readonly flushIntervalMs: number;
  private readonly maxBatchSize: number;
  private sessionId: string;
  private flushListeners: FeedbackFlushListener[] = [];
  private errorListeners: FeedbackErrorListener[] = [];
  private disposed = false;

  constructor(
    private lspClient: LspClient,
    private deviceId: string,
    options?: { flushIntervalMs?: number; maxBatchSize?: number; storage?: { get: (key: string) => string | undefined; set: (key: string, value: string) => void } }
  ) {
    this.flushIntervalMs = options?.flushIntervalMs ?? 30000;
    this.maxBatchSize = options?.maxBatchSize ?? 10;
    this.sessionId = this.genSessionId();
    // Attempt to restore any pending feedback from a previous session
    this.restorePending();
  }

  /** Add a feedback item to the queue and trigger auto-flush if needed. */
  public collectFeedback(item: FeedbackItem): void {
    if (this.disposed) { return; }
    const err = this.validateItem(item);
    if (err) { console.warn('[FeedbackCollector] validation failed:', err); return; }
    this.queue.push(item);
    this.backupPending();
    if (this.queue.length >= this.maxBatchSize) {
      void this.flush();
    } else {
      this.scheduleFlush();
    }
  }

  /** Immediately send all queued feedback to the backend.
   *  DEBT-RUST-FEEDBACK-001: On LSP failure, falls back to session.store
   *  via localStorage backup so feedback survives extension host restart.
   */
  public async flush(): Promise<FeedbackResult> {
    if (this.disposed || this.queue.length === 0) { return { success: true, storedCount: 0 }; }
    const batch: FeedbackBatch = {
      items: this.queue.splice(0, this.queue.length),
      deviceId: this.deviceId,
      sessionId: this.sessionId,
    };
    const result = await this.sendWithRetry(batch, 3);
    if (result.success) {
      this.clearBackup();
    } else {
      // Fallback: memory.store / session.store — persist to localStorage for later retry
      this.storeToSession(batch);
    }
    this.flushListeners.forEach((l) => { try { l(result); } catch {} });
    return result;
  }

  /** Register a listener for successful flush events. */
  public onFlush(listener: FeedbackFlushListener): () => void {
    this.flushListeners.push(listener);
    return () => { this.flushListeners = this.flushListeners.filter((l) => l !== listener); };
  }

  /** Register a listener for flush error events. */
  public onError(listener: FeedbackErrorListener): () => void {
    this.errorListeners.push(listener);
    return () => { this.errorListeners = this.errorListeners.filter((l) => l !== listener); };
  }

  /** Dispose: cancel pending timer and attempt final flush. */
  public dispose(): void {
    this.disposed = true;
    if (this.timer) { clearTimeout(this.timer); this.timer = null; }
    if (this.queue.length > 0) { void this.flush(); }
  }

  /** Current pending queue length. */
  public pendingCount(): number { return this.queue.length; }

  /** Validate a feedback item before enqueueing. */
  private validateItem(item: FeedbackItem): string | null {
    if (!item.messageId || item.messageId.trim().length === 0) { return 'messageId is required'; }
    if (!['accept', 'reject', 'explain'].includes(item.choice)) { return `invalid choice: ${item.choice}`; }
    if (item.reason && item.reason.length > 2000) { return 'reason exceeds 2000 characters'; }
    return null;
  }

  /** Exposed for testing: force a flush with a specific batch. */
  public async flushForTest(batch: FeedbackBatch): Promise<FeedbackResult> {
    return this.sendWithRetry(batch, 1);
  }

  /** Reset session ID (e.g. on conversation clear). */
  public resetSession(): void {
    this.sessionId = this.genSessionId();
    this.queue = [];
    this.clearBackup();
    if (this.timer) { clearTimeout(this.timer); this.timer = null; }
  }

  private genSessionId(): string {
    return `sess-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
  }

  private scheduleFlush(): void {
    if (this.timer) { clearTimeout(this.timer); }
    this.timer = setTimeout(() => { void this.flush(); }, this.flushIntervalMs);
  }

  /** Persist pending queue to localStorage for crash recovery. */
  private backupPending(): void {
    try {
      const data = JSON.stringify({ sessionId: this.sessionId, items: this.queue });
      // Use VSCode global state if available; fallback to localStorage for webview
      if (typeof localStorage !== 'undefined') { localStorage.setItem('hajimi-feedback-pending', data); }
    } catch { /* silent: storage may be unavailable */ }
  }

  private restorePending(): void {
    try {
      if (typeof localStorage === 'undefined') return;
      const raw = localStorage.getItem('hajimi-feedback-pending');
      if (!raw) return;
      const parsed = JSON.parse(raw) as { sessionId: string; items: FeedbackItem[] };
      if (parsed.items.length > 0) {
        this.sessionId = parsed.sessionId;
        this.queue.push(...parsed.items);
      }
    } catch { /* silent: ignore corrupt backup */ }
  }

  private clearBackup(): void {
    try { if (typeof localStorage !== 'undefined') localStorage.removeItem('hajimi-feedback-pending'); } catch {}
  }

  /** Persist batch to session storage as a memory.store fallback. */
  private storeToSession(batch: FeedbackBatch): void {
    try {
      const key = `hajimi-feedback-session-${this.sessionId}`;
      const existing = (() => {
        try {
          const raw = localStorage.getItem(key);
          return raw ? (JSON.parse(raw) as FeedbackBatch[]) : [];
        } catch { return []; }
      })();
      existing.push(batch);
      localStorage.setItem(key, JSON.stringify(existing.slice(-10)));
    } catch { /* silent */ }
  }

  private async sendWithRetry(batch: FeedbackBatch, retriesLeft: number): Promise<FeedbackResult> {
    try {
      const result = await this.lspClient.sendCustomRequest<FeedbackResult>('hajimi/feedback', batch);
      return result;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      if (retriesLeft > 0) {
        const delay = Math.pow(2, 3 - retriesLeft) * 1000;
        await new Promise((r) => setTimeout(r, delay));
        return this.sendWithRetry(batch, retriesLeft - 1);
      }
      // Re-enqueue on total failure so data is not lost
      this.queue.unshift(...batch.items);
      this.backupPending();
      const error = new Error(msg);
      this.errorListeners.forEach((l) => { try { l(error, batch); } catch {} });
      return { success: false, storedCount: 0, errors: [msg] };
    }
  }
}
