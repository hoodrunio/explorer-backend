use serde::{Deserialize, Serialize};

use crate::fetch::socket::EvmPollVote;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EvmPoll {
    pub timestamp: u64,
    pub tx_height: u64,
    pub poll_id: String,
    pub chain_name: String,
    pub status: String,
    pub action: String,
    pub evm_tx_id: String,
    pub evm_deposit_address: String,
    pub participants: Vec<EvmPollParticipant>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EvmPollParticipant {
    pub operator_address: String,
    pub poll_id: String,
    pub vote: EvmPollVote,
    pub chain_name: String,
    pub time: u64,
    pub tx_height: u64,
    pub tx_hash: String,
    pub voter_address: String,
}

impl EvmPollParticipant {
    pub fn from_info(operator_address: String, poll_id: String, chain_name: String) -> Self {
        Self {
            vote: EvmPollVote::UnSubmit,
            time: 0,
            tx_height: 0,
            tx_hash: String::from(""),
            voter_address: "".to_string(),
            poll_id,
            chain_name,
            operator_address,
        }
    }
}
