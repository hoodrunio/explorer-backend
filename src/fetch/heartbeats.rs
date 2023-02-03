use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::chain::Chain;
use crate::database::HeartbeatForDb;
use crate::routes::{TNRAppError, ValidatorHeartbeatsQBody};

impl Chain {
    pub async fn get_val_heartbeats(&self, operator_address: String, heartbeats_query: HeartbeatsQuery) -> Result<Vec<HeartbeatForDb>, TNRAppError> {
        let query = doc! {"operator_address": operator_address};
        let val_voter_address = match self.database.find_validator(query).await {
            Ok(res) => {
                match res.voter_address {
                    Some(res) => { res }
                    None => { return Err(TNRAppError::from(format!("Validator does not have voter address"))); }
                }
            }
            Err(e) => { return Err(TNRAppError::from(e)); }
        };

        let match_pipe = doc! {"$match":{"sender": val_voter_address}};
        let mut pipeline = vec![match_pipe];
        match (heartbeats_query.from_block, heartbeats_query.to_block) {
            (Some(from), (Some(to))) => {
                let range_match_pipe = doc! {"$match":{"period_height":{"$gte":from as i64,"$lt":to as i64}}};
                pipeline.push(range_match_pipe);
            }
            _ => {}
        };

        let heartbeats = self.database.find_heartbeats(pipeline).await?;

        Ok(heartbeats)
    }
}

pub struct HeartbeatsQuery {
    pub sender: String,
    pub from_block: Option<i64>,
    pub to_block: Option<i64>,
}

impl HeartbeatsQuery {
    pub fn new(sender: String, from_block: Option<i64>, to_block: Option<i64>) -> Result<Self, String> {
        if (from_block.is_some() && to_block.is_none()) || (to_block.is_some() && from_block.is_none()) {
            return Err(String::from("Please specify from_block and to_block properties together"));
        };

        Ok(Self {
            sender,
            from_block,
            to_block,
        })
    }
}