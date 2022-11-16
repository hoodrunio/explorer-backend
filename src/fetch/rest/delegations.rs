use chrono::DateTime;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tokio::join;

use super::others::{DenomAmount, Pagination, PaginationConfig};
use crate::{
    chain::Chain,
    routes::rest::{calc_pages, OutRestResponse},
    utils::get_validator_logo,
};

impl Chain {
    /// Returns the delegations of given address.
    pub async fn get_delegations(&self, delegator_addr: &str, config: PaginationConfig) -> Result<OutRestResponse<Vec<InternalDelegation>>, String> {
        let path = format!("/cosmos/staking/v1beta1/delegations/{delegator_addr}");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<DelagationsResp>(&path, &query).await?;

        let mut jobs = vec![];

        for delegation_response in resp.delegation_responses {
            jobs.push(async move {
                let validator = self.get_validator(&delegation_response.delegation.validator_address).await?;
                let validator_logo = get_validator_logo(self.inner.client.clone(), &validator.description.identity).await?;
                let validator_name = validator.description.moniker;
                let amount = (delegation_response
                    .balance
                    .amount
                    .parse::<u128>()
                    .or_else(|_| Err(format!("Cannot parse delegation amount, '{}'", delegation_response.balance.amount)))?
                    / self.inner.decimals_pow as u128) as f64;
                let reward = 0.0;
                Ok::<InternalDelegation, String>(InternalDelegation {
                    amount,
                    reward,
                    validator_logo,
                    validator_name,
                })
            })
        }

        let mut delegations = vec![];

        let resps = join_all(jobs).await;

        for delegation in resps {
            delegations.push(delegation?);
        }

        let pages = calc_pages(resp.pagination, config)?;

        OutRestResponse::new(delegations, pages)
    }

