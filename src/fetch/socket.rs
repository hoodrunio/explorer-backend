use std::sync::Arc;

use chrono::{DateTime, Utc};
use futures::future::join_all;
use futures::{SinkExt, StreamExt};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use tokio::try_join;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::chain::Chain;
use crate::database::{
    BlockForDb, DatabaseTR, EvmPollForDb, EvmPollParticipantForDb, HeartbeatForDb, HeartbeatRawForDb, ProposalVoteForDb, ProposalVoteOptionForDb,
};
use crate::events::WsEvent;
use crate::fetch::blocks::{Block, CosmosEvent, ResultBeginBlock, ResultEndBlock};
use crate::fetch::evm::PollStatus;
use crate::fetch::heartbeats::HeartbeatStatus;
use crate::fetch::transactions::InternalTransaction;
use crate::routes::TNRAppError;

use super::evm_socket_handler::EvmSocketHandler;
use super::{blocks::BlockHeader, transactions::TransactionItem};

const SUBSCRIBE_BLOCK: &str = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='NewBlock'"], "id": 0 }"#;
// const SUBSCRIBE_HEADER: &str = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='NewBlockHeader'"], "id": 1 }"#;
const SUBSCRIBE_TX: &str = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='Tx'"], "id": 2 }"#;
const SUBSCRIBE_PROPOSAL_VOTE_TX: &str =
    r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='Tx' AND message.action CONTAINS 'MsgVote'"], "id": 2 }"#;

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
        let chain_name = clone.config.name.clone();
        let url = &clone.config.wss_url;

        // Connect to the `wss://` URL.
        let (ws_stream, _) = connect_async(url).await.map_err(|e| format!("Failed to connect to {url}: {e}"))?;

        // Split the connection into two parts.
        let (mut write, mut read) = ws_stream.split();

        let events = vec![
            SUBSCRIBE_BLOCK, // SUBSCRIBE_HEADER
            SUBSCRIBE_TX,
        ];

        // Subscribe to txs which are related with chain.
        for event in events {
            write
                .send(event.into())
                .await
                .map_err(|e| format!("Can't subscribe to confirm {} for {}: {e}", event, chain_name))?;
        }

        // The variable to hold the previous block header response to have block hash value.
        let previous_block_header_resp: Arc<Mutex<Option<NewBlockValue>>> = Arc::new(Mutex::new(None));

        while let Some(msg) = read.next().await {
            // Run the function below for each message received.
            if let Ok(Message::Text(msg)) = msg {
                match serde_json::from_str::<SocketMessage>(&msg) {
                    Ok(msg) => match msg.result {
                        SocketResult::NonEmpty(SocketResultNonEmpty::Tx { events }) => {
                            tracing::info!("wss: new tx on {}", clone.config.name);
                            let tx_fee_denom = events.tx_fee[0].clone();

                            let tx_item = TransactionItem {
                                amount: clone
                                    .string_amount_parser(
                                        events
                                            .transfer_amount
                                            .iter()
                                            .filter(|str| str.to_string() != tx_fee_denom)
                                            .map(String::from)
                                            .collect::<Vec<String>>()
                                            .get(0)
                                            .map(|amount| amount.replace(clone.config.main_denom.as_str(), ""))
                                            .unwrap_or(String::from("0.00"))
                                            .clone(),
                                        None,
                                    )
                                    .await?,
                                fee: clone
                                    .string_amount_parser(tx_fee_denom.replace(clone.config.main_denom.as_str(), "").clone(), None)
                                    .await?,
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
                            let _ = self.database.add_transaction(tx_item.into()).await;
                        }
                        SocketResult::NonEmpty(SocketResultNonEmpty::Block { data }) => {
                            tracing::info!("wss: new block on {}", clone.config.name);
                            let data = data;
                            let current_resp = data.value;

                            let mut mutex_previous_resp = previous_block_header_resp.lock().await;
                            match mutex_previous_resp.as_ref() {
                                Some(previous_resp) => {
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
                                            .map_err(|_e| format!("Cannot parse datetime, {}: e", prev_block_data.header.time))?,
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
            } else if let Err(e) = msg {
                return Err(format!("Websocket error for {}: {}", clone.config.name, e));
            }
        }

        Ok(())
    }

    pub async fn sub_proposal_events(&self) -> Result<(), String> {
        let clone = self.clone();
        let url = &clone.config.wss_url;
        // Connect to the `wss://` URL.
        let (ws_stream, _) = connect_async(url).await.map_err(|e| format!("Failed to connect to {url}: {e}"))?;
        let (mut write, mut read) = ws_stream.split();

        write
            .send(SUBSCRIBE_PROPOSAL_VOTE_TX.into())
            .await
            .map_err(|e| format!("Can't subscribe to blocks for {}: {e}", clone.config.name))?;

        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(msg)) = msg {
                match serde_json::from_str::<SocketMessage>(&msg) {
                    Ok(msg) => match msg.result {
                        SocketResult::NonEmpty(SocketResultNonEmpty::ProposalVoteTx { events }) => {
                            let proposal_id = events.proposal_id[0].clone();
                            let proposal_vote_option =
                                match serde_json::from_str::<ProposalVoteOption>(events.vote_option[0].replace(r#"\"#, "").as_str()) {
                                    Ok(option) => option,
                                    Err(e) => {
                                        tracing::error!("Error parsing vote option: {e} proposal id {proposal_id}");
                                        continue;
                                    }
                                };

                            let voter = match events.voter.get(0) {
                                Some(voter) => String::from(voter),
                                None => {
                                    tracing::error!("Error parsing voter proposal id {proposal_id}");
                                    continue;
                                }
                            };

                            let proposal_vote_option_db = ProposalVoteOptionForDb {
                                option: proposal_vote_option.option,
                                weight: proposal_vote_option.weight.parse::<f32>().unwrap_or(0.0),
                            };

                            let proposal_vote = ProposalVoteForDb {
                                proposal_id,
                                voter,
                                option: proposal_vote_option_db,
                                tx_hash: events.tx_hash[0].clone(),
                                timestamp: Utc::now().timestamp_millis(),
                            };

                            let _ = self.database.add_propsal_vote(proposal_vote).await;
                        }
                        SocketResult::Empty {} => {
                            tracing::info!("Websocket empty response for {}", clone.config.name);
                        }
                        _ => (),
                    },
                    Err(error) => {
                        tracing::info!("Websocket JSON parse error for {}: {error}", clone.config.name);
                    }
                }
            }
        }

        Ok(())
    }
    pub async fn sub_axelar_events(axelar: Chain, tx: Sender<(String, WsEvent)>) -> Result<(), String> {
        let poll = axelar.sub_for_axelar_evm_polls(tx.clone());
        let heartbeats = axelar.sub_for_axelar_heartbeats();
        match try_join!(poll, heartbeats) {
            Ok(..) => {}
            Err(e) => {
                return Err(e.message.unwrap_or(String::from("")));
            }
        };

        Ok(())
    }

    async fn sub_for_axelar_evm_polls(&self, ws_tx: Sender<(String, WsEvent)>) -> Result<(), TNRAppError> {
        let ws_url = self.config.wss_url.clone();
        let chain_name = self.config.name.clone();

        let (ws_stream, _) = connect_async(ws_url.clone())
            .await
            .map_err(|_| TNRAppError::from("Can not connect".to_string()))?;

        // Split the connection into two parts.
        let (mut write, mut read) = ws_stream.split();

        let events = vec![
            AXELAR_SUB_CONFIRM_DEPOSIT_TX,
            AXELAR_SUB_CONFIRM_ERC20_DEPOSIT_TX,
            AXELAR_SUB_CONFIRM_TRANSFER_KEY_TX,
            AXELAR_SUB_CONFIRM_GATEWAY_TX,
            SUBSCRIBE_BLOCK,
            AXELAR_SUB_VOTE_TX,
        ];

        // Subscribe to txs which are related evm polls.
        for event in events {
            write
                .send(event.into())
                .await
                .map_err(|e| format!("Can't subscribe to confirm {} for {}: {e}", event, chain_name,))?;
        }

        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text_msg)) = msg {
                match serde_json::from_str::<SocketMessage>(&text_msg) {
                    Ok(socket_msg) => {
                        let handler = EvmSocketHandler::new(self.clone(), ws_tx.clone());

                        match socket_msg.result {
                            SocketResult::NonEmpty(SocketResultNonEmpty::Block { data }) => {
                                tokio::spawn(async move {
                                    handler.handle_evm_poll(data).await;
                                });
                            }
                            SocketResult::NonEmpty(SocketResultNonEmpty::VotedTx { events: voted_tx }) => {
                                tokio::spawn(async move {
                                    handler.handle_evm_poll_status(voted_tx).await;
                                });
                            }
                            SocketResult::NonEmpty(evm_poll_msg) => {
                                tokio::spawn(async move {
                                    handler.handle_evm_poll_any_message(evm_poll_msg).await;
                                });
                            }
                            SocketResult::Empty { .. } => {}
                        };
                    }
                    Err(error) => {
                        tracing::error!("Websocket JSON parse error for {}: {error}", chain_name);
                    }
                }
            };
        }

        Ok(())
    }
    async fn sub_for_axelar_heartbeats(&self) -> Result<(), TNRAppError> {
        let ws_url = self.config.wss_url.clone();
        let chain_name = self.config.name.clone();

        let (ws_stream, _) = connect_async(ws_url.clone())
            .await
            .map_err(|_| TNRAppError::from("Can not connect".to_string()))?;

        // Split the connection into two parts.
        let (mut write, mut read) = ws_stream.split();

        // Subscribe to txs which are for heartbeats.
        write
            .send(SUBSCRIBE_BLOCK.into())
            .await
            .map_err(|e| format!("Can't subscribe to confirm AXELAR SUB HEARTBEAT TX for {}: {e}", chain_name))?;

        let mut heartbeat_begin_height: u64 = 0;
        let heartbeat_block_check_range = 6;
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text_msg)) = msg {
                match serde_json::from_str::<SocketMessage>(&text_msg) {
                    Ok(socket_msg) => {
                        match socket_msg.result {
                            SocketResult::NonEmpty(SocketResultNonEmpty::Block { data }) => {
                                let current_height = data.value.block.header.height.parse::<u64>().unwrap_or(0);

                                if data.value.result_end_block.is_heartbeat_begin() {
                                    heartbeat_begin_height = current_height;

                                    if let Ok(res) = self
                                        .database
                                        .find_validators(Some(doc! {"$match":{"voter_address":{"$exists":true}}}))
                                        .await
                                    {
                                        let period_height = heartbeat_begin_height + 1;
                                        let mut initial_period_heartbeats = vec![];
                                        for validator in res.into_iter() {
                                            match validator.voter_address.clone() {
                                                None => {}
                                                Some(sender_address) => {
                                                    let generated_id = self.generate_heartbeat_id(sender_address.clone(), period_height);
                                                    let heartbeat = HeartbeatForDb {
                                                        heartbeat_raw: None,
                                                        period_height,
                                                        status: HeartbeatStatus::Fail,
                                                        sender: sender_address.clone(),
                                                        id: generated_id,
                                                    };
                                                    initial_period_heartbeats.push(heartbeat);
                                                }
                                            };
                                        }

                                        match self.database.add_heartbeat_many(initial_period_heartbeats).await {
                                            Ok(_) => {
                                                tracing::info!("Current period initial heartbeats inserted");
                                            }
                                            Err(_) => {
                                                tracing::info!("Current period initial heartbeats could not inserted");
                                            }
                                        };
                                    };
                                };

                                if heartbeat_begin_height + heartbeat_block_check_range >= current_height {
                                    let block_result = self.get_block_result_by_height(Some(current_height)).await;

                                    if let Ok(block_result) = block_result {
                                        let mut block_res_txs_handler_futures = vec![];
                                        for block_res_tx_res in block_result.value.txs_results {
                                            let sender_address = block_res_tx_res.get_sender_address().unwrap_or(String::from("")).clone();
                                            block_res_txs_handler_futures.push(async move {
                                                let heartbeat_info = self.get_axelar_sender_heartbeat_info(&sender_address, current_height).await;
                                                if let Ok(info) = heartbeat_info {
                                                    let period_height = heartbeat_begin_height + 1;
                                                    let generated_id = self.generate_heartbeat_id(info.sender.clone(), period_height);
                                                    let sender = info.sender.clone();
                                                    let heartbeat_raw = HeartbeatRawForDb {
                                                        height: current_height,
                                                        tx_hash: info.tx_hash.clone(),
                                                        timestamp: info.timestamp as u64,
                                                        signatures: info.signatures.clone(),
                                                        key_ids: info.key_ids.clone(),
                                                        sender: sender.clone(),
                                                        period_height,
                                                    };

                                                    let db_heartbeat = HeartbeatForDb {
                                                        id: generated_id.clone(),
                                                        status: HeartbeatStatus::Success,
                                                        heartbeat_raw: Some(heartbeat_raw),
                                                        sender,
                                                        period_height,
                                                    };
                                                    match self.database.upsert_heartbeat(db_heartbeat).await {
                                                        Ok(_) => {
                                                            tracing::info!("Successfully inserted heartbeat id {}", &generated_id)
                                                        }
                                                        Err(_) => {
                                                            tracing::error!("Could not inserted heartbeat id {}", &generated_id)
                                                        }
                                                    };
                                                };
                                            });
                                        }

                                        join_all(block_res_txs_handler_futures).await;
                                    }
                                }
                            }
                            SocketResult::Empty { .. } => {}
                            _ => {}
                        };
                    }
                    Err(error) => {
                        tracing::error!("Websocket JSON parse error for {}: {error}", chain_name);
                    }
                }
            };
        }
        Ok(())
    }

    pub fn convert_to_evm_hex(&self, string_byte_array: &String) -> Option<String> {
        let mut result: Option<String> = None;

        if string_byte_array.is_empty() {
            return result;
        };

        let mut prefix = String::from("0x");
        match serde_json::from_str::<Vec<u8>>(string_byte_array) {
            Ok(res) => {
                let hex_res = hex::encode(res);
                prefix.push_str(hex_res.as_str());
                result = Some(prefix);
            }
            Err(_) => {
                tracing::error!("Error while evm tx id byte array converting to hex");
            }
        }

        result
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
    #[serde(rename = "tm.event='Tx' AND message.action CONTAINS 'MsgVote'")]
    ProposalVoteTx { events: ProposalVoteEvents },
    #[serde(
        rename = "tm.event='Tx' AND message.action='ConfirmERC20Deposit' AND axelar.evm.v1beta1.ConfirmDepositStarted.participants CONTAINS 'participants'"
    )]
    ConfirmERC20DepositStartedTx { events: ConfirmDepositStartedEvents },

    #[serde(
        rename = "tm.event='Tx' AND message.action='ConfirmDeposit' AND axelar.evm.v1beta1.ConfirmDepositStarted.participants CONTAINS 'participants'"
    )]
    ConfirmDepositStartedTx { events: ConfirmDepositStartedEvents },

    #[serde(
        rename = "tm.event='Tx' AND message.action='ConfirmGatewayTx' AND axelar.evm.v1beta1.ConfirmGatewayTxStarted.participants CONTAINS 'participants'"
    )]
    ConfirmGatewayTxStartedTx { events: ConfirmGatewayTxStartedEvents },

    #[serde(
        rename = "tm.event='Tx' AND message.action='ConfirmTransferKey' AND axelar.evm.v1beta1.ConfirmKeyTransferStarted.participants CONTAINS 'participants'"
    )]
    ConfirmKeyTransferStartedTx { events: ConfirmKeyTransferStartedEvents },

    #[serde(rename = "tm.event='Tx' AND axelar.vote.v1beta1.Voted.action CONTAINS 'vote'")]
    VotedTx { events: VotedTxEvents },
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
    pub result_begin_block: ResultBeginBlock,
    pub result_end_block: ResultEndBlock,
}

