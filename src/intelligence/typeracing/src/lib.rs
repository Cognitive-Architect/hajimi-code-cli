//! TypeRacing - LSP驱动的类型预测引擎
//!
//! 基于 tool-system LSP 工具实现的智能代码补全引擎。
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use typeracing::Engine;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut engine = Engine::new();
//! engine.init("rust-analyzer", vec![]).await?;
//!
//! let hover_info = engine.hover("file:///main.rs", 10, 5).await?;
//! let definitions = engine.definition("file:///main.rs", 10, 5).await?;
//! 
//! // 异步预测
//! let handle = engine.predict("file:///main.rs".to_string(), 10, 5);
//! let predictions = handle.await??;
//! # Ok(())
//! # }
//! ```

mod algorithm;
mod engine;
mod terminal_adapter;

pub use engine::{
    Engine,
    TypeTree,
    PredictionNode,
    PredictionSource,
};

pub use algorithm::{
    calculate_weighted_confidence,
    rank_predictions,
    merge_predictions,
    select_top_k,
    average_confidence,
    binary_search_by_type,
};

pub use terminal_adapter::{
    TerminalAdapter,
    AdapterState,
    AdapterError,
    AdapterResult,
};

// 重新导出 tool-system 的错误类型
pub use engine_tool_system::ToolError;
