use crate::protocol::{error_codes, JsonRpcRequest, JsonRpcResponse, ProtocolError};
use serde_json::json;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type HandlerFn = Arc<
    dyn Fn(
            JsonRpcRequest,
        )
            -> Pin<Box<dyn Future<Output = Result<serde_json::Value, ProtocolError>> + Send>>
        + Send
        + Sync,
>;

#[derive(Clone)]
pub struct HandlerRegistry {
    handlers: Arc<RwLock<HashMap<String, HandlerFn>>>,
}

impl std::fmt::Debug for HandlerRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandlerRegistry")
            .field("handlers", &"<Map>")
            .finish()
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register<F, Fut>(&self, method: impl Into<String>, handler: F)
    where
        F: Fn(JsonRpcRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<serde_json::Value, ProtocolError>> + Send + 'static,
    {
        let method = method.into();
        let boxed_handler: HandlerFn = Arc::new(move |req| Box::pin(handler(req)));
        let mut handlers = self.handlers.write().await;
        handlers.insert(method, boxed_handler);
    }

    pub async fn handle(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let handlers = self.handlers.read().await;
        match handlers.get(&request.method) {
            Some(handler) => match handler(request.clone()).await {
                Ok(result) => JsonRpcResponse::success(result, request.id),
                Err(e) => match e {
                    ProtocolError::InvalidParams(msg) => {
                        JsonRpcResponse::error(error_codes::INVALID_PARAMS, msg, request.id)
                    }
                    _ => {
                        JsonRpcResponse::error(error_codes::SERVER_ERROR, e.to_string(), request.id)
                    }
                },
            },
            None => JsonRpcResponse::error(
                error_codes::METHOD_NOT_FOUND,
                format!("Method '{}' not found", request.method),
                request.id,
            ),
        }
    }
}

pub struct BuiltinHandlers;

impl BuiltinHandlers {
    pub async fn create_registry() -> HandlerRegistry {
        use std::sync::Arc;
        use tokio::sync::RwLock;
        let feedback_store: Arc<RwLock<Vec<serde_json::Value>>> = Arc::new(RwLock::new(Vec::new()));

        let registry = HandlerRegistry::new();
        registry
            .register(
                "health",
                |_req| async move { Ok(json!({"status": "healthy"})) },
            )
            .await;
        registry
            .register("echo", |req| async move { Ok(json!({"echo": req.params})) })
            .await;
        registry
            .register("server_info", |_req| async move {
                Ok(json!({
                    "name": "ws_server",
                    "version": env!("CARGO_PKG_VERSION"),
                    "protocol": "jsonrpc-2.0",
                }))
            })
            .await;

        // Week 4: hajimi/feedback endpoint — stores feedback in memory for now.
        // DEBT-RUST-FEEDBACK-001: Route to MemoryGateway + Governance in Week 5–6.
        let store = feedback_store.clone();
        registry
            .register("hajimi/feedback", move |req| {
                let store = store.clone();
                async move {
                    let params = req.params.unwrap_or(json!({}));
                    let items = params
                        .get("items")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    let mut guard = store.write().await;
                    for item in items {
                        guard.push(item);
                    }
                    let count = guard.len();
                    drop(guard);
                    Ok(json!({ "success": true, "storedCount": count }))
                }
            })
            .await;

        registry
    }
}

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub connection_id: String,
    pub remote_addr: String,
    pub request_time: std::time::Instant,
}

impl RequestContext {
    pub fn new(connection_id: impl Into<String>, remote_addr: impl Into<String>) -> Self {
        Self {
            connection_id: connection_id.into(),
            remote_addr: remote_addr.into(),
            request_time: std::time::Instant::now(),
        }
    }
}

pub mod utils {
    use super::*;

    pub fn validate_params(
        params: &Option<serde_json::Value>,
        required: &[&str],
    ) -> Result<serde_json::Map<String, serde_json::Value>, ProtocolError> {
        let params = params
            .as_ref()
            .ok_or_else(|| ProtocolError::InvalidParams("Missing params".to_string()))?;
        let obj = params
            .as_object()
            .ok_or_else(|| ProtocolError::InvalidParams("Params must be object".to_string()))?;
        for &field in required {
            if !obj.contains_key(field) {
                return Err(ProtocolError::InvalidParams(format!("Missing: {}", field)));
            }
        }
        Ok(obj.clone())
    }

    pub fn success(data: serde_json::Value) -> Result<serde_json::Value, ProtocolError> {
        Ok(data)
    }

    pub fn error(msg: impl Into<String>) -> Result<serde_json::Value, ProtocolError> {
        Err(ProtocolError::InternalError(msg.into()))
    }
}
