/**
 * HNSW (Hierarchical Navigable Small World) - Full Rust Implementation
 * 
 * 目标: 比JavaScript实现快5倍的向量检索
 * 实现: 完整HNSW算法（分层导航图 + 贪心搜索 + 邻居选择）
 */

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;

// 当panic时输出到console
#[cfg(feature = "console_error_panic_hook")]
extern crate console_error_panic_hook;

/// 优先队列项（用于贪心搜索）
#[derive(Clone, Debug)]
struct Candidate {
    id: u32,
    distance: f32,
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for Candidate {}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // 最小堆：距离小的优先
        other.distance.partial_cmp(&self.distance)
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// HNSW节点
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HNSWNode {
    pub id: u32,
    pub vector: Vec<f32>,
    pub level: u8,
    // 每层连接的邻居ID: level -> [neighbor_id, ...]
    pub connections: HashMap<u8, Vec<u32>>,
}

impl HNSWNode {
    fn new(id: u32, vector: Vec<f32>, level: u8) -> Self {
        let mut connections = HashMap::new();
        for i in 0..=level {
            connections.insert(i, Vec::new());
        }
        HNSWNode { id, vector, level, connections }
    }
}

/// 搜索结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: u32,
    pub distance: f32,
}

/// HNSW索引 (WASM导出)
#[wasm_bindgen]
pub struct HNSWIndex {
    nodes: HashMap<u32, HNSWNode>,
    entry_point: Option<u32>,
    max_level: u8,
    m: usize,              // 每层最大连接数
    ef_construction: usize, // 构建时的搜索深度
    ef_search: usize,       // 搜索时的搜索深度
    dimension: usize,
    level_mult: f64,        // 层概率因子
}

#[wasm_bindgen]
impl HNSWIndex {
    /// 创建新索引
    #[wasm_bindgen(constructor)]
    pub fn new(dimension: usize, m: Option<usize>, ef_construction: Option<usize>) -> Self {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        let m = m.unwrap_or(16);
        HNSWIndex {
            nodes: HashMap::new(),
            entry_point: None,
            max_level: 0,
            m,
            ef_construction: ef_construction.unwrap_or(200),
            ef_search: 64,
            dimension,
            level_mult: 1.0 / (m as f64).ln(),
        }
    }

    /// 设置搜索参数
    pub fn set_ef_search(&mut self, ef: usize) {
        self.ef_search = ef;
    }

    /// 插入向量
    pub fn insert(&mut self, id: u32, vector: Vec<f32>) -> Result<(), JsValue> {
        if vector.len() != self.dimension {
            return Err(JsValue::from_str(&format!(
                "Dimension mismatch: expected {}, got {}", 
                self.dimension, vector.len()
            )));
        }

        // 检查是否已存在
        if self.nodes.contains_key(&id) {
            return Err(JsValue::from_str(&format!("Node {} already exists", id)));
        }

        let level = self._random_level();
        let mut node = HNSWNode::new(id, vector, level);

        // 第一个节点作为入口
        if self.entry_point.is_none() {
            self.entry_point = Some(id);
            self.max_level = level;
            // 初始化各层连接
            for i in 0..=level {
                node.connections.insert(i, Vec::new());
            }
            self.nodes.insert(id, node);
            return Ok(());
        }

        // 从最高层开始搜索入口点
        let mut current_entry = self.entry_point.unwrap();
        
        // 如果新节点层数高于当前最大层，更新入口
        if level > self.max_level {
            self.max_level = level;
            self.entry_point = Some(id);
        }

        // 贪心搜索：从最高层下降到level+1层
        for i in (level + 1)..=self.max_level {
            if let Some(new_entry) = self._search_layer_simple(&node.vector, current_entry, i) {
                current_entry = new_entry;
            }
        }

        // 从level层开始逐层建立连接
        for i in (0..=level.min(self.max_level)).rev() {
            // 搜索ef个最近邻
            let neighbors = self._search_layer_ef(&node.vector, current_entry, self.ef_construction, i);
            
            // 选择M个邻居（启发式）
            let selected = self._select_neighbors(&node.vector, &neighbors, self.m);
            
            // 建立双向连接
            node.connections.insert(i, selected.clone());
            
            for &neighbor_id in &selected {
                if let Some(neighbor) = self.nodes.get_mut(&neighbor_id) {
                    neighbor.connections.entry(i).or_insert_with(Vec::new).push(id);
                    
                    // 如果邻居连接过多，进行裁剪
                    self._prune_connections(neighbor_id, i);
                }
            }
            
            // 更新当前入口为下一层的起点
            if !neighbors.is_empty() {
                current_entry = neighbors[0].id;
            }
        }

        self.nodes.insert(id, node);
        Ok(())
    }

