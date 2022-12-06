use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Validator {
    name: String,
    logo_url: String,
    bonded_height: u64,
    change_24h: u64,
    hex_address: String,
    operator_address: String,
    consensus_address: String,
    self_delegate_address: String,
}
