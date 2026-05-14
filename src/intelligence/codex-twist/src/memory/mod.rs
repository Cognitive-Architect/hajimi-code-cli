//! MemGPT四级内存架构 (Focus/Working/Archive/RAG)
//!
//! 模块结构:
//! - memory-tier: 统一trait接口
//! - focus-memory: LRU焦点内存层
//! - memory-gateway: 四级内存网关

pub mod archive_memory;
pub mod focus_memory;
pub mod memory_gateway;
pub mod memory_tier;
pub mod rag_index;
pub mod token_tracker;
pub mod working_memory;

pub use archive_memory::ArchiveMemory;
pub use focus_memory::{FocusKey, FocusMemory, FocusValue};
pub use memory_gateway::{GatewayStats, MemoryGateway};
pub use memory_tier::{MemoryLevel, MemoryStats, MemoryTier, TokenBudget};
pub use rag_index::RAGIndex;
pub use token_tracker::{GlobalStats, SessionStats, TokenUsageTracker};
pub use working_memory::WorkingMemory;
