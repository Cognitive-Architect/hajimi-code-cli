//! Memory Bootstrapper E2E Tests (B-10A/10) — Cross-session memory recovery validation.
//! Verifies the complete lifecycle: bootstrap → 3 dialogue rounds → checkpoint save
//! → destroy → MemoryBootstrapper restore from disk → verify summary + Blackboard.
//! Mock-based test: no real LLM API calls are made during this test.
use agent_core::{Blackboard, LoopState, MemoryBootstrapper};
use async_trait::async_trait;
use std::sync::Arc;

/// Remove leftover checkpoint files from previous test runs to ensure isolation.
async fn cleanup_test_checkpoint(agent_id: &str) {
    #[cfg(target_os = "windows")]
    let base = std::env::var("APPDATA").ok().map(std::path::PathBuf::from);
    #[cfg(not(target_os = "windows"))]
    let base = std::env::var("HOME")
        .ok()
        .map(|h| std::path::PathBuf::from(h).join(".config"));
    if let Some(dir) = base {
        let p = dir
            .join(".hajimi")
            .join("checkpoints")
            .join(format!("{}.jsonl", agent_id));
        if p.exists() {
            let _ = tokio::fs::remove_file(&p).await;
        }
    }
}

async fn cleanup_test_summary(project_id: &str) {
    #[cfg(target_os = "windows")]
    let base = std::env::var("APPDATA").ok().map(std::path::PathBuf::from);
    #[cfg(not(target_os = "windows"))]
    let base = std::env::var("HOME")
        .ok()
        .map(|h| std::path::PathBuf::from(h).join(".config"));
    if let Some(dir) = base {
        let p = dir
            .join(".hajimi")
            .join("memory")
            .join(project_id)
            .join("summary.md");
        if p.exists() {
            let _ = tokio::fs::remove_file(&p).await;
        }
    }
}

#[tokio::test]
async fn test_cross_session_memory_recovery() {
    let project_id = "e2e_memory_test";
    let device_id = "e2e_device";
    let agent_id = "e2e_test_agent".to_string();

    // Pre-clean to ensure isolation between test runs
    cleanup_test_checkpoint(&agent_id).await;
    cleanup_test_summary(project_id).await;

    // =======================================================================
    // PHASE 1: Initial bootstrap — load project memory, simulate 3 dialogue
    // rounds by writing to a Blackboard, then persist a checkpoint to disk.
    // =======================================================================
    let bootstrapper = MemoryBootstrapper::new(project_id, device_id, &agent_id);
    let result = bootstrapper.load_project_memory().await.unwrap();

    // First load with no prior checkpoint should produce the fallback summary
    assert!(
        result.summary.contains("No checkpoint available")
            || result.summary.contains("plan_summary=")
            || result.summary.contains("项目状态摘要"),
        "Unexpected initial summary: {}",
        result.summary
    );

    // Verify the shared gateway Arc is valid
    assert!(
        Arc::strong_count(&result.gateway) >= 1,
        "Gateway Arc should be held by BootstrapResult"
    );

    // Simulate 3 dialogue rounds on a dedicated blackboard
    let blackboard = Arc::new(Blackboard::new());
    for i in 0..3 {
        blackboard
            .write(
                &format!("dialogue_round_{}", i),
                &format!("user_message_{}", i),
                &agent_id,
            )
            .await;
    }

    // Verify blackboard contains all dialogue rounds before checkpoint save
    for i in 0..3 {
        let entry = blackboard.read(&format!("dialogue_round_{}", i)).await;
        assert!(
            entry.is_some(),
            "Missing blackboard entry for dialogue_round_{}",
            i
        );
        assert_eq!(entry.unwrap().value, format!("user_message_{}", i));
    }

    // Persist checkpoint to disk via CheckpointManager; this writes to
    // config_dir/.hajimi/checkpoints/{agent_id}.jsonl for cross-session recovery
    let chk = result
        .checkpoint_mgr
        .save(&agent_id, None, vec![], vec![], &blackboard)
        .await
        .unwrap();
    assert!(
        chk.id.starts_with("chk_"),
        "Checkpoint ID format invalid: {}",
        chk.id
    );
    assert_eq!(chk.agent_id, agent_id, "Checkpoint agent_id mismatch");

    // Verify the checkpoint is immediately listable in memory before destruction
    let list_before = result.checkpoint_mgr.list(&agent_id).await;
    assert_eq!(list_before.len(), 1, "Expected 1 checkpoint in memory list");
    assert_eq!(
        list_before[0].id, chk.id,
        "Checkpoint ID mismatch in memory list"
    );

    // Destroy all handles — simulate complete session termination
    drop(result);
    drop(blackboard);

    // =======================================================================
    // PHASE 2: Re-bootstrap after destruction — MemoryBootstrapper must restore
    // the checkpoint from disk and produce a summary that reflects saved state.
    // =======================================================================
    let bootstrapper2 = MemoryBootstrapper::new(project_id, device_id, &agent_id);
    let result2 = bootstrapper2.load_project_memory().await.unwrap();

    // Verify that the checkpoint was successfully restored from disk.
    // The summary must reflect restored state (either old kv format or new emoji format).
    assert!(
        !result2.summary.contains("No checkpoint available"),
        "Expected restored checkpoint summary, got fallback: {}",
        result2.summary
    );
    assert!(
        result2.summary.contains("plan_summary=")
            || result2.summary.contains("项目状态摘要")
            || result2.summary.contains("📋"),
        "Missing expected summary markers in: {}",
        result2.summary
    );

    // Verify restore_latest_from_disk returns the same checkpoint that was saved
    let disk_chk = result2
        .checkpoint_mgr
        .restore_latest_from_disk(project_id, &agent_id)
        .await
        .unwrap();
    assert_eq!(disk_chk.id, chk.id, "Disk restore checkpoint ID mismatch");

    // Verify that build_agent_loop_with_memory injects project_memory_summary
    // into the AgentLoop's Blackboard and returns a fully functional loop.
    let agent_loop = bootstrapper2.build_agent_loop_with_memory().await.unwrap();
    assert_eq!(
        agent_loop.current_state().await,
        LoopState::Idle,
        "AgentLoop should start in Idle state after rebuild"
    );
    assert!(
        agent_loop
            .inject_memory("verification_key", "verification_value", &agent_id)
            .await
            .is_ok(),
        "AgentLoop should be functional after cross-session rebuild"
    );

    cleanup_test_checkpoint(&agent_id).await;
    cleanup_test_summary(project_id).await;
}

