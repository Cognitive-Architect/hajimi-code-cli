//! 召回率测试数据集 - 100条384维向量
//! 
//! 生成方式: 使用 src/test_utils/onnx_mock.rs 的 generate_test_dataset_100()
//! 语义分组: 10个主题 × 10条相似向量
//! 
//! DEBT-ONNX-API-W28: 基于ONNX mock生成，非真实ONNX推理输出
//! 债务编号: TEST-014

/// 维度常量
pub const EMBEDDING_DIM: usize = 384;

/// 主题名称列表（10个语义主题）
pub const TOPIC_NAMES: [&str; 10] = [
    "rust_programming",    // 0: 系统编程、内存安全、所有权
    "machine_learning",    // 1: 神经网络、深度学习、训练
    "distributed_systems", // 2: 微服务、一致性、CAP
    "web_development",     // 3: HTTP、API、前端框架
    "database_design",     // 4: SQL、索引、事务ACID
    "cloud_computing",     // 5: AWS、容器、Kubernetes
    "security_practices",  // 6: 加密、认证、授权OAuth
    "algorithm_theory",    // 7: 排序、图论、复杂度分析
    "software_engineering", // 8: 设计模式、敏捷开发
    "data_visualization",  // 9: 图表、D3.js、BI工具
];

/// 主题种子基数（用于确定性生成）
pub const TOPIC_SEED_BASES: [u64; 10] = [
    1000, 2000, 3000, 4000, 5000,
    6000, 7000, 8000, 9000, 10000,
];

/// 获取主题ID对应的索引
pub fn get_topic_index(id: &str) -> Option<usize> {
    TOPIC_NAMES.iter().position(|t| id.starts_with(t))
}

/// 获取主题数量
pub const fn topic_count() -> usize {
    TOPIC_NAMES.len()
}

/// 获取每主题向量数量
pub const fn vectors_per_topic() -> usize {
    10
}

/// 获取总向量数量
pub const fn total_vectors() -> usize {
    topic_count() * vectors_per_topic()
}

/// 生成主题内的所有ID
pub fn generate_topic_ids(topic_idx: usize) -> Vec<String> {
    if topic_idx >= TOPIC_NAMES.len() {
        return Vec::new();
    }
    (0..vectors_per_topic())
        .map(|i| format!("{}_{:02}", TOPIC_NAMES[topic_idx], i))
        .collect()
}

/// 生成所有100条ID
pub fn generate_all_ids() -> Vec<String> {
    (0..topic_count())
        .flat_map(|t| generate_topic_ids(t))
        .collect()
}

/// 获取主题的种子
pub fn get_topic_seed(topic_idx: usize, item_idx: usize) -> u64 {
    if topic_idx >= TOPIC_NAMES.len() || item_idx >= vectors_per_topic() {
        return 0;
    }
    TOPIC_SEED_BASES[topic_idx] + (item_idx as u64 * 7)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_vectors() {
        assert_eq!(total_vectors(), 100);
    }

    #[test]
    fn test_topic_count() {
        assert_eq!(topic_count(), 10);
    }

    #[test]
    fn test_vectors_per_topic() {
        assert_eq!(vectors_per_topic(), 10);
    }

    #[test]
    fn test_generate_all_ids() {
        let ids = generate_all_ids();
        assert_eq!(ids.len(), 100);
        assert_eq!(ids[0], "rust_programming_00");
        assert_eq!(ids[99], "data_visualization_09");
    }

    #[test]
    fn test_get_topic_index() {
        assert_eq!(get_topic_index("rust_programming_05"), Some(0));
        assert_eq!(get_topic_index("machine_learning_03"), Some(1));
        assert_eq!(get_topic_index("data_visualization_09"), Some(9));
        assert_eq!(get_topic_index("unknown_topic"), None);
    }

    #[test]
    fn test_topic_seeds_unique() {
        // 确保不同主题的种子不重叠
        let mut seeds = std::collections::HashSet::new();
        for t in 0..topic_count() {
            for i in 0..vectors_per_topic() {
                let seed = get_topic_seed(t, i);
                assert!(
                    seeds.insert(seed),
                    "种子应唯一: topic={}, item={}, seed={}",
                    t, i, seed
                );
            }
        }
        assert_eq!(seeds.len(), 100);
    }

    #[test]
    fn test_embedding_dim() {
        assert_eq!(EMBEDDING_DIM, 384);
    }
}
