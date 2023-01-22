use std::sync::Arc;

use chrono::DateTime;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::chain::Chain;
use crate::database::BlockForDb;
use crate::events::WsEvent;
use crate::fetch::blocks::Block;

use super::{
    blocks::{BlockHeader, BlockItem},
    transactions::TransactionItem,
};

const SUBSCRIBE_BLOCK: &str = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='NewBlock'"], "id": 0 }"#;
const SUBSCRIBE_HEADER: &str = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='NewBlockHeader'"], "id": 1 }"#;
const SUBSCRIBE_TX: &str = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='Tx'"], "id": 2 }"#;

const AXELAR_SUB_CONFIRM_DEPOSIT_TX: &str = r#"{
    "jsonrpc": "2.0",
    "method": "subscribe",
    "id": "0",
    "params": {
        "query": "tm.event='Tx' AND message.action='ConfirmDeposit' AND axelar.evm.v1beta1.ConfirmDepositStarted.participants CONTAINS 'participants'"
    }
}"#;
const AXELAR_SUB_CONFIRM_ERC20_DEPOSIT_TX: &str = r#"{
    "jsonrpc": "2.0",
    "method": "subscribe",
    "id": "0",
    "params": {
        "query": "tm.event='Tx' AND message.action='ConfirmERC20Deposit' AND axelar.evm.v1beta1.ConfirmDepositStarted.participants CONTAINS 'participants'"
    }
}"#;
const AXELAR_SUB_CONFIRM_TRANSFER_KEY_TX: &str = r#"{
    "jsonrpc": "2.0",
    "method": "subscribe",
    "id": "0",
    "params": {
        "query": "tm.event='Tx' AND message.action='ConfirmTransferKey' AND axelar.evm.v1beta1.ConfirmKeyTransferStarted.participants CONTAINS 'participants'"
    }
}"#;
const AXELAR_SUB_CONFIRM_GATEWAY_TX: &str = r#"{
    "jsonrpc": "2.0",
    "method": "subscribe",
    "id": "0",
    "params": {
        "query": "tm.event='Tx' AND message.action='ConfirmGatewayTx' AND axelar.evm.v1beta1.ConfirmGatewayTxStarted.participants CONTAINS 'participants'"
    }
}"#;

const AXELAR_SUB_VOTE_TX: &str = r#"{
    "jsonrpc": "2.0",
    "method": "subscribe",
    "id": "0",
    "params": {
        "query": "tm.event='Tx' AND axelar.vote.v1beta1.Voted.action CONTAINS 'vote'"
    }
}"#;

