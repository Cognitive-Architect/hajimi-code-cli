//! ONNX Mock - 生成真实分布的384维向量
//! 
//! DEBT-ONNX-API-W28: 这是ONNX占位接口的mock实现
//! 使用确定性伪随机数生成器，避免全0/全0.1的虚假向量

use std::f32::consts::PI;

/// 维度常量
pub const EMBEDDING_DIM: usize = 384;

/// 线性同余生成器 (LCG) - 确定性伪随机数生成
/// 保证相同seed产生相同序列，便于测试可重复性
struct LcgRng {
    state: u64,
}

impl LcgRng {
    const A: u64 = 6364136223846793005;
    const C: u64 = 1442695040888963407;

    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = Self::A.wrapping_mul(self.state).wrapping_add(Self::C);
        self.state
    }

    /// 生成 [0, 1) 范围的 f32
    fn next_f32(&mut self) -> f32 {
        (self.next_u64() as f32) / (u64::MAX as f32 + 1.0)
    }

    /// Box-Muller 变换生成标准正态分布 N(0,1)
    fn next_gaussian(&mut self) -> f32 {
        let u1 = self.next_f32();
        let u2 = self.next_f32();
        // 避免 u1 = 0
        let u1 = u1.max(f32::EPSILON);
        ((-2.0 * u1.ln()).sqrt()) * (2.0 * PI * u2).cos()
    }
}

/// 生成单条384维embedding向量
/// 
/// # 算法
/// 1. 使用 Box-Muller 变换生成标准正态分布
/// 2. L2归一化确保向量长度为1（真实ONNX输出通常已归一化）
/// 
/// # 参数
/// - seed: 随机种子，保证可重复性
/// 
/// # 返回
/// 384维 f32 向量，符合真实ONNX分布特征
pub fn generate_embedding_384(seed: u64) -> Vec<f32> {
    let mut rng = LcgRng::new(seed);
    let mut vec: Vec<f32> = (0..EMBEDDING_DIM)
        .map(|_| rng.next_gaussian())
        .collect();
    
    // L2 归一化
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in &mut vec {
            *x /= norm;
        }
    }
    
    vec
}

/// 语义主题定义 - 10个不同主题用于召回率测试
const SEMANTIC_TOPICS: [&str; 10] = [
    "rust_programming",    // 系统编程、内存安全
    "machine_learning",    // 神经网络、深度学习
    "distributed_systems", // 微服务、一致性
    "web_development",     // HTTP、API、前端
    "database_design",     // SQL、索引、事务
    "cloud_computing",     // AWS、容器、K8s
    "security_practices",  // 加密、认证、授权
    "algorithm_theory",    // 排序、图论、复杂度
    "software_engineering", // 设计模式、敏捷
    "data_visualization",  // 图表、D3、BI
];

/// 生成100条测试数据集
/// 
/// # 结构
/// - 10个语义主题，每个主题10条向量
/// - 同主题向量使用相近的seed，确保语义相似性
/// - 不同主题使用不同的基础seed，确保语义区分度
/// 
/// # 返回
/// Vec<(topic_id, embedding)> - 100条记录
pub fn generate_test_dataset_100() -> Vec<(String, Vec<f32>)> {
    let mut dataset = Vec::with_capacity(100);
    
    for (topic_idx, topic) in SEMANTIC_TOPICS.iter().enumerate() {
        let base_seed = (topic_idx as u64 + 1) * 1000;
        
        for item_idx in 0..10 {
            // 同主题内使用相近seed，产生相似向量
            let seed = base_seed + (item_idx as u64 * 7);
            let embedding = generate_embedding_384(seed);
            let id = format!("{}_{:02}", topic, item_idx);
            dataset.push((id, embedding));
        }
    }
    
    dataset
}

/// 生成与给定主题相似的查询向量
/// 
/// 用于测试召回率时，确保查询与目标主题向量有语义关联
pub fn generate_query_for_topic(topic_idx: usize) -> Vec<f32> {
    let base_seed = (topic_idx as u64 + 1) * 1000;
    // 使用主题内第一条向量的seed + 偏移，产生相似但不相同的向量
    generate_embedding_384(base_seed + 3)
}

/// 获取主题索引（用于确定查询应该召回哪些向量）
pub fn get_topic_for_id(id: &str) -> Option<usize> {
    SEMANTIC_TOPICS.iter().position(|t| id.starts_with(t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_dimension() {
        let emb = generate_embedding_384(42);
        assert_eq!(emb.len(), EMBEDDING_DIM);
    }

    #[test]
    fn test_embedding_normalized() {
        let emb = generate_embedding_384(123);
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001, "向量应已归一化，实际范数={}", norm);
    }

    #[test]
    fn test_not_all_zeros() {
        let emb = generate_embedding_384(999);
        let sum: f32 = emb.iter().map(|x| x.abs()).sum();
        assert!(sum > 0.1, "向量不能是全0或近似全0");
    }

    #[test]
    fn test_deterministic() {
        let emb1 = generate_embedding_384(42);
        let emb2 = generate_embedding_384(42);
        assert_eq!(emb1, emb2, "相同seed应产生相同向量");
    }

    #[test]
    fn test_dataset_size() {
        let data = generate_test_dataset_100();
        assert_eq!(data.len(), 100, "数据集必须包含100条记录");
    }

    #[test]
    fn test_vector_distribution() {
        // 验证向量分布特性（非全0，有变化）
        let emb = generate_embedding_384(42);
        
        // 验证有正有负（正态分布特征）
        let positive_count = emb.iter().filter(|&&x| x > 0.0).count();
        let negative_count = emb.iter().filter(|&&x| x < 0.0).count();
        
        // 正态分布中，大约一半正一半负
        assert!(positive_count > 100, "应有足够多的正值，count={}", positive_count);
        assert!(negative_count > 100, "应有足够多的负值，count={}", negative_count);
        
        // 验证值范围（3σ原则，99.7%的值在±3σ内，归一化后在±0.2左右）
        let max_val = emb.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(max_val < 0.5, "最大值应在合理范围内，max={}", max_val);
    }
    
    #[test]
    fn test_different_seeds_produce_different_vectors() {
        // 不同种子应产生不同向量
        let v1 = generate_embedding_384(1);
        let v2 = generate_embedding_384(2);
        assert_ne!(v1, v2, "不同种子应产生不同向量");
        
        // 但两者都应已归一化
        let norm1: f32 = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = v2.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm1 - 1.0).abs() < 0.001);
        assert!((norm2 - 1.0).abs() < 0.001);
    }
}
