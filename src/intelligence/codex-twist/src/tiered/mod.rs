//! 四级存储分层架构 (Hot/Warm/Cold/Archive)
//! 
//! 模块结构:
//! - tiered-storage: 统一trait接口
//! - hot-tier: 热存储层（内存）

pub mod tiered_storage;
pub mod hot_tier;
pub mod warm_tier;
pub mod cold_tier;
pub mod archive_tier;
pub mod storage_gateway;

pub use tiered_storage::{TierLevel, TieredStorage, StorageStats};
pub use hot_tier::HotTier;
pub use warm_tier::WarmTier;
pub use cold_tier::ColdTier;
pub use archive_tier::ArchiveTier;
pub use storage_gateway::{StorageGateway, GatewayStats};
