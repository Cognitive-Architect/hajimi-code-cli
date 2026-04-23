//! TypeRacing Algorithm - 类型预测算法
//!
//! 提供 confidence 权重计算、候选排序和预测合并功能。
//! 算法复杂度: O(N log N)

use crate::engine::{PredictionNode, PredictionSource};

/// 权重配置常量
const WEIGHT_LSP_HOVER: f64 = 0.95;
const WEIGHT_LSP_DEFINITION: f64 = 0.90;
const WEIGHT_LSP_REFERENCES: f64 = 0.75;
const WEIGHT_HEURISTIC: f64 = 0.50;
const WEIGHT_HISTORICAL: f64 = 0.60;
const DECAY_FACTOR: f64 = 0.95;

/// 计算预测的加权 confidence 分数
pub fn calculate_weighted_confidence(source: &PredictionSource, raw_confidence: f64) -> f64 {
    let source_weight = match source {
        PredictionSource::LspHover => WEIGHT_LSP_HOVER,
        PredictionSource::LspDefinition => WEIGHT_LSP_DEFINITION,
        PredictionSource::LspReferences => WEIGHT_LSP_REFERENCES,
        PredictionSource::Heuristic => WEIGHT_HEURISTIC,
        PredictionSource::Historical => WEIGHT_HISTORICAL,
    };
    (raw_confidence * source_weight).min(1.0).max(0.0)
}

/// 对预测候选进行排序 (O(N log N))
pub fn rank_predictions(predictions: &mut [PredictionNode]) {
    predictions.sort_by(|a, b| {
        let score_a = calculate_weighted_confidence(&a.source, a.confidence);
        let score_b = calculate_weighted_confidence(&b.source, b.confidence);
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// 合并相同类型的预测，累加 confidence (O(N log N))
pub fn merge_predictions(mut predictions: Vec<PredictionNode>) -> Vec<PredictionNode> {
    if predictions.len() <= 1 {
        return predictions;
    }
    predictions.sort_by(|a, b| a.type_name.cmp(&b.type_name));
    let mut merged: Vec<PredictionNode> = Vec::new();
    for pred in predictions {
        if let Some(last) = merged.last_mut() {
            if last.type_name == pred.type_name {
                last.confidence = (last.confidence + pred.confidence * DECAY_FACTOR).min(1.0);
                continue;
            }
        }
        merged.push(pred);
    }
    rank_predictions(&mut merged);
    merged
}

/// 选择 top-k 预测结果
pub fn select_top_k(predictions: &[PredictionNode], k: usize) -> Vec<PredictionNode> {
    let mut sorted = predictions.to_vec();
    rank_predictions(&mut sorted);
    sorted.into_iter().take(k).collect()
}

/// 计算预测列表的平均 confidence
pub fn average_confidence(predictions: &[PredictionNode]) -> f64 {
    if predictions.is_empty() {
        return 0.0;
    }
    let sum: f64 = predictions
        .iter()
        .map(|p| calculate_weighted_confidence(&p.source, p.confidence))
        .sum();
    sum / predictions.len() as f64
}

/// 根据类型名称进行二分查找
pub fn binary_search_by_type(predictions: &[PredictionNode], type_name: &str) -> Result<usize, usize> {
    predictions.binary_search_by(|p| p.type_name.as_str().cmp(type_name))
}
