use std::sync::Arc;

use chrono::DateTime;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing_subscriber::fmt::format;

use crate::chain::Chain;

use super::{
    blocks::{BlockHeader, BlockItem},
    transactions::TransactionItem,
};

const SUBSCRIBE_HEADER: &str = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='NewBlockHeader'"], "id": 1 }"#;
const SUBSCRIBE_TX: &str = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='Tx'"], "id": 2 }"#;

impl Chain {
    /// Subscribes to all the events.
    pub async fn subscribe_to_events(&self) -> Result<(), String> {
        // Define the URL.
        let clone = self.clone();
        let url = &clone.config.wss_url;

        // Connect to the `wss://` URL.
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| format!("Failed to connect to {url}: {e}"))?;

        // Split the connection into two parts.
        let (mut write, mut read) = ws_stream.split();

        // Subscribe to block headers.
        write.send(SUBSCRIBE_HEADER.into()).await.map_err(|e| format!("Can't subscribe to block headers for {}: {e}", clone.config.name))?;

        // Subscribe to block txs.
        write.send(SUBSCRIBE_TX.into()).await.map_err(|e| format!("Can't subscribe to txs for {}: {e}", clone.config.name))?;

        // The variable to hold the previous block header response to have block hash value.
        let previous_block_header_resp: Arc<Mutex<Option<NewBlockHeaderValue>>> = Arc::new(Mutex::new(None));

        while let Some(msg) = read.next().await {
            // Run the function below for each message received.
            if let Ok(Message::Text(msg)) = msg {
                match serde_json::from_str::<SocketMessage>(&msg) {
                    Ok(msg) => match msg.result {
                        SocketResult::NonEmpty(SocketResultNonEmpty::Header { data }) => {
                            let current_resp = data.value;

                            tracing::info!("wss: new block on {}", self.config.name);
                            let mut mutex_previous_resp = previous_block_header_resp.lock().await;
                            match mutex_previous_resp.as_ref() {
                                Some(mut previous_resp) => {
                                    let proposer_metadata = self
                                        .database
                                        .find_validator_by_hex_addr(&previous_resp.header.proposer_address.clone())
                                        .await
                                        .map_err(|e| format!("block+ error: {e}"))?;

                                    let block_item = BlockItem {
                                        hash: current_resp.header.last_block_id.hash.clone(),
                                        height: previous_resp.header.height.parse::<u64>().map_err(|e| format!("Cannot parse block height, {}: {e}", previous_resp.header.height))?,
                                        timestamp: DateTime::parse_from_rfc3339(&previous_resp.header.time).map(|d| d.timestamp_millis()).map_err(|e| format!("Cannot parse datetime, {}: e", previous_resp.header.time))?,
                                        tx_count: previous_resp.num_txs.parse::<u64>().map_err(|e| format!("Cannot parse Tx count, {}: {e}", previous_resp.num_txs))?,
                                        proposer_logo_url: proposer_metadata.logo_url,
                                        proposer_name: proposer_metadata.name,
                                        proposer_address: proposer_metadata.operator_address,
                                    };

                                    // STORE BLOCKS TO MONGO_DB HERE
                                    // clone.store_new_block(block_item);

                                    *mutex_previous_resp = Some(current_resp);
                                }
                                None => *mutex_previous_resp = Some(current_resp),
                            }
                        }
                        SocketResult::NonEmpty(SocketResultNonEmpty::Tx { events }) => {
                            tracing::info!("wss: new tx on {}", clone.config.name);

                            let tx_item = TransactionItem {
                                amount: events.transfer_amount.get(0).map(|amount| clone._get_amount(amount)).unwrap_or(0.00),
                                fee: clone._get_amount(&events.tx_fee[0]),
                                hash: events.tx_hash[0].clone(),
                                height: events.tx_height[0].parse::<u64>().map_err(|e| format!("Cannot parse tx height {}: {e}", events.tx_height[0]))?,
                                time: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64,
                                result: "Success".to_string(),
                                r#type: events.message_action[0]
                                    .split_once("Msg")
                                    .map(|(_, r)| r)
                                    .unwrap_or(events.message_action[0].split('.').last().unwrap_or("Unknown"))
                                    .to_string(),
                            };

                            // STORE TXS TO MONGO_DB HERE
                            // clone.store_new_tx(tx_item);
                        }
                        SocketResult::Empty {} => (),
                    },
                    Err(error) => tracing::info!("Websocket JSON parse error for {}: {error}", clone.config.name),
                }
            }
        }
        Ok(())
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