impl Chain {
    /// Subscribes to all the events.
    pub async fn subscribe_to_events(&self, tx: Sender<(String, WsEvent)>) -> Result<(), String> {
        // Define the URL.
        let clone = self.clone();
        let url = &clone.config.wss_url;

        // Connect to the `wss://` URL.
        let (ws_stream, _) = connect_async(url).await.map_err(|e| format!("Failed to connect to {url}: {e}"))?;

        // Split the connection into two parts.
        let (mut write, mut read) = ws_stream.split();

        // Subscribe to blocks.
        write
            .send(SUBSCRIBE_BLOCK.into())
            .await
            .map_err(|e| format!("Can't subscribe to blocks for {}: {e}", clone.config.name))?;

        // Subscribe to block headers.
        // write.send(SUBSCRIBE_HEADER.into()).await.map_err(|e| format!("Can't subscribe to block headers for {}: {e}", clone.config.name))?;

        // Subscribe to block txs.
        write
            .send(SUBSCRIBE_TX.into())
            .await
            .map_err(|e| format!("Can't subscribe to txs for {}: {e}", clone.config.name))?;

        // The variable to hold the previous block header response to have block hash value.
        let previous_block_header_resp: Arc<Mutex<Option<NewBlockValue>>> = Arc::new(Mutex::new(None));

        while let Some(msg) = read.next().await {
            // Run the function below for each message received.
            if let Ok(Message::Text(msg)) = msg {
                match serde_json::from_str::<SocketMessage>(&msg) {
                    Ok(msg) => match msg.result {
                        SocketResult::NonEmpty(SocketResultNonEmpty::Tx { events }) => {
                            tracing::info!("wss: new tx on {}", clone.config.name);

                            let tx_item = TransactionItem {
                                amount: events.transfer_amount.get(0).map(|amount| clone._get_amount(amount)).unwrap_or(0.00),
                                fee: clone._get_amount(&events.tx_fee[0]),
                                hash: events.tx_hash[0].clone(),
                                height: events.tx_height[0]
                                    .parse::<u64>()
                                    .map_err(|e| format!("Cannot parse tx height {}: {e}", events.tx_height[0]))?,
                                time: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64,
                                result: "Success".to_string(),
                                r#type: events.message_action[0]
                                    .split_once("Msg")
                                    .map(|(_, r)| r)
                                    .unwrap_or(events.message_action[0].split('.').last().unwrap_or("Unknown"))
                                    .to_string(),
                            };

                            tx.send((self.config.name.clone(), WsEvent::NewTX(tx_item.clone()))).ok();
                            // STORE TXS TO MONGO_DB HERE
                            // clone.store_new_tx(tx_item);
                        }
                        SocketResult::NonEmpty(SocketResultNonEmpty::Block { data }) => {
                            tracing::info!("wss: new block on {}", clone.config.name);
                            let data = data;
                            let current_resp = data.value;

                            let mut mutex_previous_resp = previous_block_header_resp.lock().await;
                            match mutex_previous_resp.as_ref() {
                                Some(mut previous_resp) => {
                                    let proposer_metadata = self
                                        .database
                                        .find_validator_by_hex_addr(&previous_resp.block.header.proposer_address.clone())
                                        .await
                                        .map_err(|e| format!("block+ error: {e}"))?;

                                    let prev_block_data = &previous_resp.block;
                                    let current_block_data = &current_resp.block;

                                    let block_item = BlockForDb {
                                        hash: current_block_data.header.last_block_id.hash.clone(),
                                        height: prev_block_data
                                            .header
                                            .height
                                            .parse::<u64>()
                                            .map_err(|e| format!("Cannot parse block height, {}: {e}", prev_block_data.header.height))?,
                                        timestamp: DateTime::parse_from_rfc3339(&prev_block_data.header.time)
                                            .map(|d| d.timestamp_millis())
                                            .map_err(|e| format!("Cannot parse datetime, {}: e", prev_block_data.header.time))?,
                                        tx_count: prev_block_data.data.txs.len() as u64,
                                        proposer_logo_url: proposer_metadata.logo_url,
                                        proposer_name: proposer_metadata.name,
                                        proposer_address: proposer_metadata.operator_address,
                                        signatures: current_block_data.last_commit.signatures.clone(),
                                    };

                                    tx.send((self.config.name.clone(), WsEvent::NewBLock(block_item.clone()))).ok();

                                    if let Err(e) = self.database.upsert_block(block_item).await {
                                        tracing::error!("Error saving block to the database: {e} ")
                                    }

                                    *mutex_previous_resp = Some(current_resp);
                                }
                                None => *mutex_previous_resp = Some(current_resp),
                            }
                        }
                        SocketResult::Empty {} => (),
                        _ => {}
                    },
                    Err(error) => tracing::info!("Websocket JSON parse error for {}: {error}", clone.config.name),
                }
            }
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SocketMessage {
    pub result: SocketResult,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum SocketResult {
    NonEmpty(SocketResultNonEmpty),
    Empty {},
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "query")]
pub enum SocketResultNonEmpty {
    #[serde(rename = "tm.event='Tx'")]
    Tx { events: TxEvents },
    #[serde(rename = "tm.event='NewBlockHeader'")]
    Header { data: NewBlockHeaderData },
    #[serde(rename = "tm.event='NewBlock'")]
    Block { data: NewBlockData },
}

#[derive(Deserialize)]
pub struct NewBlockHeaderMessage {
    pub data: Option<NewBlockHeaderData>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NewBlockHeaderData {
    pub value: NewBlockHeaderValue,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NewBlockData {
    pub value: NewBlockValue,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NewBlockHeaderValue {
    pub header: BlockHeader,
    pub num_txs: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NewBlockValue {
    pub block: Block,
}

#[derive(Deserialize)]
pub struct TxMessage {
    pub events: Option<TxEvents>,
}

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
pub struct ConfirmDepositStartedEvents {
    #[serde(rename = "tx.height")]
    pub tx_height: [String; 1],
    #[serde(rename = "axelar.evm.v1beta1.ConfirmDepositStarted.chain")]
    pub chain: [String; 1],
    #[serde(rename = "axelar.evm.v1beta1.ConfirmDepositStarted.participants")]
    pub participants: [String; 1],
    #[serde(rename = "axelar.evm.v1beta1.ConfirmDepositStarted.tx_id")]
    pub tx_id: [String; 1],
    #[serde(rename = "message.action")]
    pub message_action: [String; 1],
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConfirmGatewayTxStartedEvents {
    #[serde(rename = "tx.height")]
    pub tx_height: [String; 1],
    #[serde(rename = "axelar.evm.v1beta1.ConfirmGatewayTxStarted.chain")]
    pub chain: [String; 1],
    #[serde(rename = "axelar.evm.v1beta1.ConfirmGatewayTxStarted.participants")]
    pub participants: [String; 1],
    #[serde(rename = "axelar.evm.v1beta1.ConfirmGatewayTxStarted.tx_id")]
    pub tx_id: [String; 1],
    #[serde(rename = "message.action")]
    pub message_action: [String; 1],
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConfirmKeyTransferStartedEvents {
    #[serde(rename = "tx.height")]
    pub tx_height: [String; 1],
    #[serde(rename = "axelar.evm.v1beta1.ConfirmKeyTransferStarted.chain")]
    pub chain: [String; 1],
    #[serde(rename = "axelar.evm.v1beta1.ConfirmKeyTransferStarted.participants")]
    pub participants: [String; 1],
    #[serde(rename = "axelar.evm.v1beta1.ConfirmKeyTransferStarted.tx_id")]
    pub tx_id: [String; 1],
    #[serde(rename = "message.action")]
    pub message_action: [String; 1],
}

impl SocketResultNonEmpty {
    pub async fn get_evm_poll_item(&self, chain: &Chain) -> Result<EvmPollItem, TNRAppError> {
        let tx_height = self.get_tx_height();
        let chain_name = self.get_chain_name();
        let action_name = self.get_action_name();
        let participants_raw = self.get_participants_raw();
        let tx_id = self.get_tx_id();

        Ok(EvmPollItem::new(&EvmPollItemEventParams {
            chain: chain_name,
            tx_height,
            action_name,
            participants_raw,
            tx_id,
        }, &chain).await.unwrap()
        )
    }

    fn get_tx_height(&self) -> u64 {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => { events.tx_height.get(0).unwrap().parse::<u64>().unwrap() }
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => { events.tx_height.get(0).unwrap().parse::<u64>().unwrap() }
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => { events.tx_height.get(0).unwrap().parse::<u64>().unwrap() }
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => { events.tx_height.get(0).unwrap().parse::<u64>().unwrap() }
            _ => 0,
        }
    }
    fn get_chain_name(&self) -> String {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => { events.chain.get(0).unwrap().to_string() }
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => { events.chain.get(0).unwrap().to_string() }
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => { events.chain.get(0).unwrap().to_string() }
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => { events.chain.get(0).unwrap().to_string() }
            _ => String::from(""),
        }
    }
    fn get_action_name(&self) -> String {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => { events.message_action.get(0).unwrap().to_string() }
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => { events.message_action.get(0).unwrap().to_string() }
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => { events.message_action.get(0).unwrap().to_string() }
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => { events.message_action.get(0).unwrap().to_string() }
            _ => String::from(""),
        }
    }
    fn get_participants_raw(&self) -> String {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => { events.participants.get(0).unwrap().to_string() }
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => { events.participants.get(0).unwrap().to_string() }
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => { events.participants.get(0).unwrap().to_string() }
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => { events.participants.get(0).unwrap().to_string() }
            _ => String::from(""),
        }
    }
    fn get_tx_id(&self) -> String {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => { events.tx_id.get(0).unwrap().to_string() }
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => { events.tx_id.get(0).unwrap().to_string() }
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => { events.tx_id.get(0).unwrap().to_string() }
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => { events.tx_id.get(0).unwrap().to_string() }
            _ => String::from(""),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct VotedTxEvents {
    #[serde(rename = "tx.height")]
    pub tx_height: [String; 1],
    #[serde(rename = "tx.hash")]
    pub tx_hash: [String; 1],
}

#[derive(Deserialize, Debug, Clone)]
pub struct PoolParticipants {
    pub poll_id: String,

    #[serde(rename = "participants")]
    pub participants_operator_address: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EvmPollItem {
    pub tx_height: u64,
    pub action: String,
    pub poll_id: String,
    pub chain_name: String,
    pub status: String,
    pub evm_tx_id: String,
    pub participants_operator_address: Vec<String>,
    pub time: u64,
}

impl EvmPollItem {
    async fn new(params: &EvmPollItemEventParams, chain: &Chain) -> Result<Self, TNRAppError> {
        let rmv_backslash_participants = str::replace(&params.participants_raw, r#"\"#, "");
        let poll_info = match serde_json::from_str::<PoolParticipants>(&rmv_backslash_participants) {
            Ok(res) => res,
            Err(e) => { return Err(TNRAppError::from(format!("error {}", e))); }
        };

        let tx_height = params.tx_height;
        let time = match chain.get_block_by_height(Some(tx_height)).await {
            Ok(res) => res.value.time as u64,
            Err(_) => { 0 }
        };

        let chain_name = str::replace(&params.chain, r#"\"#, "");
        let evm_tx_id = chain.convert_to_evm_hex(&params.tx_id).unwrap();
        let action = String::from(&params.action_name);

        Ok(Self {
            poll_id: poll_info.poll_id.clone(),
            status: String::from("Pending"),
            participants_operator_address: poll_info.participants_operator_address.clone(),
            action,
            evm_tx_id,
            chain_name,
            time,
            tx_height,
        })
    }
}

struct EvmPollItemEventParams {
    pub tx_height: u64,
    pub chain: String,
    pub action_name: String,
    pub participants_raw: String,
    pub tx_id: String,
}

impl From<EvmPollItem> for EvmPollForDb {
    fn from(value: EvmPollItem) -> Self {
        let participants: Vec<EvmPollParticipantForDb> = value.participants_operator_address.into_iter().map(|address| { EvmPollParticipantForDb::from(address) }).collect();

        EvmPollForDb {
            timestamp: value.time.clone(),
            tx_height: value.tx_height.clone(),
            poll_id: value.poll_id.clone(),
            action: value.action.clone(),
            status: value.status.clone(),
            evm_tx_id: value.evm_tx_id.clone(),
            chain_name: value.chain_name.clone(),
            participants,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum EvmPollVote {
    UN_SUBMIT,
    YES,
    NO,
}

impl EvmPollVote {
    pub fn to_db_str(&self) -> String {
        match self {
            EvmPollVote::UN_SUBMIT => format!("UN_SUBMIT"),
            EvmPollVote::YES => format!("YES"),
            EvmPollVote::NO => format!("NO")
        }
    }
}