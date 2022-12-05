use chrono::DateTime;
use serde::{Deserialize, Serialize};
use tokio::join;

use super::others::{DenomAmount, Pagination, PaginationConfig};
use crate::{
    chain::Chain,
    routes::{calc_pages, OutRestResponse},
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

        let mut delegations = vec![];

        for delegation_response in resp.delegation_responses {
            if let Ok(validator_metadata) = self
                .get_validator_metadata_by_valoper_addr(delegation_response.delegation.validator_address)
                .await
            {
                delegations.push({
                    InternalDelegation {
                        amount: self.calc_amount_u128_to_f64(
                            delegation_response
                                .balance
                                .amount
                                .parse::<u128>()
                                .map_err(|_| format!("Cannot parse delegation amount, '{}'", delegation_response.balance.amount))?,
                        ),
                        validator_logo_url: validator_metadata.logo_url,
                        validator_name: validator_metadata.name,
                        validator_address: validator_metadata.valoper_address,
                    }
                })
            }
        }

        let pages = calc_pages(resp.pagination, config)?;

        Ok(OutRestResponse::new(delegations, pages))
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

        let mut redelegations = vec![];

        for redelegation_response in resp.redelegation_responses {
            if let (Ok(validator_from), Ok(validator_to)) = join!(
                self.get_validator_metadata_by_valoper_addr(redelegation_response.redelegation.validator_src_address),
                self.get_validator_metadata_by_valoper_addr(redelegation_response.redelegation.validator_dst_address),
            ) {
                redelegations.push({
                    let redelegation_resp_entry = redelegation_response
                        .redelegation
                        .entries
                        .get(0)
                        .ok_or_else(|| "There is no redelegation entry.".to_string())?;

                    let amount = self.calc_amount_u128_to_f64(
                        redelegation_resp_entry
                            .balance
                            .parse::<u128>()
                            .map_err(|_| format!("Cannot parse redelegation amount, '{}'", redelegation_resp_entry.balance))?,
                    );

                    let completion_time = DateTime::parse_from_rfc3339(&redelegation_resp_entry.redelegation_entry.completion_time)
                        .map_err(|_| {
                            format!(
                                "Cannot parse redelegation completion datetime, '{}'.",
                                redelegation_resp_entry.redelegation_entry.completion_time
                            )
                        })?
                        .timestamp_millis();

                    InternalRedelegation {
                        amount,
                        completion_time,
                        validator_from_logo_url: validator_from.logo_url,
                        validator_from_name: validator_from.name,
                        validator_from_address: validator_from.valoper_address,
                        validator_to_logo_url: validator_to.logo_url,
                        validator_to_name: validator_to.name,
                        validator_to_address: validator_to.valoper_address,
                    }
                })
            }
        }

        let pages = calc_pages(resp.pagination, config)?;

        Ok(OutRestResponse::new(redelegations, pages))
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

        let mut unbondings = vec![];

        for unbonding_response in resp.unbonding_responses {
            if let Ok(validator_metadata) = self.get_validator_metadata_by_valoper_addr(unbonding_response.validator_address).await {
                unbondings.push({
                    let unbonding_entry = unbonding_response
                        .entries
                        .get(0)
                        .ok_or_else(|| "There is no unbonding delegation entry.".to_string())?;

                    InternalUnbonding {
                        balance: self.calc_amount_u128_to_f64(
                            unbonding_entry
                                .balance
                                .parse::<u128>()
                                .map_err(|_| format!("Cannot parse unbonding delegation balance, '{}'", unbonding_entry.balance))?,
                        ),
                        completion_time: DateTime::parse_from_rfc3339(&unbonding_entry.completion_time)
                            .map_err(|_| {
                                format!(
                                    "Cannot parse unbonding delegation completion datetime, '{}'.",
                                    unbonding_entry.completion_time
                                )
                            })?
                            .timestamp_millis(),
                        validator_logo_url: validator_metadata.logo_url,
                        validator_name: validator_metadata.name,
                        validator_address: validator_metadata.valoper_address,
                    }
                })
            }
        }

        let pages = calc_pages(resp.pagination, config)?;

        Ok(OutRestResponse::new(unbondings, pages))
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
    pub validator_logo_url: String,
    pub validator_name: String,
    pub validator_address: String,
    pub amount: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalRedelegation {
    pub validator_from_logo_url: String,
    pub validator_from_name: String,
    pub validator_from_address: String,
    pub validator_to_logo_url: String,
    pub validator_to_name: String,
    pub validator_to_address: String,
    pub amount: f64,
    pub completion_time: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalUnbonding {
    pub validator_logo_url: String,
    pub validator_name: String,
    pub validator_address: String,
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
                .map_err(|_| format!("Cannot parse redelegation creation height, '{}'.", value.creation_height))?,
            completion_time: DateTime::parse_from_rfc3339(&value.completion_time)
                .map_err(|_| format!("Cannot parse redelegation completion datetime, '{}'.", value.completion_time))?
                .timestamp_millis(),
            initial_balance: value.initial_balance,
            shares_dst: value.shares_dst,
        })
    }
}
