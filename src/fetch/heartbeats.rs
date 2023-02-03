use mongodb::bson::doc;
use crate::chain::Chain;
use crate::routes::TNRAppError;

impl Chain {
    pub async fn get_val_heartbeats(&self, operator_address: &String) -> Result<String, TNRAppError> {
        // let query = doc! {"operator_address": operator_address};
        // let val = self.database.find_validator(query).await?;
        // let val_voter_address = val.voter_address;

        Ok(String::from("Heartbeat"))
    }
}