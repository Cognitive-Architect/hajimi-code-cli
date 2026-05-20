//! DEBT-LINES-B0301A: Extracted from agent_loop.rs.
//! Phase 4 Day 2: Added optional AST context injection via ASTContextProvider.
//! Phase 4 Day 11: Added retrieve_for_context for ContextWindowManager integration.
use crate::blackboard::Blackboard;
use crate::checkpoint::CheckpointManager;
use crate::context_window_manager::{estimate_tokens, ContentType, ContextBlock, ContextPriority};
use chimera_repl::traits::ReplResult;
use engine_tool_system::lsp_integration::ASTContextProvider;
use memory::sync_gateway::MemoryTier;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{info, warn};

const RETRIEVAL_CACHE_TTL_SECS: u64 = 30;
const MAX_RETRIEVAL_TOKENS: usize = 4096;

#[derive(Debug)]
pub enum RetrieveOutcome {
    CacheHit(String),
    Retrieved { summary: String },
    Error(String),
}

pub struct MemoryRetriever {
    blackboard: Arc<Blackboard>,
    sync_gateway: Option<memory::sync_gateway::SyncGatewayHandle>,
    memory: Option<Arc<Mutex<memory::memory_gateway::MemoryGateway>>>,
    cache: Arc<Mutex<HashMap<String, (String, Instant)>>>,
    ast_provider: Option<Arc<dyn ASTContextProvider>>,
}

impl MemoryRetriever {
    pub fn new(
        blackboard: Arc<Blackboard>,
        sync_gateway: Option<memory::sync_gateway::SyncGatewayHandle>,
        memory: Option<Arc<Mutex<memory::memory_gateway::MemoryGateway>>>,
    ) -> Self {
        Self {
            blackboard,
            sync_gateway,
            memory,
            cache: Arc::new(Mutex::new(HashMap::new())),
            ast_provider: None,
        }
    }

    pub fn with_ast_provider(mut self, provider: Arc<dyn ASTContextProvider>) -> Self {
        self.ast_provider = Some(provider);
        self
    }

    /// `retrieve_for_context` retrieves memory entries as `ContextBlock`s for the `ContextWindowManager`.
    ///
    /// Maps memory tiers to context priorities:
    /// - **Focus Memory** → always P1 (high priority, always included even if budget tight)
    /// - **Working Memory** → P2 (compressed summary)
    /// - **Archive Memory** → P3 (top-ranked fragments only)
    ///
    /// Returns a `Vec<ContextBlock>` ready for `ContextWindowManager::assemble`.
    pub async fn retrieve_for_context(&self, agent_id: &str, budget: usize) -> Vec<ContextBlock> {
        let mut blocks = Vec::new();

        let focus_limit = std::cmp::min(budget * 10 / 100, 32_000);
        let working_limit = std::cmp::min(budget * 25 / 100, 128_000);
        let archive_limit = std::cmp::min(budget * 65 / 100, 400_000);

        if let Some(ref memory) = self.memory {
            let mem = memory.lock().await;

            // Focus Memory → always P1 (high priority, always included)
            if let Some(focus) = mem.session.get(&format!("ctx_{}", agent_id)) {
                let content = format!("{:?}", focus);
                let truncated = truncate_to_tokens(&content, focus_limit, "Focus");
                blocks.push(ContextBlock {
                    name: format!("focus_memory_Focus_score_1.0_key_ctx_{}", agent_id),
                    priority: ContextPriority::P1,
                    content_type: ContentType::Text,
                    content: truncated.clone(),
                    token_estimate: estimate_tokens(&truncated),
                    truncatable: true,
                });
            }

            // Working Memory → P2 (compressed summary)
            if let Some(working) = mem.session.get(&format!("working_{}", agent_id)) {
                let content = format!("{:?}", working);
                let truncated = truncate_to_tokens(&content, working_limit, "Working");
                blocks.push(ContextBlock {
                    name: format!("working_memory_Working_score_1.0_key_working_{}", agent_id),
                    priority: ContextPriority::P2,
                    content_type: ContentType::Text,
                    content: truncated.clone(),
                    token_estimate: estimate_tokens(&truncated),
                    truncatable: true,
                });
            }

            // Archive Memory → P3 (top-ranked fragments)
            if let Some(archive) = mem.session.get(&format!("archive_{}", agent_id)) {
                let content = format!("{:?}", archive);
                let truncated = truncate_to_tokens(&content, archive_limit, "Archive");
                blocks.push(ContextBlock {
                    name: format!("archive_memory_Archive_score_1.0_key_archive_{}", agent_id),
                    priority: ContextPriority::P3,
                    content_type: ContentType::Text,
                    content: truncated.clone(),
                    token_estimate: estimate_tokens(&truncated),
                    truncatable: true,
                });
            }
        }

        blocks
    }

