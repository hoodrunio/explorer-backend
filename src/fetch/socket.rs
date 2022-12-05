use std::sync::Arc;

use chrono::DateTime;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::chain::Chain;

use super::{
    blocks::{BlockHeader, BlockItem},
    transactions::TransactionItem,
};

const SUBSCRIBE_HEADER: &str = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='NewBlockHeader'"], "id": 1 }"#;
const SUBSCRIBE_TX: &str = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='Tx'"], "id": 2 }"#;

impl Chain {
    /// Subscribes to all the events.
    pub async fn subscribe_to_events(&self) {
        // Define the URL.
        let url = self.inner.wss_url;

        // Connect to the `wss://` URL.
        let (ws_stream, _) = match connect_async(url).await {
            Ok(connection) => connection,
            _ => return eprintln!("Failed to connect to {url}"),
        };

        // Split the connection into two parts.
        let (mut write, read) = ws_stream.split();

        // Subscribe to block headers.
        match write.send(SUBSCRIBE_HEADER.into()).await {
            Ok(()) => (),
            _ => return eprintln!("Can't subscribe to block headers for {}.", self.inner.name),
        }

        // Subscribe to block txs.
        match write.send(SUBSCRIBE_TX.into()).await {
            Ok(()) => (),
            _ => return eprintln!("Can't subscribe to txs for {}.", self.inner.name),
        }

        // The variable to hold the previous block header response to have block hash value.
        let previous_block_header_resp: Arc<Mutex<Option<NewBlockHeaderValue>>> = Arc::new(Mutex::new(None));

        // Run the function below for each message received.
        read.for_each(|msg| async {
            if let Ok(Message::Text(msg)) = msg {
                match serde_json::from_str::<SocketMessage>(&msg) {
                    Ok(msg) => match msg.result {
                        SocketResult::NonEmpty(SocketResultNonEmpty::Header { data }) => {
                            let current_resp = data.value;

                            println!("wss: new block on {}", self.inner.name);
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
                        SocketResult::NonEmpty(SocketResultNonEmpty::Tx { events }) => {
                            println!("wss: new tx on {}", self.inner.name);

                            let tx_item = TransactionItem {
                                amount: events.transfer_amount.get(0).map(|amount| self._get_amount(amount)).unwrap_or(0.00),
                                fee: self._get_amount(&events.tx_fee[0]),
                                hash: events.tx_hash[0].clone(),
                                height: match events.tx_height[0].parse::<u64>() {
                                    Ok(tx_height) => tx_height,
                                    _ => return eprintln!("Cannot parse tx height, {}.", events.tx_height[0]),
                                },
                                time: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64,
                                result: "Success".to_string(),
                                r#type: events.message_action[0]
                                    .split_once("Msg")
                                    .map(|(_, r)| r)
                                    .unwrap_or(events.message_action[0].split('.').last().unwrap_or("Unknown"))
                                    .to_string(),
                            };

                            self.store_new_tx(tx_item);
                        }
                        SocketResult::Empty {} => (),
                    },
                    Err(error) => eprintln!("Websocket JSON parse error for {}.\n{error}", self.inner.name),
                }
            }
        })
        .await;
    }
}

#[derive(Deserialize)]
pub struct SocketMessage {
    pub result: SocketResult,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum SocketResult {
    NonEmpty(SocketResultNonEmpty),
    Empty {},
}

#[derive(Deserialize)]
#[serde(tag = "query")]
pub enum SocketResultNonEmpty {
    #[serde(rename = "tm.event='Tx'")]
    Tx { events: TxEvents },
    #[serde(rename = "tm.event='NewBlockHeader'")]
    Header { data: NewBlockHeaderData },
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
    /// `[ "F0E26D70191E27C8AB6249DE9C088B8C2812443CDF0DF04D7C83AE76A117C083" ]`
    #[serde(rename = "tx.hash")]
    pub tx_hash: [String; 1],

    /// `[ "2931697000000000aevmos" ]`
    #[serde(rename = "tx.fee")]
    pub tx_fee: [String; 1],

    /// `[ "8076531" ]`
    #[serde(rename = "tx.height")]
    pub tx_height: [String; 1],

    /// `[ "/ethermint.evm.v1.MsgEthereumTx" ]`
    #[serde(rename = "message.action")]
    pub message_action: [String; 1],

    /// `[ "1535902500000000aevmos" ]`
    #[serde(rename = "transfer.amount")]
    pub transfer_amount: Vec<String>,
}
