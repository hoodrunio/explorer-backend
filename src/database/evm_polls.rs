use serde::{Deserialize, Serialize};
use crate::fetch::socket::EVM_POLL_VOTE;
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
    pub participants: Vec<EvmPollParticipant>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EvmPollParticipant {
    pub operator_address:String,
    pub vote:EVM_POLL_VOTE,
}

impl From<String> for EvmPollParticipant {
    fn from(operator_address: String) -> Self {
        Self{
            operator_address,
            vote: EVM_POLL_VOTE::UN_SUBMIT,
        }
    }
}
