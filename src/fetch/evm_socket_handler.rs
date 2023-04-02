use crate::{
    chain::Chain,
    database::{EvmPollForDb, EvmPollParticipantForDb},
    events::WsEvent,
};
use mongodb::bson::doc;
use tokio::sync::broadcast::Sender;

use super::{
    evm::PollStatus,
    socket::{NewBlockData, SocketResultNonEmpty, VotedTxEvents},
    transactions::{AxelarKnownVote, AxelarVote, InnerMessage, InnerMessageKnown, InternalTransactionContent, InternalTransactionContentKnowns},
};

pub struct EvmSocketHandler {
    pub chain: Chain,
    pub ws_tx_sender: Sender<(String, WsEvent)>,
}

impl EvmSocketHandler {
    pub fn new(chain: Chain, ws_tx_sender: Sender<(String, WsEvent)>) -> Self {
        Self { chain, ws_tx_sender }
    }
    pub async fn handle_evm_poll(&self, evm_poll_block_data: NewBlockData) {
        if let Some(polls) = &evm_poll_block_data.value.extract_evm_poll_completed_events() {
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
    pub async fn handle_evm_poll_status(&self, voted_tx: VotedTxEvents) {
        let tx_hash = voted_tx.get_tx_hash();
        let tx = match voted_tx.fetch_tx(&self.chain).await {
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
    pub async fn handle_evm_poll_any_message(&self, non_empty_message: SocketResultNonEmpty) {
        let evm_poll_item = match non_empty_message.get_evm_poll_item(&self.chain).await {
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
}
