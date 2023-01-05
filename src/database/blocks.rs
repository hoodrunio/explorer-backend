use serde::{Deserialize, Serialize};
use crate::fetch::blocks::{BlockItem, BlockLastCommitSignatures};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Block {
    pub proposer_address: String,
    pub proposer_name: String,
    pub proposer_logo_url: String,
    pub height: u64,
    pub hash: String,
    pub tx_count: u64,
    pub timestamp: i64,
    pub signatures: Vec<BlockLastCommitSignatures>
}
