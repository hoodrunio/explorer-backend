use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Heartbeat {
    pub tx_hash: String,
    pub height: u64,
    pub period_height: u64,
    pub timestamp: u64,
    pub signatures: Vec<String>,
    pub sender: String,
    pub key_ids: Vec<String>,
    pub id: String,
}