    /// 搜索最近邻
    pub fn search(&self, query: Vec<f32>, k: usize) -> Result<JsValue, JsValue> {
        if query.len() != self.dimension {
            return Err(JsValue::from_str("Dimension mismatch"));
        }

        if self.entry_point.is_none() || self.nodes.is_empty() {
            return Ok(serde_wasm_bindgen::to_value(&Vec::<SearchResult>::new())?);
        }

        let k = k.min(self.nodes.len());
        let entry = self.entry_point.unwrap();

        // 从最高层贪心下降到第1层
        let mut current = entry;
        for i in (1..=self.max_level).rev() {
            if let Some(new_current) = self._search_layer_simple(&query, current, i) {
                current = new_current;
            }
        }

        // 在第0层使用ef搜索
        let ef = self.ef_search.max(k);
        let results = self._search_layer_ef(&query, current, ef, 0);
        
        // 返回前k个结果
        let final_results: Vec<SearchResult> = results.into_iter()
            .take(k)
            .map(|c| SearchResult { id: c.id, distance: c.distance })
            .collect();

        Ok(serde_wasm_bindgen::to_value(&final_results)?)
    }

    /// 批量搜索（RISK-02 FIX: 真·批量API，减少WASM边界跨越）
    /// 
    /// 参数:
    /// - queries: 扁平化的查询向量数组 [query1_dim1, query1_dim2, ..., queryN_dimD]
    /// - query_count: 查询数量
    /// - k: 每个查询返回的结果数
    /// 
    /// 返回: 扁平化的搜索结果数组 [[result1, result2, ...], ...]
    #[wasm_bindgen(js_name = searchBatch)]
    pub fn search_batch(&self, queries: Vec<f32>, query_count: usize, k: usize) -> Result<JsValue, JsValue> {
        if query_count == 0 {
            return Ok(serde_wasm_bindgen::to_value(&Vec::<Vec<SearchResult>>::new())?);
        }
        
        if queries.len() != query_count * self.dimension {
            return Err(JsValue::from_str(&format!(
                "Batch dimension mismatch: expected {} dims per query, got {} total for {} queries", 
                self.dimension, queries.len(), query_count
            )));
        }

        let k = k.min(self.nodes.len());
        let mut all_results: Vec<Vec<SearchResult>> = Vec::with_capacity(query_count);

        // 批量处理所有查询（单次WASM调用）
        for i in 0..query_count {
            let start = i * self.dimension;
            let end = start + self.dimension;
            // 零拷贝：直接切片，不分配新Vec
            let query = &queries[start..end];
            
            let results = self._search_single(query, k);
            all_results.push(results);
        }

        Ok(serde_wasm_bindgen::to_value(&all_results)?)
    }

    /// 零拷贝批量搜索（修复FIND-025-01）
    /// 
    /// 直接接收&[f32]切片，避免Vec<f32>分配，实现JS→WASM零拷贝
    /// JS调用: index.searchBatchZeroCopy(float32Array, dim, k)
    #[wasm_bindgen(js_name = "searchBatchZeroCopy")]
    pub fn search_batch_zero_copy(
        &self,
        data: &[f32],  // 关键：&[f32]而非Vec<f32>，零拷贝
        dim: usize,
        k: usize,
    ) -> Result<JsValue, JsValue> {
        // 参数验证
        if data.is_empty() {
            return Err(JsValue::from_str("Empty data"));
        }
        if dim == 0 {
            return Err(JsValue::from_str("Zero dimension"));
        }
        if data.len() % dim != 0 {
            return Err(JsValue::from_str("Data length not divisible by dimension"));
        }
        let num_vectors = data.len() / dim;
        let k = k.min(self.nodes.len());
        
        // 批量处理，零拷贝切片访问
        let mut all_results: Vec<Vec<SearchResult>> = Vec::with_capacity(num_vectors);
        for i in 0..num_vectors {
            let start = i * dim;
            let end = start + dim;
            let query = &data[start..end];  // 切片，零拷贝
            let results = self._search_single(query, k);
            all_results.push(results);
        }
        
        Ok(serde_wasm_bindgen::to_value(&all_results)?)
    }

