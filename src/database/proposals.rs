use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ProposalVote {
    pub voter: String,
    pub tx_hash: String,
    pub proposal_id: String,
    pub option: ProposalVoteOption,
    pub timestamp: i64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ProposalVoteOption {
    pub option: u8,
    pub weight: f32,
}