/// Verify that build_agent_loop_with_memory always returns a functional AgentLoop
/// even when no prior checkpoint exists (cold-start scenario).
#[tokio::test]
async fn test_build_agent_loop_with_memory_cold_start() {
    let agent_id = "cold_agent".to_string();
    let bootstrapper = MemoryBootstrapper::new("cold_proj", "cold_dev", &agent_id);
    let result = bootstrapper.load_project_memory().await.unwrap();

    // Cold start should produce the fallback summary
    assert!(
        result.summary.contains("No checkpoint available")
            || result.summary.contains("plan_summary=")
            || result.summary.contains("项目状态摘要"),
        "Cold start summary unexpected: {}",
        result.summary
    );

    // Build AgentLoop and verify it is functional
    let agent_loop = bootstrapper.build_agent_loop_with_memory().await.unwrap();
    assert_eq!(agent_loop.current_state().await, LoopState::Idle);
    assert!(agent_loop
        .inject_memory("cold_test", "ok", &agent_id)
        .await
        .is_ok());
}

// --- Mock LLM client for natural-language summary tests ---

struct MockLlmClient {
    response: String,
    provider: engine_llm_core::LlmProvider,
}

impl MockLlmClient {
    fn new(response: &str) -> Self {
        Self {
            response: response.to_string(),
            provider: engine_llm_core::LlmProvider::Ollama {
                base_url: "http://localhost".to_string(),
                model: "mock".to_string(),
            },
        }
    }
}

#[async_trait]
impl engine_llm_core::LlmClient for MockLlmClient {
    async fn stream_chat(
        &self,
        _prompt: String,
    ) -> Result<engine_llm_core::ChannelStream, engine_llm_core::EngineError> {
        let (stream, tx) = engine_llm_core::ChannelStream::new(10);
        let _ = tx
            .send(engine_llm_core::StreamChunk::Output(self.response.clone()))
            .await;
        let _ = tx.send(engine_llm_core::StreamChunk::Done).await;
        Ok(stream)
    }

    async fn stream_chat_with_context(
        &self,
        _messages: Vec<engine_llm_core::ChatMessage>,
        _system_prompt: Option<String>,
    ) -> Result<engine_llm_core::ChannelStream, engine_llm_core::EngineError> {
        self.stream_chat("".to_string()).await
    }

    fn provider(&self) -> &engine_llm_core::LlmProvider {
        &self.provider
    }

    fn count_tokens(
        &self,
        _messages: Vec<engine_llm_core::ChatMessage>,
        _model: &str,
    ) -> Result<usize, engine_llm_core::EngineError> {
        Ok(0)
    }

