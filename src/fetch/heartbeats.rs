use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::chain::Chain;
use crate::database::{HeartbeatForDb, ListDbResult};
use crate::routes::{PaginationData, TNRAppError};

impl Chain {
    pub async fn get_val_heartbeats(
        &self,
        operator_address: String,
        heartbeats_query: HeartbeatsQuery,
        config: PaginationData,
    ) -> Result<ListDbResult<HeartbeatForDb>, TNRAppError> {
        let query = doc! {"operator_address": operator_address};
        let val_voter_address = match self.database.find_validator(query).await {
            Ok(res) => match res.voter_address {
                Some(res) => res,
                None => {
                    return Err(TNRAppError::from(format!("Validator does not have voter address")));
                }
            },
            Err(e) => {
                return Err(TNRAppError::from(e));
            }
        };

        let match_pipe = doc! {"$match":{"sender": val_voter_address}};

        let heartbeats = self.database.find_paginated_heartbeats(Some(match_pipe), Some(config)).await?;

        Ok(heartbeats)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HeartbeatsQuery {
    pub from_block: Option<i64>,
    pub to_block: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HeartbeatsListElement {
    pub status: HeartbeatStatus,
    pub period_height: u64,
    pub sender: String,
    pub id: String,
    pub heartbeat_raw: Option<HeartbeatsListRawElement>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct HeartbeatsListRawElement {
    pub tx_hash: String,
    pub height: u64,
    pub period_height: u64,
    pub timestamp: u64,
    pub signatures: Vec<String>,
    pub sender: String,
    pub key_ids: Vec<String>,
}

impl HeartbeatsQuery {
    pub fn new(from_block: Option<i64>, to_block: Option<i64>) -> Result<Self, String> {
        if (from_block.is_some() && to_block.is_none()) || (to_block.is_some() && from_block.is_none()) {
            return Err(String::from("Please specify from_block and to_block properties together"));
        };

        Ok(Self { from_block, to_block })
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum HeartbeatStatus {
    Success,
    Fail,
}
