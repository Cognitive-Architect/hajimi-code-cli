use crate::state::{AppState, EditHistoryEntry};
use crate::ProviderConfig;
use crate::StreamEvent;
use agent_core::agent_loop::{TraceEvent, TraceStepType};
use codex_twist::memory::MemoryTier;
use engine_llm_core::ChatMessage;
use futures::StreamExt;
use std::sync::Arc;
use tauri::ipc::Channel;
use tauri::Emitter;

#[derive(serde::Serialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AgentUiEvent {
    Status { message: String },
    Trace { event: TraceEvent },
    Result { output: String },
    Error { message: String },
    Done,
}

#[tauri::command]
pub async fn stream_chat(
    provider: String,
    prompt: String,
    messages: Option<Vec<ChatMessage>>,
    config: Option<ProviderConfig>,
    on_event: Channel<StreamEvent>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let model = config.as_ref().map(|c| c.model.clone()).unwrap_or_default();
    let system_prompt = config.as_ref().and_then(|c| c.system_prompt.clone());
    let diagnostic_session_id = format!(
        "stream:{}:{}",
        provider,
        chrono::Utc::now().timestamp_millis()
    );

    let msg_count = messages.as_ref().map(|m| m.len()).unwrap_or(1);
    crate::write_stream_diagnostic(
        "backend_command_start",
        Some(&diagnostic_session_id),
        serde_json::json!({
            "provider": provider.clone(),
            "model": model.clone(),
            "messageCount": msg_count,
            "hasConfig": config.is_some(),
            "baseUrl": config.as_ref().map(|c| c.base_url.clone()),
        }),
    );

    let chat_result = async {
        let client = crate::create_llm_client(&provider, profile.as_deref(), config)?;

        let msgs = if let Some(msgs) = messages {
            if !msgs.is_empty() {
                msgs
            } else {
                vec![ChatMessage {
                    role: "user".into(),
                    content: prompt,
                    timestamp: None,
                }]
            }
        } else {
            vec![ChatMessage {
                role: "user".into(),
                content: prompt,
                timestamp: None,
            }]
        };
        let msgs_for_opt = msgs.clone();

        let gateway = state.memory_gateway.clone();
        let token_tracker = state.token_tracker.clone();
        let session_key = format!("chat:{}:{}", provider, chrono::Utc::now().timestamp());
        let ctx_json = serde_json::to_string(&msgs).map_err(|e| e.to_string())?;

        let _ = gateway.working().put(session_key.clone(), ctx_json).await;

        let stats_before = gateway.stats().await;
        let token_before = stats_before.working_tokens as u64;
        let precise_prompt_start = client
            .count_tokens(msgs_for_opt.clone(), &model)
            .ok()
            .map(|n| n as u64);

        let _ = crate::audit::log_usage(&crate::audit::KeyUsageRecord {
            timestamp: chrono::Utc::now().to_rfc3339(),
            provider_name: provider.clone(),
            model: model.clone(),
            status: "started".into(),
            estimated_tokens: Some(msg_count as u64 * 50),
            precise_prompt: precise_prompt_start,
            precise_completion: None,
            token_before: Some(token_before),
            token_after: None,
        });

        let mut stream = client
            .stream_chat_with_context(msgs, system_prompt)
            .await
            .map_err(|e| format!("stream start failed: {}", e))?;

        crate::write_stream_diagnostic(
            "backend_stream_started",
            Some(&diagnostic_session_id),
            serde_json::json!({ "provider": provider.clone(), "model": model.clone() }),
        );

        let mut output_events = 0_u64;
        let mut output_chars = 0_u64;
        let mut done_events = 0_u64;
        let mut error_events = 0_u64;
        let mut channel_send_events = 0_u64;
        let mut channel_send_errors = 0_u64;
        #[cfg(feature = "stream-diagnostics")]
        let mut first_output_preview: Option<String> = None;
        #[cfg(not(feature = "stream-diagnostics"))]
        let first_output_preview: Option<String> = None;

        while let Some(chunk) = stream.next().await {
            let (text, is_done, is_error) = match chunk {
                engine_llm_core::StreamChunk::Output(t) => (t, false, false),
                engine_llm_core::StreamChunk::Error(e) => (e, false, true),
                engine_llm_core::StreamChunk::Done => (String::new(), true, false),
                _ => (String::new(), false, false),
            };
            if is_done {
                done_events += 1;
            } else if is_error {
                error_events += 1;
            } else {
                output_events += 1;
                output_chars += text.chars().count() as u64;
                #[cfg(feature = "stream-diagnostics")]
                if first_output_preview.is_none() && !text.is_empty() {
                    first_output_preview = Some(crate::preview_for_diagnostic(&text));
                }
            }
            let usage = if is_done { client.last_usage() } else { None };
            channel_send_events += 1;
            let send_result = on_event.send(StreamEvent {
                chunk: text,
                done: is_done,
                error: if is_error {
                    Some("LLM error".into())
                } else {
                    None
                },
                prompt_tokens: usage.as_ref().map(|u| u.prompt_tokens),
                completion_tokens: usage.as_ref().map(|u| u.completion_tokens),
            });
            if let Err(e) = send_result {
                let err_str = e.to_string();
                channel_send_errors += 1;
                crate::write_stream_diagnostic(
                    "backend_channel_send_error",
                    Some(&diagnostic_session_id),
                    serde_json::json!({
                        "error": err_str,
                        "outputEvents": output_events,
                        "outputChars": output_chars,
                        "doneEvents": done_events,
                        "errorEvents": error_events,
                        "channelSendEvents": channel_send_events,
                        "channelSendErrors": channel_send_errors,
                    }),
                );
                return Err(err_str);
            }
            if is_done {
                break;
            }
        }

        let usage = client.last_usage();
        crate::write_stream_diagnostic(
            "backend_stream_summary",
            Some(&diagnostic_session_id),
            serde_json::json!({
                "outputEvents": output_events,
                "outputChars": output_chars,
                "doneEvents": done_events,
                "errorEvents": error_events,
                "channelSendEvents": channel_send_events,
                "channelSendErrors": channel_send_errors,
                "firstOutputPreview": first_output_preview,
                "promptTokens": usage.as_ref().map(|u| u.prompt_tokens),
                "completionTokens": usage.as_ref().map(|u| u.completion_tokens),
            }),
        );

        if let Some(ref u) = usage {
            token_tracker
                .record_usage(
                    &session_key,
                    &provider,
                    u.prompt_tokens,
                    u.completion_tokens,
                )
                .await;
        }

        crate::write_stream_diagnostic(
            "backend_optimize_start",
            Some(&diagnostic_session_id),
            serde_json::json!({ "messageCount": msgs_for_opt.len() }),
        );
        let _ = gateway.optimize(msgs_for_opt, client.as_ref()).await;
        crate::write_stream_diagnostic(
            "backend_optimize_done",
            Some(&diagnostic_session_id),
            serde_json::json!({}),
        );
        let stats_after = gateway.stats().await;
        let token_after = stats_after.working_tokens as u64;

        let _retrieved = gateway.working().get(&session_key).await;

        Ok((token_before, token_after, usage))
    }
    .await;

    let (chat_result, token_before_val, token_after_val, usage_val) = match chat_result {
        Ok((tb, ta, u)) => (Ok(()), tb, ta, u),
        Err(e) => (Err(e), 0, 0, None),
    };

    let (precise_prompt_end, precise_completion_end) = if let Some(u) = usage_val {
        (Some(u.prompt_tokens), Some(u.completion_tokens))
    } else {
        (None, None)
    };

    crate::write_stream_diagnostic(
        "backend_command_result",
        Some(&diagnostic_session_id),
        serde_json::json!({
            "ok": chat_result.is_ok(),
            "error": chat_result.as_ref().err(),
            "tokenBefore": token_before_val,
            "tokenAfter": token_after_val,
            "promptTokens": precise_prompt_end,
            "completionTokens": precise_completion_end,
        }),
    );

    let _ = crate::audit::log_usage(&crate::audit::KeyUsageRecord {
        timestamp: chrono::Utc::now().to_rfc3339(),
        provider_name: provider,
        model,
        status: if chat_result.is_ok() {
            "completed".into()
        } else {
            "failed".into()
        },
        estimated_tokens: Some(msg_count as u64 * 50),
        precise_prompt: precise_prompt_end,
        precise_completion: precise_completion_end,
        token_before: Some(token_before_val),
        token_after: Some(token_after_val),
    });

    chat_result
}

