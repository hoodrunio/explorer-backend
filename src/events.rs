use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::database::{BlockForDb, EvmPollForDb, EvmPollParticipantForDb};
use crate::fetch::transactions::TransactionItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsEvent {
    NewTX(TransactionItem),
    NewBLock(BlockForDb),
    NewEvmPoll(EvmPollForDb),
    UpdateEvmPollParticipant(EvmPollParticipantForDb),
}

impl Display for WsEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WsEvent::NewTX(tx) => {
                let hash = tx.hash.clone();
                write!(f, "WsEvent (NewTX), hash: {hash}")
            }
            WsEvent::NewBLock(block) => {
                let hash = block.hash.clone();
                write!(f, "WsEvent (NewBlock), hash: {hash}")
            }
            WsEvent::NewEvmPoll(poll) => {
                let poll_id = poll.poll_id.clone();
                write!(f, "WsEvent (NewEvmPoll), id: {poll_id}")
            }
            WsEvent::UpdateEvmPollParticipant(participant) => {
                let participant_address = participant.voter_address.clone();
                let poll_id = participant.poll_id.clone();
                write!(
                    f,
                    "WsEvent (UpdateEvmPollParticipant), poll_id: {poll_id}, participant_hash: {participant_address}"
                )
            }
        }
    }
}

// Helper function to create a broadcast channel for WsEvents
pub fn create_event_channel(capacity: usize) -> (broadcast::Sender<(String, WsEvent)>, broadcast::Receiver<(String, WsEvent)>) {
    broadcast::channel(capacity)
}
