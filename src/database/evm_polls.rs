use crate::fetch::socket::EvmPollVote;
use crate::fetch::validators::{ValidatorListValidatorCommission, ValidatorStatus};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EvmPoll {
    pub timestamp: u64,
    pub tx_height: u64,
    pub poll_id: String,
    pub chain_name: String,
    pub status: String,
    pub action: String,
    pub evm_tx_id: String,
    pub participants: Vec<EvmPollParticipant>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EvmPollParticipant {
    pub operator_address: String,
    pub vote: EvmPollVote,
}

impl From<String> for EvmPollParticipant {
    fn from(operator_address: String) -> Self {
        Self {
            operator_address,
            vote: EvmPollVote::UnSubmit,
        }
    }
}
