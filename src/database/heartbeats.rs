use serde::{Deserialize, Serialize};

use crate::fetch::heartbeats::HeartbeatStatus;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Heartbeat {
    pub period_height: u64,
    pub status: HeartbeatStatus,
    pub sender: String,
    pub id: String,
    pub heartbeat_raw: Option<HeartbeatRaw>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct HeartbeatRaw {
    pub tx_hash: String,
    pub height: u64,
    pub period_height: u64,
    pub timestamp: u64,
    pub signatures: Vec<String>,
    pub sender: String,
    pub key_ids: Vec<String>,
}

