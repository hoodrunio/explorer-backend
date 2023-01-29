use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tendermint::PublicKey;
use futures::future::join_all;
use mongodb::bson::doc;

use crate::database::{ValidatorForDb, VotingPowerForDb};
use crate::utils::{convert_consensus_pubkey_to_consensus_address, convert_consensus_pubkey_to_hex_address, get_validator_logo};
use crate::{chain::Chain, fetch::others::PaginationConfig};
use crate::fetch::validators::ValidatorStatus;

impl Chain {
    pub async fn cron_job_validator(&self) -> Result<(), String> {
        let resp = self.get_validators_unspecified(PaginationConfig::new().limit(10000)).await?;
        let staking_pool = self.get_staking_pool().await?.value;

        let validators = resp.validators;

        for validator in validators {
            let mut val_delegator_shares = 0.0;
            match self.format_delegator_share(&validator.delegator_shares) {
                Ok(delegator_shares) => {
                    val_delegator_shares = delegator_shares;
                    let voting_power_db = VotingPowerForDb {
                        voting_power: delegator_shares,
                        voting_power_percentage: (delegator_shares / (staking_pool.bonded as f64)) * 100.0,
                        ..Default::default()
                    }.init();

                    self.database.upsert_voting_power_data(&validator.operator_address, voting_power_db).await?;
                }
                Err(err) => { tracing::error!("{}",err) }
            }

            let is_active = &validator.status == "BOND_STATUS_BONDED";
            let consensus_address = convert_consensus_pubkey_to_consensus_address(&validator.consensus_pubkey.key, &format!("{}valcons", self.config.base_prefix));
            let logo_url = get_validator_logo(self.client.clone(), &validator.description.identity).await;
            let uptime = match self.get_validator_uptime(&consensus_address, Some(ValidatorStatus::Active)).await {
                Ok(res) => res,
                Err(_) => 0.0
            };

            let voter_address = match self.get_validator_voter_address(&validator.operator_address).await {
                Ok(res) => res,
                Err(_) => None
            };


            let db_val = ValidatorForDb {
                bonded_height: None,     // Find way to fetch and store.
                change_24h: None,        // Find way to fetch and store
                consensus_address, // use it after it get's complete: `convert_consensus_pubkey_to_consensus_address()`
                hex_address: convert_consensus_pubkey_to_hex_address(&validator.consensus_pubkey.key)
                    .ok_or_else(|| format!("Cannot parse self delegate address, {}.", validator.operator_address))?,
                logo_url,
                uptime,
                name: validator.description.moniker,
                operator_address: validator.operator_address.clone(),
                is_active,
                self_delegate_address: self
                    .convert_valoper_to_self_delegate_address(&validator.operator_address)
                    .ok_or_else(|| format!("Cannot parse self delegate address, {}.", validator.operator_address))?,
                delegator_shares: val_delegator_shares,
                validator_commissions: validator.commission,
                cumulative_bonded_tokens: None,
                voter_address,
                supported_evm_chains: None,
            };

            self.database.upsert_validator(db_val).await?;
        }

        Ok(())
    }
    pub async fn cron_job_val_supported_chains(&self) -> Result<(), String> {
        if self.config.name != "axelar" {
            return Ok(());
        };
        let validators = self.database.find_validators(Some(doc! {"$match":{"is_active":true}})).await?;
        let supported_chains = self.get_evm_supported_chains().await?;
        let mut chains_maintainers: HashMap<String, Vec<String>> = HashMap::new();

        for supported_chain in supported_chains {
            let maintainers = self.get_evm_chain_maintainers(&supported_chain).await?;
            chains_maintainers.insert(supported_chain.to_string(), maintainers);
        }

        for validator in validators {
            let mut val_supported_chains: Vec<String> = vec![];
            let operator_address = validator.operator_address.clone();
            for (chain, maintainers) in &chains_maintainers {
                let is_suppoerted = maintainers.contains(&operator_address);
                if is_suppoerted {
                    val_supported_chains.push(chain.clone());
                }
            }
            self.database.update_validator_supported_chains(&operator_address, val_supported_chains).await?;
            val_supported_chains = vec![];
        }

        Ok(())
    }
}
