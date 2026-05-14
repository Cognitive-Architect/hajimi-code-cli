//! Token usage tracker — session-level and global accumulation.
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Token statistics for a session, provider, or day.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SessionStats {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub request_count: u64,
}

/// Global accumulated statistics (by provider / by day / total).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GlobalStats {
    pub by_provider: HashMap<String, SessionStats>,
    pub by_day: HashMap<String, SessionStats>,
    pub total: SessionStats,
}

/// Thread-safe token usage tracker with interior mutability.
#[derive(Clone)]
pub struct TokenUsageTracker {
    sessions: Arc<RwLock<HashMap<String, SessionStats>>>,
    global: Arc<RwLock<GlobalStats>>,
}

fn upsert(
    map: &mut HashMap<String, SessionStats>,
    key: String,
    prompt: u64,
    completion: u64,
    total: u64,
) {
    map.entry(key)
        .and_modify(|s| {
            s.prompt_tokens = s.prompt_tokens.saturating_add(prompt);
            s.completion_tokens = s.completion_tokens.saturating_add(completion);
            s.total_tokens = s.total_tokens.saturating_add(total);
            s.request_count = s.request_count.saturating_add(1);
        })
        .or_insert_with(|| SessionStats {
            prompt_tokens: prompt,
            completion_tokens: completion,
            total_tokens: total,
            request_count: 1,
        });
}

impl TokenUsageTracker {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            global: Arc::new(RwLock::new(GlobalStats::default())),
        }
    }

    /// Record usage for a session and update global counters (provider + day + total).
    pub async fn record_usage(
        &self,
        session_id: &str,
        provider: &str,
        prompt_tokens: u64,
        completion_tokens: u64,
    ) {
        let total_tokens = prompt_tokens.saturating_add(completion_tokens);
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        upsert(
            &mut *self.sessions.write().await,
            session_id.to_string(),
            prompt_tokens,
            completion_tokens,
            total_tokens,
        );
        let mut g = self.global.write().await;
        upsert(
            &mut g.by_provider,
            provider.to_string(),
            prompt_tokens,
            completion_tokens,
            total_tokens,
        );
        upsert(
            &mut g.by_day,
            today,
            prompt_tokens,
            completion_tokens,
            total_tokens,
        );
        g.total.prompt_tokens = g.total.prompt_tokens.saturating_add(prompt_tokens);
        g.total.completion_tokens = g.total.completion_tokens.saturating_add(completion_tokens);
        g.total.total_tokens = g.total.total_tokens.saturating_add(total_tokens);
        g.total.request_count = g.total.request_count.saturating_add(1);
    }

    /// Get stats for a session. Returns Default (zeros) if not found.
    pub async fn get_token_stats(&self, session_id: &str) -> SessionStats {
        self.sessions
            .read()
            .await
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get global accumulated statistics.
    pub async fn get_global_stats(&self) -> GlobalStats {
        self.global.read().await.clone()
    }
}

impl Default for TokenUsageTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_accumulation() {
        let t = TokenUsageTracker::new();
        t.record_usage("s1", "openai", 10, 20).await;
        t.record_usage("s1", "openai", 5, 15).await;
        let s = t.get_token_stats("s1").await;
        assert_eq!(s.prompt_tokens, 15);
        assert_eq!(s.completion_tokens, 35);
        assert_eq!(s.total_tokens, 50);
        assert_eq!(s.request_count, 2);
    }

    #[tokio::test]
    async fn test_session_isolation() {
        let t = TokenUsageTracker::new();
        t.record_usage("sa", "openai", 10, 20).await;
        t.record_usage("sb", "anthropic", 30, 40).await;
        assert_eq!(t.get_token_stats("sa").await.prompt_tokens, 10);
        assert_eq!(t.get_token_stats("sb").await.prompt_tokens, 30);
    }

    #[tokio::test]
    async fn test_global_by_provider() {
        let t = TokenUsageTracker::new();
        t.record_usage("s1", "openai", 10, 20).await;
        t.record_usage("s2", "anthropic", 30, 40).await;
        t.record_usage("s3", "openai", 5, 10).await;
        let g = t.get_global_stats().await;
        assert_eq!(g.by_provider.get("openai").unwrap().request_count, 2);
        assert_eq!(g.by_provider.get("anthropic").unwrap().request_count, 1);
    }

    #[tokio::test]
    async fn test_global_by_day() {
        let t = TokenUsageTracker::new();
        t.record_usage("s1", "openai", 10, 20).await;
        t.record_usage("s2", "openai", 5, 10).await;
        let g = t.get_global_stats().await;
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let d = g.by_day.get(&today).unwrap();
        assert_eq!(d.prompt_tokens, 15);
        assert_eq!(d.request_count, 2);
    }

    #[tokio::test]
    async fn test_cumulative_consistency() {
        let t = TokenUsageTracker::new();
        t.record_usage("s1", "openai", 10, 20).await;
        t.record_usage("s2", "anthropic", 30, 40).await;
        t.record_usage("s1", "openai", 5, 10).await;
        let g = t.get_global_stats().await;
        assert_eq!(g.total.prompt_tokens, 45);
        assert_eq!(g.total.completion_tokens, 70);
        assert_eq!(g.total.total_tokens, 115);
        assert_eq!(g.total.request_count, 3);
        let psum: SessionStats =
            g.by_provider
                .values()
                .cloned()
                .fold(SessionStats::default(), |mut acc, s| {
                    acc.prompt_tokens += s.prompt_tokens;
                    acc.completion_tokens += s.completion_tokens;
                    acc.total_tokens += s.total_tokens;
                    acc.request_count += s.request_count;
                    acc
                });
        assert_eq!(psum, g.total);
    }

    #[tokio::test]
    async fn test_get_token_stats_missing() {
        let t = TokenUsageTracker::new();
        assert_eq!(t.get_token_stats("none").await, SessionStats::default());
    }
}
