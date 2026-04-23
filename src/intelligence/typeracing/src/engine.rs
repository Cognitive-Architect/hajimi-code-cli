//! TypeRacing Engine - LSP驱动的类型预测引擎
//!
//! 基于 tool-system LSP 工具实现智能类型预测，支持异步预测和结果缓存。

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use engine_tool_system::lsp::{LspInitTool, LspDefinitionTool, LspReferencesTool, LspHoverTool};
use engine_tool_system::{Tool, ToolArgs, ToolError};
use crate::algorithm::{merge_predictions, rank_predictions};

/// 类型预测树节点
#[derive(Debug, Clone)]
pub struct PredictionNode {
    pub type_name: String,
    pub confidence: f64,
    pub source: PredictionSource,
    pub children: Vec<PredictionNode>,
}

/// 预测来源类型
#[derive(Debug, Clone)]
pub enum PredictionSource { LspHover, LspDefinition, LspReferences, Heuristic, Historical }

/// 类型预测树
#[derive(Debug)]
pub struct TypeTree { pub root: PredictionNode, pub index: HashMap<String, Vec<PredictionNode>> }

/// 预测结果缓存项
#[derive(Debug, Clone)]
struct CacheEntry { predictions: Vec<PredictionNode>, timestamp: std::time::Instant }

/// TypeRacing 引擎结构体
pub struct Engine {
    init_tool: LspInitTool, definition_tool: LspDefinitionTool,
    references_tool: LspReferencesTool, hover_tool: LspHoverTool,
    type_tree: TypeTree, initialized: bool,
    prediction_cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    cache_ttl: std::time::Duration,
}

impl Engine {
    /// 创建新的引擎实例
    pub fn new() -> Self {
        let root = PredictionNode { type_name: "root".to_string(), confidence: 1.0, source: PredictionSource::Heuristic, children: Vec::new() };
        Self {
            init_tool: LspInitTool::new(), definition_tool: LspDefinitionTool::new(),
            references_tool: LspReferencesTool::new(), hover_tool: LspHoverTool::new(),
            type_tree: TypeTree { root, index: HashMap::new() }, initialized: false,
            prediction_cache: Arc::new(Mutex::new(HashMap::new())),
            cache_ttl: std::time::Duration::from_secs(300),
        }
    }

    /// 初始化 LSP 连接
    pub async fn init(&mut self, cmd: &str, args: Vec<String>) -> Result<Value, ToolError> {
        let tool_args = ToolArgs::from(serde_json::json!({ "cmd": cmd, "args": args }));
        let output = self.init_tool.execute(tool_args).await?;
        self.initialized = true;
        Ok(serde_json::from_str(&output.stdout).unwrap_or_default())
    }

    /// 获取悬停类型信息
    pub async fn hover(&self, uri: &str, line: u32, character: u32) -> Result<Value, ToolError> {
        if !self.initialized { return Err(ToolError::new("Engine not initialized")); }
        let args = ToolArgs::from(serde_json::json!({ "uri": uri, "line": line, "character": character }));
        let output = self.hover_tool.execute(args).await?;
        Ok(serde_json::json!({"hover": output.stdout}))
    }

    /// 获取类型定义
    pub async fn definition(&self, uri: &str, line: u32, character: u32) -> Result<Value, ToolError> {
        if !self.initialized { return Err(ToolError::new("Engine not initialized")); }
        let args = ToolArgs::from(serde_json::json!({ "uri": uri, "line": line, "character": character }));
        let output = self.definition_tool.execute(args).await?;
        Ok(serde_json::json!({"definition": output.stdout}))
    }

    /// 获取引用信息
    pub async fn references(&self, uri: &str, line: u32, character: u32) -> Result<Value, ToolError> {
        if !self.initialized { return Err(ToolError::new("Engine not initialized")); }
        let args = ToolArgs::from(serde_json::json!({ "uri": uri, "line": line, "character": character }));
        let output = self.references_tool.execute(args).await?;
        Ok(serde_json::json!({"references": output.stdout}))
    }

