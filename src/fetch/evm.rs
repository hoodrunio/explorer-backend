use std::fmt::format;
use std::num::ParseFloatError;

use chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::fetch::blocks::BlockResp;
use crate::fetch::params::ParamsResp;
use crate::{chain::Chain, routes::OutRestResponse};
use crate::database::{EvmPollForDb, EvmPollParticipantForDb};
use crate::fetch::others::PaginationDb;
use crate::routes::TNRAppError;

impl Chain {
    pub async fn get_supported_chains(&self, operator_address: &String) -> Result<EvmSupportedChains, TNRAppError> {
        let res = self.database.find_validator_supported_chains(operator_address).await?;

        Ok(res)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EvmPollListDbResp {
    pub polls: Vec<EvmPollForDb>,
    pub pagination: PaginationDb,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EvmPollListResp {
    pub polls: Vec<EvmPollListRespElement>,
    pub pagination: PaginationDb,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EvmPollListRespElement {
    pub deposit_address: String,
    pub event: String,
    pub status: String,
    pub height: u64,
    pub id: String,
    pub participants: Vec<EvmPollParticipantForDb>,
    pub sender_chain: String,
    pub tx_id: String,
}

impl EvmPollListResp {
    pub fn from_db_list(other: EvmPollListDbResp) -> Self {
        let mut polls = vec![];

        for p in (&other.polls).iter() {
            polls.push(EvmPollListRespElement {
                deposit_address: p.evm_deposit_address.clone(),
                event: p.action.clone(),
                status: p.status.clone(),
                height: p.tx_height.clone(),
                id: p.poll_id.clone(),
                participants: p.participants.clone(),
                sender_chain: p.chain_name.clone(),
                tx_id: p.evm_tx_id.clone(),
            }
            )
        };


        Self {
            polls,
            pagination: other.pagination,
        }
    }
}

pub type EvmSupportedChains = Vec<String>;
