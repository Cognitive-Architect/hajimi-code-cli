//! MemoryTier Trait - MemGPT四级内存统一接口契约
//!
//! 架构层级: Focus → Working → Archive → RAG
//! 设计原则: 抽象trait层，零具体实现，零外部依赖
//!
//! 四级内存模型:
//! - **Focus**: 高频焦点内存（~4000 tokens），LRU淘汰，RwLock并发
//! - **Working**: 工作内存（~32K tokens），近期活跃数据
//! - **Archive**: 归档内存（~1M tokens），压缩存储，按需加载
//! - **RAG**: 检索增强内存（无上限），向量索引，语义检索

use std::future::Future;

/// 内存层级枚举
///
/// MemGPT四级内存架构从高频到低频访问:
/// - Focus: 焦点内存层，O(1)访问，LRU淘汰策略
/// - Working: 工作内存层，平衡容量与访问速度
/// - Archive: 归档内存层，压缩存储，低频访问
/// - Rag: 检索增强层，向量索引，语义检索
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MemoryLevel {
    #[default]
    Focus, // 焦点层 O(1) LRU
    Working, // 工作层 近期数据
    Archive, // 归档层 压缩存储
    Rag,     // RAG层 向量检索
}

/// 四级内存统一接口
///
/// 设计约束:
/// - 自动实现Send + Sync（线程安全）
/// - 关联类型抽象Key/Value/Error
/// - 支持异步Future操作
/// - 零unsafe代码保证
///
/// 实现要求:
/// - Focus层: 必须使用RwLock（非Mutex），支持并发读
/// - Working层: 可选持久化，平衡策略
/// - Archive层: 压缩+懒加载，内存优化
/// - RAG层: 向量索引，近似最近邻检索
pub trait MemoryTier: Send + Sync {
    /// 错误类型: 标准Error + Send + Sync
    type Error: std::error::Error + Send + Sync;
    /// 键类型: 字符串或结构化ID
    type Key: AsRef<str> + Send + Sync;
    /// 值类型: 内存条目（文本+元数据）
    type Value: Send + Sync;

    /// 根据键获取内存条目
    ///
    /// # 性能预期
    /// - Focus: O(1)，内存直接访问
    /// - Working: O(1)~O(log n)，取决于实现
    /// - Archive: O(1)索引+解压延迟
    /// - RAG: O(1)向量检索+HNSW近似
    fn get(
        &self,
        key: &Self::Key,
    ) -> impl Future<Output = Result<Option<Self::Value>, Self::Error>> + Send;

    /// 存储内存条目（所有权转移）
    ///
    /// # 约束
    /// - Focus层LRU满时自动淘汰最久未使用
    /// - RAG层自动更新向量索引
    fn put(
        &self,
        key: Self::Key,
        value: Self::Value,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// 删除内存条目
    ///
    /// # 副作用
    /// - RAG层需同步删除向量索引
    fn delete(&self, key: &Self::Key) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// 列出所有内存键
    ///
    /// # 性能警告
    /// - Archive/RAG层可能较慢，建议分页
    fn list_keys(&self) -> impl Future<Output = Result<Vec<Self::Key>, Self::Error>> + Send;

    /// 获取内存统计信息
    ///
    /// 包含: 条目数、总token数、内存占用
    fn stats(&self) -> impl Future<Output = Result<MemoryStats, Self::Error>> + Send;

    /// 获取当前内存层级
    fn memory_level(&self) -> MemoryLevel;

    /// 搜索内存（RAG层核心功能）
    ///
    /// # 参数
    /// * `query` - 查询向量或文本
    /// * `top_k` - 返回最相似的K条
    ///
    /// # 默认实现
    /// 非RAG层返回空结果
    fn search(
        &self,
        _query: &str,
        _top_k: usize,
    ) -> impl Future<Output = Result<Vec<(Self::Key, f32)>, Self::Error>> + Send {
        std::future::ready(Ok(Vec::new()))
    }
}

/// 内存统计信息结构体
///
/// 包含内存层级的基本统计指标:
/// - entry_count: 内存条目数量
/// - total_tokens: 总token数量
/// - memory_bytes: 内存占用字节数
/// - level: 所属内存层级
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub entry_count: usize,
    pub total_tokens: usize,
    pub memory_bytes: usize,
    pub level: MemoryLevel,
}

/// Token预算配置
///
/// 用于MemoryGateway的Token预算管理
#[derive(Debug, Clone)]
pub struct TokenBudget {
    pub focus_limit: usize,   // 默认4000
    pub working_limit: usize, // 默认32000
    pub archive_limit: usize, // 默认1000000
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            focus_limit: 4000,
            working_limit: 32000,
            archive_limit: 1000000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_level_default() {
        assert_eq!(MemoryLevel::default(), MemoryLevel::Focus);
    }

    #[test]
    fn test_token_budget_default() {
        let budget = TokenBudget::default();
        assert_eq!(budget.focus_limit, 4000);
        assert_eq!(budget.working_limit, 32000);
    }

    #[test]
    fn test_memory_stats_default() {
        let stats = MemoryStats::default();
        assert_eq!(stats.entry_count, 0);
        assert_eq!(stats.total_tokens, 0);
    }
}