impl NewBlockValue {
    fn extract_evm_poll_info(&self, event: &CosmosEvent, status: PollStatus) -> AxelarCompletedPoll {
        let mut poll_id: String = String::from("");
        let mut chain: String = String::from("");
        let mut tx_id: String = String::from("");

        for attribute in event.attributes.clone() {
            if attribute.key == "poll_id" {
                poll_id = attribute.value.clone().replace('"', "");
            };
            if attribute.key == "chain" {
                chain = attribute.value.clone().replace('"', "");
            };
            if attribute.key == "tx_id" {
                tx_id = attribute.value.clone();
            };
        }

        AxelarCompletedPoll {
            chain,
            poll_id,
            tx_id,
            poll_status: status,
        }
    }

    pub fn extract_evm_poll_completed_events(&self) -> Option<Vec<AxelarCompletedPoll>> {
        let end_block_events = &self.result_end_block.events;
        if end_block_events.is_empty() {
            return None;
        };
        let mut poll_completed_axelar_polls: Vec<AxelarCompletedPoll> = vec![];

        for event in end_block_events {
            if event.r#type == "axelar.evm.v1beta1.PollCompleted" {
                let completed_axelar_poll_info = self.extract_evm_poll_info(event, PollStatus::Completed);
                let ignore = poll_completed_axelar_polls
                    .clone()
                    .into_iter()
                    .any(|poll| poll.poll_id == completed_axelar_poll_info.poll_id);

                if !ignore {
                    poll_completed_axelar_polls.push(completed_axelar_poll_info);
                };
            };
            if event.r#type == "axelar.evm.v1beta1.NoEventsConfirmed" {
                let axelar_poll_info = self.extract_evm_poll_info(event, PollStatus::Failed);
                poll_completed_axelar_polls.push(axelar_poll_info);
            };
        }

        if poll_completed_axelar_polls.is_empty() {
            return None;
        }

        Some(poll_completed_axelar_polls)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct AxelarCompletedPoll {
    pub chain: String,
    pub poll_id: String,
    pub tx_id: String,
    pub poll_status: PollStatus,
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
    #[serde(rename = "axelar.evm.v1beta1.ConfirmDepositStarted.deposit_address")]
    pub evm_deposit_address: [String; 1],
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

#[derive(Deserialize, Debug, Clone)]
pub struct ProposalVoteEvents {
    //Comes as string json with possible backslash
    #[serde(rename = "proposal_vote.option")]
    pub vote_option: [String; 1],
    #[serde(rename = "message.sender")]
    pub voter: Vec<String>,
    #[serde(rename = "proposal_vote.proposal_id")]
    pub proposal_id: [String; 1],
    #[serde(rename = "tx.hash")]
    pub tx_hash: [String; 1],
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProposalVoteOption {
    pub weight: String,
    pub option: u8,
}

impl SocketResultNonEmpty {
    pub async fn get_evm_poll_item(&self, chain: &Chain) -> Result<EvmPollItem, TNRAppError> {
        let tx_height = self.get_tx_height();
        let chain_name = self.get_chain_name();
        let action_name = self.get_action_name();
        let participants_raw = self.get_participants_raw();
        let tx_id = self.get_tx_id();
        let deposit_address = self.get_deposit_address();

        let evm_poll_item = match EvmPollItem::new(
            &EvmPollItemEventParams {
                chain: chain_name,
                deposit_address,
                tx_height,
                action_name,
                participants_raw,
                tx_id,
            },
            chain,
        )
        .await
        {
            Ok(res) => res,
            Err(e) => {
                return Err(e);
            }
        };

        Ok(evm_poll_item)
    }

    fn get_tx_height(&self) -> u64 {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => {
                events.tx_height.get(0).unwrap_or(&String::from("0")).parse::<u64>().unwrap_or(0)
            }
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => {
                events.tx_height.get(0).unwrap_or(&String::from("0")).parse::<u64>().unwrap_or(0)
            }
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => {
                events.tx_height.get(0).unwrap_or(&String::from("0")).parse::<u64>().unwrap_or(0)
            }
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => {
                events.tx_height.get(0).unwrap_or(&String::from("0")).parse::<u64>().unwrap_or(0)
            }
            _ => 0,
        }
    }
    fn get_chain_name(&self) -> String {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => events.chain.get(0).unwrap_or(&String::from("")).to_string(),
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => events.chain.get(0).unwrap_or(&String::from("")).to_string(),
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => events.chain.get(0).unwrap_or(&String::from("")).to_string(),
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => events.chain.get(0).unwrap_or(&String::from("")).to_string(),
            _ => String::from(""),
        }
    }
    fn get_action_name(&self) -> String {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => events.message_action.get(0).unwrap_or(&String::from("")).to_string(),
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => events.message_action.get(0).unwrap_or(&String::from("")).to_string(),
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => events.message_action.get(0).unwrap_or(&String::from("")).to_string(),
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => events.message_action.get(0).unwrap_or(&String::from("")).to_string(),
            _ => String::from(""),
        }
    }
    fn get_participants_raw(&self) -> String {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => events.participants.get(0).unwrap_or(&String::from("")).to_string(),
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => events.participants.get(0).unwrap_or(&String::from("")).to_string(),
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => events.participants.get(0).unwrap_or(&String::from("")).to_string(),
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => events.participants.get(0).unwrap_or(&String::from("")).to_string(),
            _ => String::from(""),
        }
    }
    fn get_tx_id(&self) -> String {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => events.tx_id.get(0).unwrap_or(&String::from("")).to_string(),
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => events.tx_id.get(0).unwrap_or(&String::from("")).to_string(),
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => events.tx_id.get(0).unwrap_or(&String::from("")).to_string(),
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => events.tx_id.get(0).unwrap_or(&String::from("")).to_string(),
            _ => String::from(""),
        }
    }

    fn get_deposit_address(&self) -> String {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => {
                events.evm_deposit_address.get(0).unwrap_or(&String::from("")).to_string()
            }
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => events.evm_deposit_address.get(0).unwrap_or(&String::from("")).to_string(),
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events: _ } => String::from(""),
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events: _ } => String::from(""),
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
    #[serde(rename = "axelar.vote.v1beta1.Voted.state")]
    pub poll_state: [String; 1],
}

