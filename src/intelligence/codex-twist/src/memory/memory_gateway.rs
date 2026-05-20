//! MemoryGateway - MemGPT四级内存网关
use super::{ArchiveMemory, FocusMemory, MemoryLevel, MemoryTier, TokenBudget, WorkingMemory};
use engine_llm_core::{ChatMessage, LlmClient, StreamChunk};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct MemoryGateway {
    focus: Arc<FocusMemory<String, String>>,
    working: Arc<WorkingMemory>,
    archive: Arc<ArchiveMemory>,
    budget: Arc<RwLock<TokenBudget>>,
}

fn memory_path(home: &str, subdir: &str) -> PathBuf {
    PathBuf::from(home).join(".codex/memory").join(subdir)
}

macro_rules! merge_stats {
    ($f:expr, $w:expr, $a:expr) => {
        GatewayStats {
            focus_entries: $f.entry_count,
            focus_tokens: $f.total_tokens,
            working_entries: $w.entry_count,
            working_tokens: $w.total_tokens,
            archive_entries: $a.entry_count,
            archive_tokens: $a.total_tokens,
            rag_entries: 0,
            rag_tokens: 0,
        }
    };
}

impl MemoryGateway {
    pub fn new() -> Self {
        Self::with_budget(TokenBudget::default())
    }
    pub fn with_budget(budget: TokenBudget) -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Self {
            focus: Arc::new(FocusMemory::new()),
            working: Arc::new(WorkingMemory::with_persistence(
                budget.working_limit,
                memory_path(&home, "working"),
            )),
            archive: Arc::new(ArchiveMemory::with_path(
                budget.archive_limit,
                memory_path(&home, "archive"),
            )),
            budget: Arc::new(RwLock::new(budget)),
        }
    }
    pub fn focus(&self) -> &FocusMemory<String, String> {
        &self.focus
    }
    pub fn working(&self) -> &WorkingMemory {
        &self.working
    }
    pub fn archive(&self) -> &ArchiveMemory {
        &self.archive
    }
    pub async fn budget(&self) -> TokenBudget {
        self.budget.read().await.clone()
    }
    pub async fn set_budget(&self, budget: TokenBudget) {
        *self.budget.write().await = budget;
    }
    pub async fn get(&self, k: &str) -> Option<String> {
        if let Some(v) = self.focus.get(&k.to_string()).await.ok().flatten() {
            return Some(v);
        }
        if let Some(v) = self.working.get(&k.to_string()).await.ok().flatten() {
            return Some(v);
        }
        self.archive.get(&k.to_string()).await.ok().flatten()
    }
    pub async fn put(&self, k: String, v: String, level: MemoryLevel) {
        match level {
            MemoryLevel::Focus => {
                self.focus.put(k, v).await.ok();
            }
            MemoryLevel::Working => {
                self.working.put(k, v).await.ok();
            }
            MemoryLevel::Archive => {
                self.archive.put(k, v).await.ok();
            }
            _ => {}
        }
    }
    pub async fn stats(&self) -> GatewayStats {
        let focus = self.focus.stats().await.unwrap_or_default();
        let working = self.working.stats().await.unwrap_or_default();
        let archive = self.archive.stats().await.unwrap_or_default();
        merge_stats!(focus, working, archive)
    }
    pub async fn clear_focus(&self) {
        self.focus.clear().await;
    }
    pub async fn clear_working(&self) {
        self.working.clear().await;
    }
    pub async fn clear_archive(&self) {
        self.archive.clear().await;
    }
    pub async fn optimize(
        &self,
        messages: Vec<ChatMessage>,
        client: &dyn LlmClient,
    ) -> Result<String, String> {
        if messages.len() <= 2 {
            return Ok("对话轮次不足，无需压缩".to_string());
        }

        let summary_msgs: Vec<ChatMessage> = messages[..messages.len() - 2]
            .iter()
            .map(|m| ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
                timestamp: m.timestamp,
            })
            .collect();

        let system_prompt = "请将以下对话历史压缩为一段简洁的摘要（200字以内），保留关键决策、代码变更和上下文信息。只输出摘要内容，不要添加任何前缀或解释。".to_string();

        let mut stream = client
            .stream_chat_with_context(summary_msgs, Some(system_prompt))
            .await
            .map_err(|e| format!("摘要生成失败: {}", e))?;

        let mut summary = String::new();
        while let Some(chunk) = stream.next().await {
            match chunk {
                StreamChunk::Output(text) => summary.push_str(&text),
                StreamChunk::Error(err) => return Err(format!("LLM错误: {}", err)),
                StreamChunk::Done => break,
            }
        }

        Ok(summary.trim().to_string())
    }
}

impl Default for MemoryGateway {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct GatewayStats {
    pub focus_entries: usize,
    pub focus_tokens: usize,
    pub working_entries: usize,
    pub working_tokens: usize,
    pub archive_entries: usize,
    pub archive_tokens: usize,
    pub rag_entries: usize,
    pub rag_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gateway_new() {
        let g = MemoryGateway::new();
        assert_eq!(g.budget.blocking_read().focus_limit, 4000);
    }
    #[tokio::test]
    async fn test_gateway_budget_update() {
        let g = MemoryGateway::new();
        g.set_budget(TokenBudget {
            focus_limit: 8000,
            working_limit: 64000,
            archive_limit: 2000000,
        })
        .await;
        assert_eq!(g.budget().await.focus_limit, 8000);
    }
    macro_rules! test_op {
        ($name:ident, $level:expr) => {
            #[tokio::test]
            async fn $name() {
                let g = MemoryGateway::new();
                g.put("k".into(), "v".into(), $level).await;
                assert_eq!(g.get("k").await, Some("v".into()));
            }
        };
    }
    test_op!(test_focus, MemoryLevel::Focus);
    test_op!(test_working, MemoryLevel::Working);
    test_op!(test_archive, MemoryLevel::Archive);
    #[tokio::test]
    async fn test_fallback() {
        let g = MemoryGateway::new();
        g.put("k".into(), "v".into(), MemoryLevel::Working).await;
        assert_eq!(g.get("k").await, Some("v".into()));
    }
    #[tokio::test]
    async fn test_stats() {
        let g = MemoryGateway::new();
        g.put("k".into(), "v".into(), MemoryLevel::Focus).await;
        assert_eq!(g.stats().await.focus_entries, 1);
    }
    #[tokio::test]
    async fn test_stats_all() {
        let g = MemoryGateway::new();
        g.put("f1".into(), "v1".into(), MemoryLevel::Focus).await;
        g.put("w1".into(), "v2".into(), MemoryLevel::Working).await;
        g.put("a1".into(), "v3".into(), MemoryLevel::Archive).await;
        let s = g.stats().await;
        assert_eq!(s.focus_entries, 1);
        assert_eq!(s.working_entries, 1);
        assert_eq!(s.archive_entries, 1);
    }
}
