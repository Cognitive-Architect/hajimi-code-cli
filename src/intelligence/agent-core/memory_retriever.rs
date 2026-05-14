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
    pub async fn retrieve_for_context(&self, agent_id: &str, _budget: usize) -> Vec<ContextBlock> {
        let mut blocks = Vec::new();

        if let Some(ref memory) = self.memory {
            let mem = memory.lock().await;

            // Focus Memory → always P1 (high priority, always included)
            if let Some(focus) = mem.session.get(&format!("ctx_{}", agent_id)) {
                let content = format!("{:?}", focus);
                blocks.push(ContextBlock {
                    name: "focus_memory".to_string(),
                    priority: ContextPriority::P1,
                    content_type: ContentType::Text,
                    content: content.clone(),
                    token_estimate: estimate_tokens(&content),
                    truncatable: true,
                });
            }

            // Working Memory → P2 (compressed summary)
            if let Some(working) = mem.session.get(&format!("working_{}", agent_id)) {
                let content = format!("{:?}", working);
                let summary = if content.len() > 500 {
                    format!("{}…", &content[..find_summary_end(&content, 500)])
                } else {
                    content.clone()
                };
                blocks.push(ContextBlock {
                    name: "working_memory".to_string(),
                    priority: ContextPriority::P2,
                    content_type: ContentType::Text,
                    content: summary.clone(),
                    token_estimate: estimate_tokens(&summary),
                    truncatable: true,
                });
            }

            // Archive Memory → P3 (top-ranked fragments)
            if let Some(archive) = mem.session.get(&format!("archive_{}", agent_id)) {
                let content = format!("{:?}", archive);
                let fragment = if content.len() > 300 {
                    format!("{}…", &content[..find_summary_end(&content, 300)])
                } else {
                    content.clone()
                };
                blocks.push(ContextBlock {
                    name: "archive_memory".to_string(),
                    priority: ContextPriority::P3,
                    content_type: ContentType::Text,
                    content: fragment.clone(),
                    token_estimate: estimate_tokens(&fragment),
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

/// Find a safe truncation point at or before `max_len`, respecting char boundaries.
fn find_summary_end(s: &str, max_len: usize) -> usize {
    if max_len >= s.len() {
        return s.len();
    }
    let mut end = max_len;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    end
}