    /// 单条搜索内部方法（零拷贝，使用切片）
    fn _search_single(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        if self.entry_point.is_none() || self.nodes.is_empty() {
            return Vec::new();
        }

        let entry = self.entry_point.unwrap();
        
        // 从最高层贪心下降到第1层
        let mut current = entry;
        for level in (1..=self.max_level).rev() {
            if let Some(new_current) = self._search_layer_simple(query, current, level) {
                current = new_current;
            }
        }

        // 在第0层使用ef搜索
        let ef = self.ef_search.max(k);
        let candidates = self._search_layer_ef(query, current, ef, 0);
        
        // 返回前k个结果
        candidates.into_iter()
            .take(k)
            .map(|c| SearchResult { id: c.id, distance: c.distance })
            .collect()
    }

    /// 批量插入（高性能）
    pub fn insert_batch(&mut self, ids: Vec<u32>, vectors: Vec<f32>) -> Result<(), JsValue> {
        let batch_size = ids.len();
        if vectors.len() != batch_size * self.dimension {
            return Err(JsValue::from_str("Batch dimension mismatch"));
        }

        for (i, id) in ids.iter().enumerate() {
            let start = i * self.dimension;
            let end = start + self.dimension;
            let vector = vectors[start..end].to_vec();
            self.insert(*id, vector)?;
        }

        Ok(())
    }