    fn last_usage(&self) -> Option<engine_llm_core::Usage> {
        None
    }
}

#[tokio::test]
async fn test_natural_language_summary_quality() {
    let agent_id = "summary_quality_agent".to_string();
    cleanup_test_checkpoint(&agent_id).await;
    cleanup_test_summary("summary_quality_proj").await;

    // PHASE 0: Seed a checkpoint so that load_project_memory has context to summarize.
    let seed_bootstrapper = MemoryBootstrapper::new("summary_quality_proj", "dev", &agent_id);
    let seed = seed_bootstrapper.load_project_memory().await.unwrap();
    let bb = Arc::new(Blackboard::new());
    bb.write("dialogue_round_0", "Implemented auth refactor", &agent_id)
        .await;
    let _ = seed
        .checkpoint_mgr
        .save(&agent_id, None, vec![], vec![], &bb)
        .await
        .unwrap();
    drop(seed);
    drop(bb);

    // PHASE 1: Bootstrap with mock LLM — should generate natural-language summary.
    let llm = Arc::new(MockLlmClient::new(
        "上次我完成了用户认证模块的重构，当前正在处理文件上传的错误边界情况，下一步计划是优化数据库查询性能。"
    ));
    let bootstrapper =
        MemoryBootstrapper::new("summary_quality_proj", "dev", &agent_id).with_llm_client(llm);

    let result = bootstrapper.load_project_memory().await.unwrap();

    // Quality checks
    assert!(
        result.summary.len() > 50,
        "摘要过短: {}",
        result.summary.len()
    );
    assert!(
        result.summary.contains("上次")
            || result.summary.contains("当前")
            || result.summary.contains("下一步"),
        "应包含时间上下文词: {}",
        result.summary
    );
    assert!(
        !result.summary.contains("plan_summary"),
        "不应包含原始键名: {}",
        result.summary
    );
    assert!(
        !result.summary.contains("reflections="),
        "不应包含原始键名: {}",
        result.summary
    );

    // Verify persistence: summary.md should exist on disk
    let base = dirs::config_dir().unwrap();
    let path = base
        .join(".hajimi")
        .join("memory")
        .join("summary_quality_proj")
        .join("summary.md");
    assert!(path.exists(), "summary.md 应被持久化到磁盘");

    // PHASE 2: Re-bootstrap without LLM client — should load cached summary.md
    let bootstrapper2 = MemoryBootstrapper::new("summary_quality_proj", "dev", &agent_id);
    let result2 = bootstrapper2.load_project_memory().await.unwrap();
    assert_eq!(
        result2.summary, result.summary,
        "二次加载应从磁盘恢复相同摘要"
    );

    cleanup_test_checkpoint(&agent_id).await;
    cleanup_test_summary("summary_quality_proj").await;
}

#[tokio::test]
async fn test_summary_fallback_on_llm_error() {
    let agent_id = "fallback_agent".to_string();
    cleanup_test_checkpoint(&agent_id).await;
    cleanup_test_summary("fallback_proj").await;

    // No LLM client configured → should trigger fallback (format_raw_summary with emoji)
    let bootstrapper = MemoryBootstrapper::new("fallback_proj", "dev", &agent_id);
    let result = bootstrapper.load_project_memory().await.unwrap();

    // Cold start with no checkpoint → "No checkpoint available"
    assert!(
        result.summary.contains("No checkpoint available"),
        "Expected fallback: {}",
        result.summary
    );

    cleanup_test_checkpoint(&agent_id).await;
    cleanup_test_summary("fallback_proj").await;
}

#[tokio::test]
async fn test_empty_reflections_graceful() {
    let agent_id = "empty_reflections_agent".to_string();
    cleanup_test_checkpoint(&agent_id).await;
    cleanup_test_summary("empty_reflections_proj").await;

    // Mock LLM that returns a fixed response
    let llm = Arc::new(MockLlmClient::new(
        "项目刚开始，还没有太多进展，下一步是搭建基础框架。",
    ));
    let bootstrapper =
        MemoryBootstrapper::new("empty_reflections_proj", "dev", &agent_id).with_llm_client(llm);

    // With no prior checkpoint, summary is fallback text — does not panic
    let result = bootstrapper.load_project_memory().await.unwrap();
    assert!(
        result.summary.contains("No checkpoint available")
            || result.summary.contains("项目状态摘要")
            || result.summary.contains("刚开始"),
        "Empty reflections should not panic, got: {}",
        result.summary
    );

    cleanup_test_checkpoint(&agent_id).await;
    cleanup_test_summary("empty_reflections_proj").await;
}
