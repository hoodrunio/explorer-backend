use chrono::{DateTime, NaiveDateTime};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use tokio::join;
use tonic::transport::Endpoint;

use crate::chain::Chain;
use crate::database::ListDbResult;
use crate::routes::{ChainAmountItem, PaginationData};

use super::others::{DenomAmount, Pagination};

impl Chain {
    /// Returns the delegations of given address.
    pub async fn get_delegations(&self, delegator_addr: &str, config: PaginationData) -> Result<ListDbResult<InternalDelegation>, String> {
        use crate::fetch::cosmos::staking::v1beta1::{query_client::QueryClient, QueryDelegatorDelegationsRequest};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryDelegatorDelegationsRequest {
            delegator_addr: delegator_addr.to_string(),
            pagination: Some(config.into()),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .delegator_delegations(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let mut delegations = vec![];

        for delegation_response in resp.delegation_responses {
            if let Ok(validator_metadata) = self
                .database
                .find_validator_by_operator_addr(&delegation_response.delegation.unwrap().validator_address)
                .await
            {
                let amount = self
                    .string_amount_parser(delegation_response.balance.unwrap().amount.clone(), None)
                    .await?;
                delegations.push({
                    InternalDelegation {
                        amount,
                        validator_logo_url: validator_metadata.logo_url,
                        validator_name: validator_metadata.name,
                        validator_address: validator_metadata.operator_address,
                    }
                })
            }
        }

        Ok(ListDbResult {
            data: delegations,
            pagination: resp.pagination.unwrap_or_default().into(),
        })
    }

    /// Returns the redelegations of given address.
    pub async fn get_redelegations(&self, delegator_addr: &str, config: PaginationData) -> Result<ListDbResult<InternalRedelegation>, String> {
        use crate::fetch::cosmos::staking::v1beta1::{query_client::QueryClient, QueryRedelegationsRequest, QueryRedelegationsResponse};

        let endoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryRedelegationsRequest {
            delegator_addr: delegator_addr.to_string(),
            src_validator_addr: "".to_string(),
            dst_validator_addr: "".to_string(),
            pagination: Some(config.into()),
        };

        let resp = QueryClient::connect(endoint)
            .await
            .unwrap()
            .redelegations(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let mut redelegations = vec![];

        for redelegation_response in resp.redelegation_responses {
            let src_address = redelegation_response.redelegation.clone().unwrap().validator_src_address;
            let dst_address = redelegation_response.redelegation.clone().unwrap().validator_dst_address;

            if let (Ok(validator_from), Ok(validator_to)) = join!(
                self.database.find_validator_by_operator_addr(src_address.as_str()),
                self.database.find_validator_by_operator_addr(dst_address.as_str()),
            ) {
                redelegations.push({
                    let redelegation_resp_entry = redelegation_response
                        .entries
                        .get(0)
                        .ok_or_else(|| "There is no redelegation entry.".to_string())?;

                    let amount = self.string_amount_parser(redelegation_resp_entry.balance.clone(), None).await?;

                    let completion_time = NaiveDateTime::from_timestamp_millis(
                        (redelegation_resp_entry.redelegation_entry.clone().unwrap().completion_time.unwrap().nanos / 1_000_000) as i64,
                    )
                    .ok_or(format!("Cannot parse redelegation completion datetime",))?
                    .timestamp_millis();

                    InternalRedelegation {
                        amount,
                        completion_time,
                        validator_from_logo_url: validator_from.logo_url,
                        validator_from_name: validator_from.name,
                        validator_from_address: validator_from.operator_address,
                        validator_to_logo_url: validator_to.logo_url,
                        validator_to_name: validator_to.name,
                        validator_to_address: validator_to.operator_address,
                    }
                })
            }
        }

        Ok(ListDbResult {
            data: redelegations,
            pagination: resp.pagination.unwrap_or_default().into(),
        })
    }

    /// Returns the unbonding delegations of given address.
    pub async fn get_delegations_unbonding(&self, delegator_addr: &str, config: PaginationData) -> Result<ListDbResult<InternalUnbonding>, String> {
        use crate::fetch::cosmos::staking::v1beta1::{query_client::QueryClient, QueryDelegatorUnbondingDelegationsRequest};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryDelegatorUnbondingDelegationsRequest {
            delegator_addr: delegator_addr.to_string(),
            pagination: Some(config.into()),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .delegator_unbonding_delegations(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let mut unbondings = vec![];

        for unbonding_response in resp.unbonding_responses {
            if let Ok(validator_metadata) = self.database.find_validator_by_operator_addr(&unbonding_response.validator_address).await {
                unbondings.push({
                    let unbonding_entry = unbonding_response
                        .entries
                        .get(0)
                        .ok_or_else(|| "There is no unbonding delegation entry.".to_string())?;

                    let amount = self.string_amount_parser(unbonding_entry.balance.clone(), None).await?;
                    InternalUnbonding {
                        balance: amount,
                        completion_time: NaiveDateTime::from_timestamp_millis(
                            (&unbonding_entry.completion_time.clone().unwrap().nanos / 1_000_000) as i64,
                        )
                        .ok_or(format!("Cannot parse unbonding delegation completion datetime",))?
                        .timestamp_millis(),
                        validator_logo_url: validator_metadata.logo_url,
                        validator_name: validator_metadata.name,
                        validator_address: validator_metadata.operator_address,
                    }
                })
            }
        }

        Ok(ListDbResult {
            data: unbondings,
            pagination: resp.pagination.unwrap_or_default().into(),
        })
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
pub struct SelfDelagationResp {
    /// Array of delegation responses.
    pub delegation_response: DelegationResponse,
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
    pub amount: ChainAmountItem,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalRedelegation {
    pub validator_from_logo_url: String,
    pub validator_from_name: String,
    pub validator_from_address: String,
    pub validator_to_logo_url: String,
    pub validator_to_name: String,
    pub validator_to_address: String,
    pub amount: ChainAmountItem,
    pub completion_time: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalUnbonding {
    pub validator_logo_url: String,
    pub validator_name: String,
    pub validator_address: String,
    pub balance: ChainAmountItem,
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
    pub entries: Vec<RedelegationResponseEntry>,
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
    pub entries: Option<Vec<RedelegationResponseEntry>>,
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
    pub creation_height: u64,
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
            creation_height: value.creation_height,
            completion_time: DateTime::parse_from_rfc3339(&value.completion_time)
                .map_err(|_| format!("Cannot parse redelegation completion datetime, '{}'.", value.completion_time))?
                .timestamp_millis(),
            initial_balance: value.initial_balance,
            shares_dst: value.shares_dst,
        })
    }
}
