//! TieredStorage Trait - 四级存储统一接口契约
//! 
//! 架构层级: Hot → Warm → Cold → Archive
//! 设计原则: 抽象trait层，零具体实现，零外部依赖

use std::future::Future;

/// 存储层级枚举
/// 
/// 四级存储架构从高到低性能递减:
/// - Hot: 内存驻留层，O(1)访问延迟，最高性能
/// - Warm: SSD本地存储，异步IO，平衡性能与容量
/// - Cold: 压缩存储层，zstd算法，节省空间
/// - Archive: 归档冷存储，长期保存，最低成本
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TierLevel {
    #[default]
    Hot,     // 内存层 O(1)
    Warm,    // SSD层 异步IO
    Cold,    // 压缩层 zstd
    Archive, // 归档层 冷存储
}

/// 四级存储统一接口
/// 
/// 设计约束:
/// - 自动实现Send + Sync（线程安全）
/// - 关联类型抽象Key/Value/Error
/// - 支持异步Future操作
/// - 零unsafe代码保证
pub trait TieredStorage: Send + Sync {
    /// 错误类型: 标准Error + Send + Sync
    type Error: std::error::Error + Send + Sync;
    /// 键类型: AsRef<[u8]>序列化支持
    type Key: AsRef<[u8]> + Send + Sync;
    /// 值类型: 线程安全传输
    type Value: Send + Sync;

    /// 根据键获取值，返回Some(Value)或None
    fn get(&self, key: &Self::Key) -> impl Future<Output = Result<Option<Self::Value>, Self::Error>> + Send;
    /// 存储键值对（所有权转移）
    fn put(&self, key: Self::Key, value: Self::Value) -> impl Future<Output = Result<(), Self::Error>> + Send;
    /// 删除键值对
    fn delete(&self, key: &Self::Key) -> impl Future<Output = Result<(), Self::Error>> + Send;
    /// 列出所有键
    fn list_keys(&self) -> impl Future<Output = Result<Vec<Self::Key>, Self::Error>> + Send;
    /// 获取存储统计信息
    fn stats(&self) -> impl Future<Output = Result<StorageStats, Self::Error>> + Send;
    /// 获取当前存储层级
    fn tier_level(&self) -> TierLevel;
}

/// 存储统计信息结构体
/// 
/// 包含存储层级的基本统计指标:
/// - entry_count: 键值对条目数量
/// - total_bytes: 总占用字节数
/// - tier: 所属存储层级
#[derive(Debug, Clone, Default)]
pub struct StorageStats {
    pub entry_count: usize,
    pub total_bytes: usize,
    pub tier: TierLevel,
}

/// HctxStorage兼容性适配器
/// 
/// 向后兼容性说明:
/// - 现有HctxStorage可通过实现TieredStorage扩展
/// - 或作为HotTier<T>的底层存储适配
/// 
/// 推荐迁移路径:
/// HctxStorage → HotTier<HctxStorage> → TieredStorage
pub type HctxStorageAdapter<T> = T;