pub(crate) fn agent_outcome_output(outcome: agent_core::agent_loop::LoopOutcome) -> String {
    match outcome {
        agent_core::agent_loop::LoopOutcome::SuccessWithMessage(message) => message,
        other => format!("{:?}", other),
    }
}

#[tauri::command]
pub async fn run_agent_task(
    agent_id: String,
    goal: String,
    provider_id: Option<String>,
    on_event: Channel<AgentUiEvent>,
    state: tauri::State<'_, AppState>,
    agent_loop: tauri::State<'_, std::sync::Arc<agent_core::agent_loop::AgentLoop>>,
) -> Result<(), String> {
    let trimmed_goal = goal.trim();
    if trimmed_goal.is_empty() {
        return Err("goal cannot be empty".to_string());
    }

    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let provider = provider_id.clone().unwrap_or_else(|| "openai".to_string());

    let config = if provider == "ollama" || provider == "anthropic" || provider == "openai" {
        None
    } else {
        let configs = crate::read_merged_configs(None, profile.as_deref());
        configs.into_iter().find(|c| c.id == provider)
    };

    crate::write_provider_caps_to_blackboard(
        agent_loop.blackboard(),
        &agent_id,
        &provider,
        config.as_ref(),
    )
    .await;

    let client_box = crate::create_llm_client(&provider, profile.as_deref(), config.clone())
        .map_err(|e| format!("Failed to create LLM client for agent: {}", e))?;
    let client_arc: Arc<dyn engine_llm_core::LlmClient> = Arc::from(client_box);
    *state.agent_llm_client.write().await = Some(client_arc);

    let _ = on_event.send(AgentUiEvent::Status {
        message: format!("Agent task started with goal: {}", trimmed_goal),
    });

    if let Some(mut rx) = agent_loop.subscribe_trace() {
        let on_event_trace = on_event.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        if on_event_trace.send(AgentUiEvent::Trace { event }).is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    }
                }
            }
        });
    }

    match agent_loop.execute_goal(agent_id, trimmed_goal).await {
        Ok(outcome) => {
            let output = agent_outcome_output(outcome);
            let _ = on_event.send(AgentUiEvent::Result { output });
            let _ = on_event.send(AgentUiEvent::Done);
        }
        Err(e) => {
            let _ = on_event.send(AgentUiEvent::Error {
                message: format!("Agent loop execution failed: {}", e),
            });
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn run_agent_command(
    cmd: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let trimmed = cmd.trim();
    if trimmed.starts_with("@agent refactor ") {
        let target = trimmed
            .strip_prefix("@agent refactor ")
            .unwrap_or("")
            .to_string();
        return Ok(format!("Refactor request queued for: {}", target));
    }
    if trimmed.starts_with("@agent review-pr") {
        return Ok("PR review mode activated".to_string());
    }
    if trimmed.starts_with("@agent continue-background") {
        *state.paused.lock().unwrap_or_else(|e| e.into_inner()) = false;
        return Ok("Agent resumed in background".to_string());
    }
    if trimmed.starts_with("@agent pause") {
        *state.paused.lock().unwrap_or_else(|e| e.into_inner()) = true;
        return Ok("Agent paused".to_string());
    }
    if trimmed.starts_with("@agent status") {
        let paused = *state.paused.lock().unwrap_or_else(|e| e.into_inner());
        let level = state
            .approval_level
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        return Ok(format!(
            "Agent status: paused={}, approval_level={}",
            paused, level
        ));
    }
    Err(format!("Unknown agent command: {}", cmd))
}

#[tauri::command]
pub async fn subscribe_agent_trace(
    on_event: Channel<TraceEvent>,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let tx = state
        .trace_tx
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let Some(tx) = tx else {
        return Ok(());
    };
    let mut rx = tx.subscribe();
    let history_clone = state.edit_history.clone();
    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if matches!(
                event.step_type,
                TraceStepType::EditProposed
                    | TraceStepType::EditApplied
                    | TraceStepType::EditRejected
            ) {
                let checkpoint = crate::checkpoint_record_from_trace(&event);
                if let Err(e) = crate::write_checkpoint_record(&app_clone, &checkpoint) {
                    eprintln!("checkpoint write failed: {}", e);
                }
                let mut hist = history_clone.lock().await;
                let entry = EditHistoryEntry {
                    id: format!("edit_{}_{}", event.iteration, hist.len()),
                    timestamp: event.timestamp.to_rfc3339(),
                    step_type: format!("{:?}", event.step_type),
                    summary: event.details.clone(),
                    confidence: event.confidence_score,
                    token_before: None,
                    token_after: None,
                    checkpoint_id: Some(checkpoint.id),
                };
                hist.push(entry);
                if hist.len() > 200 {
                    hist.remove(0);
                }
            } else if crate::is_checkpoint_store_trace(&event) {
                let checkpoint = crate::checkpoint_record_from_trace(&event);
                if let Err(e) = crate::write_checkpoint_record(&app_clone, &checkpoint) {
                    eprintln!("checkpoint write failed: {}", e);
                }
            }
            let _ = on_event.send(event.clone());
            let _ = app_clone.emit("agent:trace", &event);
        }
    });
    Ok(())
}

