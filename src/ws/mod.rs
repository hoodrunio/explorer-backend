use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock, Mutex, oneshot};
use tokio::time::{Duration, interval};
use tokio_tungstenite::tungstenite::Message;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use serde_querystring::{ParseMode, from_str};

use crate::events::WsEvent;
use crate::logging::{LogLevel::*, log_socket};

pub struct WsManager {
    supported_chains: HashSet<String>,
    connections: Arc<RwLock<HashMap<String, ConnectionStats>>>,
    event_tx: broadcast::Sender<(String, WsEvent)>,
    senders: Arc<RwLock<HashMap<String, broadcast::Sender<(String, WsEvent)>>>>,
}

#[derive(Debug)]
pub struct ConnectionStats {
    pub connected_at: std::time::Instant,
    pub message_count: u64,
    pub last_message_at: std::time::Instant,
}

impl WsManager {
    pub fn new(chains: HashSet<String>, channel_capacity: usize) -> Self {
        let (event_tx, _) = broadcast::channel(channel_capacity);
        Self {
            supported_chains: chains,
            connections: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            senders: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_event_sender(&self) -> broadcast::Sender<(String, WsEvent)> {
        self.event_tx.clone()
    }

    pub async fn start_server(self: Arc<Self>) -> std::io::Result<()> {
        let addr = "127.0.0.1:8081";
        let listener = TcpListener::bind(&addr).await?;
        log_socket(INFO, &format!("WebSocket server listening on: {}", addr));

        let cleanup_interval = interval(Duration::from_secs(60));
        tokio::pin!(cleanup_interval);

        loop {
            tokio::select! {
                Ok((stream, addr)) = listener.accept() => {
                    let ws_manager = Arc::clone(&self);
                    tokio::spawn(async move {
                        match ws_manager.handle_connection(stream, addr).await {
                            Ok(_) => log_socket(INFO, &format!("Connection handled successfully: {}", addr)),
                            Err(e) => log_socket(ERROR, &format!("Error handling connection {}: {:?}", addr, e)),
                        }
                    });
                }
                _ = cleanup_interval.tick() => {
                    self.cleanup_stale_connections().await;
                }
            }
        }
    }

    async fn cleanup_stale_connections(&self) {
        let mut connections = self.connections.write().await;
        let now = std::time::Instant::now();
        connections.retain(|_, stats| {
            now.duration_since(stats.last_message_at) < Duration::from_secs(300)
        });
    }

    async fn handle_connection(
        &self,
        stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<(), ErrorResponse> {
        let (mode_tx, mode_rx) = oneshot::channel();
        let mode_tx = Arc::new(Mutex::new(Some(mode_tx)));

        let callback = |req: &Request, response: Response| {
            let uri = req.uri();
            let query = uri.query().unwrap_or("");

            if query.is_empty() {
                return Err(ErrorResponse::new(Some("Please provide subscription parameters".to_string())));
            };

            let Ok(mode) = from_str::<SubscriptionMode>(query, ParseMode::UrlEncoded) else {
                return Err(ErrorResponse::new(Some("Invalid query parameters".to_string())));
            };

            let chain = mode.chain.clone();
            if !self.supported_chains.contains(&chain) {
                return Err(ErrorResponse::new(Some(format!("Unsupported chain: {}", chain))));
            }

            // Send the mode through the channel
            if let Some(tx) = mode_tx.try_lock().ok().and_then(|mut guard| guard.take()) {
                let _ = tx.send(mode);
            }

            Ok(response)
        };

        let ws_stream = tokio_tungstenite::accept_hdr_async(stream, callback)
            .await
            .map_err(|e| ErrorResponse::new(Some(e.to_string())))?;

        // Receive the mode
        let mode = mode_rx.await.map_err(|_| ErrorResponse::new(Some("Failed to get subscription mode".to_string())))?;

        log_socket(INFO, &format!("WebSocket connection established from: {addr}"));

        let (ws_sender, mut ws_receiver) = ws_stream.split();
        let (tx, mut rx) = broadcast::channel(32);
        let addr_str = addr.to_string();

        // Store connection stats
        self.connections.write().await.insert(addr_str.clone(), ConnectionStats {
            connected_at: std::time::Instant::now(),
            message_count: 0,
            last_message_at: std::time::Instant::now(),
        });

        // Store message sender
        self.senders.write().await.insert(addr_str.clone(), tx);

        // Create a mutex for the sender
        let ws_sender = Arc::new(Mutex::new(ws_sender));
        let ws_sender_clone = Arc::clone(&ws_sender);

        // Spawn task to handle incoming messages
        let connections = Arc::clone(&self.connections);
        let addr_clone = addr_str.clone();
        tokio::spawn(async move {
            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(Message::Ping(data)) => {
                        let mut sender = ws_sender.lock().await;
                        if let Err(e) = sender.send(Message::Pong(data)).await {
                            log_socket(ERROR, &format!("Error sending pong: {}", e));
                            break;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        break;
                    }
                    Ok(_) => {
                        if let Some(stats) = connections.write().await.get_mut(&addr_clone) {
                            stats.message_count += 1;
                            stats.last_message_at = std::time::Instant::now();
                        }
                    }
                    Err(e) => {
                        log_socket(ERROR, &format!("WebSocket error: {}", e));
                        break;
                    }
                }
            }
        });

        // Spawn task to handle outgoing messages
        let addr_clone2 = addr_str.clone();
        tokio::spawn(async move {
            while let Ok((_chain, event)) = rx.recv().await {
                let should_send = match &event {
                    WsEvent::NewTX(_) => mode.tx,
                    WsEvent::NewBLock(_) => mode.block,
                    WsEvent::NewEvmPoll(_) | WsEvent::UpdateEvmPollParticipant(_) => mode.poll,
                };

                if !should_send {
                    continue;
                }

                let json = serde_json::to_string(&event).unwrap_or_else(|e| {
                    log_socket(ERROR, &format!("Error serializing event: {}", e));
                    String::from("{}")
                });

                let mut sender = ws_sender_clone.lock().await;
                if let Err(e) = sender.send(Message::Text(json)).await {
                    log_socket(ERROR, &format!("Error sending message to {}: {}", addr_clone2, e));
                    break;
                }
            }
        });

        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
struct SubscriptionMode {
    chain: String,
    tx: bool,
    block: bool,
    poll: bool,
}
