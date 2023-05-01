use serde::{Deserialize, Serialize};

use super::TXMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewProposalVoteEvent {
    pub vote_option: ProposalVoteOption,
    pub voter: String,
    pub proposal_id: String,
    pub tx_hash: String,
}

impl NewProposalVoteEvent {
    pub fn from_tx_events(ev: TXMap) -> Self {
        let vote_option = serde_json::from_str(ev["proposal_vote.option"].get(0).unwrap()).unwrap();

        Self {
            vote_option,
            voter: ev["message.sender"].get(0).unwrap().to_string(),
            proposal_id: ev["proposal_vote.proposal_id"].get(0).unwrap().to_string(),
            tx_hash: ev["tx.hash"].get(0).unwrap().to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalVoteOption {
    pub weight: String,
    pub option: u8,
}
