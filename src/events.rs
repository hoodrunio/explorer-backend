use crate::database::BlockForDb;
use crate::fetch::transactions::TransactionItem;
use actix::{Actor, AsyncContext, StreamHandler};
use actix_web::web::Data;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use futures::{SinkExt, StreamExt};
use serde::de::Error;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio_tungstenite::tungstenite::Message;
use tracing_subscriber::fmt::format;

pub async fn handle_connection(tx: Sender<WsEvent>, raw_stream: TcpStream, addr: SocketAddr) -> Result<(), String> {
    tracing::info!("Incoming TCP connection from: {addr}");

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .map_err(|e| format!("Error creating websocket connection: {e}"))?;

    tracing::info!("WebSocket connection established: {addr}");

    let (mut outgoing, mut incoming) = ws_stream.split();

    let mut rx = tx.subscribe();

    // while let Ok(msg) = rx.recv() {
    //     tracing::debug!("Got message from channel: {msg}");
    //     let msg = serde_json::to_string(&msg) else {
    //         continue
    //     }
    //     outgoing.send(Message::Text(msg))
    // }
    loop {
        tokio::select! {
            Some(Ok(msg)) = incoming.next() => {
                tracing::debug!("Got message from ws: {msg}");
                match msg {
                    Message::Ping(bytes) => { outgoing.send(Message::Pong(bytes)); },
                    Message::Close(_) => {break }
                    _ => {}
                };
            },
            Ok(msg) = rx.recv() => {

                tracing::debug!("Got message from channel: {msg}");
                outgoing.send(Message::Text(serde_json::to_string(&msg).unwrap())).await;
            }
        }
    }

    Ok(())
}

pub async fn run_ws(tx: Sender<WsEvent>) -> Result<(), String> {
    let listener = TcpListener::bind("127.0.0.1:8081").await.map_err(|e| format!("Error binding: {e}"))?;

    tracing::info!("BOUND");
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(tx.clone(), stream, addr));
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsEvent {
    NewTX(TransactionItem),
    NewBLock(BlockForDb),
}

impl Display for WsEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WsEvent::NewTX(tx) => {
                let hash = tx.hash.clone();
                write!(f, "WsEvent (NewTX), hash: {hash}")
            }
            WsEvent::NewBLock(block) => {
                let hash = block.hash.clone();
                write!(f, "WsEvent (NewBlock), hash: {hash}")
            }
        }
    }
}