    pub async fn retrieve(&self, agent_id: &str) -> RetrieveOutcome {
        self.retrieve_with_ast(agent_id, None).await
    }

    pub async fn retrieve_with_ast(
        &self,
        agent_id: &str,
        ast_query: Option<&str>,
    ) -> RetrieveOutcome {
        // Phase 4 Day 2: Optional AST-enhanced retrieval
        if let Some(query) = ast_query {
            if let Some(ref provider) = self.ast_provider {
                match provider.enhance_retrieve_with_ast(query).await {
                    Ok(ctx) => {
                        self.blackboard
                            .write(&format!("ast_context_{}", agent_id), &ctx, agent_id)
                            .await;
                        info!("AST-enhanced retrieval: {}", ctx);
                        return RetrieveOutcome::Retrieved {
                            summary: format!("AST-enhanced: {}", ctx),
                        };
                    }
                    Err(e) => {
                        warn!("AST enhancement failed (falling back to text): {}", e);
                    }
                }
            }
        }

        // Also check blackboard for ast_query set by planner
        let bb = self.blackboard.snapshot().await;
        let bb_ast_query = bb
            .get(&format!("ast_query_{}", agent_id))
            .map(|e| e.value.clone());
        if let Some(query) = bb_ast_query {
            if let Some(ref provider) = self.ast_provider {
                match provider.enhance_retrieve_with_ast(&query).await {
                    Ok(ctx) => {
                        self.blackboard
                            .write(&format!("ast_context_{}", agent_id), &ctx, agent_id)
                            .await;
                        info!("AST-enhanced retrieval (from blackboard): {}", ctx);
                        return RetrieveOutcome::Retrieved {
                            summary: format!("AST-enhanced: {}", ctx),
                        };
                    }
                    Err(e) => {
                        warn!(
                            "AST enhancement from blackboard failed (falling back): {}",
                            e
                        );
                    }
                }
            }
        }

        // Standard text-based retrieval
        let query = format!(
            "agent:{} keys:{:?}",
            agent_id,
            self.blackboard
                .snapshot()
                .await
                .keys()
                .cloned()
                .collect::<Vec<_>>()
        );
        if let Some((summary, ts)) = self.cache.lock().await.get(&query) {
            if ts.elapsed() < Duration::from_secs(RETRIEVAL_CACHE_TTL_SECS) {
                return RetrieveOutcome::CacheHit(summary.clone());
            }
        }
        if let Some(ref sg) = self.sync_gateway {
            let mut sg_guard = sg.lock().await;
            match sg_guard
                .retrieve_multi(MemoryTier::fallback_order(), &query)
                .await
            {
                Ok(results) => {
                    let total = results
                        .iter()
                        .map(|(_, entries)| entries.len())
                        .sum::<usize>();
                    let mut tokens = 0usize;
                    for (tier, entries) in &results {
                        for entry in entries {
                            if tokens + entry.tokens > MAX_RETRIEVAL_TOKENS {
                                break;
                            }
                            self.blackboard
                                .write(
                                    &format!("retrieved_{:?}_{}", tier, agent_id),
                                    &entry.content,
                                    agent_id,
                                )
                                .await;
                            tokens += entry.tokens;
                        }
                    }
                    let summary = format!(
                        "{} entries in {} tiers ({} tokens)",
                        total,
                        results.len(),
                        tokens
                    );
                    self.cache
                        .lock()
                        .await
                        .insert(query, (summary.clone(), Instant::now()));
                    info!("Retrieved: {}", summary);
                    return RetrieveOutcome::Retrieved { summary };
                }
                Err(e) => return RetrieveOutcome::Error(e.to_string()),
            }
        }
        RetrieveOutcome::Error("No sync_gateway".to_string())
    }

    pub async fn query_legacy(&self, agent_id: &str) {
        if let Some(ref memory) = self.memory {
            let mem = memory.lock().await;
            if let Some(entry) = mem.session.get(&format!("ctx_{}", agent_id)) {
                let content = format!("{:?}", entry);
                self.blackboard
                    .write(
                        &format!("retrieved_legacy_{}", agent_id),
                        &content,
                        agent_id,
                    )
                    .await;
                info!("Legacy query hit: {} bytes", content.len());
            }
        }
    }