#[tauri::command]
pub fn get_agent_providers(
    state: tauri::State<'_, AppState>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let map = state
        .agent_providers
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    Ok(map)
}

#[tauri::command]
pub async fn set_agent_provider(
    agent_id: String,
    provider_id: Option<String>,
    state: tauri::State<'_, AppState>,
    agent_loop: tauri::State<'_, std::sync::Arc<agent_core::agent_loop::AgentLoop>>,
) -> Result<(), String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let provider = provider_id.clone().unwrap_or_else(|| "openai".to_string());

    {
        let mut map = state
            .agent_providers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(ref pid) = provider_id {
            map.insert(agent_id.clone(), pid.clone());
        } else {
            map.remove(&agent_id);
        }
    }

    let config = if provider == "ollama" || provider == "anthropic" || provider == "openai" {
        None
    } else {
        let configs = crate::read_merged_configs(None, profile.as_deref());
        configs.into_iter().find(|c| c.id == provider)
    };

    crate::write_provider_caps_to_blackboard(
        agent_loop.blackboard(),
        &agent_id,
        &provider,
        config.as_ref(),
    )
    .await;

    Ok(())
}

#[tauri::command]
pub async fn create_agent_with_provider(
    agent_id: String,
    goal: String,
    provider_id: Option<String>,
    state: tauri::State<'_, AppState>,
    agent_loop: tauri::State<'_, std::sync::Arc<agent_core::agent_loop::AgentLoop>>,
) -> Result<String, String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let provider = provider_id.clone().unwrap_or_else(|| "openai".to_string());

    {
        let mut map = state
            .agent_providers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(pid) = provider_id.clone() {
            map.insert(agent_id.clone(), pid);
        } else {
            map.remove(&agent_id);
        }
    }

    let config = if provider == "ollama" || provider == "anthropic" || provider == "openai" {
        None
    } else {
        let configs = crate::read_merged_configs(None, profile.as_deref());
        configs.into_iter().find(|c| c.id == provider)
    };

    crate::write_provider_caps_to_blackboard(
        agent_loop.blackboard(),
        &agent_id,
        &provider,
        config.as_ref(),
    )
    .await;

    let model = config.as_ref().map(|c| c.model.clone()).unwrap_or_default();

    let result = async {
        let client = crate::create_llm_client(&provider, profile.as_deref(), config)?;
        let precise_prompt_start = client
            .count_tokens(
                vec![ChatMessage {
                    role: "user".into(),
                    content: goal.clone(),
                    timestamp: None,
                }],
                &model,
            )
            .ok()
            .map(|n| n as u64);

        let _ = crate::audit::log_usage(&crate::audit::KeyUsageRecord {
            timestamp: chrono::Utc::now().to_rfc3339(),
            provider_name: provider.clone(),
            model: model.clone(),
            status: "started".into(),
            estimated_tokens: None,
            precise_prompt: precise_prompt_start,
            precise_completion: None,
            token_before: None,
            token_after: None,
        });

        let mut stream = client
            .stream_chat(goal)
            .await
            .map_err(|e| format!("stream start failed: {}", e))?;

        let mut output = String::new();
        while let Some(chunk) = stream.next().await {
            match chunk {
                engine_llm_core::StreamChunk::Output(text) => output.push_str(&text),
                engine_llm_core::StreamChunk::Error(e) => return Err(format!("LLM error: {}", e)),
                engine_llm_core::StreamChunk::Done => break,
                _ => {}
            }
        }
        let usage = client.last_usage();
        Ok((output, usage))
    }
    .await;

    let (_output_val, usage_val) = match &result {
        Ok((out, usage)) => (Some(out.clone()), *usage),
        Err(_) => (None, None),
    };

    let (precise_prompt_end, precise_completion_end) = if let Some(u) = usage_val {
        (Some(u.prompt_tokens), Some(u.completion_tokens))
    } else {
        (None, None)
    };

    let _ = crate::audit::log_usage(&crate::audit::KeyUsageRecord {
        timestamp: chrono::Utc::now().to_rfc3339(),
        provider_name: provider,
        model,
        status: if result.is_ok() {
            "completed".into()
        } else {
            "failed".into()
        },
        estimated_tokens: None,
        precise_prompt: precise_prompt_end,
        precise_completion: precise_completion_end,
        token_before: None,
        token_after: None,
    });

    match result {
        Ok((output, _)) => Ok(format!("Agent {} completed. Output:\n{}", agent_id, output)),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn subscribe_resource_alerts(on_event: Channel<TraceEvent>) -> Result<(), String> {
    on_event
        .send(TraceEvent {
            step: agent_core::LoopState::Idle,
            details: "Resource alerts subscription started".to_string(),
            iteration: 0,
            timestamp: chrono::Utc::now(),
            step_type: TraceStepType::Other,
            plan_summary: None,
            reflection_key_points: vec![],
            confidence_score: None,
            edit_payload: None,
            operation_summary: None,
            thinking_content: None,
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}
