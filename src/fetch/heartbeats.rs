use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::chain::Chain;
use crate::database::HeartbeatForDb;
use crate::routes::{TNRAppError, ValidatorHeartbeatsQBody};

impl Chain {
    pub async fn get_val_heartbeats(&self, operator_address: String, hearbeats_body: ValidatorHeartbeatsQBody) -> Result<Vec<HeartbeatForDb>, TNRAppError> {
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
        match (hearbeats_body.from_block, hearbeats_body.to_block) {
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