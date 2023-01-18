use std::collections::HashSet;
use crate::database::BlockForDb;
use crate::fetch::transactions::TransactionItem;
use actix::{Actor, AsyncContext, StreamHandler};
use actix_web::web::Data;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use futures::{SinkExt, StreamExt, TryStreamExt};
use serde::de::Error;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use tokio::sync::oneshot;
use cosmrs::bip32::secp256k1::elliptic_curve::weierstrass::add;
use dashmap::{DashMap, DashSet};
use serde_json::to_string;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio_tungstenite::tungstenite::Message;
use tracing_subscriber::fmt::format;
use tokio_tungstenite::tungstenite::handshake::server::{Callback, ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::header::SEC_WEBSOCKET_PROTOCOL;
use tokio_tungstenite::tungstenite::http::StatusCode;

pub type PeerMap = DashMap<SocketAddr, DashSet<String>>;

pub async fn handle_connection(tx: Sender<(String, WsEvent)>, raw_stream: TcpStream, addr: SocketAddr, chains: HashSet<String>) -> Result<(), String> {
    tracing::info!("Incoming TCP connection from: {addr}");

    let (tx_config, rx_config) =  oneshot::channel();
    let callback = |request: &Request, mut response: Response| -> Result<Response, ErrorResponse> {
        let Some(chain) = request.uri().to_string()[1..].split("/" ).next().map(|s| s.to_string()) else {
            return Err(ErrorResponse::new(Some("No chain specified".to_string())));
        };

        if !chains.contains(&chain) {
            return Err(ErrorResponse::new(Some("Chain is not found".to_string())));
        }
        tx_config.send(chain.to_string()).ok();

        // let protocol = request.headers().get(SEC_WEBSOCKET_PROTOCOL).expect("the client should specify a protocol").to_owned(); //save the protocol to use outside the closure
        // let response_protocol = request.headers().get(SEC_WEBSOCKET_PROTOCOL).expect("the client should specify a protocol").to_owned();
        // response.headers_mut().insert(SEC_WEBSOCKET_PROTOCOL, response_protocol);

        Ok(response)
    };

    let ws_stream = tokio_tungstenite::accept_hdr_async(raw_stream, callback)
        .await
        .map_err(|e| format!("Error creating websocket connection: {e}"))?;


    let wanted_chain = rx_config.await.map_err(|e| format!("Error getting the subjects: {e}"))?;

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
            Ok((chain, msg)) = rx.recv() => {
                tracing::debug!("Got message from channel for chain {chain}: {msg}");

                if chain == wanted_chain {
                    outgoing.send(Message::Text(serde_json::to_string(&msg).unwrap())).await;
                }
            }
        }
    }

    Ok(())
}

pub async fn run_ws(tx: Sender<(String, WsEvent)>, chains: HashSet<String>) -> Result<(), String> {
    let listener = TcpListener::bind("127.0.0.1:8081").await.map_err(|e| format!("Error binding: {e}"))?;

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(tx.clone(), stream, addr, chains.clone()));
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
