use crate::utils::Base64Convert;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use futures::future::join_all;
use futures::stream::select;
use futures::SinkExt;
use futures::StreamExt;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use tendermint::block::commit_sig::CommitSig;
use tendermint::{Block, Time};
use tendermint_rpc::event::EventData;
use tendermint_rpc::query::EventType;
use tendermint_rpc::{SubscriptionClient, WebSocketClient};
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::chain::Chain;
use crate::database::{BlockForDb, DatabaseTR, EvmPollForDb, EvmPollParticipantForDb, ProposalVoteForDb, ProposalVoteOptionForDb};
use crate::events::WsEvent;
use crate::fetch::blocks::{CosmosEvent, ResultBeginBlock, ResultEndBlock};
use crate::fetch::evm::PollStatus;
use crate::fetch::transactions::InternalTransaction;
use crate::routes::TNRAppError;

use super::blocks::{BlockLastCommitSignatures, CosmosEventAttribute};
use super::evm_socket_handler::EvmSocketHandler;
use super::{blocks::BlockHeader, transactions::TransactionItem};

const SUBSCRIBE_BLOCK: &str = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='NewBlock'"], "id": 0 }"#;
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseTransaction {
    /// `[ "F0E26D70191E27C8AB6249DE9C088B8C2812443CDF0DF04D7C83AE76A117C083" ]`
    // #[serde(rename = "tx.hash")]
    pub hash: String,
    /// `[ "2931697000000000aevmos" ]`
    // #[serde(rename = "tx.fee")]
    pub fee: String,
    /// `[ "8076531" ]`
    // #[serde(rename = "tx.height")]
    pub height: String,
    /// `[ "/ethermint.evm.v1.MsgEthereumTx" ]`
    // #[serde(rename = "message.action")]
    pub message_action: String,
    /// `[ "1535902500000000aevmos" ]`
    // #[serde(rename = "transfer.amount")]
    pub transfer_amount: String,
}

impl BaseTransaction {
    fn from_tx_events(ev: TXMap) -> Option<Self> {
        let tx_fee_denom = ev.get("tx.fee")?.get(0)?.to_string();
        let transfer_amount = ev
            .get("transfer.amount")?
            .iter()
            .filter(|str| str.to_string() != tx_fee_denom)
            .map(String::from)
            .collect::<Vec<String>>()
            .get(0)
            .unwrap_or(&String::from("0.00"))
            .clone();

        Some(Self {
            hash: ev.get("tx.hash")?.get(0)?.to_string(),
            fee: tx_fee_denom,
            height: ev.get("tx.height")?.get(0)?.to_string(),
            message_action: ev.get("message.action")?.get(0)?.to_string(),
            transfer_amount,
        })
    }
}

