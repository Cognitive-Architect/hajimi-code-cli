/* tslint:disable */
/* eslint-disable */

/**
 * 基准测试工具
 */
export class Benchmark {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * 运行搜索基准测试
     */
    static benchmark_search(index: HNSWIndex, query: Float32Array, k: number, iterations: number): number;
}

/**
 * HNSW索引 (WASM导出)
 */
export class HNSWIndex {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * 清空索引
     */
    clear(): void;
    /**
     * 插入向量
     */
    insert(id: number, vector: Float32Array): void;
    /**
     * 批量插入（高性能）
     */
    insert_batch(ids: Uint32Array, vectors: Float32Array): void;
    /**
     * 创建新索引
     */
    constructor(dimension: number, m?: number | null, ef_construction?: number | null);
    /**
     * 获取节点数量
     */
    node_count(): number;
    /**
     * 搜索最近邻
     */
    search(query: Float32Array, k: number): any;
    /**
     * 批量搜索（RISK-02 FIX: 真·批量API，减少WASM边界跨越）
     *
     * 参数:
     * - queries: 扁平化的查询向量数组 [query1_dim1, query1_dim2, ..., queryN_dimD]
     * - query_count: 查询数量
     * - k: 每个查询返回的结果数
     *
     * 返回: 扁平化的搜索结果数组 [[result1, result2, ...], ...]
     */
    searchBatch(queries: Float32Array, query_count: number, k: number): any;
    /**
     * 设置搜索参数
     */
    set_ef_search(ef: number): void;
    /**
     * 获取索引统计
     */
    stats(): any;
}

/**
 * WASM内存管理工具
 */
export class MemoryManager {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * 建议的最大内存 (400MB)
     */
    static max_memory(): number;
    /**
     * 获取当前内存使用 (字节)
     */
    static memory_usage(): number;
}

/**
 * 初始化日志
 */
export function main(): void;
