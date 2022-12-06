use std::time::Duration;

use futures::future::join_all;

use crate::database::ValidatorForDb;
use crate::utils::{convert_consensus_pubkey_to_hex_address, get_validator_logo};
use crate::{chain::Chain, fetch::others::PaginationConfig};

impl Chain {
    pub async fn cron_job_validator(&self) -> Result<(), String> {
        let resp = self.get_validators_unspecified(PaginationConfig::new().limit(10000)).await?;

        let validators = resp.validators;

        let jobs: Vec<_> = validators
            .into_iter()
            .map(|validator| async move {
                Ok::<_, String>(ValidatorForDb {
                    bonded_height: None,     // Find way to fetch and store.
                    change_24h: None,        // Find way to fetch and store
                    consensus_address: None, // use it after it get's complete: `convert_consensus_pubkey_to_consensus_address()`
                    hex_address: convert_consensus_pubkey_to_hex_address(&validator.consensus_pubkey.key)
                        .ok_or_else(|| format!("Cannot parse self delegate address, {}.", validator.operator_address))?,
                    logo_url: get_validator_logo(self.inner.client.clone(), &validator.description.identity).await,
                    name: validator.description.moniker,
                    operator_address: validator.operator_address.clone(),
                    self_delegate_address: self
                        .convert_valoper_to_self_delegate_address(&validator.operator_address)
                        .ok_or_else(|| format!("Cannot parse self delegate address, {}.", validator.operator_address))?,
                })
            })
            .collect();

        let resps = join_all(jobs).await;

        let mut validators = vec![];

        for resp in resps {
            validators.push(resp?)
        }

        // Save to database.
        self.inner.database.add_validators(validators).await?;

        Ok(())
    }
}
