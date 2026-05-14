use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};

pub mod handlers;
pub mod protocol;

use crate::handlers::HandlerRegistry;
use crate::protocol::{
    error_codes, ConnectionState, JsonRpcRequest, JsonRpcResponse, ProtocolError, ServerConfig,
    WsMessage,
};
use std::sync::atomic::{AtomicU64, Ordering};

static CONN_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub struct Connection {
    pub id: String,
    pub remote_addr: SocketAddr,
    pub state: ConnectionState,
    pub sender: mpsc::UnboundedSender<WsMessage>,
}

// SAFETY: connection cleanup on drop - all resources are properly managed via RAII
#[derive(Debug, Clone)]
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, Connection>>>,
    max_connections: usize,
}

impl ConnectionManager {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            max_connections,
        }
    }

    pub async fn add_connection(&self, conn: Connection) -> Result<(), ProtocolError> {
        let mut conns = self.connections.write().await;
        if conns.len() >= self.max_connections {
            return Err(ProtocolError::InternalError("Max connections".to_string()));
        }
        conns.insert(conn.id.clone(), conn);
        Ok(())
    }

    pub async fn remove_connection(&self, id: &str) {
        self.connections.write().await.remove(id);
    }
}

#[derive(Debug, Clone)]
pub struct WebSocketServer {
    config: ServerConfig,
    handler_registry: HandlerRegistry,
    connection_manager: ConnectionManager,
}

impl WebSocketServer {
    pub fn new(config: ServerConfig, handler_registry: HandlerRegistry) -> Self {
        let max_conns = config.max_connections;
        Self {
            config,
            handler_registry,
            connection_manager: ConnectionManager::new(max_conns),
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = self.config.bind_addr.parse()?;
        let listener = TcpListener::bind(addr).await?;
        info!("WebSocket server on {}", addr);

        loop {
            match listener.accept().await {
                Ok((stream, remote_addr)) => {
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream, remote_addr).await {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => error!("Accept error: {}", e),
            }
        }
    }

    async fn handle_connection(
        &self,
        stream: TcpStream,
        remote_addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ws_stream = accept_async(stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let connection_id = format!(
            "{}-{:016x}",
            ts,
            CONN_COUNTER.fetch_add(1, Ordering::SeqCst)
        );

        info!("New connection {} from {}", connection_id, remote_addr);

        let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();

        let connection = Connection {
            id: connection_id.clone(),
            remote_addr,
            state: ConnectionState::Connected,
            sender: tx.clone(),
        };

        self.connection_manager.add_connection(connection).await?;

        let heartbeat_interval = self.config.heartbeat_interval;
        let handler_registry = self.handler_registry.clone();
        let connection_manager = self.connection_manager.clone();
        let conn_id = connection_id.clone();

        tokio::spawn(async move {
            let mut heartbeat = interval(heartbeat_interval);
            let mut last_pong = std::time::Instant::now();
            let timeout = Duration::from_secs(60);

            loop {
                tokio::select! {
                    _ = heartbeat.tick() => {
                        if last_pong.elapsed() > timeout {
                            warn!("Heartbeat timeout for {}", conn_id);
                            break;
                        }
                        let ping_msg = Message::Ping(vec![]);
                        if ws_sender.send(ping_msg).await.is_err() {
                            break;
                        }
                    }
                    Some(msg) = rx.recv() => {
                        let ws_msg = match msg {
                            WsMessage::Text(s) => Message::Text(s),
                            WsMessage::Binary(b) => Message::Binary(b),
                            WsMessage::Ping(p) => Message::Ping(p),
                            WsMessage::Pong(p) => Message::Pong(p),
                            WsMessage::Close(c, r) => Message::Close(c.map(|code|
                                tokio_tungstenite::tungstenite::protocol::CloseFrame {
                                    code: code.into(),
                                    reason: r.into(),
                                })),
                        };
                        if ws_sender.send(ws_msg).await.is_err() {
                            break;
                        }
                    }
                    Some(msg) = ws_receiver.next() => {
                        match msg {
                            Ok(Message::Text(text)) => {
                                match Self::handle_message(&text, &handler_registry).await {
                                    Ok(response) => {
                                        if let Ok(json) = serde_json::to_string(&response) {
                                            let _ = tx.send(WsMessage::Text(json));
                                        }
                                    }
                                    Err(e) => error!("Handler error: {}", e),
                                }
                            }
                            Ok(Message::Binary(bin)) => {
                                debug!("Binary: {} bytes", bin.len());
                            }
                            Ok(Message::Ping(_)) => {
                                let _ = tx.send(WsMessage::Pong(vec![]));
                            }
                            Ok(Message::Pong(_)) => {
                                last_pong = std::time::Instant::now();
                            }
                            Ok(Message::Close(_)) => {
                                info!("Connection {} closed", conn_id);
                                break;
                            }
                            _ => {}
                        }
                    }
                    else => break,
                }
            }

            connection_manager.remove_connection(&conn_id).await;
            info!("Connection {} cleaned up", conn_id);
        });

        Ok(())
    }

    async fn handle_message(
        text: &str,
        registry: &HandlerRegistry,
    ) -> Result<JsonRpcResponse, ProtocolError> {
        let request: JsonRpcRequest = match serde_json::from_str::<JsonRpcRequest>(text) {
            Ok(req) => req,
            Err(e) => {
                return Ok(JsonRpcResponse::error(
                    error_codes::PARSE_ERROR,
                    format!("Parse error: {}", e),
                    None,
                ));
            }
        };

        if request.jsonrpc != "2.0" {
            return Ok(JsonRpcResponse::error(
                error_codes::INVALID_REQUEST,
                "Invalid JSON-RPC version",
                request.id,
            ));
        }

        Ok(registry.handle(request).await)
    }
}
