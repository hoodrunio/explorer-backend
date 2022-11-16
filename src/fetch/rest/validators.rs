use serde::{Deserialize, Serialize};

use super::others::{DenomAmount, Pagination, PaginationConfig, PublicKey};
use crate::chain::Chain;

impl Chain {
    /// Returns validator by given validator address.
    pub async fn get_validator(&self, validator_addr: &str) -> Result<ValidatorListValidator, String> {
        let path = format!("/cosmos/distribution/v1beta1/validators/{validator_addr}");

        match self.rest_api_request::<ValidatorResp>(&path, &[]).await {
            Ok(res) => Ok(res.validator),
            Err(error) => Err(error),
        }
    }

    /// Returns all the validators by given delegator address.
    pub async fn get_validators_by_delegator(&self, delegator_addr: &str, pagination_config: PaginationConfig) -> Result<ValidatorsResp, String> {
        let path = format!("/cosmos/staking/v1beta1/delegators/{delegator_addr}/validators");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request(&path, &query).await
    }

    /// Returns accumulated commission of given validator.
    pub async fn get_validator_commission(&self, validator_addr: &str) -> Result<ValidatorCommisionResp, String> {
        let path = format!("/cosmos/distribution/v1beta1/validators/{validator_addr}/commission");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns rewards of given validator.
    pub async fn get_validator_rewards(&self, validator_addr: &str) -> Result<ValidatorRewardsResp, String> {
        let path = format!("/cosmos/distribution/v1beta1/validators/{validator_addr}/outstanding_rewards");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns the list of validators with bonded status.
    pub async fn get_validators_bonded(&self, pagination_config: PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_BONDED".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }

    /// Returns the list of validators with unbonded status.
    pub async fn get_validators_unbonded(&self, pagination_config: PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_UNBONDED".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }

    /// Returns the list of validators with unbonding status.
    pub async fn get_validators_unbonding(&self, pagination_config: PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_UNBONDING".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }

    /// Returns the list of validators with unspecified status.
    pub async fn get_validators_unspecified(&self, pagination_config: PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_UNSPECIFIED".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }

    /// Returns validator information by given delegator validator pair.
    pub async fn get_delegator_validator_pair_info(&self, delegator_addr: &str, validator_addr: &str) -> Result<ValidatorResp, String> {
        let path = format!("/cosmos/staking/v1beta1/delegators/{delegator_addr}/validators/{validator_addr}");

        self.rest_api_request(&path, &[]).await
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorResp {
    /// Validator.
    pub validator: ValidatorListValidator,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorsResp {
    /// Array of validators.
    pub validators: Vec<ValidatorListValidator>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorCommisionResp {
    /// Validator commission.
    pub commission: ValidatorCommision,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorCommision {
    /// Array of amounts and demons.
    pub commission: Vec<DenomAmount>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorRewardsResp {
    /// Validator rewards.
    pub rewards: ValidatorCommision,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorRewards {
    /// Array of amounts and denoms.
    pub rewards: Vec<DenomAmount>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorListResp {
    /// Array of validators.
    pub validators: Vec<ValidatorListValidator>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorListValidator {
    /// Operator address. Eg: `"evmosvaloper1qq95x6dhrdnrfunlth5uh24tkrfphzl9crd3xr"`
    pub operator_address: String,
    /// Consensus public key.
    pub consensus_pubkey: PublicKey,
    /// Jailed state. Eg: `false`
    pub jailed: bool,
    /// Status. Eg: `"BOND_STATUS_BONDED"`
    pub status: String,
    /// Tokens. Eg: `"145722654634775400576772"`
    pub tokens: String,
    /// Delegator shares. Eg: `"146454922655204548581706.446790192014497216"`
    pub delegator_shares: String,
    /// Description.
    pub description: ValidatorListValidatorDescription,
    /// Unbonding height. Eg: `"2580496"`
    pub unbonding_height: String,
    /// Unbonding time. Eg: `"2022-08-21T03:48:38.952541966Z"`
    pub unbonding_time: String,
    /// Validator commission.
    pub commission: ValidatorListValidatorCommission,
    /// Minimum self delegation. Eg: `"1"`
    pub min_self_delegation: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorListValidatorCommission {
    /// Validator commission rates.
    pub commission_rates: ValidatorListValidatorCommissionRates,
    /// Validator commission update time. Eg: `"2022-03-02T19:00:00Z"`
    pub update_time: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorListValidatorCommissionRates {
    /// Validator commission rate. Eg: `"0.050000000000000000"`
    pub rate: String,
    /// Validator maximum commission rate. Eg: `"0.200000000000000000"`
    pub max_rate: String,
    /// Validator maximum commission change rate. Eg: `"0.010000000000000000"`
    pub max_change_rate: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorListValidatorDescription {
    /// Validator moniker. Eg: `"heisenbug"`
    pub moniker: String,
    /// Validator identity. Eg: `"367960C067E253A4"`
    pub identity: String,
    /// Validator website. Eg: `"https://heisenbug.one"`
    pub website: String,
    /// Validator security contact. Eg: `"@heisenbug_evmos"`
    pub security_contact: String,
    /// Validator details. Eg: `"reliable \u0026\u0026 secure staking"`
    pub details: String,
}
