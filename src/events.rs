use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;

use dashmap::{DashMap, DashSet};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_querystring::de::ParseMode;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::Sender;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::Message;

use crate::database::{BlockForDb, EvmPollForDb, EvmPollParticipantForDb};
use crate::fetch::transactions::TransactionItem;

pub type PeerMap = DashMap<SocketAddr, DashSet<String>>;

#[derive(Serialize, Deserialize, Debug)]
struct SubscriptionMode {
    #[serde(default)]
    tx: bool,
    #[serde(default)]
    block: bool,
    #[serde(default)]
    poll: bool,
}

pub async fn handle_connection(tx: Sender<(String, WsEvent)>, raw_stream: TcpStream, addr: SocketAddr, chains: HashSet<String>) -> Result<(), String> {
    tracing::info!("Incoming TCP connection from: {addr}");

    let (tx_config, rx_config) = oneshot::channel();
    let callback = |request: &Request, response: Response| -> Result<Response, ErrorResponse> {
        let Some(chain) = request.uri().path().to_string()[1..].split("/").next().map(|s| s.to_string()) else {
            return Err(ErrorResponse::new(Some("No chain specified".to_string())));
        };

        dbg!("path");
        if !chains.contains(&chain) {
            return Err(ErrorResponse::new(Some("Chain is not found".to_string())));
        }
        dbg!("chain");

        let Some(query) = request.uri().query() else {
            return Err(ErrorResponse::new(Some("Please provide the subjects as parameters".to_string())));
        };

        dbg!("query");

        let Ok(parsed) = dbg!(serde_querystring::from_str::<SubscriptionMode>(query, ParseMode::UrlEncoded)) else {
            return Err(ErrorResponse::new(Some("Invalid query parameters".to_string())));
        };

        dbg!("parse");

        tx_config.send((chain.to_string(), parsed)).ok();

        // let protocol = request.headers().get(SEC_WEBSOCKET_PROTOCOL).expect("the client should specify a protocol").to_owned(); //save the protocol to use outside the closure
        // let response_protocol = request.headers().get(SEC_WEBSOCKET_PROTOCOL).expect("the client should specify a protocol").to_owned();
        // response.headers_mut().insert(SEC_WEBSOCKET_PROTOCOL, response_protocol);

        Ok(response)
    };

    let ws_stream = tokio_tungstenite::accept_hdr_async(raw_stream, callback)
        .await
        .map_err(|e| format!("Error creating websocket connection: {e}"))?;


    let (wanted_chain, mode) = rx_config.await.map_err(|e| format!("Error getting the subjects: {e}"))?;

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

                let should_send = match msg {
                    WsEvent::NewTX(_) => mode.tx,
                    WsEvent::NewBLock(_) => mode.block,
                    WsEvent::NewEvmPoll(_) => mode.poll,
                    WsEvent::UpdateEvmPollParticipant(_) => mode.poll,
                };
                if chain == wanted_chain && should_send {
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
    NewEvmPoll(EvmPollForDb),
    UpdateEvmPollParticipant((String, EvmPollParticipantForDb)),
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
            WsEvent::NewEvmPoll(poll) => {
                let poll_id = poll.poll_id.clone();
                write!(f, "WsEvent (NewEvmPoll), id: {poll_id}")
            }
            WsEvent::UpdateEvmPollParticipant((poll_id, participant)) => {
                let participant_address = participant.voter_address.clone();
                write!(f, "WsEvent (UpdateEvmPollParticipant), poll_id: {poll_id}, participant_hash: {participant_address}")
            }
        }
    }
}
