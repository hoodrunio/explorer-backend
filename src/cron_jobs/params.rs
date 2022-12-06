use std::time::Duration;

use futures::future::join_all;

use crate::database::ValidatorForDb;
use crate::utils::{convert_consensus_pubkey_to_hex_address, get_validator_logo};
use crate::{chain::Chain, fetch::others::PaginationConfig};

impl Chain {
    pub async fn cron_job_params(&self) -> Result<(), String> {
        let resp = self.get_params_all().await?;


        let all_params = resp.value;

        // Implement `params` in database folder and then save it here.
        // Finally add this cron job to `cron_jobs_all` method in `all.rs` file. 

        Ok(())
    }
}
