use serde::{Deserialize, Serialize};
use crate::fetch::validators::{ValidatorListValidatorCommission, ValidatorStatus};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EvmPoll {
    pub timestamp: String,
    pub tx_height: u64,
    pub poll_id: String,
    pub status: String,
    pub evm_tx_id: String,
    pub participants_operator_addresses: Vec<String>,
}
