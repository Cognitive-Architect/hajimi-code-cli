/**
 * TypeScript definitions for hajimi-hnsw
 */

export interface SearchResult {
  id: number;
  distance: number;
}

export interface HNSWStats {
  nodeCount: number;
  maxLevel: number;
  entryPoint: number | null;
  dimension: number;
  m: number;
}

export class HNSWIndex {
  constructor(dimension: number, m?: number, ef_construction?: number);
  insert(id: number, vector: Float32Array | number[]): void;
  search(query: Float32Array | number[], k?: number): SearchResult[];
  stats(): HNSWStats;
  free(): void;
}

export class MemoryManager {
  static memoryUsage(): number;
  static maxMemory(): number;
}

export const __wasm: any;