impl VotedTxEvents {
    pub async fn fetch_tx(&self, chain: &Chain) -> Result<InternalTransaction, TNRAppError> {
        let tx_hash = self.get_tx_hash();

        let internal_tx = match chain.get_tx_by_hash(&tx_hash).await {
            Ok(res) => res.value,
            Err(e) => {
                tracing::error!("tx could not fetched retrying 1 {}", e);
                match chain.get_tx_by_hash(&tx_hash).await {
                    Ok(res) => res.value,
                    Err(e) => {
                        tracing::error!("tx could not fetched retrying 2  {}", e);
                        match chain.get_tx_by_hash(&tx_hash).await {
                            Ok(res) => res.value,
                            Err(e) => {
                                tracing::error!("tx could not fetched  {}", e);
                                return Err(TNRAppError::from(e));
                            }
                        }
                    }
                }
            }
        };

        Ok(internal_tx)
    }

    pub fn get_tx_hash(&self) -> String {
        self.tx_hash.get(0).unwrap_or(&String::from("")).to_string()
    }
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
    pub evm_deposit_address: String,
    pub participants_operator_address: Vec<String>,
    pub time: u64,
}

impl EvmPollItem {
    async fn new(params: &EvmPollItemEventParams, chain: &Chain) -> Result<Self, TNRAppError> {
        let rmv_backslash_participants = str::replace(&params.participants_raw, r#"\"#, "");
        let poll_info = match serde_json::from_str::<PoolParticipants>(&rmv_backslash_participants) {
            Ok(res) => res,
            Err(e) => {
                return Err(TNRAppError::from(format!("error {}", e)));
            }
        };

        let tx_height = params.tx_height;
        let time = match chain.get_block_by_height(Some(tx_height)).await {
            Ok(res) => res.value.time as u64,
            Err(_) => 0,
        };

        let chain_name = str::replace(&params.chain, "\"", "");
        let evm_tx_id = chain.convert_to_evm_hex(&params.tx_id).unwrap_or(String::from(""));
        let evm_deposit_address = chain.convert_to_evm_hex(&params.deposit_address).unwrap_or(String::from(""));
        let action = String::from(&params.action_name);

        Ok(Self {
            poll_id: poll_info.poll_id.clone(),
            status: String::from("Pending"),
            participants_operator_address: poll_info.participants_operator_address.clone(),
            evm_deposit_address,
            action,
            evm_tx_id,
            chain_name,
            time,
            tx_height,
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
struct EvmPollItemEventParams {
    pub tx_height: u64,
    pub chain: String,
    pub action_name: String,
    pub participants_raw: String,
    pub tx_id: String,
    pub deposit_address: String,
}

impl From<EvmPollItem> for EvmPollForDb {
    fn from(value: EvmPollItem) -> Self {
        EvmPollForDb {
            timestamp: value.time,
            tx_height: value.tx_height,
            poll_id: value.poll_id.clone(),
            action: value.action.clone(),
            status: value.status.clone(),
            evm_tx_id: value.evm_tx_id.clone(),
            chain_name: value.chain_name.clone(),
            evm_deposit_address: value.evm_deposit_address,
        }
    }
}

impl EvmPollItem {
    pub async fn upsert_participants(&self, db: &DatabaseTR) -> Result<(), String> {
        let participants: Vec<EvmPollParticipantForDb> = self
            .participants_operator_address
            .iter()
            .map(|address| EvmPollParticipantForDb::from_info(address.clone(), self.poll_id.clone(), self.chain_name.clone()))
            .collect();

        let mut db_jobs = vec![];
        for participant in participants {
            db_jobs.push(async move { db.upsert_evm_poll_participant(participant).await });
        }
        join_all(db_jobs).await;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum EvmPollVote {
    UnSubmit,
    Yes,
    No,
}
