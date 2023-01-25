use serde::{Deserialize, Serialize};
use crate::fetch::socket::EvmPollVote;
use crate::fetch::validators::{ValidatorListValidatorCommission, ValidatorStatus};

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
    pub operator_address:String,
    pub vote: EvmPollVote,
    pub time: u64,
    pub tx_height: u64,
    pub tx_hash: String,
    pub voter_address: String,
}

impl From<String> for EvmPollParticipant {
    fn from(operator_address: String) -> Self {
        Self{
            operator_address,
            vote: EvmPollVote::UnSubmit,
            time: 0,
            tx_height: 0,
            tx_hash: String::from(""),
            voter_address: "".to_string(),
        }
    }
}
