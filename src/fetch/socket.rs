use std::sync::Arc;

use chrono::DateTime;
use futures::{Future, SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Deserialize};
use std::sync::Mutex;
use tokio::join;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::chain::Chain;

use super::blocks::{BlockHeader, BlockItem};

/// Enum for each web socket event.
pub enum SocketEvent {
    NewBlockHeader,
    Tx,
}

impl SocketEvent {
    /// Returns the message that should be sent to Web Socket provider to subscribe preferred event.
    pub fn name(&self) -> &str {
        match self {
            Self::NewBlockHeader => "NewBlockHeader",
            Self::Tx => "Tx",
        }
    }

    /// Returns the message that should be sent to Web Socket provider to subscribe preferred event.
    pub fn subscribing_msg(&self) -> &str {
        match self {
            Self::NewBlockHeader => r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='NewBlockHeader'"], "id": 2 }"#,
            Self::Tx => r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='Tx'"], "id": 1 }"#,
        }
    }
}

impl Chain {
    /// Subscribes to all the events.
    pub async fn subscribe_to_events(&self) {
        join!(self.subscribe_to_new_blocks(), self.subscribe_to_new_transactions());
    }

    /// Subscribes to preferred event.
    ///
    /// It is the base function for WebSocket subscribtions.
    ///
    /// Generic `T` is the expected return type of the WebSocket.
    ///
    /// Generic `F` is the type for the function that will rerun for each message received.
    async fn subscribe_to_event<T, F, Fut>(&self, event: SocketEvent, f: F)
    where
        T: DeserializeOwned,
        F: Fn(T) -> Fut,
        Fut: Future<Output = ()>,
    {
        let url = self.inner.wss_url;

        println!("aa");
        let (ws_stream, _) = match connect_async(url).await {
            Ok(connection) => connection,
            _ => return eprintln!("Failed to connect to {url}"),
        };

        let (mut write, read) = ws_stream.split();

        match write.send(event.subscribing_msg().into()).await {
            Ok(()) => (),
            _ => return eprintln!("Can't subscribe to {} for {}.", event.name(), self.inner.name),
        }

        read.for_each(|msg| async {
            if let Ok(Message::Text(msg)) = msg {
                match serde_json::from_str::<T>(&msg) {
                    Ok(msg) => {
                        f(msg).await;
                    }
                    Err(error) => eprintln!("Websocket JSON parse error for {}.\n{error}", self.inner.name),
                }
            }
        })
        .await;
    }

    /// Subscribes to `NewBlockHeader` event.
    async fn subscribe_to_new_blocks(&self) {
        let previous_block_header_resp: Arc<Mutex<Option<NewBlockHeaderValue>>> = Arc::new(Mutex::new(None));

        self.subscribe_to_event(SocketEvent::NewBlockHeader, |msg: SocketMessage<NewBlockHeaderMessage>| async {
            if let Some(data) = msg.result.data {
                let current_resp = data.value;

                if let Ok(mut mutex_previous_resp) = previous_block_header_resp.lock() {
                    match &*mutex_previous_resp {
                        Some(previous_resp) => {
                            let proposer_metadata =
                                match self.get_validator_metadata_by_hex_addr_blocking(previous_resp.header.proposer_address.clone()) {
                                    Some(proposer_metadata) => proposer_metadata,
                                    None => return,
                                };

                            let block_item = BlockItem {
                                hash: current_resp.header.last_block_id.hash.clone(),
                                height: match previous_resp.header.height.parse::<u64>() {
                                    Ok(height) => height,
                                    _ => return eprintln!("Cannot parse block height, {}.", previous_resp.header.height),
                                },
                                timestamp: match DateTime::parse_from_rfc3339(&previous_resp.header.time) {
                                    Ok(date_time) => date_time.timestamp_millis(),
                                    _ => return eprintln!("Cannot parse datetime, {}.", previous_resp.header.time),
                                },
                                tx_count: match previous_resp.num_txs.parse::<u64>() {
                                    Ok(num_txs) => num_txs,
                                    _ => return eprintln!("Cannot parse Tx count, {}.", previous_resp.num_txs),
                                },
                                proposer_logo_url: proposer_metadata.logo_url,
                                proposer_name: proposer_metadata.name,
                            };

                            self.store_new_block(block_item);

                            *mutex_previous_resp = Some(current_resp);
                        }
                        None => *mutex_previous_resp = Some(current_resp),
                    }
                }
            }
        })
        .await;
    }

    /// Subscribes to `Tx` event.
    async fn subscribe_to_new_transactions(&self) {
        self.subscribe_to_event(SocketEvent::Tx, |_msg: SocketMessage<TxMessage>| async {

            /*
            match msg.result.events {
                Some(events) => match events.tx_hash.get(0) {
                    Some(hash) => match self.get_tx_by_hash(hash).await {
                        Ok(tx) => self.store_new_tx(tx.value.into()),
                        Err(error) => eprintln!("Tx WebSocket for {}. {error}", self.inner.name),
                    },
                    None => eprintln!("Tx WebSocket for {}. No Tx hash exists.", self.inner.name),
                },
                None => (),
            };
            */
        })
        .await
    }
}

#[derive(Deserialize)]
pub struct SocketMessage<T> {
    pub result: T,
}

#[derive(Deserialize)]
pub struct NewBlockHeaderMessage {
    pub data: Option<NewBlockHeaderData>,
}

#[derive(Deserialize)]
pub struct NewBlockHeaderData {
    pub value: NewBlockHeaderValue,
}

#[derive(Deserialize)]
pub struct NewBlockHeaderValue {
    pub header: BlockHeader,
    pub num_txs: String,
}

#[derive(Deserialize)]
pub struct TxMessage {
    pub events: Option<TxEvents>,
}

#[derive(Deserialize)]
pub struct TxEvents {
    #[serde(rename = "tx.hash")]
    pub tx_hash: [String; 1],
}
