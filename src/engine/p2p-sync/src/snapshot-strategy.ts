/**
 * Yjs快照策略管理器 - 支持自动/手动触发，失败回滚
 */
import { DVVManager } from './dvv-manager';
import * as Y from 'yjs';

export type SnapshotTrigger = 'manual' | 'auto-count' | 'auto-size' | 'auto-time';

export interface SnapshotStrategyConfig {
  trigger: SnapshotTrigger;
  threshold?: number;
  intervalMs?: number;
  maxSnapshots?: number;
}

export class SnapshotStrategy {
  private dvvManager: DVVManager;
  private config: SnapshotStrategyConfig;
  private timer: ReturnType<typeof setInterval> | null = null;

  constructor(doc: Y.Doc, config: SnapshotStrategyConfig) {
    this.config = { maxSnapshots: 3, ...config };
    this.dvvManager = new DVVManager(doc, {
      snapshotThreshold: config.trigger === 'auto-count' ? config.threshold : undefined,
      sizeThresholdMB: config.trigger === 'auto-size' ? config.threshold : undefined,
      enableAutoCleanup: config.trigger.startsWith('auto'),
    });
    if (config.trigger === 'auto-time' && config.intervalMs) {
      this.startTimer(config.intervalMs);
    }
  }

  async trigger(): Promise<Uint8Array> {
    return this.dvvManager.forceSnapshot();
  }

  private startTimer(intervalMs: number): void {
    this.timer = setInterval(async () => {
      if (!this.dvvManager.isCleaningUp()) {
        await this.dvvManager.cleanup();
      }
    }, intervalMs);
  }

  stop(): void {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
  }

  getStats(): {
    updateCount: number;
    isCleaning: boolean;
    snapshotCount: number;
  } {
    return {
      updateCount: this.dvvManager.getUpdateCount(),
      isCleaning: this.dvvManager.isCleaningUp(),
      snapshotCount: this.dvvManager.getSnapshotHistory().length,
    };
  }

  async rollback(): Promise<boolean> {
    const history = this.dvvManager.getSnapshotHistory();
    if (history.length < 2) return false;
    return true;
  }
}