impl BaseTransaction {
    pub async fn as_tx_item(&self, chain: &Chain) -> Result<TransactionItem, String> {
        let tx_fee_denom = self.fee.clone();
        let amount = chain
            .string_amount_parser(self.transfer_amount.replace(chain.config.main_denom.as_str(), "").clone(), None)
            .await?;

        let fee = chain
            .string_amount_parser(tx_fee_denom.replace(chain.config.main_denom.as_str(), "").clone(), None)
            .await?;

        Ok(TransactionItem {
            amount,
            fee,
            hash: self.hash.clone(),
            height: self
                .height
                .parse::<u64>()
                .map_err(|e| format!("Cannot parse tx height {}: {e}", self.height))?,
            time: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64,
            result: "Success".to_string(),
            tx_type: self
                .message_action
                .split_once("Msg")
                .map(|(_, r)| r)
                .unwrap_or(self.message_action.split('.').last().unwrap_or("Unknown"))
                .to_string(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ChainEvent {
    Tx(BaseTransaction, ExtraTxEventData),
    Block(BaseTransaction, ExtraBlockEventData),
}

pub type TXMap = BTreeMap<String, Vec<String>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfirmDepositStarted {
    chain: String,
    participants: PollParticipants,
    tx_id: String,
    evm_deposit_address: String,
    action: String,
    height: String,
}

impl ConfirmDepositStarted {
    fn from_tx_events(ev: TXMap) -> Self {
        let participants = serde_json::from_str(ev["axelar.evm.v1beta1.ConfirmDepositStarted.participants"].get(0).unwrap()).unwrap();

        Self {
            chain: ev["axelar.evm.v1beta1.ConfirmDepositStarted.chain"].get(0).unwrap().to_string(),
            participants,
            tx_id: ev["axelar.evm.v1beta1.ConfirmDepositStarted.tx_id"].get(0).unwrap().to_string(),
            evm_deposit_address: ev["axelar.evm.v1beta1.ConfirmDepositStarted.deposit_address"].get(0).unwrap().to_string(),
            action: ev["message.action"].get(0).unwrap().to_string(),
            height: ev["tx.height"].get(0).unwrap().to_string(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfirmGatewayTxStartedEvents {
    chain: String,
    participants: PollParticipants,
    tx_id: String,
    height: String,
    message_action: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PollParticipants {
    poll_id: String,
    participants: Vec<String>,
}

impl ConfirmGatewayTxStartedEvents {
    fn from_tx_events(ev: TXMap) -> Self {
        let participants = serde_json::from_str(ev["axelar.evm.v1beta1.ConfirmGatewayTxStarted.participants"].get(0).unwrap()).unwrap();

        Self {
            chain: ev["axelar.evm.v1beta1.ConfirmGatewayTxStarted.chain"].get(0).unwrap().to_string(),
            participants,
            tx_id: ev["axelar.evm.v1beta1.ConfirmGatewayTxStarted.tx_id"].get(0).unwrap().to_string(),
            message_action: ev["message.action"].get(0).unwrap().to_string(),
            height: ev["tx.height"].get(0).unwrap().to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfirmKeyTransferStartedEvents {
    chain: String,
    participants: PollParticipants,
    tx_id: String,
    message_action: String,
    height: String,
}

impl ConfirmKeyTransferStartedEvents {
    fn from_tx_events(ev: TXMap) -> Self {
        let participants = serde_json::from_str(ev["axelar.evm.v1beta1.ConfirmKeyTransferStarted.participants"].get(0).unwrap()).unwrap();

        Self {
            chain: ev["axelar.evm.v1beta1.ConfirmKeyTransferStarted.chain"].get(0).unwrap().to_string(),
            participants,
            tx_id: ev["axelar.evm.v1beta1.ConfirmKeyTransferStarted.tx_id"].get(0).unwrap().to_string(),
            message_action: ev["message.action"].get(0).unwrap().to_string(),
            height: ev["tx.height"].get(0).unwrap().to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewPollEvent {
    chain: String,
    poll_participants: PollParticipants,
    tx_id: String,
    height: u64,
    evm_deposit_address: String,
    message_action: String,
}

impl NewPollEvent {
    pub async fn get_evm_poll_item(&self, chain: &Chain) -> Result<EvmPollItem, TNRAppError> {
        let tx_height = self.height;
        let chain_name = self.chain.clone();
        let action_name = self.message_action.clone();
        let poll_participants = self.poll_participants.clone();
        let tx_id = self.tx_id.clone();
        let deposit_address = self.evm_deposit_address.clone();

        EvmPollItem::new(
            &EvmPollItemEventParams {
                chain: chain_name,
                deposit_address,
                tx_height,
                action_name,
                poll_participants,
                tx_id,
            },
            chain,
        )
        .await
    }
}

impl Default for NewPollEvent {
    fn default() -> Self {
        Self {
            chain: "".to_string(),
            poll_participants: PollParticipants {
                poll_id: String::default(),
                participants: vec![],
            },
            tx_id: "".to_string(),
            evm_deposit_address: "".to_string(),
            message_action: "".to_string(),
            height: 0,
        }
    }
}

impl From<ConfirmDepositStarted> for NewPollEvent {
    fn from(e: ConfirmDepositStarted) -> Self {
        Self {
            chain: e.chain,
            poll_participants: e.participants,
            tx_id: e.tx_id,
            evm_deposit_address: e.evm_deposit_address,
            message_action: e.action,
            height: e.height.parse::<u64>().unwrap_or(0),
        }
    }
}
impl From<ConfirmGatewayTxStartedEvents> for NewPollEvent {
    fn from(e: ConfirmGatewayTxStartedEvents) -> Self {
        Self {
            chain: e.chain,
            poll_participants: e.participants,
            tx_id: e.tx_id,
            message_action: e.message_action,
            height: e.height.parse::<u64>().unwrap_or(0),
            ..Default::default()
        }
    }
}
impl From<ConfirmKeyTransferStartedEvents> for NewPollEvent {
    fn from(e: ConfirmKeyTransferStartedEvents) -> Self {
        Self {
            chain: e.chain,
            poll_participants: e.participants,
            tx_id: e.tx_id,
            message_action: e.message_action,
            height: e.height.parse::<u64>().unwrap_or(0),
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PollVoteEvent {
    pub poll_state: String,
    pub hash: String,
}

impl PollVoteEvent {
    fn from_tx_events(ev: TXMap) -> Self {
        Self {
            poll_state: ev["axelar.vote.v1beta1.Voted.state"].get(0).unwrap().to_string(),
            hash: ev.get("tx.hash").unwrap().get(0).unwrap().to_string(),
        }
    }

    pub async fn fetch_tx(&self, chain: &Chain, tx_hash: String) -> Result<InternalTransaction, TNRAppError> {
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewProposalVoteEvent {
    pub vote_option: ProposalVoteOption,
    pub voter: String,
    pub proposal_id: String,
    pub tx_hash: String,
}

impl NewProposalVoteEvent {
    fn from_tx_events(ev: TXMap) -> Self {
        let vote_option = serde_json::from_str(ev["proposal_vote.option"].get(0).unwrap()).unwrap();

        Self {
            vote_option,
            voter: ev["message.sender"].get(0).unwrap().to_string(),
            proposal_id: ev["proposal_vote.proposal_id"].get(0).unwrap().to_string(),
            tx_hash: ev["tx.hash"].get(0).unwrap().to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExtraTxEventData {
    ConfirmDepositStarted(ConfirmDepositStarted),
    ConfirmGatewayTxStarted(ConfirmGatewayTxStartedEvents),
    ConfirmKeyTransferStarted(ConfirmKeyTransferStartedEvents),
    NewPoll(NewPollEvent),
    PollVote(PollVoteEvent),
    NewProposalVote(NewProposalVoteEvent),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseBlock {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExtraBlockEventData {}

#[derive(Debug, Clone)]
pub enum ParseError {
    ParseIntError(ParseIntError),
    MissingData,
}

impl From<ParseIntError> for ParseError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ParseIntError(e) => {
                write!(f, "Failed to parse number: {}", e)
            }
            ParseError::MissingData => {
                write!(f, "Some data is missing")
            }
        }
    }
}

pub fn parse_transaction(events: TXMap) -> Result<(BaseTransaction, Option<ExtraTxEventData>), ParseError> {
    let tx = BaseTransaction::from_tx_events(events.clone()).ok_or(ParseError::MissingData)?;

    match tx.message_action.as_str() {
        "ConfirmERC20Deposit" | "ConfirmDeposit" => {
            if events.contains_key("axelar.evm.v1beta1.ConfirmDepositStarted.participants") {
                let sp_tx = ConfirmDepositStarted::from_tx_events(events);
                return Ok((tx, Some(ExtraTxEventData::NewPoll(sp_tx.into()))));
            }
        }
        "ConfirmGatewayTx" => {
            if events.contains_key("axelar.evm.v1beta1.ConfirmGatewayTxStarted.participants") {
                let sp_tx = ConfirmGatewayTxStartedEvents::from_tx_events(events);
                return Ok((tx, Some(ExtraTxEventData::NewPoll(sp_tx.into()))));
            }
        }
        "ConfirmTransferKey" => {
            if events.contains_key("axelar.evm.v1beta1.ConfirmKeyTransferStarted.participants") {
                let sp_tx = ConfirmKeyTransferStartedEvents::from_tx_events(events);
                return Ok((tx, Some(ExtraTxEventData::NewPoll(sp_tx.into()))));
            }
        }
        "/cosmos.gov.v1beta1.MsgVote" => {
            if events.contains_key("proposal_vote.option") {
                let sp_tx = NewProposalVoteEvent::from_tx_events(events);
                return Ok((tx, Some(ExtraTxEventData::NewProposalVote(sp_tx))));
            };
        }
        other => {
            if events.contains_key("axelar.vote.v1beta1.Voted.state") {
                let sp_tx = PollVoteEvent::from_tx_events(events);
                return Ok((tx, Some(ExtraTxEventData::PollVote(sp_tx))));
            }
        } // m => { if m != "RefundMsgRequest" { dbg!(m); } }
    }
    Ok((tx, None))
}

impl Chain {
    pub async fn subscribe_events(&self, tx: Sender<(String, WsEvent)>) -> Result<(), String> {
        let (client, driver) = WebSocketClient::new(self.config.wss_url.as_str())
            .await
            .map_err(|e| format!("Failed to connect to the websocket endpoint: {e}"))?;

        let driver_handle = tokio::spawn(async move { driver.run().await });

        let mut txs = client
            .subscribe(EventType::Tx.into())
            .await
            .map_err(|e| format!("Failed to subscribe to new transactions: {e}"))?;

        let mut blocks = client
            .subscribe(EventType::NewBlock.into())
            .await
            .map_err(|e| format!("Failed to subscribe to new blocks: {e}"))?;

        let mut bundled = select(txs, blocks);

        let previous_block_header_resp: Arc<Mutex<Option<Block>>> = Arc::new(Mutex::new(None));

        let mut heartbeat_begin_height: u64 = 0;

        while let Some(ev) = bundled.next().await {
            let Ok(ev) = ev else {
                continue
            };

            let events = ev.events.clone().unwrap();
            let handler = EvmSocketHandler::new(self.clone(), tx.clone());

            match ev.data {
                EventData::NewBlock {
                    block,
                    result_begin_block,
                    result_end_block,
                } => {
                    let (Some(block), Some(result_begin_block), Some(result_end_block)) = (block, result_begin_block, result_end_block) else {
                        continue
                    };
                    tracing::info!("wss: new block on {}", self.config.name);

                    if vec![String::from("axelar"), String::from("axelar-testnet")].contains(&self.config.name) {
                        let is_hearbeat_begin = result_end_block.clone().events.iter().any(|e| e.kind == "heartbeat");
                        let current_height = block.header.height.value();

                        let handler_params = HeartbeatStateParams::from_ws_block(current_height, is_hearbeat_begin, &mut heartbeat_begin_height);

                        handler.heartbeat_handler(handler_params).await;

                        let evm_poll_block_info = EvmPollBlockInfo {
                            events: result_end_block
                                .events
                                .into_iter()
                                .map(|e| {
                                    let attributes = e
                                        .attributes
                                        .into_iter()
                                        .map(|a| {
                                            let key = String::base64_to_string(&a.key);
                                            let value = String::base64_to_string(&a.value);
                                            let index = a.index;

                                            CosmosEventAttribute { key, value, index }
                                        })
                                        .collect();

                                    CosmosEvent { r#type: e.kind, attributes }
                                })
                                .collect::<Vec<CosmosEvent>>(),
                        };

                        handler.new_evm_poll_from_block(evm_poll_block_info).await;
                    }

                    let mut mutex_previous_resp = previous_block_header_resp.lock().await;
                    match mutex_previous_resp.as_ref() {
                        Some(previous_resp) => {
                            let hex_res = hex::encode(previous_resp.header.proposer_address.as_bytes());

                            let proposer_metadata = self
                                .database
                                .find_validator_by_hex_addr(&hex_res)
                                .await
                                .map_err(|e| format!("block+ error: {e}"))?;

                            let prev_header = &previous_resp.header;
                            let current_heder = &block.header;
                            let signatures: Vec<BlockLastCommitSignatures> = block.last_commit().as_ref().map_or_else(Vec::new, |c| {
                                c.signatures
                                    .iter()
                                    .map(|cs| {
                                        let (block_id_flag, validator_address, timestamp, signature) = match cs {
                                            CommitSig::BlockIdFlagAbsent => (0, String::from(""), Time::now().to_rfc3339(), None),
                                            CommitSig::BlockIdFlagCommit {
                                                validator_address,
                                                timestamp,
                                                signature,
                                            } => (
                                                1,
                                                validator_address.to_string(),
                                                timestamp.to_rfc3339(),
                                                signature.as_ref().map(|s| base64::encode(s.as_bytes())),
                                            ),
                                            CommitSig::BlockIdFlagNil {
                                                validator_address,
                                                timestamp,
                                                signature,
                                            } => (
                                                2,
                                                validator_address.to_string(),
                                                timestamp.to_rfc3339(),
                                                signature.as_ref().map(|s| base64::encode(s.as_bytes())),
                                            ),
                                        };

                                        BlockLastCommitSignatures {
                                            block_id_flag,
                                            validator_address,
                                            timestamp,
                                            signature,
                                        }
                                    })
                                    .collect::<Vec<BlockLastCommitSignatures>>()
                            });

                            let block_item = BlockForDb {
                                hash: current_heder.last_block_id.map(|id| id.hash.to_string()).unwrap_or_default(),
                                height: prev_header.height.value(),
                                timestamp: DateTime::parse_from_rfc3339(&prev_header.time.to_rfc3339())
                                    .map(|dt| dt.timestamp_millis())
                                    .unwrap_or_default(),
                                tx_count: previous_resp.data.len() as u64,
                                proposer_logo_url: proposer_metadata.logo_url,
                                proposer_name: proposer_metadata.name,
                                proposer_address: proposer_metadata.operator_address,
                                signatures,
                            };

                            tx.send((self.config.name.clone(), WsEvent::NewBLock(block_item.clone()))).ok();

                            if let Err(e) = self.database.upsert_block(block_item).await {
                                tracing::error!("Error saving block to the database: {e} ")
                            }

                            *mutex_previous_resp = Some(block);
                        }
                        None => *mutex_previous_resp = Some(block),
                    };
                }
                EventData::Tx { tx_result } => {
                    let Ok((base, extra)) = parse_transaction(events) else {
                        continue
                    };
                    tracing::info!("wss: new tx on {}", self.config.name);

                    //All Tx Flow
                    let chain = self.clone();
                    let tx_sender_clone = tx.clone();
                    tokio::spawn(async move {
                        if let Ok(tx_item) = base.clone().as_tx_item(&chain).await {
                            tx_sender_clone.send((chain.config.name.clone(), WsEvent::NewTX(tx_item.clone()))).ok();
                            let _ = chain.database.add_transaction(tx_item.into()).await;
                        };
                    });

                    //Axelar tx flow
                    if vec![String::from("axelar"), String::from("axelar-testnet")].contains(&self.config.name) {
                        if let Some(extra_data) = extra {
                            match extra_data {
                                ExtraTxEventData::NewPoll(p) => {
                                    handler.new_evm_poll_from_tx(p).await;
                                }
                                ExtraTxEventData::PollVote(v) => {
                                    handler.evm_poll_status_handler(v).await;
                                }
                                ExtraTxEventData::NewProposalVote(np) => {
                                    handler.new_proposal_vote(np).await;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                EventData::GenericJsonEvent(_) => {}
            }
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalVoteOption {
    pub weight: String,
    pub option: u8,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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
        let poll_info = params.poll_participants.clone();

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
            participants_operator_address: poll_info.participants.clone(),
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
    pub poll_participants: PollParticipants,
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

#[derive(Deserialize, Debug, Clone)]
pub struct EvmPollBlockInfo {
    events: Vec<CosmosEvent>,
}

impl EvmPollBlockInfo {
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
        let end_block_events = &self.events;
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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct HeartbeatStateParams {
    pub current_height: u64,
    pub is_heartbeat_begin: bool,
    pub period_height: u64,
    pub is_in_period: bool,
}

impl HeartbeatStateParams {
    pub fn from_ws_block(current_height: u64, is_hearbeat_begin: bool, heartbeat_begin_height: &mut u64) -> Self {
        let heartbeat_block_check_range = 6;
        if is_hearbeat_begin {
            *heartbeat_begin_height = current_height;
        }
        let is_in_period = *heartbeat_begin_height + heartbeat_block_check_range >= current_height;
        let period_height = *heartbeat_begin_height + 1;

        Self {
            current_height,
            is_heartbeat_begin: is_hearbeat_begin,
            period_height,
            is_in_period,
        }
    }
}
