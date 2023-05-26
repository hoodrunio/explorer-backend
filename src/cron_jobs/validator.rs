use std::collections::HashMap;
use std::ops::Div;

use futures::future::join_all;
use mongodb::bson::doc;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::chain::Chain;
use crate::database::{ValidatorForDb, VotingPowerForDb};
use crate::fetch::amount_util::TnrDecimal;
use crate::fetch::evm::EvmSupportedChains;
use crate::fetch::others::InternalStakingPool;
use crate::fetch::validators::{ValidatorListValidator, ValidatorListValidatorCommission, ValidatorListValidatorCommissionRates, ValidatorStatus};
use crate::routes::PaginationData;
use crate::utils::{convert_consensus_pubkey_to_consensus_address, convert_consensus_pubkey_to_hex_address, get_validator_logo, str_to_dec};

impl Chain {
    pub async fn cron_job_validator(&self) -> Result<(), String> {
        let resp = self
            .get_validators_unspecified(PaginationData {
                cursor: None,
                offset: None,
                limit: Some(10000),
                direction: None,
            })
            .await?;
        let staking_pool = self.get_staking_pool().await?.value;

        let validators = resp.data;
        let mut jobs = vec![];

        for validator in validators {
            jobs.push(async move { self.to_job_validator(validator, staking_pool).await });
        }

        let resp = join_all(jobs).await;
        let mut job_validators: Vec<JobValidator> = vec![];
        for res in resp {
            job_validators.push(res?);
        }

        //Sort depending on delegator shares
        job_validators.sort_by(|a, b| b.delegator_shares.partial_cmp(&a.delegator_shares).unwrap());
        for (i, jv) in job_validators.iter_mut().enumerate() {
            let rank = (i as u64) + 1;
            jv.rank(rank);
        }

        let mut db_jobs = vec![];
        for job in job_validators {
            db_jobs.push(async move { self.database.upsert_validator(job.into()).await });
        }

        join_all(db_jobs).await;

        Ok(())
    }

