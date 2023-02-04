use std::fmt;
use std::fmt::format;
use std::num::ParseFloatError;

use chrono::DateTime;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{chain::Chain, routes::OutRestResponse};
use crate::database::{EvmPollForDb, EvmPollParticipantForDb, PaginationDb};
use crate::fetch::blocks::BlockResp;
use crate::fetch::params::ParamsResp;
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
    pub deposit_address: String,
    pub participants: Vec<EvmPollParticipantForDb>,
}

impl From<EvmPollForDb> for EvmPollRespElement {
    fn from(value: EvmPollForDb) -> Self {
        Self {
            deposit_address: value.evm_deposit_address.clone(),
            event: value.action.clone(),
            status: value.status.to_string(),
            height: value.tx_height.clone(),
            id: value.poll_id.clone(),
            participants: value.participants.clone(),
            sender_chain: value.chain_name.clone(),
            tx_id: value.evm_tx_id.clone(),
        }
    }
}

impl EvmPollListResp {
    pub fn from_db_list(other: EvmPollListDbResp) -> Self {
        let mut polls: Vec<EvmPollRespElement> = vec![];

        for evm_poll in (&other.polls).iter() {
            polls.push(evm_poll.clone().into());
        };


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