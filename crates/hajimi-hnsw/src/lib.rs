/**
 * HNSW Rust Core - WASM优化版本
 * 
 * 目标: 比JavaScript实现快5倍的向量检索
 */

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 当panic时输出到console
#[cfg(feature = "console_error_panic_hook")]
extern crate console_error_panic_hook;

/// HNSW节点
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HNSWNode {
    pub id: u32,
    pub vector: Vec<f32>,
    pub level: u8,
    pub connections: HashMap<u8, Vec<u32>>, // level -> neighbor ids
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
    m: usize,           // 每层最大连接数
    ef_construction: usize,
    ef_search: usize,
    dimension: usize,
}

#[wasm_bindgen]
impl HNSWIndex {
    /// 创建新索引
    #[wasm_bindgen(constructor)]
    pub fn new(dimension: usize, m: Option<usize>, ef_construction: Option<usize>) -> Self {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        HNSWIndex {
            nodes: HashMap::new(),
            entry_point: None,
            max_level: 0,
            m: m.unwrap_or(16),
            ef_construction: ef_construction.unwrap_or(200),
            ef_search: 50,
            dimension,
        }
    }

    /// 插入向量
    pub fn insert(&mut self, id: u32, vector: Vec<f32>) -> Result<(), JsValue> {
        if vector.len() != self.dimension {
            return Err(JsValue::from_str("Dimension mismatch"));
        }

        let level = self._random_level();
        
        let node = HNSWNode {
            id,
            vector,
            level,
            connections: HashMap::new(),
        };

        // 第一层直接设为entry point
        if self.entry_point.is_none() {
            self.entry_point = Some(id);
            self.max_level = level;
        }

        self.nodes.insert(id, node);
        
        // 更新最大层数
        if level > self.max_level {
            self.max_level = level;
            self.entry_point = Some(id);
        }

        Ok(())
    }

    /// 搜索最近邻
    pub fn search(&self, query: Vec<f32>, k: usize) -> Result<JsValue, JsValue> {
        if query.len() != self.dimension {
            return Err(JsValue::from_str("Dimension mismatch"));
        }

        let entry = match self.entry_point {
            Some(id) => id,
            None => return Ok(serde_wasm_bindgen::to_value(&Vec::<SearchResult>::new())?),
        };

        // 简化版搜索: 计算所有距离返回最近的k个
        let mut results: Vec<SearchResult> = self
            .nodes
            .values()
            .map(|node| SearchResult {
                id: node.id,
                distance: Self::_cosine_distance(&query, &node.vector),
            })
            .collect();

        results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        results.truncate(k);

        Ok(serde_wasm_bindgen::to_value(&results)?)
    }

    /// 获取索引统计
    pub fn stats(&self) -> Result<JsValue, JsValue> {
        let stats = serde_json::json!({
            "nodeCount": self.nodes.len(),
            "maxLevel": self.max_level,
            "entryPoint": self.entry_point,
            "dimension": self.dimension,
            "m": self.m,
        });
        
        Ok(serde_wasm_bindgen::to_value(&stats)?)
    }

    /// 生成随机层数
    fn _random_level(&self) -> u8 {
        // 简化: 使用均匀分布
        let mut level = 0u8;
        while level < 16 && js_sys::Math::random() < 0.5 {
            level += 1;
        }
        level
    }

    /// 余弦距离 (越小越相似)
    fn _cosine_distance(a: &[f32], b: &[f32]) -> f32 {
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for i in 0..a.len() {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }

        let denom = (norm_a * norm_b).sqrt();
        if denom == 0.0 {
            return 1.0;
        }

        // 转换为距离: 1 - similarity
        1.0 - (dot / denom)
    }
}

/// WASM内存管理工具
#[wasm_bindgen]
pub struct MemoryManager;

#[wasm_bindgen]
impl MemoryManager {
    /// 获取当前内存使用 (字节)
    pub fn memory_usage() -> usize {
        // 这是WASM线性内存的大小
        (js_sys::Function::new_no_args("return wasm.memory.buffer.byteLength")
            .call0(&JsValue::NULL)
            .unwrap()
            .as_f64()
            .unwrap() as usize)
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
