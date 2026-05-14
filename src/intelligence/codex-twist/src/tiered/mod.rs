//! 四级存储分层架构 (Hot/Warm/Cold/Archive)
//!
//! 模块结构:
//! - tiered-storage: 统一trait接口
//! - hot-tier: 热存储层（内存）

pub mod archive_tier;
pub mod cold_tier;
pub mod hot_tier;
pub mod storage_gateway;
pub mod tiered_storage;
pub mod warm_tier;

pub use archive_tier::ArchiveTier;
pub use cold_tier::ColdTier;
pub use hot_tier::HotTier;
pub use storage_gateway::{GatewayStats, StorageGateway};
pub use tiered_storage::{StorageStats, TierLevel, TieredStorage};
pub use warm_tier::WarmTier;