    /// 构建类型预测树
    pub fn build_type_tree(&mut self, uri: &str, candidates: Vec<PredictionNode>) {
        self.type_tree.index.insert(uri.to_string(), candidates);
    }

    /// 获取预测候选列表
    pub fn get_predictions(&self, uri: &str) -> Vec<&PredictionNode> {
        self.type_tree.index.get(uri).map(|n| n.iter().collect()).unwrap_or_default()
    }

    /// 预测类型 - 核心算法方法
    ///
    /// 异步调用 LSP 工具获取类型信息，按 confidence 排序返回预测结果
    /// 使用 tokio::spawn 确保非阻塞执行，支持结果缓存
    pub fn predict(&self, uri: String, line: u32, character: u32) -> tokio::task::JoinHandle<Result<Vec<PredictionNode>, ToolError>> {
        let hover_tool = LspHoverTool::new();
        let definition_tool = LspDefinitionTool::new();
        let cache = Arc::clone(&self.prediction_cache);
        let cache_ttl = self.cache_ttl;
        let uri_key = uri.clone();

        tokio::spawn(async move {
            // 检查缓存
            {
                let cache_guard = cache.lock().await;
                if let Some(entry) = cache_guard.get(&uri_key) {
                    if entry.timestamp.elapsed() < cache_ttl { return Ok(entry.predictions.clone()); }
                }
            }

            // 并行调用 LSP 工具
            let hover_args = ToolArgs::from(serde_json::json!({"uri": uri.clone(), "line": line, "character": character}));
            let def_args = ToolArgs::from(serde_json::json!({"uri": uri.clone(), "line": line, "character": character}));
            
            let (hover_result, def_result) = tokio::join!(
                async { hover_tool.execute(hover_args).await },
                async { definition_tool.execute(def_args).await }
            );

            let mut predictions: Vec<PredictionNode> = Vec::new();

            if let Ok(output) = hover_result {
                let type_name = Self::extract_type(&output.stdout);
                if !type_name.is_empty() {
                    predictions.push(PredictionNode { type_name, confidence: 0.9, source: PredictionSource::LspHover, children: Vec::new() });
                }
            }

            if let Ok(output) = def_result {
                let type_name = Self::extract_type(&output.stdout);
                if !type_name.is_empty() && !predictions.iter().any(|p| p.type_name == type_name) {
                    predictions.push(PredictionNode { type_name, confidence: 0.85, source: PredictionSource::LspDefinition, children: Vec::new() });
                }
            }

            // 合并相同类型并排序 (O(N log N))
            predictions = merge_predictions(predictions);
            rank_predictions(&mut predictions);

            // 更新缓存
            if !predictions.is_empty() {
                let mut cache_guard = cache.lock().await;
                cache_guard.insert(uri, CacheEntry { predictions: predictions.clone(), timestamp: std::time::Instant::now() });
            }
            Ok(predictions)
        })
    }

    /// 从 LSP 输出中提取类型名称
    fn extract_type(output: &str) -> String {
        if output.is_empty() || output.contains("No ") { return String::new(); }
        if let Ok(json) = serde_json::from_str::<Value>(output) {
            if let Some(content) = json.get("hover").or_else(|| json.get("definition")).and_then(|v| v.as_str()) {
                return Self::parse_type(content);
            }
        }
        Self::parse_type(output)
    }

    /// 解析类型字符串
    fn parse_type(s: &str) -> String {
        let types = ["i32", "i64", "u32", "u64", "f32", "f64", "bool", "String", "str", "Vec<", "Option<", "Result<"];
        for t in &types { if s.contains(t) { return t.to_string(); } }
        s.lines().next().unwrap_or("").trim().to_string()
    }

    /// 清除预测缓存
    pub async fn clear_cache(&self) { self.prediction_cache.lock().await.clear(); }
    
    /// 获取缓存统计
    pub async fn cache_stats(&self) -> usize { self.prediction_cache.lock().await.len() }
}

impl Default for Engine { fn default() -> Self { Self::new() } }
