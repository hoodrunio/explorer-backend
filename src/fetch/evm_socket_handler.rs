use crate::{
    chain::Chain,
    database::{EvmPollForDb, EvmPollParticipantForDb, HeartbeatForDb, HeartbeatRawForDb},
    events::WsEvent,
};
use futures::future::join_all;
use mongodb::bson::doc;
use tokio::sync::broadcast::Sender;

use super::{
    evm::PollStatus,
    heartbeats::HeartbeatStatus,
    socket::{EvmPollBlockInfo, NewPollEvent, PollVoteEvent},
    transactions::{
        AxelarKnownVote, AxelarVote, InnerMessage, InnerMessageKnown, InternalTransactionContent, InternalTransactionContentKnowns, TransactionItem,
    },
};

pub struct EvmSocketHandler {
    pub chain: Chain,
    pub ws_tx_sender: Sender<(String, WsEvent)>,
}

impl EvmSocketHandler {
    pub fn new(chain: Chain, ws_tx_sender: Sender<(String, WsEvent)>) -> Self {
        Self { chain, ws_tx_sender }
    }
    pub async fn new_evm_poll_from_block(&self, evm_poll_block_info: EvmPollBlockInfo) {
        if let Some(polls) = &evm_poll_block_info.extract_evm_poll_completed_events() {
            if !polls.is_empty() {
                for completed_poll in polls.clone() {
                    match self
                        .chain
                        .database
                        .update_evm_poll_status(&completed_poll.poll_id, &completed_poll.poll_status)
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!("Could not update evm poll cause of {}", e);
                        }
                    };
                }
            };
        }
    }
    pub async fn new_evm_poll_from_tx(&self, new_poll_event: NewPollEvent, base_tx: TransactionItem) {
        dbg!(&new_poll_event);

        let evm_poll_item = match new_poll_event.get_evm_poll_item(&self.chain, base_tx).await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Could not get evm poll item {}", e);
                return;
            }
        };

        let _ = evm_poll_item.upsert_participants(&self.chain.database).await;

        let evm_poll: EvmPollForDb = evm_poll_item.clone().into();
        if let Err(e) = self
            .ws_tx_sender
            .send((self.chain.config.name.clone(), WsEvent::NewEvmPoll(evm_poll.clone())))
        {
            tracing::error!("Error dispatching evm poll event: {e}");
        }
        match self.chain.database.upsert_evm_poll(evm_poll).await {
            Ok(_) => {
                tracing::info!("evm poll successfully created by poll id {}", &evm_poll_item.poll_id);
            }
            Err(e) => {
                tracing::error!("evm poll could not created {}, Error: {}", &evm_poll_item.poll_id, e);
            }
        };
    }
    pub async fn evm_poll_status_handler(&self, poll_vote_event: PollVoteEvent, base_tx: TransactionItem) {
        let tx_hash = base_tx.hash.clone();
        let tx = match poll_vote_event.fetch_tx(&self.chain, &base_tx).await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Axelar evm poll vote tx fetcher error {}", &e);
                return;
            }
        };
        let tx_content = match tx.content.get(0) {
            Some(res) => res,
            None => {
                tracing::error!("Axelar evm poll tx does not have content which hash is {}", &tx_hash);
                return;
            }
        };

        match tx_content {
            InternalTransactionContent::Known(InternalTransactionContentKnowns::AxelarRefundRequest { sender: _, inner_message }) => {
                match inner_message {
                    InnerMessage::Known(InnerMessageKnown::VoteRequest { sender, vote, poll_id }) => {
                        let mut is_confirmation_tx = false;
                        if tx.raw.contains("POLL_STATE_COMPLETED") {
                            let mut poll_status = None;
                            let is_poll_failed = &tx.is_evm_poll_failed();
                            if *is_poll_failed {
                                poll_status = Some(PollStatus::Failed);
                            } else {
                                is_confirmation_tx = tx.is_evm_poll_confirmation_tx();
                                if is_confirmation_tx {
                                    poll_status = Some(PollStatus::Completed);
                                }
                            }

                            if let Some(poll_status) = poll_status {
                                match self.chain.database.update_evm_poll_status(poll_id, &poll_status).await {
                                    Ok(_) => {
                                        tracing::info!("Successfully updated evm poll status completed for which poll id is {}", &poll_id);
                                    }
                                    Err(e) => {
                                        tracing::error!("Can not updated evm poll participant {}", e);
                                    }
                                };
                            }
                        };

                        match vote {
                            AxelarVote::Known(axelar_known_vote) => {
                                let vote = axelar_known_vote.evm_vote();
                                let time = tx.time as u64;
                                let tx_height = tx.height;
                                let chain = match axelar_known_vote {
                                    AxelarKnownVote::VoteEvent { chain, .. } => chain,
                                };

                                let validator = self.chain.database.find_validator(doc! {"voter_address":sender.clone()}).await;
                                if let Ok(validator) = validator {
                                    let voter_address = validator.voter_address.unwrap_or(String::from(sender));
                                    let evm_poll_participant = EvmPollParticipantForDb {
                                        operator_address: validator.operator_address.clone(),
                                        tx_hash: tx_hash.to_string(),
                                        poll_id: poll_id.clone(),
                                        chain_name: String::from(chain),
                                        vote,
                                        time,
                                        tx_height,
                                        voter_address,
                                        confirmation: is_confirmation_tx,
                                    };
                                    match self.chain.database.upsert_evm_poll_participant(evm_poll_participant.clone()).await {
                                        Ok(_) => {
                                            tracing::info!(
                                                "Successfully updated evm poll participant {} for which poll id is {}",
                                                &validator.operator_address,
                                                &poll_id
                                            );
                                        }
                                        Err(e) => {
                                            tracing::error!("Can not updated evm poll participant {}", e);
                                        }
                                    };

                                    if let Err(e) = self.ws_tx_sender.send((
                                        self.chain.config.name.clone(),
                                        WsEvent::UpdateEvmPollParticipant((poll_id.clone(), evm_poll_participant)),
                                    )) {
                                        tracing::error!("Error dispatching Evm Poll Update event: {e}");
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
    pub async fn heartbeat_handler(&self, current_height: u64, is_heartbeat_begin: bool, mut heartbeat_begin_height: u64) {
        let heartbeat_block_check_range = 6;
        if is_heartbeat_begin {
            heartbeat_begin_height = current_height;

            if let Ok(res) = self
                .chain
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
                            let generated_id = self.chain.generate_heartbeat_id(sender_address.clone(), period_height);
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

                match self.chain.database.add_heartbeat_many(initial_period_heartbeats).await {
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
            let block_result = self.chain.get_block_result_by_height(Some(current_height)).await;

            if let Ok(block_result) = block_result {
                let mut block_res_txs_handler_futures = vec![];
                for block_res_tx_res in block_result.value.txs_results {
                    let sender_address = block_res_tx_res.get_sender_address().unwrap_or(String::from("")).clone();
                    block_res_txs_handler_futures.push(async move {
                        let heartbeat_info = self.chain.get_axelar_sender_heartbeat_info(&sender_address, current_height).await;
                        if let Ok(info) = heartbeat_info {
                            let period_height = heartbeat_begin_height + 1;
                            let generated_id = self.chain.generate_heartbeat_id(info.sender.clone(), period_height);
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
                            match self.chain.database.upsert_heartbeat(db_heartbeat).await {
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
}