    async fn to_job_validator(&self, validator: ValidatorListValidator, staking_pool: InternalStakingPool) -> Result<JobValidator, String> {
        let val_delegator_shares = self.format_delegator_share(&str_to_dec(validator.delegator_shares.as_str()));
        let bonded_staking_poll = TnrDecimal::from(staking_pool.bonded);

        let voting_power = val_delegator_shares.to_u64().unwrap_or(0);
        let voting_power_percentage = val_delegator_shares.div(bonded_staking_poll).to_f64().unwrap_or(0.0);

        let voting_power_db = VotingPowerForDb {
            voting_power: val_delegator_shares.to_f64().unwrap_or(0.0),
            voting_power_percentage,
            ..Default::default()
        }
        .init();

        self.database
            .upsert_voting_power_data(&validator.operator_address, voting_power_db)
            .await?;

        let is_active = &validator.status == "BOND_STATUS_BONDED";
        let consensus_address =
            convert_consensus_pubkey_to_consensus_address(&validator.consensus_pubkey.key, &format!("{}valcons", self.config.base_prefix));
        let logo_url = get_validator_logo(self.client.clone(), &validator.description.identity).await;
        let uptime = self
            .get_validator_uptime(&consensus_address, Some(ValidatorStatus::Active))
            .await
            .unwrap_or(0.0);

        let voter_address = match self.get_validator_voter_address(&validator.operator_address).await {
            Ok(res) => res,
            Err(_) => None,
        };

        let supported_evm_chains = match self.get_supported_chains(&validator.operator_address).await {
            Ok(res) => Some(res),
            Err(_) => None,
        };

        let self_delegation_amount: Option<f64> = match self.get_val_self_delegations(validator.operator_address.clone()).await {
            Ok(res) => Some(res.amount.amount.to_f64().unwrap_or(0.0)),
            Err(_) => None,
        };

        let validator_commissions = ValidatorListValidatorCommission {
            commission_rates: ValidatorListValidatorCommissionRates {
                rate: String::from(str_to_dec(&validator.commission.commission_rates.rate)),
                max_rate: String::from(str_to_dec(&validator.commission.commission_rates.max_rate)),
                max_change_rate: String::from(str_to_dec(&validator.commission.commission_rates.max_change_rate)),
            },
            update_time: validator.commission.update_time,
        };

        let job_val = JobValidator {
            rank: 0,
            bonded_height: None, // Find way to fetch and store.
            change_24h: None,    // Find way to fetch and store
            consensus_address,   // use it after it get's complete: `convert_consensus_pubkey_to_consensus_address()`
            hex_address: convert_consensus_pubkey_to_hex_address(&validator.consensus_pubkey.key)
                .ok_or_else(|| format!("Cannot parse self delegate address, {}.", validator.operator_address))?,
            logo_url,
            uptime,
            name: validator.description.moniker,
            operator_address: validator.operator_address.clone(),
            is_active,
            self_delegation_amount,
            self_delegate_address: self
                .convert_valoper_to_self_delegate_address(&validator.operator_address)
                .ok_or_else(|| format!("Cannot parse self delegate address, {}.", validator.operator_address))?,
            delegator_shares: val_delegator_shares.to_f64().unwrap_or(0.0),
            voting_power,
            voting_power_ratio: voting_power_percentage,
            validator_commissions,
            cumulative_bonded_tokens: None,
            voter_address,
            supported_evm_chains,
        };

        Ok(job_val)
    }
    pub async fn cron_job_val_supported_chains(&self) -> Result<(), String> {
        if self.config.name != "axelar" {
            return Ok(());
        };
        let validators = self.database.find_validators(Some(doc! {"$match":{"is_active":true}})).await?;
        let supported_chains = self.get_evm_supported_chains().await?;
        let mut chains_maintainers: HashMap<String, Vec<String>> = HashMap::new();

        for supported_chain in supported_chains {
            let maintainers = match self.get_evm_chain_maintainers(&supported_chain).await {
                Ok(res) => res,
                Err(_) => {
                    tracing::error!("Could not fetched supported chain maintainers");
                    continue;
                }
            };
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

            match self
                .database
                .update_validator_supported_chains(&operator_address, val_supported_chains)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("{}", e);
                }
            };
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct JobValidator {
    pub rank: u64,
    pub name: String,
    pub logo_url: String,
    pub bonded_height: Option<u64>,
    pub change_24h: Option<u64>,
    pub hex_address: String,
    pub delegator_shares: f64,
    pub voting_power: u64,
    pub voting_power_ratio: f64,
    pub is_active: bool,
    pub uptime: f64,
    pub validator_commissions: ValidatorListValidatorCommission,
    pub operator_address: String,
    pub consensus_address: String,
    pub self_delegation_amount: Option<f64>,
    pub self_delegate_address: String,
    pub cumulative_bonded_tokens: Option<f64>,
    pub voter_address: Option<String>,
    pub supported_evm_chains: Option<EvmSupportedChains>,
}

impl JobValidator {
    fn rank(&mut self, rank: u64) {
        self.rank = rank;
    }
}

impl From<JobValidator> for ValidatorForDb {
    fn from(value: JobValidator) -> Self {
        Self {
            rank: value.rank,
            name: value.name,
            logo_url: value.logo_url,
            bonded_height: value.bonded_height,
            change_24h: value.change_24h,
            hex_address: value.hex_address,
            delegator_shares: value.delegator_shares,
            voting_power: value.voting_power,
            voting_power_ratio: value.voting_power_ratio,
            is_active: value.is_active,
            uptime: value.uptime,
            validator_commissions: value.validator_commissions,
            operator_address: value.operator_address,
            consensus_address: value.consensus_address,
            self_delegation_amount: value.self_delegation_amount,
            self_delegate_address: value.self_delegate_address,
            cumulative_bonded_tokens: value.cumulative_bonded_tokens,
            voter_address: value.voter_address,
            supported_evm_chains: value.supported_evm_chains,
        }
    }
}
