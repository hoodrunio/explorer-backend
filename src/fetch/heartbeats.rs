use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::chain::Chain;
use crate::fetch::others::PaginationConfig;
use crate::routes::{HeartbeatsListResp, TNRAppError};

impl Chain {
    pub async fn get_val_heartbeats(
        &self,
        operator_address: String,
        heartbeats_query: HeartbeatsQuery,
        config: PaginationConfig,
    ) -> Result<HeartbeatsListResp, TNRAppError> {
        let query = doc! {"operator_address": operator_address};
        let val_voter_address = match self.database.find_validator(query).await {
            Ok(res) => match res.voter_address {
                Some(res) => res,
                None => {
                    return Err(TNRAppError::from("Validator does not have voter address".to_string()));
                }
            },
            Err(e) => {
                return Err(TNRAppError::from(e));
            }
        };

        let match_pipe = doc! {"$match":{"sender": val_voter_address}};
        let mut pipeline = vec![match_pipe];
        if let (Some(from), Some(to)) = (heartbeats_query.from_block, heartbeats_query.to_block) {
            let range_match_pipe = doc! {"$match":{"period_height":{"$gte":from,"$lt":to}}};
            pipeline.push(range_match_pipe);
        };

        let heartbeats = self.database.find_paginated_heartbeats(pipeline, config).await?;
        let res = HeartbeatsListResp::from_db_list(heartbeats)?;

        Ok(res)
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
