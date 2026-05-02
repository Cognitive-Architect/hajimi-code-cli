//! MemGPT四级内存架构 (Focus/Working/Archive/RAG)
//! 
//! 模块结构:
//! - memory-tier: 统一trait接口
//! - focus-memory: LRU焦点内存层
//! - memory-gateway: 四级内存网关

pub mod memory_tier;
pub mod focus_memory;
pub mod working_memory;
pub mod archive_memory;
pub mod rag_index;
pub mod memory_gateway;
pub mod token_tracker;

pub use memory_tier::{MemoryLevel, MemoryTier, MemoryStats, TokenBudget};
pub use focus_memory::{FocusMemory, FocusKey, FocusValue};
pub use working_memory::WorkingMemory;
pub use archive_memory::ArchiveMemory;
pub use rag_index::RAGIndex;
pub use memory_gateway::{MemoryGateway, GatewayStats};
pub use token_tracker::{TokenUsageTracker, SessionStats, GlobalStats};
