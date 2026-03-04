/**
 * Write Amplification监控器
 * WA = total_bytes_written / user_bytes_written
 */
export class WriteAmpMonitor {
  private userBytes = 0;
  private totalBytes = 0;
  private compactionBytes = 0;

  recordWrite(userBytes: number, totalDelta: number): void {
    this.userBytes += userBytes;
    this.totalBytes += totalDelta;
    // Compaction导致的额外写入
    if (totalDelta > userBytes) {
      this.compactionBytes += (totalDelta - userBytes);
    }
  }

  getWriteAmplification(): number {
    if (this.userBytes === 0) return 1.0;
    return this.totalBytes / this.userBytes;
  }

  getCompactionRatio(): number {
    if (this.userBytes === 0) return 0;
    return this.compactionBytes / this.userBytes;
  }

  reset(): void {
    this.userBytes = 0;
    this.totalBytes = 0;
    this.compactionBytes = 0;
  }

  metrics(): {
    writeAmplification: number;
    compactionRatio: number;
    userBytes: number;
    totalBytes: number;
  } {
    return {
      writeAmplification: this.getWriteAmplification(),
      compactionRatio: this.getCompactionRatio(),
      userBytes: this.userBytes,
      totalBytes: this.totalBytes,
    };
  }
}
