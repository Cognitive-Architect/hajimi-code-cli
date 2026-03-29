/**
 * RocksDB适配器 - Tiered Compaction
 * Write Amplification目标：< 2x
 * 使用Universal Compaction减少写放大
 */
import type { IQueueDb } from './queue-db-interface';

const BYTES_PER_MB = 1024 * 1024;
const WRITE_BUFFER_MB = 64;
const TARGET_FILE_SIZE_MB = 64;
const MAX_WRITE_BUFFER_NUMBER = 3;
const TARGET_FILE_SIZE_MULTIPLIER = 2;
const LEVEL0_COMPACTION_TRIGGER = 4;
const LEVEL0_SLOWDOWN_TRIGGER = 20;
const LEVEL0_STOP_TRIGGER = 36;

export interface RocksDBOptions {
  path: string;
  compactionStyle?: 'level' | 'universal' | 'fifo';
}

interface RocksDBInstance {
  put(key: string, value: string, cb: (err: Error | null) => void): void;
  get(key: string, cb: (err: Error | null, val: string) => void): void;
  del(key: string, cb: (err: Error | null) => void): void;
  close(cb: () => void): void;
  open(opts: Record<string, unknown>, cb: (err: Error | null) => void): void;
}

export class RocksDBAdapter implements IQueueDb {
  private db: RocksDBInstance;

  constructor(options: RocksDBOptions) {
    const rocksdb = require('rocksdb');
    this.db = rocksdb(options.path) as RocksDBInstance;
    this.db.open({
      create_if_missing: true,
      compaction_style: options.compactionStyle ?? 'universal',  // Tiered Compaction
      write_buffer_size: WRITE_BUFFER_MB * BYTES_PER_MB,         // 64MB
      max_write_buffer_number: MAX_WRITE_BUFFER_NUMBER,
      target_file_size_base: TARGET_FILE_SIZE_MB * BYTES_PER_MB,  // 64MB
      target_file_size_multiplier: TARGET_FILE_SIZE_MULTIPLIER,
      level0_file_num_compaction_trigger: LEVEL0_COMPACTION_TRIGGER,
      level0_slowdown_writes_trigger: LEVEL0_SLOWDOWN_TRIGGER,
      level0_stop_writes_trigger: LEVEL0_STOP_TRIGGER,
    }, (err: Error | null) => { if (err) throw err; });
  }

  put(key: string, value: string): Promise<void> {
    return new Promise((resolve, reject) => {
      this.db.put(key, value, (err) => err ? reject(err) : resolve());
    });
  }

  get(key: string): Promise<string> {
    return new Promise((resolve, reject) => {
      this.db.get(key, (err, val) => err ? reject(err) : resolve(val));
    });
  }

  del(key: string): Promise<void> {
    return new Promise((resolve, reject) => {
      this.db.del(key, (err) => err ? reject(err) : resolve());
    });
  }

  close(): Promise<void> {
    return new Promise((resolve) => this.db.close(resolve));
  }
}