    /// 获取索引统计
    pub fn stats(&self) -> Result<JsValue, JsValue> {
        let total_connections: usize = self.nodes.values()
            .map(|n| n.connections.values().map(|v| v.len()).sum::<usize>())
            .sum();

        let avg_connections = if !self.nodes.is_empty() {
            total_connections as f64 / self.nodes.len() as f64
        } else {
            0.0
        };

        let stats = serde_json::json!({
            "nodeCount": self.nodes.len(),
            "maxLevel": self.max_level,
            "entryPoint": self.entry_point,
            "dimension": self.dimension,
            "m": self.m,
            "efConstruction": self.ef_construction,
            "efSearch": self.ef_search,
            "avgConnections": avg_connections,
            "totalConnections": total_connections,
        });
        
        Ok(serde_wasm_bindgen::to_value(&stats)?)
    }

    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// 清空索引
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.entry_point = None;
        self.max_level = 0;
    }

    // ============ 私有方法 ============

    /// 生成随机层数（指数分布）
    fn _random_level(&self) -> u8 {
        let mut level = 0u8;
        let max_level = 16u8;
        
        while level < max_level {
            let r = js_sys::Math::random();
            if r >= std::f64::consts::E.powf(-1.0 / self.level_mult) {
                break;
            }
            level += 1;
        }
        
        level
    }

    /// 余弦距离（越小越相似）
    fn _cosine_distance(a: &[f32], b: &[f32]) -> f32 {
        let mut dot = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;

        for i in 0..a.len() {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }

        let denom = (norm_a * norm_b).sqrt();
        if denom == 0.0 {
            return 1.0;
        }

        1.0 - (dot / denom)
    }

    /// 计算查询向量与节点的距离
    fn _distance_to_query(&self, query: &[f32], node_id: u32) -> f32 {
        if let Some(node) = self.nodes.get(&node_id) {
            Self::_cosine_distance(query, &node.vector)
        } else {
            f32::INFINITY
        }
    }

    /// 单层简单贪心搜索（返回最近的一个）
    fn _search_layer_simple(&self, query: &[f32], entry_id: u32, level: u8) -> Option<u32> {
        let mut current = entry_id;
        let mut current_dist = self._distance_to_query(query, current);
        let mut changed = true;

        while changed {
            changed = false;
            
            if let Some(node) = self.nodes.get(&current) {
                if let Some(neighbors) = node.connections.get(&level) {
                    for &neighbor_id in neighbors {
                        let dist = self._distance_to_query(query, neighbor_id);
                        if dist < current_dist {
                            current = neighbor_id;
                            current_dist = dist;
                            changed = true;
                        }
                    }
                }
            } else {
                break;
            }
        }

        Some(current)
    }

    /// 多层搜索（ef控制）
    fn _search_layer_ef(&self, query: &[f32], entry_id: u32, ef: usize, level: u8) -> Vec<Candidate> {
        let mut visited = HashMap::new();
        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();

        let entry_dist = self._distance_to_query(query, entry_id);
        visited.insert(entry_id, entry_dist);
        candidates.push(Candidate { id: entry_id, distance: entry_dist });
        results.push(Candidate { id: entry_id, distance: entry_dist });

        while let Some(current) = candidates.pop() {
            // 获取结果堆中的最差结果
            let worst_result_dist = results.peek().map(|c| c.distance).unwrap_or(f32::INFINITY);
            
            if current.distance > worst_result_dist {
                break;
            }

            if let Some(node) = self.nodes.get(&current.id) {
                if let Some(neighbors) = node.connections.get(&level) {
                    for &neighbor_id in neighbors {
                        if visited.contains_key(&neighbor_id) {
                            continue;
                        }

                        let dist = self._distance_to_query(query, neighbor_id);
                        visited.insert(neighbor_id, dist);

                        if results.len() < ef || dist < worst_result_dist {
                            candidates.push(Candidate { id: neighbor_id, distance: dist });
                            results.push(Candidate { id: neighbor_id, distance: dist });
                            
                            if results.len() > ef {
                                results.pop();
                            }
                        }
                    }
                }
            }
        }

        // 转换为Vec并排序（距离从小到大）
        let mut result_vec: Vec<Candidate> = results.into_iter().collect();
        result_vec.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Ordering::Equal));
        result_vec
    }

    /// 选择邻居（启发式，保持图连通性）
    fn _select_neighbors(&self, _query: &[f32], candidates: &[Candidate], m: usize) -> Vec<u32> {
        if candidates.len() <= m {
            return candidates.iter().map(|c| c.id).collect();
        }

        // 简化策略：选择距离最近的m个
        // 更复杂的策略可以加入多样性启发式
        candidates.iter().take(m).map(|c| c.id).collect()
    }

    /// 裁剪连接（保持最多2*M个连接）
    /// 返回true表示进行了裁剪
    fn _prune_connections(&mut self, node_id: u32, level: u8) -> bool {
        // 首先检查是否需要裁剪（使用不可变借用）
        let needs_prune = if let Some(node) = self.nodes.get(&node_id) {
            if let Some(connections) = node.connections.get(&level) {
                connections.len() > self.m * 2
            } else {
                false
            }
        } else {
            false
        };

        if !needs_prune {
            return false;
        }

        // 计算到所有连接的距离并排序
        let node_vector = if let Some(node) = self.nodes.get(&node_id) {
            node.vector.clone()
        } else {
            return false;
        };

        let connections_to_sort: Vec<u32> = if let Some(node) = self.nodes.get(&node_id) {
            if let Some(conns) = node.connections.get(&level) {
                conns.clone()
            } else {
                return false;
            }
        } else {
            return false;
        };

        // 计算距离并排序
        let mut with_distances: Vec<(u32, f32)> = connections_to_sort
            .iter()
            .map(|&conn_id| {
                let dist = if let Some(conn_node) = self.nodes.get(&conn_id) {
                    Self::_cosine_distance(&node_vector, &conn_node.vector)
                } else {
                    f32::INFINITY
                };
                (conn_id, dist)
            })
            .collect();

        // 按距离排序（最近的在前）
        with_distances.sort_by(|a, b| {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        // 保留最近的m*2个
        let keep_count = self.m * 2;
        let to_keep: Vec<u32> = with_distances
            .into_iter()
            .take(keep_count)
            .map(|(id, _)| id)
            .collect();

        // 更新连接（可变借用）
        if let Some(node) = self.nodes.get_mut(&node_id) {
            if let Some(connections) = node.connections.get_mut(&level) {
                *connections = to_keep;
                return true;
            }
        }

        false
    }
}

/// WASM内存管理工具
#[wasm_bindgen]
pub struct MemoryManager;

#[wasm_bindgen]
impl MemoryManager {
    /// 获取当前内存使用 (字节)
    pub fn memory_usage() -> usize {
        js_sys::Function::new_no_args("return wasm.memory.buffer.byteLength")
            .call0(&JsValue::NULL)
            .unwrap()
            .as_f64()
            .unwrap() as usize
    }

    /// 建议的最大内存 (400MB)
    pub fn max_memory() -> usize {
        400 * 1024 * 1024
    }
}

/// 初始化日志
#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// ============ 测试工具 ============

/// 基准测试工具
#[wasm_bindgen]
pub struct Benchmark;

#[wasm_bindgen]
impl Benchmark {
    /// 运行搜索基准测试
    pub fn benchmark_search(index: &HNSWIndex, query: Vec<f32>, k: usize, iterations: usize) -> f64 {
        let start = js_sys::Date::now();
        
        for _ in 0..iterations {
            let _ = index.search(query.clone(), k);
        }
        
        let end = js_sys::Date::now();
        (end - start) / iterations as f64
    }
}
