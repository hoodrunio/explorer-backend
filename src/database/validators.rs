use serde::{Deserialize, Serialize};
use crate::fetch::validators::{ValidatorListValidatorCommission, ValidatorStatus};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Validator {
    pub name: String,
    pub logo_url: String,
    pub bonded_height: Option<u64>,
    pub change_24h: Option<u64>,
    pub hex_address: String,
    pub delegator_shares: f64,
    pub is_active: bool,
    pub validator_commissions: ValidatorListValidatorCommission,
    pub operator_address: String,
    pub consensus_address: String,
    pub self_delegate_address: String,
}
