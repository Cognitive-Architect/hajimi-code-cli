/**
 * LevelDB优化配置 - Write Amplification < 3x
 * 策略：增大写缓冲(64MB)，延迟compaction触发
 * Leveled Compaction调优配置
 */
import { Level } from 'level';
import { IQueueDb } from './queue-db-interface';
import { WriteAmpMonitor } from './write-amp-monitor';
import * as fs from 'fs/promises';
import * as path from 'path';

const BYTES_PER_MB = 1024 * 1024;
const DEFAULT_WRITE_BUFFER_MB = 64;     // 64MB vs 4MB默认
const LEVEL0_COMPACTION_TRIGGER = 10;   // 默认4→10，延迟compaction
const LEVEL0_SLOWDOWN_TRIGGER = 20;     // 默认8→20，延迟写入降速
const LEVEL0_STOP_TRIGGER = 36;         // 默认12→36，延迟写入停止
const SSTABLE_FILE_SIZE_MB = 8;         // 8MB vs 2MB默认

export interface OptimizedLevelDBOptions {
  path: string;
  writeBufferSizeMB?: number;
  enableMonitoring?: boolean;
}

export class OptimizedLevelDB implements IQueueDb {
  private db: Level<string, string>;
  private monitor: WriteAmpMonitor;
  private dbPath: string;

  constructor(options: OptimizedLevelDBOptions) {
    const writeBufferSize = (options.writeBufferSizeMB ?? DEFAULT_WRITE_BUFFER_MB) * BYTES_PER_MB;
    this.dbPath = options.path;
    
    // LevelDB优化配置：减少Write Amplification
    // 使用类型断言传递底层LevelDB选项（level包类型定义未包含）
    this.db = new Level(options.path, {
      valueEncoding: 'utf8',
      writeBufferSize,  // 64MB vs 4MB默认
      level0FileNumCompactionTrigger: LEVEL0_COMPACTION_TRIGGER,  // Leveled Compaction调优
      level0SlowdownWritesTrigger: LEVEL0_SLOWDOWN_TRIGGER,
      level0StopWritesTrigger: LEVEL0_STOP_TRIGGER,
      maxFileSize: SSTABLE_FILE_SIZE_MB * BYTES_PER_MB,
    } as unknown as ConstructorParameters<typeof Level<string, string>>[1]);
    this.monitor = new WriteAmpMonitor();
  }

  async put(key: string, value: string): Promise<void> {
    const before = await this.getDBSize();
    await this.db.put(key, value);
    const after = await this.getDBSize();
    this.monitor.recordWrite(value.length, after - before);
  }

  private async getDBSize(): Promise<number> {
    let total = 0;
    try {
      const files = await fs.readdir(this.dbPath);
      for (const f of files) {
        if (f.endsWith('.ldb') || f.endsWith('.log')) {
          total += (await fs.stat(path.join(this.dbPath, f))).size;
        }
      }
    } catch { /* 目录可能不存在，忽略 */ }
    return total;
  }

  getWriteAmplification(): number {
    return this.monitor.getWriteAmplification();
  }

  getMetrics() { return this.monitor.metrics(); }

  async get(key: string): Promise<string> { 
    return this.db.get(key); 
  }
  
  async del(key: string): Promise<void> { 
    return this.db.del(key); 
  }
  
  async close(): Promise<void> { 
    return this.db.close(); 
  }
}

export default OptimizedLevelDB;
