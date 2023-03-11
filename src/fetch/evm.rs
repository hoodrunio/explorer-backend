use std::fmt;

use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::chain::Chain;
use crate::database::{EvmPollForDb, EvmPollParticipantForDb, PaginationDb};
use crate::fetch::socket::EvmPollVote;
use crate::routes::TNRAppError;

impl Chain {
    pub async fn get_supported_chains(&self, operator_address: &String) -> Result<EvmSupportedChains, TNRAppError> {
        let res = self.database.find_validator_supported_chains(operator_address).await?;

        Ok(res)
    }

    pub async fn get_evm_poll(&self, poll_id: &String) -> Result<EvmPollRespElement, TNRAppError> {
        let query = doc! {"poll_id": poll_id};
        let res = self.database.find_evm_poll(query).await?;

        Ok(res.into())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EvmPollListDbResp {
    pub polls: Vec<EvmPollForDb>,
    pub pagination: PaginationDb,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EvmPollListResp {
    pub polls: Vec<EvmPollRespElement>,
    pub pagination: PaginationDb,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EvmPollRespElement {
    pub id: String,
    pub tx_id: String,
    pub sender_chain: String,
    pub event: String,
    pub status: String,
    pub height: u64,
    pub timestamp: u64,
    pub deposit_address: String,
    pub vote_count_info: EvmPollVoteCountInfoElement,
    pub participants: Vec<EvmPollParticipantForDb>,
}

impl From<EvmPollForDb> for EvmPollRespElement {
    fn from(value: EvmPollForDb) -> Self {
        let mut vote_count_info = EvmPollVoteCountInfoElement::default();
        value
            .participants
            .iter()
            .for_each(|participant| vote_count_info.increment_count(&participant.vote));

        Self {
            deposit_address: value.evm_deposit_address.clone(),
            event: value.action.clone(),
            status: value.status.to_string(),
            height: value.tx_height,
            id: value.poll_id.clone(),
            participants: value.participants.clone(),
            sender_chain: value.chain_name.clone(),
            tx_id: value.evm_tx_id,
            timestamp: value.timestamp,
            vote_count_info,
        }
    }
}

impl EvmPollListResp {
    pub fn from_db_list(other: EvmPollListDbResp) -> Self {
        let mut polls: Vec<EvmPollRespElement> = vec![];

        for evm_poll in other.polls.iter() {
            polls.push(evm_poll.clone().into());
        }

        Self {
            polls,
            pagination: other.pagination,
        }
    }
}

pub type EvmSupportedChains = Vec<String>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum PollStatus {
    Pending,
    Completed,
    Failed,
}

impl fmt::Display for PollStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PollStatus::Pending => write!(f, "Pending"),
            PollStatus::Completed => write!(f, "Completed"),
            PollStatus::Failed => write!(f, "Failed"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EvmVotesListResp {
    pub list: Vec<EvmVoteRespElement>,
    pub pagination: PaginationDb,
}

impl EvmVotesListResp {
    pub fn from_db_list(list_from_db: EvmPollListDbResp, operator_address: String) -> Self {
        let mut votes: Vec<EvmVoteRespElement> = vec![];

        for evm_poll in list_from_db.polls.iter() {
            match &evm_poll
                .participants
                .iter()
                .find(|participant| participant.operator_address == operator_address)
            {
                None => {}
                Some(evm_vote) => {
                    votes.push(EvmVoteRespElement {
                        operator_address: evm_vote.operator_address.clone(),
                        poll_id: evm_vote.poll_id.clone(),
                        chain_name: evm_vote.chain_name.clone(),
                        vote: evm_vote.vote.clone(),
                        time: evm_vote.time,
                        tx_height: evm_vote.tx_height,
                        tx_hash: evm_vote.tx_hash.clone(),
                        voter_address: evm_vote.voter_address.clone(),
                    });
                }
            };
        }

        Self {
            list: votes,
            pagination: list_from_db.pagination,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EvmVoteRespElement {
    pub operator_address: String,
    pub poll_id: String,
    pub chain_name: String,
    pub vote: EvmPollVote,
    pub time: u64,
    pub tx_height: u64,
    pub tx_hash: String,
    pub voter_address: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EvmPollVoteCountInfoElement {
    pub yes: u32,
    pub no: u32,
    pub unsubmit: u32,
}

impl EvmPollVoteCountInfoElement {
    pub fn increment_count(&mut self, vote: &EvmPollVote) {
        match vote {
            EvmPollVote::Yes => self.yes += 1,
            EvmPollVote::No => self.no += 1,
            EvmPollVote::UnSubmit => self.unsubmit += 1,
        }
    }
}

impl Default for EvmPollVoteCountInfoElement {
    fn default() -> Self {
        Self { yes: 0, no: 0, unsubmit: 0 }
    }
}
