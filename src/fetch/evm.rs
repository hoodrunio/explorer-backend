use std::fmt;

use futures::future::join_all;
use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};

use crate::chain::Chain;
use crate::database::{EvmPollForDb, EvmPollParticipantForDb, ListDbResult, ValidatorForDb};
use crate::fetch::chain_socket::EvmPollVote;
use crate::routes::{PaginationData, TNRAppError};

impl Chain {
    pub async fn get_supported_chains(&self, operator_address: &String) -> Result<EvmSupportedChains, TNRAppError> {
        let res = self.database.find_validator_supported_chains(operator_address).await?;

        Ok(res)
    }

    pub async fn get_evm_poll(&self, poll_id: &String) -> Result<EvmPollRespElement, TNRAppError> {
        let query = doc! {"poll_id": poll_id};
        let res = self.database.find_evm_poll(query).await?;

        Ok(EvmPollRespElement::new(self, res).await?)
    }

    pub async fn get_evm_polls(&self, query: Option<Document>, config: PaginationData) -> Result<ListDbResult<EvmPollRespElement>, TNRAppError> {
        let evm_polls_from_db = self.database.find_paginated_evm_polls(query, config).await?;

        let elements_jobs_resp = join_all(
            evm_polls_from_db
                .data
                .iter()
                .map(|poll| async move { EvmPollRespElement::new(self, poll.clone()).await }),
        )
        .await;

        let mut evm_poll_elements: Vec<EvmPollRespElement> = vec![];

        for res in elements_jobs_resp {
            evm_poll_elements.push(res?);
        }

        Ok(ListDbResult {
            data: evm_poll_elements,
            pagination: evm_polls_from_db.pagination,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EvmPollRespElement {
    pub timestamp: u64,
    pub height: u64,
    pub id: String,
    pub sender_chain: String,
    pub status: String,
    pub event: String,
    pub tx_id: String,
    pub deposit_address: String,
    pub vote_count_info: EvmPollVoteCountInfoElement,
    pub participants: Vec<EvmPollParticipantRespElement>,
}

impl EvmPollRespElement {
    pub async fn new(chain: &Chain, value: EvmPollForDb) -> Result<Self, String> {
        let mut vote_count_info = EvmPollVoteCountInfoElement::default();

        let participants = chain
            .database
            .find_paginated_evm_poll_participants(
                Some(doc! {"poll_id":value.poll_id.clone()}),
                PaginationData {
                    limit: Some(10000),
                    ..Default::default()
                },
            )
            .await?
            .data;

        participants
            .iter()
            .for_each(|participant| vote_count_info.increment_count(&participant.vote));

        let mut val_query_jobs = vec![];
        for participant_from_db in participants.iter() {
            val_query_jobs.push(async move {
                let doc = doc! {"operator_address": participant_from_db.operator_address.clone()};
                let val_res = chain.database.find_validator(doc).await;
                let operator_info: EvmPollOperatorInfo = val_res.map(|val| val.into()).unwrap_or_default();
                EvmPollParticipantRespElement::new(participant_from_db.clone(), operator_info)
            });
        }

        let participants = join_all(val_query_jobs).await.into_iter().collect();

        Ok(Self {
            deposit_address: value.evm_deposit_address.clone(),
            event: value.action.clone(),
            status: value.status.to_string(),
            height: value.tx_height,
            id: value.poll_id.clone(),
            sender_chain: value.chain_name.clone(),
            tx_id: value.evm_tx_id,
            timestamp: value.timestamp,
            vote_count_info,
            participants,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EvmPollParticipantRespElement {
    pub confirmation: bool,
    pub poll_id: String,
    pub vote: EvmPollVote,
    pub chain_name: String,
    pub time: u64,
    pub tx_height: u64,
    pub tx_hash: String,
    pub voter_address: String,
    pub operator_info: EvmPollOperatorInfo,
}

impl EvmPollParticipantRespElement {
    pub fn new(participant: EvmPollParticipantForDb, operator_info: EvmPollOperatorInfo) -> Self {
        Self {
            confirmation: participant.confirmation,
            poll_id: participant.poll_id,
            vote: participant.vote,
            chain_name: participant.chain_name,
            time: participant.time,
            tx_height: participant.tx_height,
            tx_hash: participant.tx_hash,
            voter_address: participant.voter_address,
            operator_info,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EvmPollOperatorInfo {
    pub operator_address: String,
    pub name: String,
    pub logo_url: String,
    pub voting_power: u64,
    pub voting_power_percent: f64,
}

impl Default for EvmPollOperatorInfo {
    fn default() -> Self {
        Self {
            operator_address: "".to_string(),
            name: "".to_string(),
            logo_url: "".to_string(),
            voting_power: 0,
            voting_power_percent: 0.0,
        }
    }
}

impl From<ValidatorForDb> for EvmPollOperatorInfo {
    fn from(value: ValidatorForDb) -> Self {
        Self {
            operator_address: value.operator_address,
            name: value.name,
            logo_url: value.logo_url,
            voting_power: value.voting_power,
            voting_power_percent: value.voting_power_ratio,
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

#[derive(Deserialize, Serialize, Debug, Default)]
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
