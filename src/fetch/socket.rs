use std::sync::Arc;

use chrono::DateTime;
use futures::{SinkExt, StreamExt};
use futures::future::join_all;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use tokio::try_join;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::chain::Chain;
use crate::database::{BlockForDb, EvmPollForDb, EvmPollParticipantForDb, HeartbeatForDb, HeartbeatRawForDb};
use crate::events::WsEvent;
use crate::fetch::blocks::{Block, ResultBeginBlock, ResultEndBlock};
use crate::fetch::evm::PollStatus;
use crate::fetch::heartbeats::HeartbeatStatus;
use crate::fetch::transactions::{AxelarKnownVote, AxelarVote, InnerMessage, InnerMessageKnown, InternalTransaction, InternalTransactionContent, InternalTransactionContentKnowns};
use crate::routes::TNRAppError;

use super::{
    blocks::BlockHeader,
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
            }
        }
        Ok(())
    }

    pub async fn sub_axelar_events(axelar: Chain, tx: Sender<(String, WsEvent)>) -> Result<(), String> {
        let poll = axelar.sub_for_axelar_evm_polls(tx.clone());
        let vote = axelar.sub_for_axelar_evm_poll_votes(tx.clone());
        let heartbeats = axelar.sub_for_axelar_heartbeats();
        match try_join!(poll,vote,heartbeats) {
            Ok(..) => {}
            Err(e) => { return Err(e.message.unwrap_or(String::from(""))); }
        };

        Ok(())
    }

    async fn sub_for_axelar_evm_polls(&self, tx: Sender<(String, WsEvent)>) -> Result<(), TNRAppError> {
        let ws_url = self.config.wss_url.clone();
        let chain_name = self.config.name.clone();


        loop {
            let (ws_stream, _) = connect_async(ws_url.clone()).await.map_err(|_| TNRAppError::from("Can not connect".to_string()))?;

            // Split the connection into two parts.
            let (mut write, mut read) = ws_stream.split();

            // Subscribe to txs which are related evm polls.
            write.send(AXELAR_SUB_CONFIRM_DEPOSIT_TX.into()).await.map_err(|e| format!("Can't subscribe to confirm AXELAR CONFIRM DEPOSIT TX for {}: {e}", chain_name))?;
            write.send(AXELAR_SUB_CONFIRM_ERC20_DEPOSIT_TX.into()).await.map_err(|e| format!("Can't subscribe to AXELAR CONFIRM ERC20_DEPOSIT TX for {}: {e}", chain_name))?;
            write.send(AXELAR_SUB_CONFIRM_TRANSFER_KEY_TX.into()).await.map_err(|e| format!("Can't subscribe to AXELAR CONFIRM TRANSFER_KEY TX for {}: {e}", chain_name))?;
            write.send(AXELAR_SUB_CONFIRM_GATEWAY_TX.into()).await.map_err(|e| format!("Can't subscribe to AXELAR CONFIRM GATEWAY TX for {}: {e}", chain_name))?;

            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text_msg)) = msg {
                    match serde_json::from_str::<SocketMessage>(&text_msg) {
                        Ok(socket_msg) => {
                            match socket_msg.result {
                                SocketResult::NonEmpty(evm_poll_msg) => {
                                    let evm_poll_item = match evm_poll_msg.get_evm_poll_item(&self).await {
                                        Ok(res) => res,
                                        Err(e) => {
                                            tracing::error!("Could not get evm poll item {}",e);
                                            continue;
                                        }
                                    };
                                    let evm_poll: EvmPollForDb = evm_poll_item.clone().into();
                                    if let Err(e) = tx.send((self.config.name.clone(), WsEvent::NewEvmPoll(evm_poll.clone()))) {
                                        tracing::error!("Error dispatching evm poll event: {e}");
                                    }
                                    match self.database.upsert_evm_poll(evm_poll).await {
                                        Ok(_) => {
                                            tracing::info!("evm poll successfully created by poll id {}", &evm_poll_item.poll_id);
                                        }
                                        Err(e) => {
                                            tracing::error!("evm poll could not created {}, Error: {}", &evm_poll_item.poll_id,e);
                                        }
                                    };
                                }
                                SocketResult::Empty { .. } => {}
                            };
                        }
                        Err(error) => {
                            tracing::error!("Websocket JSON parse error for {}: {error}", chain_name);
                        }
                    }
                };
            };
        }
        tracing::error!("Axelar evm poll listener stopped");
    }

    async fn sub_for_axelar_evm_poll_votes(&self, ws_tx: Sender<(String, WsEvent)>) -> Result<(), TNRAppError> {
        let ws_url = self.config.wss_url.clone();
        let chain_name = self.config.name.clone();


        loop {
            let (ws_stream, _) = connect_async(ws_url.clone()).await.map_err(|_| TNRAppError::from("Can not connect".to_string()))?;

            // Split the connection into two parts.
            let (mut write, mut read) = ws_stream.split();

            // Subscribe to txs which are related evm polls.
            write.send(AXELAR_SUB_VOTE_TX.into()).await.map_err(|e| format!("Can't subscribe to AXELAR_SUB_VOTE_TX for {}: {e}", chain_name))?;

            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text_msg)) = msg {
                    match serde_json::from_str::<SocketMessage>(&text_msg) {
                        Ok(socket_msg) => {
                            match socket_msg.result {
                                SocketResult::NonEmpty(SocketResultNonEmpty::VotedTx { events: voted_tx }) => {
                                    let tx_hash = voted_tx.get_tx_hash();
                                    let tx = match voted_tx.fetch_tx(&self).await {
                                        Ok(res) => res,
                                        Err(e) => {
                                            dbg!("Axelar evm poll vote tx fetcher error {}",&e);
                                            continue;
                                        }
                                    };
                                    let tx_content = match tx.content.get(0) {
                                        Some(res) => res,
                                        None => {
                                            dbg!("Axelar evm poll tx does not have content which hash is {}", &tx_hash);
                                            continue;
                                        }
                                    };

                                    match tx_content {
                                        InternalTransactionContent::Known(InternalTransactionContentKnowns::AxelarRefundRequest { sender: _, inner_message }) => {
                                            match inner_message {
                                                InnerMessage::Known(InnerMessageKnown::VoteRequest { sender, vote, poll_id }) => {
                                                    let mut is_confirmation_tx = false;
                                                    if tx.raw.contains("POLL_STATE_COMPLETED") {
                                                        let mut poll_status = None;
                                                        let is_poll_failed = &tx.is_poll_failed();
                                                        if *is_poll_failed {
                                                            poll_status = Some(PollStatus::Failed);
                                                        } else {
                                                            is_confirmation_tx = tx.is_evm_poll_confirmation_tx().clone();
                                                            poll_status = Some(PollStatus::Completed);
                                                        }

                                                        if let Some(poll_status) = poll_status {
                                                            match self.database.update_evm_poll_status(&poll_id, &poll_status).await {
                                                                Ok(_) => { tracing::info!("Successfully updated evm poll status completed for which poll id is {}", &poll_id); }
                                                                Err(e) => { tracing::error!("Can not updated evm poll participant {}",e); }
                                                            };
                                                        }
                                                    };

                                                    match vote {
                                                        AxelarVote::Known(axelar_known_vote) => {
                                                            let vote = axelar_known_vote.evm_vote();
                                                            let time = tx.time as u64;
                                                            let tx_height = tx.height;
                                                            let chain = match axelar_known_vote { AxelarKnownVote::VoteEvent { chain, .. } => { chain } };

                                                            let validator = self.database.find_validator(doc! {"voter_address":sender.clone()}).await;
                                                            if let Ok(validator) = validator {
                                                                let voter_address = validator.voter_address.unwrap_or(String::from(sender));
                                                                let evm_poll_participant = EvmPollParticipantForDb {
                                                                    operator_address: validator.operator_address,
                                                                    tx_hash: tx_hash.to_string(),
                                                                    poll_id: poll_id.clone(),
                                                                    chain_name: String::from(chain),
                                                                    vote,
                                                                    time,
                                                                    tx_height,
                                                                    voter_address,
                                                                    confirmation: is_confirmation_tx,
                                                                };
                                                                if let Err(e) = ws_tx.send((self.config.name.clone(), WsEvent::UpdateEvmPollParticipant((poll_id.clone(), evm_poll_participant.clone())))) {
                                                                    tracing::error!("Error dispatching Evm Poll Update event: {e}");
                                                                }
                                                                match self.database.update_evm_poll_participant(&poll_id, &evm_poll_participant).await {
                                                                    Ok(_) => { tracing::info!("Successfully updated evm poll participant {} for which poll id is {}", &evm_poll_participant.operator_address, &poll_id); }
                                                                    Err(e) => { tracing::error!("Can not updated evm poll participant {}",e); }
                                                                };
                                                            }
                                                        }
                                                        AxelarVote::Unknown(_) => {
                                                            tracing::error!("Unknown axelar evm poll vote info");
                                                        }
                                                    }
                                                }
                                                InnerMessage::Known(_) => {
                                                    tracing::warn!("Non handled message");
                                                }
                                                InnerMessage::Unknown(_) => {
                                                    tracing::error!("Unknown axelar evm poll inner message");
                                                }
                                            }
                                        }
                                        InternalTransactionContent::Unknown { .. } => {
                                            tracing::error!("Unknown InternalTransactionContent");
                                        }
                                        _ => {
                                            tracing::error!("Unknown tx content");
                                        }
                                    };
                                }
                                SocketResult::Empty { .. } => {}
                                _ => {
                                    tracing::error!("Empty axelar evm poll votes could not listen");
                                }
                            };
                        }
                        Err(error) => {
                            tracing::error!("Websocket JSON parse error for {}: {error}", chain_name);
                        }
                    }
                };
            };
        }
        tracing::error!("Axelar evm poll votes listener stopped");
    }

    async fn sub_for_axelar_heartbeats(&self) -> Result<(), TNRAppError> {
        let ws_url = self.config.wss_url.clone();
        let chain_name = self.config.name.clone();


        loop {
            let (ws_stream, _) = connect_async(ws_url.clone()).await.map_err(|_| TNRAppError::from("Can not connect".to_string()))?;

            // Split the connection into two parts.
            let (mut write, mut read) = ws_stream.split();

            // Subscribe to txs which are for heartbeats.
            write.send(SUBSCRIBE_BLOCK.into()).await.map_err(|e| format!("Can't subscribe to confirm AXELAR SUB HEARTBEAT TX for {}: {e}", chain_name))?;

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
                                        heartbeat_begin_height = current_height.clone();
                                        match self.database.find_validators(Some(doc! {"$match":{"voter_address":{"$exists":true}}})).await {
                                            Ok(res) => {
                                                let period_height = heartbeat_begin_height + 1;
                                                let mut initial_period_heartbeats = vec![];
                                                for validator in res.into_iter() {
                                                    match validator.voter_address.clone() {
                                                        None => {}
                                                        Some(sender_address) => {
                                                            let generated_id = self.generate_heartbeat_id(sender_address.clone(), period_height);
                                                            let heartbeat = HeartbeatForDb {
                                                                heartbeat_raw: None,
                                                                period_height: period_height.clone(),
                                                                status: HeartbeatStatus::Fail,
                                                                sender: sender_address.clone(),
                                                                id: generated_id,
                                                            };
                                                            initial_period_heartbeats.push(heartbeat);
                                                        }
                                                    };
                                                };

                                                match self.database.add_heartbeat_many(initial_period_heartbeats).await {
                                                    Ok(_) => { tracing::info!("Current period initial heartbeats inserted"); }
                                                    Err(_) => { tracing::info!("Current period initial heartbeats could not inserted"); }
                                                };
                                            }
                                            Err(_) => {}
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
                                                        let period_height = heartbeat_begin_height.clone() + 1;
                                                        let generated_id = self.generate_heartbeat_id(info.sender.clone(), period_height);
                                                        let sender = info.sender.clone();
                                                        let heartbeat_raw = HeartbeatRawForDb {
                                                            height: current_height.clone(),
                                                            tx_hash: info.tx_hash.clone(),
                                                            timestamp: info.timestamp.clone() as u64,
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
                                                            Ok(_) => { tracing::info!("Successfully inserted heartbeat id {}", &generated_id) }
                                                            Err(_) => { tracing::error!("Could not inserted heartbeat id {}", &generated_id) }
                                                        };
                                                    };
                                                });
                                            };

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
            };
        }
        tracing::error!("Axelar heartbeats listener stopped");
    }

    pub fn convert_to_evm_hex(&self, string_byte_array: &String) -> Option<String> {
        let mut result: Option<String> = None;

        if string_byte_array.is_empty() {
            return result;
        };

        let mut prefix = String::from("0x").to_owned();
        match serde_json::from_str::<Vec<u8>>(string_byte_array) {
            Ok(res) => {
                let hex_res = hex::encode(res).clone();
                prefix.push_str(hex_res.as_str());
                result = Some(prefix);
            }
            Err(_) => { tracing::error!("Error while evm tx id byte array converting to hex"); }
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
    #[serde(rename = "tm.event='Tx' AND message.action='ConfirmERC20Deposit' AND axelar.evm.v1beta1.ConfirmDepositStarted.participants CONTAINS 'participants'")]
    ConfirmERC20DepositStartedTx { events: ConfirmDepositStartedEvents },

    #[serde(rename = "tm.event='Tx' AND message.action='ConfirmDeposit' AND axelar.evm.v1beta1.ConfirmDepositStarted.participants CONTAINS 'participants'")]
    ConfirmDepositStartedTx { events: ConfirmDepositStartedEvents },

    #[serde(rename = "tm.event='Tx' AND message.action='ConfirmGatewayTx' AND axelar.evm.v1beta1.ConfirmGatewayTxStarted.participants CONTAINS 'participants'")]
    ConfirmGatewayTxStartedTx { events: ConfirmGatewayTxStartedEvents },

    #[serde(rename = "tm.event='Tx' AND message.action='ConfirmTransferKey' AND axelar.evm.v1beta1.ConfirmKeyTransferStarted.participants CONTAINS 'participants'")]
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

impl SocketResultNonEmpty {
    pub async fn get_evm_poll_item(&self, chain: &Chain) -> Result<EvmPollItem, TNRAppError> {
        let tx_height = self.get_tx_height();
        let chain_name = self.get_chain_name();
        let action_name = self.get_action_name();
        let participants_raw = self.get_participants_raw();
        let tx_id = self.get_tx_id();
        let deposit_address = self.get_deposit_address();

        let evm_poll_item = match EvmPollItem::new(&EvmPollItemEventParams {
            chain: chain_name,
            deposit_address,
            tx_height,
            action_name,
            participants_raw,
            tx_id,
        }, &chain).await {
            Ok(res) => res,
            Err(e) => { return Err(e); }
        };

        Ok(evm_poll_item)
    }

    fn get_tx_height(&self) -> u64 {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => { events.tx_height.get(0).unwrap_or(&String::from("0")).parse::<u64>().unwrap_or(0) }
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => { events.tx_height.get(0).unwrap_or(&String::from("0")).parse::<u64>().unwrap_or(0) }
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => { events.tx_height.get(0).unwrap_or(&String::from("0")).parse::<u64>().unwrap_or(0) }
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => { events.tx_height.get(0).unwrap_or(&String::from("0")).parse::<u64>().unwrap_or(0) }
            _ => 0,
        }
    }
    fn get_chain_name(&self) -> String {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => { events.chain.get(0).unwrap_or(&String::from("")).to_string() }
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => { events.chain.get(0).unwrap_or(&String::from("")).to_string() }
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => { events.chain.get(0).unwrap_or(&String::from("")).to_string() }
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => { events.chain.get(0).unwrap_or(&String::from("")).to_string() }
            _ => String::from(""),
        }
    }
    fn get_action_name(&self) -> String {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => { events.message_action.get(0).unwrap_or(&String::from("")).to_string() }
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => { events.message_action.get(0).unwrap_or(&String::from("")).to_string() }
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => { events.message_action.get(0).unwrap_or(&String::from("")).to_string() }
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => { events.message_action.get(0).unwrap_or(&String::from("")).to_string() }
            _ => String::from(""),
        }
    }
    fn get_participants_raw(&self) -> String {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => { events.participants.get(0).unwrap_or(&String::from("")).to_string() }
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => { events.participants.get(0).unwrap_or(&String::from("")).to_string() }
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => { events.participants.get(0).unwrap_or(&String::from("")).to_string() }
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => { events.participants.get(0).unwrap_or(&String::from("")).to_string() }
            _ => String::from(""),
        }
    }
    fn get_tx_id(&self) -> String {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => { events.tx_id.get(0).unwrap_or(&String::from("")).to_string() }
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => { events.tx_id.get(0).unwrap_or(&String::from("")).to_string() }
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events } => { events.tx_id.get(0).unwrap_or(&String::from("")).to_string() }
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events } => { events.tx_id.get(0).unwrap_or(&String::from("")).to_string() }
            _ => String::from(""),
        }
    }

    fn get_deposit_address(&self) -> String {
        match self {
            SocketResultNonEmpty::ConfirmERC20DepositStartedTx { events } => { events.evm_deposit_address.get(0).unwrap_or(&String::from("")).to_string() }
            SocketResultNonEmpty::ConfirmDepositStartedTx { events } => { events.evm_deposit_address.get(0).unwrap_or(&String::from("")).to_string() }
            SocketResultNonEmpty::ConfirmGatewayTxStartedTx { events: _ } => { String::from("") }
            SocketResultNonEmpty::ConfirmKeyTransferStartedTx { events: _ } => { String::from("") }
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
                tracing::error!("tx could not fetched retrying 1 {}",e);
                match chain.get_tx_by_hash(&tx_hash).await {
                    Ok(res) => res.value,
                    Err(e) => {
                        tracing::error!("tx could not fetched retrying 2  {}",e);
                        match chain.get_tx_by_hash(&tx_hash).await {
                            Ok(res) => res.value,
                            Err(e) => {
                                tracing::error!("tx could not fetched  {}",e);
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


    pub fn get_tx_height(&self) -> String {
        self.tx_height.get(0).unwrap_or(&String::from("")).to_string()
    }

    pub fn get_poll_state(&self) -> String {
        self.poll_state.get(0).unwrap_or(&String::from("")).replace("\"", "")
    }

    pub fn is_poll_completed(&self) -> bool {
        self.get_poll_state() == "POLL_STATE_COMPLETED"
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
            Err(e) => { return Err(TNRAppError::from(format!("error {}", e))); }
        };

        let tx_height = params.tx_height;
        let time = match chain.get_block_by_height(Some(tx_height)).await {
            Ok(res) => res.value.time as u64,
            Err(_) => { 0 }
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
        let participants: Vec<EvmPollParticipantForDb> = value.participants_operator_address.into_iter().map(|address| { EvmPollParticipantForDb::from_info(address, value.poll_id.clone(), value.chain_name.clone()) }).collect();

        EvmPollForDb {
            timestamp: value.time.clone(),
            tx_height: value.tx_height.clone(),
            poll_id: value.poll_id.clone(),
            action: value.action.clone(),
            status: value.status.clone(),
            evm_tx_id: value.evm_tx_id.clone(),
            chain_name: value.chain_name.clone(),
            evm_deposit_address: value.evm_deposit_address.clone(),
            participants,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum EvmPollVote {
    UnSubmit,
    Yes,
    No,
}