    /// Returns the redelegations of given address.
    pub async fn get_redelegations(
        &self,
        delegator_addr: &str,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalRedelegation>>, String> {
        let path = format!("/cosmos/staking/v1beta1/delegators/{delegator_addr}/redelegations");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<RedelagationsResp>(&path, &query).await?;

        let mut jobs = vec![];

        for redelegation_response in resp.redelegation_responses {
            jobs.push(async move {
                let (validator_from, validator_to) = join!(
                    self.get_validator(&redelegation_response.redelegation.validator_src_address),
                    self.get_validator(&redelegation_response.redelegation.validator_dst_address)
                );

                let validator_from = validator_from?;
                let validator_to = validator_to?;

                let (validator_from_logo, validator_to_logo) = join!(
                    get_validator_logo(self.inner.client.clone(), &validator_from.description.identity),
                    get_validator_logo(self.inner.client.clone(), &validator_to.description.identity),
                );

                let validator_from_logo = validator_from_logo?;
                let validator_to_logo = validator_to_logo?;

                let validator_from_name = validator_from.description.moniker;
                let validator_to_name = validator_to.description.moniker;

                let redelegation_resp_entry = redelegation_response
                    .redelegation
                    .entries
                    .get(0)
                    .ok_or_else(|| format!("There is no completion time."))?;

                let amount = (redelegation_resp_entry
                    .balance
                    .parse::<u128>()
                    .or_else(|_| Err(format!("Cannot parse redelegation amount, '{}'", redelegation_resp_entry.balance)))?
                    / self.inner.decimals_pow as u128) as f64;

                let completion_time = DateTime::parse_from_rfc3339(&redelegation_resp_entry.redelegation_entry.completion_time)
                    .or_else(|_| {
                        Err(format!(
                            "Cannot parse redelegation completion datetime, '{}'.",
                            redelegation_resp_entry.redelegation_entry.completion_time
                        ))
                    })?
                    .timestamp_millis();

                Ok::<InternalRedelegation, String>(InternalRedelegation {
                    amount,
                    completion_time,
                    validator_from_logo,
                    validator_from_name,
                    validator_to_logo,
                    validator_to_name,
                })
            })
        }

        let mut redelegations = vec![];

        let resps = join_all(jobs).await;

        for redelegation in resps {
            redelegations.push(redelegation?);
        }

        let pages = calc_pages(resp.pagination, config)?;

        OutRestResponse::new(redelegations, pages)
    }

    /// Returns the unbonding delegations of given address.
    pub async fn get_delegations_unbonding(
        &self,
        delegator_addr: &str,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalUnbonding>>, String> {
        let path = format!("/cosmos/staking/v1beta1/delegators/{delegator_addr}/unbonding_delegations");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<UnbondingDelegationResp>(&path, &query).await?;

        let mut jobs = vec![];

        for unbonding_response in resp.unbonding_responses {
            jobs.push(async move {
                let validator = self.get_validator(&unbonding_response.validator_address).await?;
                let validator_logo = get_validator_logo(self.inner.client.clone(), &validator.description.identity).await?;
                let validator_name = validator.description.moniker;

                let unbonding_entry = unbonding_response.entries.get(0).ok_or_else(|| format!("There is no completion time."))?;

                let balance = (unbonding_entry
                    .balance
                    .parse::<u128>()
                    .or_else(|_| Err(format!("Cannot parse unbonding delegation balance, '{}'", unbonding_entry.balance)))?
                    / self.inner.decimals_pow as u128) as f64;

                let completion_time = DateTime::parse_from_rfc3339(&unbonding_entry.completion_time)
                    .or_else(|_| {
                        Err(format!(
                            "Cannot parse unbonding delegation completion datetime, '{}'.",
                            unbonding_entry.completion_time
                        ))
                    })?
                    .timestamp_millis();

                Ok::<InternalUnbonding, String>(InternalUnbonding {
                    balance,
                    completion_time,
                    validator_logo,
                    validator_name,
                })
            })
        }

        let mut unbondings = vec![];

        let resps = join_all(jobs).await;

        for unbonding in resps {
            unbondings.push(unbonding?);
        }

        let pages = calc_pages(resp.pagination, config)?;

        OutRestResponse::new(unbondings, pages)
    }
}
#[derive(Deserialize, Serialize, Debug)]
pub struct DelegationsRedelegationsUnbondings {}

#[derive(Deserialize, Serialize, Debug)]
pub struct DelagationsResp {
    /// Array of delegation responses.
    pub delegation_responses: Vec<DelegationResponse>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DelegationResponse {
    /// Delegation.
    pub delegation: Delegation,
    /// Amount and denom.
    pub balance: DenomAmount,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalDelegation {
    pub validator_logo: String,
    pub validator_name: String,
    pub amount: f64,
    pub reward: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalRedelegation {
    pub validator_from_logo: String,
    pub validator_from_name: String,
    pub validator_to_logo: String,
    pub validator_to_name: String,
    pub amount: f64,
    pub completion_time: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalUnbonding {
    pub validator_logo: String,
    pub validator_name: String,
    pub balance: f64,
    pub completion_time: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Delegation {
    /// Delegator address. Eg: `"cosmos156gqf9837u7d4c4678yt3rl4ls9c5vuuxyhkw6"`
    pub delegator_address: String,
    /// Validator address. Eg: `"cosmosvaloper156gqf9837u7d4c4678yt3rl4ls9c5vuursrrzf"`
    pub validator_address: String,
    /// Delegation shares. Eg: `"1899999.000000000000000000"`
    pub shares: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UnbondingDelegationResp {
    /// Array of unbonding delegation responses.
    pub unbonding_responses: Vec<UnbondingDelegationResponse>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UnbondingDelegationResponse {
    /// Delegator address. Eg: `cosmos156gqf9837u7d4c4678yt3rl4ls9c5vuuxyhkw6`
    pub delegator_address: String,
    /// Validator address. Eg: `cosmosvaloper156gqf9837u7d4c4678yt3rl4ls9c5vuursrrzf`
    pub validator_address: String,
    // Array of unbonding delegation entries.
    pub entries: Vec<UnbondingDelegationEntry>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UnbondingDelegationEntry {
    /// Unbonding entry creation height. Eg: `"524000"`
    pub creation_height: String,
    /// Unbonding entry competion time. Eg: `"2022-11-06T00:14:50.583Z"`
    pub completion_time: String,
    /// Unbonding entry inital balance. Eg: `"8578951234880932833"`
    pub initial_balance: String,
    /// Unbonding entry balance. Eg: `"8578951234880932833"`
    pub balance: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RedelagationsResp {
    /// Array of redelegation responses.
    pub redelegation_responses: Vec<RedelegationResponse>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RedelegationResponse {
    /// Delegation.
    pub redelegation: Redelegation,
    /// Amount and denom.
    pub entries: Vec<RedelegationEntry>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Redelegation {
    /// Delegator address. Eg: `"cosmos156gqf9837u7d4c4678yt3rl4ls9c5vuuxyhkw6"`
    pub delegator_address: String,
    /// Validator source address. Eg: `""`
    pub validator_src_address: String,
    /// Validator destination address. Eg: `""`
    pub validator_dst_address: String,
    /// Array of redelegation entries.
    pub entries: Vec<RedelegationResponseEntry>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RedelegationResponseEntry {
    /// Redelegation entry.
    pub redelegation_entry: RedelegationEntry,
    /// Balance. Eg: `"810289999999999999999"`
    pub balance: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RedelegationEntry {
    /// Redelagation creation height. Eg: `"524000"`
    pub creation_height: String,
    /// Redelagation competion time. Eg: `"2022-11-06T00:14:50.583Z"`
    pub completion_time: String,
    /// Redelagation inital balance. Eg: `""`
    pub initial_balance: String,
    /// Redelagation shares destination. Eg: `""`
    pub shares_dst: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalRedelegationEntry {
    /// Redelagation creation height. Eg: `524000`
    pub creation_height: u64,
    /// Redelagation competion timestamp in milliseconds.
    pub completion_time: i64,
    /// Redelagation inital balance. Eg: `""`
    pub initial_balance: String,
    /// Redelagation shares destination. Eg: `""`
    pub shares_dst: String,
}

impl TryFrom<RedelegationEntry> for InternalRedelegationEntry {
    type Error = String;
    fn try_from(value: RedelegationEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            creation_height: value
                .creation_height
                .parse()
                .or_else(|_| Err(format!("Cannot parse redelegation creation height, '{}'.", value.creation_height)))?,
            completion_time: DateTime::parse_from_rfc3339(&value.completion_time)
                .or_else(|_| Err(format!("Cannot parse redelegation completion datetime, '{}'.", value.completion_time)))?
                .timestamp_millis(),
            initial_balance: value.initial_balance,
            shares_dst: value.shares_dst,
        })
    }
}
