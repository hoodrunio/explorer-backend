use chrono::DateTime;
use serde::{Deserialize, Serialize};

use super::others::{DenomAmount, InternalDenomAmount, Pagination, PaginationConfig};
use crate::{
    chain::Chain,
    routes::rest::{calc_pages, OutRestResponse},
};

impl Chain {
    /// Returns the delegations of given address.
    pub async fn get_delegations(
        &self,
        delegator_addr: &str,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalDelegationResponse>>, String> {
        let path = format!("/cosmos/staking/v1beta1/delegations/{delegator_addr}");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<DelagationsResp>(&path, &query).await?;

        let mut delegation_responses = vec![];

        for delegation_response in resp.delegation_responses {
            delegation_responses.push(delegation_response.try_into()?);
        }

        let pages = calc_pages(resp.pagination, config)?;

        OutRestResponse::new(delegation_responses, pages)
    }

    /// Returns the redelegations of given address.
    pub async fn get_redelegations(
        &self,
        delegator_addr: &str,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalRedelegationResponse>>, String> {
        let path = format!("/cosmos/staking/v1beta1/delegators/{delegator_addr}/redelegations");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<RedelagationsResp>(&path, &query).await?;

        let mut redelation_responses = vec![];

        for redelation_response in resp.redelegation_responses {
            redelation_responses.push(redelation_response.try_into()?);
        }

        let pages = calc_pages(resp.pagination, config)?;

        OutRestResponse::new(redelation_responses, pages)
    }

    /// Returns the unbonding delegations of given address.
    pub async fn get_delegations_unbonding(
        &self,
        delegator_addr: &str,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalUnbondingDelegationResponse>>, String> {
        let path = format!("/cosmos/staking/v1beta1/delegators/{delegator_addr}/unbonding_delegations");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<UnbondingDelegationResp>(&path, &query).await?;

        let mut unbonding_responses = vec![];

        for unbonding_response in resp.unbonding_responses {
            unbonding_responses.push(unbonding_response.try_into()?);
        }

        let pages = calc_pages(resp.pagination, config)?;

        OutRestResponse::new(unbonding_responses, pages)
    }
}

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
pub struct InternalDelegationResponse {
    /// Delegation.
    pub delegation: InternalDelegation,
    /// Amount and denom.
    pub balance: InternalDenomAmount,
}

impl TryFrom<DelegationResponse> for InternalDelegationResponse {
    type Error = String;
    fn try_from(value: DelegationResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            delegation: value.delegation.try_into()?,
            balance: value.balance.try_into()?,
        })
    }
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
pub struct InternalDelegation {
    /// Delegator address. Eg: `"cosmos156gqf9837u7d4c4678yt3rl4ls9c5vuuxyhkw6"`
    pub delegator_address: String,
    /// Validator address. Eg: `"cosmosvaloper156gqf9837u7d4c4678yt3rl4ls9c5vuursrrzf"`
    pub validator_address: String,
    /// Delegation shares. Eg: `1899999.000000000000000000`
    pub shares: f64,
}

impl TryFrom<Delegation> for InternalDelegation {
    type Error = String;
    fn try_from(value: Delegation) -> Result<Self, Self::Error> {
        Ok(Self {
            delegator_address: value.delegator_address,
            validator_address: value.validator_address,
            shares: value
                .shares
                .parse()
                .or_else(|_| Err(format!("Cannot parse delegation shares, '{}'.", value.shares)))?,
        })
    }
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
pub struct InternalUnbondingDelegationResponse {
    /// Delegator address. Eg: `cosmos156gqf9837u7d4c4678yt3rl4ls9c5vuuxyhkw6`
    pub delegator_address: String,
    /// Validator address. Eg: `cosmosvaloper156gqf9837u7d4c4678yt3rl4ls9c5vuursrrzf`
    pub validator_address: String,
    // Array of unbonding delegation entries.
    pub entries: Vec<InternalUnbondingDelegationEntry>,
}

impl TryFrom<UnbondingDelegationResponse> for InternalUnbondingDelegationResponse {
    type Error = String;
    fn try_from(value: UnbondingDelegationResponse) -> Result<Self, Self::Error> {
        let mut entries = vec![];

        for entry in value.entries {
            entries.push(entry.try_into()?);
        }

        Ok(Self {
            delegator_address: value.delegator_address,
            validator_address: value.validator_address,
            entries,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UnbondingDelegationEntry {
    /// Unbonding entry creation height. Eg: `"524000"`
    pub creation_height: String,
    /// Unbonding entry competion time. Eg: `"2022-11-06T00:14:50.583Z"`
    pub completion_time: String,
    /// Unbonding entry inital balance. Eg: `""`
    pub initial_balance: String,
    /// Unbonding entry balance. Eg: `""`
    pub balance: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalUnbondingDelegationEntry {
    /// Unbonding entry creation height. Eg: `524000`
    pub creation_height: u64,
    /// Unbonding entry competion timestamp in milliseconds.
    pub completion_time: i64,
    /// Unbonding entry inital balance. Eg: `""`
    pub initial_balance: String,
    /// Unbonding entry balance. Eg: `""`
    pub balance: String,
}

impl TryFrom<UnbondingDelegationEntry> for InternalUnbondingDelegationEntry {
    type Error = String;
    fn try_from(value: UnbondingDelegationEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            creation_height: value
                .creation_height
                .parse()
                .or_else(|_| Err(format!("Cannot parse unbonding delegation creation height, '{}'.", value.creation_height)))?,
            completion_time: DateTime::parse_from_rfc3339(&value.completion_time)
                .or_else(|_| Err(format!("Cannot parse redelegation completion datetime, '{}'.", value.completion_time)))?
                .timestamp_millis(),
            initial_balance: value.initial_balance,
            balance: value.balance,
        })
    }
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
pub struct InternalRedelegationResponse {
    /// Delegation.
    pub redelegation: InternalRedelegation,
    /// Amount and denom.
    pub entries: Vec<InternalRedelegationEntry>,
}

impl TryFrom<RedelegationResponse> for InternalRedelegationResponse {
    type Error = String;
    fn try_from(value: RedelegationResponse) -> Result<Self, Self::Error> {
        let mut entries = vec![];

        for entry in value.entries {
            entries.push(entry.try_into()?);
        }

        Ok(Self {
            redelegation: value.redelegation.try_into()?,
            entries,
        })
    }
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
    pub entries: Vec<RedelegationEntry>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalRedelegation {
    /// Delegator address. Eg: `"cosmos156gqf9837u7d4c4678yt3rl4ls9c5vuuxyhkw6"`
    pub delegator_address: String,
    /// Validator source address. Eg: `""`
    pub validator_src_address: String,
    /// Validator destination address. Eg: `""`
    pub validator_dst_address: String,
    /// Array of redelegation entries.
    pub entries: Vec<InternalRedelegationEntry>,
}

impl TryFrom<Redelegation> for InternalRedelegation {
    type Error = String;
    fn try_from(value: Redelegation) -> Result<Self, Self::Error> {
        let mut entries = vec![];

        for entry in value.entries {
            entries.push(entry.try_into()?);
        }

        Ok(Self {
            delegator_address: value.delegator_address,
            validator_src_address: value.validator_src_address,
            validator_dst_address: value.validator_dst_address,
            entries,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RedelegationResponseEntry {
    /// Redelegation entry.
    pub redelegation_entry: RedelegationEntry,
    /// Balance. Eg: `""`
    pub balance: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalRedelegationResponseEntry {
    /// Redelegation entry.
    pub redelegation_entry: InternalRedelegationEntry,
    /// Balance. Eg: `""`
    pub balance: String,
}

impl TryFrom<RedelegationResponseEntry> for InternalRedelegationResponseEntry {
    type Error = String;
    fn try_from(value: RedelegationResponseEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            redelegation_entry: value.redelegation_entry.try_into()?,
            balance: value.balance,
        })
    }
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
