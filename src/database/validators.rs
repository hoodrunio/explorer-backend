use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Validator {
    pub name: String,
    pub logo_url: String,
    pub bonded_height: Option<u64>,
    pub change_24h: Option<u64>,
    pub hex_address: String,
    pub operator_address: String,
    pub consensus_address: Option<String>, // It is optional for now. Once you learn how to calculate consensus address, it won't be an optional value.
    pub self_delegate_address: String,
}
