use std::ops::Div;

use chrono::{DateTime, Duration, Utc};
use futures::future::join_all;
use mongodb::bson::doc;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use tokio::join;
use tonic::transport::Endpoint;

use crate::database::{ListDbResult, ValidatorForDb};
use crate::fetch::cosmos::tx::v1beta1::OrderBy;
use crate::fetch::transactions::{InternalTransactionContent, InternalTransactionContentKnowns};
use crate::routes::ChainAmountItem;
use crate::routes::{PaginationData, TNRAppError};
use crate::utils::convert_consensus_pubkey_to_consensus_address;
use crate::{
    chain::Chain,
    routes::{calc_pages, OutRestResponse},
};
use crate::fetch::cosmos::base::query::v1beta1::PageRequest;

use crate::fetch::cosmos::slashing::v1beta1::QuerySigningInfoResponse;

use super::amount_util::TnrDecimal;
use super::delegations::SelfDelagationResp;
use super::{
    others::{DenomAmount, Pagination, PaginationConfig},
    transactions::{Tx, TxResponse, TxsResp, TxsTransactionMessage, TxsTransactionMessageKnowns},
};

impl Chain {
    /// Returns the signing info by given cons address.
    pub async fn get_validator_signing_info(&self, cons_addr: &str) -> Result<InternalSlashingSigningInfoItem, String> {
        use crate::fetch::cosmos::slashing::v1beta1::{QuerySigningInfoRequest, QuerySigningInfoResponse, query_client::QueryClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        
        let req = QuerySigningInfoRequest {
            cons_address: cons_addr.to_string(),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .signing_info(req)
            .await
            .map_err(|e| format!("{}", e))?;

        let signing_info = resp.into_inner();

        let signing_info = signing_info.try_into()?;

        Ok(signing_info)
    }

    /// Returns the delegations to given validator address.
    pub async fn get_validator_delegations(
        &self,
        validator_addr: &str,
        config: PaginationData,
    ) -> Result<ListDbResult<InternalDelegation>, String> {
        use crate::fetch::cosmos::staking::v1beta1::{QueryDelegatorDelegationsRequest, QueryDelegatorDelegationsResponse, query_client::QueryClient};
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let pagination = config.into();

        let req = QueryDelegatorDelegationsRequest {
            delegator_addr: validator_addr.to_string(),
            pagination: Some(pagination),
        };


        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .delegator_delegations(req)
            .await
            .map_err(|e| format!("{e}"))?;

        let delegations = resp.into_inner();
        
        let mut int_dels = vec![];


        for delegation in delegations.delegation_responses {
            let amount = self.string_amount_parser(delegation.balance.unwrap().amount.clone(), None).await?;
            int_dels.push(InternalDelegation {
                address: delegation.delegation.unwrap().delegator_address,
                amount,
            })
        }

        let resp = ListDbResult {
            data: int_dels,
            pagination: delegations.pagination.map(|p| p.into()).unwrap_or_default(),
        };

        Ok(resp)
    }

    /// Returns the unbonding delegations to given validator address.
    pub async fn get_validator_unbondings(
        &self,
        validator_addr: &str,
        config: PaginationData,
    ) -> Result<ListDbResult<InternalUnbonding>, String> {
        use crate::fetch::cosmos::staking::v1beta1::{QueryValidatorUnbondingDelegationsRequest, QueryValidatorUnbondingDelegationsResponse, query_client::QueryClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        let path = format!("/cosmos/staking/v1beta1/validators/{validator_addr}/unbonding_delegations");

        let req = QueryValidatorUnbondingDelegationsRequest {
            validator_addr: validator_addr.to_string(),
            pagination: Some(config.into()),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .validator_unbonding_delegations(req)
            .await
            .map_err(|e| format!("{}", e))?;


        let resp_unboundings = resp.into_inner();

        let mut unbondings = vec![];

        for unbonding in &resp_unboundings.unbonding_responses {
            for entry in &unbonding.entries {
                let balance_amount = self.string_amount_parser(entry.balance.clone(), None).await?;
                unbondings.push(InternalUnbonding {
                    address: unbonding.delegator_address.to_string(),
                    balance: balance_amount,
                    completion_time: &entry.completion_time.clone().ok_or("no completion time".to_string())?.nanos / 1_000_000,
                    // completion_time: DateTime::parse_from_rfc3339(&entry.completion_time)
                        // .map_err(|_| format!("Cannot parse unbonding delegation completion datetime, '{}'.", entry.completion_time))?
                        // .timestamp_millis(),
                })
            }
        }

        let resp = ListDbResult {
            data: unbondings,
            pagination: resp_unboundings.pagination.unwrap_or_default().into(),
        };

        Ok(resp)
    }

    /// Returns the redelegations to given validator address.
    pub async fn get_validator_redelegations(
        &self,
        validator_addr: &str,
        config: PaginationData,
        query_config: ValidatorRedelegationQuery,
    ) -> Result<ListDbResult<InternalRedelegation>, String> {
        use crate::fetch::cosmos::tx::v1beta1::{GetTxsEventRequest, GetBlockWithTxsResponse, service_client::ServiceClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        
        let is_destination = query_config.destination.unwrap_or(false);
        let is_source = query_config.source.unwrap_or(false);

        let mut events = vec![];

        if is_source {
            events.push(format!("redelegate.source_validator='{}'", validator_addr));
        };

        if is_destination {
            events.push(format!("redelegate.destination_validator='{}'", validator_addr));
        };

        let order_by = OrderBy::Unspecified;

        let limit = config.limit.unwrap_or_else(|| 50);
        let page = config.offset.map(|o| o / limit).unwrap_or_else(|| 1);
        let req = GetTxsEventRequest {
            events,
            pagination: None,
            order_by: todo!(),
            page,
            limit,
            query: todo!(),
        };



        // query.push(("message.action", "'/cosmos.staking.v1beta1.MsgBeginRedelegate'".to_string()));

        let order_by_black_list = vec!["evmos", "umee", "kyve", "quicksilver"];

        if !order_by_black_list.contains(&self.config.name.as_str()) {
            // query.push(("order_by", "ORDER_BY_DESC".to_string()));
        };

        // let resp = self.rest_api_request::<TxsResp>("/cosmos/tx/v1beta1/txs", &query).await?;

        let mut redelegations = vec![];

        // for i in 0..resp.txs.len() {
        //     let (tx, tx_response) = (
        //         resp.txs
        //             .get(i)
        //             .ok_or_else(|| "The count of transactions and transaction responses aren't the same.".to_string())?,
        //         resp.tx_responses
        //             .get(i)
        //             .ok_or_else(|| "The count of transactions and transaction responses aren't the same.".to_string())?,
        //     );
        //
        //     redelegations.push(InternalRedelegation::new(tx, tx_response, self).await?)
        // }

        // let pages = calc_pages(resp.pagination.unwrap_or(Pagination::default()), config)?;

        Ok(ListDbResult {
            data: vec![],
            pagination: Default::default(),
        });
        // Ok(todo!())
        // Ok(OutRestResponse::new(redelegations, pages))
    }

    /// Returns validator info by given validator address.
    pub async fn get_validator_info(&self, validator_addr: &str) -> Result<InternalValidator, String> {
        let path = format!("/cosmos/staking/v1beta1/validators/{validator_addr}");

        let (resp, bonded_height, staking_pool_resp) = join!(
            self.rest_api_request::<ValidatorResp>(&path, &[]),
            self.get_validator_bonded_height(validator_addr),
            self.get_staking_pool()
        );

        let bonded_height = bonded_height.unwrap_or(0);
        let validator = resp?.validator;
        let bonded_tokens = staking_pool_resp?.value.bonded as f64;

        let validator_metadata = self.database.find_validator_by_operator_addr(&validator.operator_address.clone()).await?;

        let delegator_shares = self.format_delegator_share(&validator.delegator_shares);
        let bonded_staking_poll_tnr = TnrDecimal::from_f64(bonded_tokens).unwrap_or_default();
        let voting_power_percentage = delegator_shares.div(bonded_staking_poll_tnr).to_f64().unwrap_or(0.0);

        let voting_power_change_24h = self
            .get_validator_voting_power_percentage_change(&validator.operator_address, Duration::hours(24), voting_power_percentage)
            .await
            .unwrap_or(0.0);

        let consensus_address =
            convert_consensus_pubkey_to_consensus_address(&validator.consensus_pubkey.key, &format!("{}valcons", self.config.base_prefix));
        let val_status_enum = self.get_validator_status(&validator, &consensus_address).await?;
        let uptime = self.get_validator_uptime(&consensus_address, Some(val_status_enum.clone())).await?;
        let status = val_status_enum.as_str().to_string();
        let validator = InternalValidator {
            logo_url: validator_metadata.logo_url,
            commission: validator
                .commission
                .commission_rates
                .rate
                .parse()
                .map_err(|_| format!("Cannot parse commission rate, '{}'.", validator.commission.commission_rates.rate))?,
            max_commission: validator
                .commission
                .commission_rates
                .max_rate
                .parse()
                .map_err(|_| format!("Cannot parse maximum commission rate, '{}'.", validator.commission.commission_rates.rate))?,
            self_delegation_amount: validator_metadata.self_delegation_amount.unwrap_or(0.0),
            self_delegate_address: self
                .convert_valoper_to_self_delegate_address(&validator.operator_address)
                .ok_or_else(|| format!("Cannot parse self delegate address, {}.", validator.operator_address))?,
            operator_address: validator.operator_address,
            name: validator.description.moniker,
            website: validator.description.website,
            details: validator.description.details,
            voting_power: delegator_shares.to_u64().unwrap_or(0),
            status,
            uptime,
            consensus_address,
            bonded_height,
            voting_power_percentage,
            voting_power_change_24h,
        };

        Ok(validator)
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

    /// Returns the list of validators with unbonding status.
    pub async fn get_validators_unbonding(&self, pagination_config: PaginationConfig) -> Result<ValidatorListApiResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_UNBONDING".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }

    /// Returns the list of validators with unspecified status.
    pub async fn get_validators_unspecified(&self, pagination_config: PaginationConfig) -> Result<ValidatorListApiResp, String> {
        let mut query = vec![];

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

    /// Returns the latest validator set.
    pub async fn get_validator_set(&self) -> Result<OutRestResponse<Vec<ValidatorSetValidator>>, String> {
        let config = PaginationConfig::new().limit(100);

        let mut first_resp = self._get_validator_set(config).await?;

        let mut validator_set = vec![];

        validator_set.append(&mut first_resp.value);

        let pages_to_request = first_resp.pages;

        let mut jobs = vec![];

        for page in 2..=pages_to_request {
            jobs.push(self._get_validator_set(config.page(page)))
        }

        let resps = join_all(jobs).await;

        for resp in resps {
            let mut validators = resp?.value;

            validator_set.append(&mut validators)
        }

        Ok(OutRestResponse::new(validator_set, 0))
    }

    /// Returns the latest validator set.
    async fn _get_validator_set(&self, config: PaginationConfig) -> Result<OutRestResponse<Vec<ValidatorSetValidator>>, String> {
        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self
            .rest_api_request::<ValidatorSetResp>("/cosmos/base/tendermint/v1beta1/validatorsets/latest", &query)
            .await?;

        let pages = calc_pages(resp.pagination, config)?;

        let validators = resp.validators;

        Ok(OutRestResponse::new(validators, pages))
    }

    /// Returns the validator set at given height.
    pub async fn get_validator_set_by_height(&self, height: u64) -> Result<OutRestResponse<Vec<ValidatorSetValidator>>, String> {
        let config = PaginationConfig::new().limit(100);

        let mut first_resp = self._get_validator_set_by_height(height, config).await?;

        let mut validator_set = vec![];

        validator_set.append(&mut first_resp.value);

        let pages_to_request = first_resp.pages;

        let mut jobs = vec![];

        // tracing!("{}", pages_to_request);

        for page in 2..=pages_to_request {
            jobs.push(self._get_validator_set(config.page(page)))
        }

        let resps = join_all(jobs).await;

        for resp in resps {
            let mut validators = resp?.value;

            validator_set.append(&mut validators)
        }

        Ok(OutRestResponse::new(validator_set, 0))
    }

    /// Returns the validator set at given height.
    pub async fn _get_validator_set_by_height(
        &self,
        height: u64,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<ValidatorSetValidator>>, String> {
        let path = format!("/cosmos/base/tendermint/v1beta1/validatorsets/{height}");
        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<ValidatorSetResp>(&path, &query).await?;

        let validators = resp.validators;

        // println!("{}", resp.pagination.total.clone());
        let pages = calc_pages(resp.pagination, config)?;

        Ok(OutRestResponse::new(validators, pages))
    }

    /// Returns the validator set at given height.
    async fn get_validator_bonded_height(&self, valoper_addr: &str) -> Result<u64, String> {
        let mut query = vec![];
        let default_bonded_height = "0";

        query.push(("events", format!("create_validator.validator='{}'", valoper_addr)));
        query.push(("pagination.reverse", format!("{}", true)));
        query.push(("pagination.limit", 1.to_string()));

        let resp = self.rest_api_request::<TxsResp>("/cosmos/tx/v1beta1/txs", &query).await?;

        let bonded_height_str = match resp.tx_responses.get(0) {
            Some(tx) => tx.height.clone(),
            None => default_bonded_height.into(),
        };

        let bonded_height = bonded_height_str
            .parse::<u64>()
            .map_err(|_| format!("Cannot parse bonded height, {}.", bonded_height_str))?;

        Ok(bonded_height)
    }

    pub async fn get_validator_uptime(&self, consensus_address: &str, val_status: Option<ValidatorStatus>) -> Result<f64, String> {
        let default_uptime_value = 0.0;

        if val_status.unwrap() != ValidatorStatus::Active {
            return Ok(default_uptime_value);
        }

        let (val_signing_info_resp, slashing_params) = join!(self.get_validator_signing_info(consensus_address), self.get_slashing_params());

        let val_signing_info = val_signing_info_resp?.value;
        let slashing_params = slashing_params?.value;

        Ok(1.0 - (val_signing_info.missed_blocks_counter as f64 / slashing_params.signed_blocks_window as f64))
    }

    pub async fn get_validator_status(&self, validator: &ValidatorListValidator, consensus_address: &str) -> Result<ValidatorStatus, String> {
        let signing_info = self.get_validator_signing_info(consensus_address).await?;

        let status = if validator.jailed {
            ValidatorStatus::Jailed
        } else if validator.status == "BOND_STATUS_UNBONDED" {
            ValidatorStatus::Inactive
        } else if signing_info.value.tombstoned {
            ValidatorStatus::Tombstoned
        } else {
            ValidatorStatus::Active
        };

        Ok(status)
    }

    pub async fn get_validator_voting_power_percentage_change(
        &self,
        validator_operator_address: &str,
        period: Duration,
        current_voting_power_percentage: f64,
    ) -> Result<f64, String> {
        let voting_powers_history = match self.database.find_historical_data_by_operator_address(validator_operator_address).await {
            Ok(voting_power_db) => voting_power_db.voting_power_data,
            Err(err) => return Err(err),
        };

        let period_millis = period.num_milliseconds();
        let lower_constraint_timestamp = Utc::now().timestamp_millis() - period_millis;

        let mut potential_voting_power: Option<f64> = None;

        for voting_power in voting_powers_history {
            if voting_power.ts < lower_constraint_timestamp {
                potential_voting_power = Some(voting_power.voting_power);
            } else {
                break;
            };
        }

        match potential_voting_power {
            Some(value) => {
                let bonded_tokens = self.get_staking_pool().await?.value.bonded as f64;
                Ok(((value / bonded_tokens) * 100.0) - current_voting_power_percentage)
            }
            None => Err("Could not calculate voting power change".to_string()),
        }
    }

    pub async fn get_validator_voter_address(&self, operator_address: &String) -> Result<Option<String>, TNRAppError> {
        let mut result = None;
        if self.config.name != "axelar" {
            return Ok(result);
        };

        if let Ok(res) = self.database.find_validator(doc! {"operator_address": &operator_address}).await {
            if let Some(res) = res.voter_address {
                return Ok(Some(res));
            }
        };

        let mut query = vec![];
        query.push(("events", format!("message.sender='{}'", operator_address)));
        query.push(("events", format!("message.action='{}'", "RegisterProxy")));
        let resp = self.archive_api_request::<TxsResp>("/cosmos/tx/v1beta1/txs", &query).await?;

        for tx in resp.txs.iter() {
            for message in &tx.body.messages {
                let res = message.clone().to_internal(self, &None).await?;
                if let InternalTransactionContent::Known(InternalTransactionContentKnowns::RegisterProxy { sender: _, proxy_addr }) = res {
                    result = Some(proxy_addr);
                };
            }
        }

        Ok(result)
    }

    //Self delegations of validator
    pub async fn get_val_self_delegations(&self, operator_address: String) -> Result<InternalDelegation, String> {
        let val = self.database.find_validator(doc! {"operator_address": operator_address.clone()}).await?;
        let self_delegate_address = val.self_delegate_address;
        let path = format!("/cosmos/staking/v1beta1/validators/{operator_address}/delegations/{self_delegate_address}");

        let resp = self.rest_api_request::<SelfDelagationResp>(&path, &[]).await?;
        let amount = self.string_amount_parser(resp.delegation_response.delegation.shares, None).await?;
        let internal_delegation = InternalDelegation {
            address: self_delegate_address,
            amount,
        };
        Ok(internal_delegation)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorSetResp {
    pub validators: Vec<ValidatorSetValidator>,
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorSetValidator {
    /// Validator address. Eg: `"cosmosvalcons14sk4vptumprktehmuvvf0yynarjy4gv08t64t4""`
    pub address: String,
    /// Public key.
    pub pub_key: ValidatorSetPubKey,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorSetPubKey {
    /// Validator pubkey. Eg: `"LtiHVLCcE+oFII0vpIl9mfkGDmk9BpPg1eUkvKnO4xw=""`
    pub key: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalUnbonding {
    pub address: String,
    pub balance: ChainAmountItem,
    pub completion_time: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorUnbondingsResp {
    /// Array of delegation responses.
    pub unbonding_responses: Vec<ValidatorUnbonding>,
    /// Pagination.
    pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorUnbonding {
    /// Delegator address. Eg: `"evmos1q9f3hdrm5fmllf53ne5yxcytjmhelpyuf06vtj"`
    delegator_address: String,
    /// Delegator address. Eg: `"evmosvaloper1zwr06uz8vrwkcnd05e5yddamvghn93a4hsyewa"`
    validator_address: String,
    /// Entries.
    entries: Vec<ValidatorUnbondingEntry>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorUnbondingEntry {
    /// Unbonding entry creation height. Eg: `"6883291"`
    creation_height: String,
    /// Unbonding entry completion time. Eg: `"2022-11-22T16:06:08.996987184Z"`
    completion_time: String,
    /// Unbonding entry initial balance. Eg: `"300000000000000000"`
    initial_balance: String,
    /// Unbonding entry initial balance. Eg: `"300000000000000000"`
    balance: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalDelegation {
    pub address: String,
    pub amount: ChainAmountItem,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorDelegationsResp {
    /// Array of delegation responses.
    pub delegation_responses: Vec<ValidatorDelegation>,
    /// Pagination.
    pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorDelegation {
    /// Delegation.
    delegation: ValidatorDelegationDelegation,
    /// Balance.
    balance: DenomAmount,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorDelegationDelegation {
    /// Delegator address. Eg: `"evmos1q9f3hdrm5fmllf53ne5yxcytjmhelpyuf06vtj"`
    delegator_address: String,
    /// Delegator address. Eg: `"evmosvaloper1zwr06uz8vrwkcnd05e5yddamvghn93a4hsyewa"`
    validator_address: String,
    /// Delegation shares. Eg: `"300000000000000000.000000000000000000"`
    shares: String,
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
pub struct ValidatorListApiResp {
    /// Array of validators.
    pub validators: Vec<ValidatorListValidator>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorListDbResp {
    /// Array of validators.
    pub validators: Vec<ValidatorForDb>,
    /// Pagination.
    pub pagination: PaginationData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorListResp(Vec<ValidatorListElement>);

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorListElement {
    pub rank: u64,
    pub moniker: String,
    pub voting_power: u64,
    pub voting_power_ratio: f64,
    pub cumulative_share: f64,
    pub account_address: String,
    pub operator_address: String,
    pub consensus_address: String,
    pub uptime: f64,
    pub missed_29k: u16,
    pub logo_url: String,
    pub validator_commissions: ValidatorListElementValidatorCommission,
}

impl ValidatorListResp {
    pub async fn from_db_list(other: ListDbResult<ValidatorForDb>, chain: &Chain) -> Result<Self, TNRAppError> {
        let staking_pool_resp = chain.get_staking_pool().await?.value;
        let bonded_token = staking_pool_resp.bonded;
        let mut validators = vec![];

        for v in other.data.iter() {
            let cumulative_bonded_tokens = v.cumulative_bonded_tokens.unwrap_or(0.0);
            let cumulative_share = (cumulative_bonded_tokens / bonded_token as f64) / 10000.0;
            let missed_29k = 0;
            if v.is_active {
                //WARNING This request takes too much time can turn to a cron job
                // missed_29k = chain.get_validator_signing_info(&v.consensus_address).await?.value.missed_blocks_counter;
            };

            validators.push(ValidatorListElement {
                missed_29k,
                validator_commissions: ValidatorListElementValidatorCommission::from_db(v.validator_commissions.clone()),
                moniker: v.name.clone(),
                rank: v.rank,
                cumulative_share,
                voting_power: v.voting_power,
                voting_power_ratio: v.voting_power_ratio,
                uptime: v.uptime,
                logo_url: v.logo_url.clone(),
                account_address: v.self_delegate_address.clone(),
                operator_address: v.operator_address.clone(),
                consensus_address: v.consensus_address.clone(),
            })
        }

        Ok(Self(validators))
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorListElementValidatorCommission {
    pub commission_rates: ValidatorListElementValidatorCommissionRates,
    /// Validator commission update time. Eg: `"2022-03-02T19:00:00Z"`
    pub update_time: String,
}

impl ValidatorListElementValidatorCommission {
    pub fn from_db(validator_commission: ValidatorListValidatorCommission) -> Self {
        Self {
            commission_rates: ValidatorListElementValidatorCommissionRates::from_db(validator_commission.commission_rates),
            update_time: validator_commission.update_time,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorListElementValidatorCommissionRates {
    pub rate: f64,
    pub max_rate: f64,
    pub max_change_rate: f64,
}

impl ValidatorListElementValidatorCommissionRates {
    pub fn from_db(validator_commission_rates: ValidatorListValidatorCommissionRates) -> Self {
        let default_value = 0.0;
        let rate = validator_commission_rates.rate.parse::<f64>().unwrap_or(default_value);
        let max_rate = validator_commission_rates.max_rate.parse::<f64>().unwrap_or(default_value);
        let max_change_rate = validator_commission_rates.max_change_rate.parse::<f64>().unwrap_or(default_value);
        Self {
            rate,
            max_rate,
            max_change_rate,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ConsensusPubkey {
    /// Type of public key. Eg: `"/cosmos.crypto.secp256k1.PubKey"`
    #[serde(rename = "@type")]
    pub key_type: String,
    /// Consensus public key. Eg: `"zy/GxGwk1Pm3HiG67iani1u+MUieM98ZvSIrXC8mISE="`
    pub key: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorListValidator {
    /// Operator address. Eg: `"evmosvaloper1qq95x6dhrdnrfunlth5uh24tkrfphzl9crd3xr"`
    pub operator_address: String,
    /// Consensus public key.
    pub consensus_pubkey: ConsensusPubkey,
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
pub struct InternalValidator {
    logo_url: String,
    commission: f64,
    uptime: f64,
    max_commission: f64,
    operator_address: String,
    consensus_address: String,
    name: String,
    website: String,
    self_delegation_amount: f64,
    self_delegate_address: String,
    details: String,
    voting_power_percentage: f64,
    voting_power: u64,
    bonded_height: u64,
    voting_power_change_24h: f64,
    status: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct ValidatorListValidatorCommission {
    /// Validator commission rates.
    pub commission_rates: ValidatorListValidatorCommissionRates,
    /// Validator commission update time. Eg: `"2022-03-02T19:00:00Z"`
    pub update_time: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
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

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalRedelegation {
    pub amount: ChainAmountItem,
    pub completion_time: i64,
    pub delegator_address: String,
    pub validator_to_address: String,
    pub validator_to_logo_url: String,
    pub validator_to_name: String,
}

impl InternalRedelegation {
    pub async fn new(tx: &Tx, tx_response: &TxResponse, chain: &Chain) -> Result<Self, String> {
        let (delegator_address, validator_dst_address, amount) = match tx.body.messages.get(0) {
            Some(TxsTransactionMessage::Known(TxsTransactionMessageKnowns::Redelegate {
                delegator_address,
                validator_src_address: _,
                validator_dst_address,
                amount,
            })) => (delegator_address.clone(), validator_dst_address.clone(), amount),
            _ => return Err(format!("Tx doesn't have a redelegation message, {}.", tx_response.txhash)),
        };

        let validator_to_metadata = chain.database.find_validator_by_operator_addr(&validator_dst_address).await?;
        let amount = chain.string_amount_parser(amount.amount.clone(), Some(amount.denom.clone())).await?;

        Ok(Self {
            amount,
            completion_time: match tx_response.logs.get(0) {
                Some(log) => match log.events.iter().find(|event| event.r#type == "redelegate") {
                    Some(event) => match event.attributes.iter().find(|attr| attr.key == "completion_time") {
                        Some(attr) => match DateTime::parse_from_rfc3339(&attr.value) {
                            Ok(date_time) => {
                                let ts = date_time.timestamp_millis();
                                if ts < 0 {
                                    0
                                } else {
                                    ts
                                }
                            }
                            _ => return Err(format!("Cannot parse datetime, {}.", attr.value)),
                        },
                        _ => {
                            return Err(format!(
                                "Tx redelagate event log doesn't have `completion_time` attribute, {}.",
                                tx_response.txhash
                            ));
                        }
                    },
                    _ => return Err(format!("Tx doesn't have a redelagate event log, {}.", tx_response.txhash)),
                },
                _ => return Err(format!("Tx doesn't have a log, {}.", tx_response.txhash)),
            },
            validator_to_address: validator_to_metadata.operator_address,
            validator_to_logo_url: validator_to_metadata.logo_url,
            validator_to_name: validator_to_metadata.name,
            delegator_address,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SlashingSigningInfo {
    pub info: Vec<SlashingSigningInfoItem>,
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SlashingSigningInfoItem {
    /// Validator address. Eg: `"evmosvalcons1qx4hehfny66jfzymzn6d5t38m0ely3cvw6zn06"`
    pub address: String,
    /// The block height slashing is started at. Eg: `"0"`
    pub start_height: String,
    /// Unknown. Eg: `"5888077"`
    pub index_offset: String,
    /// The time jailed until. Eg: `"2022-05-14T04:31:49.705643236Z"`
    pub jailed_until: String,
    /// Tombstoned state. Eg: `false`
    pub tombstoned: bool,
    /// The count of missed blocks. Eg: `"16433"`
    pub missed_blocks_counter: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalSlashingSigningInfoItem {
    /// Validator address. Eg: `"evmosvalcons1qx4hehfny66jfzymzn6d5t38m0ely3cvw6zn06"`
    pub address: String,
    /// The block height slashing is started at. Eg: `0`
    pub start_height: i64,
    /// Unknown. Eg: `5888077`
    pub index_offset: i64,
    /// The timestamp in milliseconds jailed until.
    pub jailed_until: i64,
    /// Tombstoned state. Eg: `false`
    pub tombstoned: bool,
    /// The count of missed blocks. Eg: `16433`
    pub missed_blocks_counter: i64,
}


impl TryFrom<QuerySigningInfoResponse> for InternalSlashingSigningInfoItem {
    type Error = String;
    fn try_from(value: QuerySigningInfoResponse) -> Result<Self, Self::Error> {
        let value = value.val_signing_info.unwrap();
        Ok(Self {
            address: value.address,
            start_height: value.start_height,
            index_offset: value.start_height,
            jailed_until: value.jailed_until.map(|ju| ju.nanos / 1_000_0000).ok_or_else(|| "Cannot parse jailed until datetime")?.into(),
            tombstoned: value.tombstoned,
            missed_blocks_counter: value.missed_blocks_counter
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SigningInfoResp {
    /// Validator signing info.
    pub val_signing_info: SlashingSigningInfoItem,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub enum ValidatorStatus {
    Active,
    Inactive,
    Jailed,
    Tombstoned,
    Unknown(String),
}

impl ValidatorStatus {
    fn as_str(&self) -> &str {
        match self {
            ValidatorStatus::Active => "Active",
            ValidatorStatus::Inactive => "Inactive",
            ValidatorStatus::Jailed => "Jailed",
            ValidatorStatus::Tombstoned => "Tombstoned",
            ValidatorStatus::Unknown(unknown_string) => unknown_string,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ValidatorRedelegationQuery {
    pub source: Option<bool>,
    pub destination: Option<bool>,
}

impl ValidatorRedelegationQuery {
    pub fn validate(&self) -> Result<(), String> {
        let error_message = Err(String::from("Please specify only one validator address type at once"));
        let is_source = self.source.unwrap_or(false);
        let is_destination = self.destination.unwrap_or(false);
        if (is_source && is_destination) || (!is_source && !is_destination) {
            error_message
        } else {
            Ok(())
        }
    }
}
