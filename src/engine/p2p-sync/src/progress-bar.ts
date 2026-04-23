/**
 * P2P Sync Progress Bar — Extracted from sync-engine.ts
 * ASCII TUI progress bar with TTY/CI auto-detection.
 */
import { SyncProgress, OnProgressCallback } from './sync-engine';

/**
 * Default TUI progress bar implementation.
 * Renders an ASCII progress bar to process.stdout with '\r' single-line refresh.
 * Falls back to console.log in non-TTY / CI environments.
 */
export function defaultTuiProgressBar(progress: SyncProgress): void {
  const { peerId, completed, total, bytesTransferred, percent, direction } = progress;

  // Guard: avoid divide-by-zero and invalid totals
  if (total <= 0) {
    const msg = `[${direction.toUpperCase()}] ${peerId}: waiting for manifest...`;
    if (!process.stdout.isTTY || process.env.CI) {
      console.log(msg);
    } else {
      process.stdout.write(`\r${msg}`);
    }
    return;
  }

  const width = 30;
  const filled = Math.min(width, Math.round((percent / 100) * width));
  const empty = width - filled;
  const bar = '='.repeat(filled) + '>'.repeat(empty > 0 ? 1 : 0) + ' '.repeat(Math.max(0, empty - 1));
  const mb = (bytesTransferred / (1024 * 1024)).toFixed(1);
  const line = `[${bar}] ${percent.toFixed(0)}% (${completed}/${total} chunks, ${mb} MB) [${direction.toUpperCase()}] ${peerId}`;

  // Non-TTY or CI: use simple log to avoid '\r' artifacts in logs
  if (!process.stdout.isTTY || process.env.CI) {
    console.log(line);
    return;
  }

  process.stdout.write(`\r${line.padEnd(80, ' ')}`);
}

/**
 * Safe wrapper for invoking onProgress callbacks.
 * Catches exceptions to prevent progress handler failures from blocking sync.
 */
export function safeOnProgress(
  onProgress: OnProgressCallback,
  progress: SyncProgress
): void {
  try {
    onProgress(progress);
  } catch (err) {
    // Progress callback failures must never block the sync pipeline.
    // Log to stderr for observability but swallow the error.
    console.error(`[SyncEngine] onProgress callback failed for ${progress.peerId}:`, err);
  }
}
