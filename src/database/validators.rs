use serde::{Deserialize, Serialize};

use crate::fetch::evm::EvmSupportedChains;
use crate::fetch::validators::ValidatorListValidatorCommission;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Validator {
    pub name: String,
    pub logo_url: String,
    pub bonded_height: Option<u64>,
    pub change_24h: Option<u64>,
    pub hex_address: String,
    pub delegator_shares: f64,
    pub voting_power: u64,
    pub voting_power_ratio: f64,
    pub is_active: bool,
    pub uptime: f64,
    pub validator_commissions: ValidatorListValidatorCommission,
    pub operator_address: String,
    pub consensus_address: String,
    pub self_delegation_amount: Option<f64>,
    pub self_delegate_address: String,

    //Calculating on query.
    pub cumulative_bonded_tokens: Option<f64>,

    //Proxy/Voter/Broadcaster address for Axelar EVM Pool participants addresses
    pub voter_address: Option<String>,
    pub supported_evm_chains: Option<EvmSupportedChains>,
}
