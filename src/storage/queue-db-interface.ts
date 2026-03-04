/**
 * 队列数据库接口定义
 * 用于抽象不同存储引擎的统一接口
 */
export interface IQueueDb {
  get(key: string): Promise<string>;
  put(key: string, value: string): Promise<void>;
  del(key: string): Promise<void>;
  close(): Promise<void>;
}
