/**
 * Yjs内存监控器
 * 追踪RSS、堆内存、文档大小趋势
 */
interface MemorySample {
  timestamp: number;
  rss: number;
  heap: number;
  docSize: number;
}

export class MemoryMonitor {
  private samples: MemorySample[] = [];
  private maxSamples = 100;

  record(docSizeBytes: number): void {
    const usage = process.memoryUsage();
    this.samples.push({
      timestamp: Date.now(),
      rss: usage.rss,
      heap: usage.heapUsed,
      docSize: docSizeBytes,
    });
    if (this.samples.length > this.maxSamples) {
      this.samples.shift();
    }
  }

  getGrowthTrend(): number {
    if (this.samples.length < 10) return 1;
    const first = this.samples[0]!;
    const last = this.samples[this.samples.length - 1]!;
    const timeDelta = (last.timestamp - first.timestamp) / 1000;
    if (timeDelta < 1) return 1;
    const growthRate = (last.docSize - first.docSize) / timeDelta;
    return growthRate > 0 ? 1 + growthRate / 1000 : 1;
  }

  getStats(): { rssMB: number; heapMB: number; docSizeMB: number; sampleCount: number; growthTrend: number; } {
    const latest = this.samples[this.samples.length - 1];
    return {
      rssMB: latest ? latest.rss / (1024 * 1024) : 0,
      heapMB: latest ? latest.heap / (1024 * 1024) : 0,
      docSizeMB: latest ? latest.docSize / (1024 * 1024) : 0,
      sampleCount: this.samples.length,
      growthTrend: this.getGrowthTrend(),
    };
  }

  shouldCleanup(thresholdGrowth = 1.5): boolean { return this.getGrowthTrend() > thresholdGrowth; }

  reset(): void {
    this.samples = [];
  }
}
