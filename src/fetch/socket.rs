use crate::fetch::chain_socket::tx::{parse_transaction, ExtraTxEventData};
use crate::fetch::chain_socket::EvmPollBlockInfo;
use crate::utils::Base64Convert;
use std::sync::Arc;

use chrono::DateTime;
use futures::stream::select;
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

use crate::chain::Chain;
use crate::database::BlockForDb;
use crate::events::WsEvent;
use crate::fetch::blocks::CosmosEvent;

use super::blocks::{BlockLastCommitSignatures, CosmosEventAttribute};
use super::evm_socket_handler::EvmSocketHandler;

impl Chain {
    pub async fn subscribe_events(&self, tx: Sender<(String, WsEvent)>) -> Result<(), String> {
        let (client, driver) = WebSocketClient::new(self.config.wss_url.as_str())
            .await
            .map_err(|e| format!("Failed to connect to the websocket endpoint: {e}"))?;

        tokio::spawn(async move { driver.run().await });

        let txs = client
            .subscribe(EventType::Tx.into())
            .await
            .map_err(|e| format!("Failed to subscribe to new transactions: {e}"))?;

        let blocks = client
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
                    let (Some(block), Some(_), Some(result_end_block)) = (block, result_begin_block, result_end_block) else {
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
                EventData::Tx { .. } => {
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
                            }
                        }
                    }
                }
                EventData::GenericJsonEvent(_) => {}
            }
        }
        Ok(())
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