    pub async fn store(
        &self,
        agent_id: &str,
        checkpoint_mgr: &CheckpointManager,
    ) -> ReplResult<()> {
        let _ = checkpoint_mgr
            .save(
                &agent_id.to_string(),
                None,
                vec![],
                vec![],
                &self.blackboard,
            )
            .await?;
        if let Some(ref sg) = self.sync_gateway {
            let bb = self.blackboard.snapshot().await;
            let snapshot = memory::sync_gateway::BlackboardSnapshot {
                entries: bb.into_iter().map(|(k, v)| (k, v.value)).collect(),
            };
            let mut sg_guard = sg.lock().await;
            if let Err(e) = sg_guard.sync_with_blackboard(&snapshot).await {
                warn!("Blackboard sync failed: {}", e);
            }
        }
        if let Some(ref memory) = self.memory {
            let mut mem = memory.lock().await;
            if let Err(e) = mem.push_vector(
                &format!("plan_{}", agent_id),
                &format!("checkpoint_{}", agent_id),
            ) {
                warn!("Memory persist failed: {}", e);
            }
        }
        Ok(())
    }
}

fn truncate_to_tokens(content: &str, limit: usize, tier: &str) -> String {
    let max_chars = limit * 4;
    if content.len() > max_chars {
        if max_chars > 50 {
            let reserved = 40;
            let mut end = max_chars.saturating_sub(reserved);
            while end > 0 && !content.is_char_boundary(end) {
                end -= 1;
            }
            format!(
                "{}... [OMITTED due to {} budget exceeded]",
                &content[..end],
                tier
            )
        } else {
            let mut end = max_chars;
            while end > 0 && !content.is_char_boundary(end) {
                end -= 1;
            }
            content[..end].to_string()
        }
    } else {
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blackboard::Blackboard;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_retrieve_for_context_dynamic_caps() {
        let bb = Arc::new(Blackboard::new());
        let mut gateway = memory::memory_gateway::MemoryGateway::new("device");

        // Populate Focus Memory (ctx_<agent_id>)
        gateway
            .session
            .insert("ctx_agent1".to_string(), "focus ".repeat(200))
            .unwrap();
        // Populate Working Memory (working_<agent_id>)
        gateway
            .session
            .insert("working_agent1".to_string(), "working ".repeat(500))
            .unwrap();
        // Populate Archive Memory (archive_<agent_id>)
        gateway
            .session
            .insert("archive_agent1".to_string(), "archive ".repeat(1000))
            .unwrap();

        let memory = Some(Arc::new(Mutex::new(gateway)));
        let retriever = MemoryRetriever::new(bb, None, memory);

        // 1. Test Small legacy budget (e.g. 500 tokens)
        // Focus limit: min(50 tokens, 32K) = 50 tokens
        // Working limit: min(125 tokens, 128K) = 125 tokens
        // Archive limit: min(325 tokens, 400K) = 325 tokens
        let blocks_small = retriever.retrieve_for_context("agent1", 500).await;
        assert_eq!(blocks_small.len(), 3);

        let focus_block = blocks_small
            .iter()
            .find(|b| b.name.contains("Focus"))
            .unwrap();
        let working_block = blocks_small
            .iter()
            .find(|b| b.name.contains("Working"))
            .unwrap();
        let archive_block = blocks_small
            .iter()
            .find(|b| b.name.contains("Archive"))
            .unwrap();

        assert!(focus_block.token_estimate <= 50);
        assert!(working_block.token_estimate <= 125);
        assert!(archive_block.token_estimate <= 325);

        assert!(focus_block
            .content
            .contains("[OMITTED due to Focus budget exceeded]"));
        assert!(working_block
            .content
            .contains("[OMITTED due to Working budget exceeded]"));

        // Verify names contains Focus / Working / Archive / score / key / agent_id
        assert!(focus_block.name.contains("Focus"));
        assert!(focus_block.name.contains("score"));
        assert!(focus_block.name.contains("key"));
        assert!(focus_block.name.contains("agent1"));

        // 2. Test Large long budget (e.g. 200,000 tokens)
        // Focus limit: min(20_000, 32K) = 20_000 tokens
        // Working limit: min(50_000, 128K) = 50_000 tokens
        // Archive limit: min(130_000, 400K) = 130_000 tokens
        let blocks_large = retriever.retrieve_for_context("agent1", 200_000).await;
        assert_eq!(blocks_large.len(), 3);

        let focus_large = blocks_large
            .iter()
            .find(|b| b.name.contains("Focus"))
            .unwrap();
        let working_large = blocks_large
            .iter()
            .find(|b| b.name.contains("Working"))
            .unwrap();
        let archive_large = blocks_large
            .iter()
            .find(|b| b.name.contains("Archive"))
            .unwrap();

        // Under huge budget, none should be truncated/omitted
        assert!(!focus_large.content.contains("[OMITTED"));
        assert!(!working_large.content.contains("[OMITTED"));
        assert!(!archive_large.content.contains("[OMITTED"));
    }
}
