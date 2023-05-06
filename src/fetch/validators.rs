use std::io::SeekFrom::End;
use std::ops::{Div, Rem};
use base64::Engine;

use chrono::{DateTime, Duration, ParseResult, Utc};
use cosmrs::proto::cosmos::tx::v1beta1::GetTxsEventRequest;
use futures::future::join_all;
use mongodb::bson::doc;
use prost::Message;
use prost_wkt_types::Any;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use tokio::join;
use tonic::transport::Endpoint;

use crate::database::{ListDbResult, PaginatedListResult, ValidatorForDb};
use crate::fetch::cosmos::tx::v1beta1::{OrderBy, service_client};
use crate::fetch::transactions::{InternalTransactionContent, InternalTransactionContentKnowns};
use crate::routes::ChainAmountItem;
use crate::routes::{PaginationData, TNRAppError};
use crate::utils::{convert_consensus_pubkey_to_consensus_address, get_key};
use crate::{chain::Chain, routes::{calc_pages, OutRestResponse}, utils};
use crate::fetch::cosmos::auth::v1beta1::query_client::QueryClient;
use crate::fetch::cosmos::base::query::v1beta1::PageRequest;

use crate::fetch::cosmos::slashing::v1beta1::QuerySigningInfoResponse;

use crate::fetch::cosmos::tx::v1beta1::Tx as GrpcTx;
use crate::fetch::cosmos::base::abci::v1beta1::{AbciMessageLog, TxResponse as GrpcTxResponse};
use crate::fetch::cosmos::staking::v1beta1::{Commission, Description, Validator};
use crate::fetch::PaginationResponse;

use super::amount_util::TnrDecimal;
use super::delegations::SelfDelagationResp;
use super::{
    others::{DenomAmount, Pagination, PaginationConfig},
    transactions::{Tx, TxResponse, TxsResp, TxsTransactionMessage, TxsTransactionMessageKnowns},
};
use base64::engine::general_purpose::STANDARD;

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
        use crate::fetch::cosmos::tx::v1beta1::{GetTxsEventRequest, GetTxsEventResponse, service_client::ServiceClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();


        let is_destination = query_config.destination.unwrap_or(false);
        let is_source = query_config.source.unwrap_or(false);

        let mut querys = vec![];

        if is_source {
            querys.push(format!("redelegate.source_validator='{}'", validator_addr));
        };

        if is_destination {
            querys.push(format!("redelegate.source_validator='{}'", validator_addr));
        };

        querys.push("message.action='/cosmos.staking.v1beta1.MsgBeginRedelegate'".to_string());

        let order_by_black_list = vec!["evmos", "umee", "kyve", "quicksilver"];

        let order_by = if !order_by_black_list.contains(&self.config.name.as_str()) {
            OrderBy::Desc
        } else {
            OrderBy::Unspecified
        };

        let query = querys.join(",");

        let limit = config.limit.unwrap_or_else(|| 50);
        let page = config.offset.map(|o| o / limit).unwrap_or_else(|| 1);

        let req = GetTxsEventRequest {
            events: vec![],
            pagination: None,
            order_by: order_by as i32,
            page,
            limit,
            query,
        };


        let resp = ServiceClient::connect(endpoint)
            .await
            .unwrap()
            .get_txs_event(req)
            .await
            .map_err(|e| format!("{}", e))?;

        let txs = resp.into_inner();


        for i in 0..txs.txs.len() {
            let tx = txs
                .txs
                .get(i)
                .ok_or_else(|| "The count of transactions and transaction responses aren't same".to_string())?;

            let tx_response = txs
                .tx_responses
                .get(i)
                .ok_or_else(|| "The count of transactions and transaction responses aren't same".to_string())?;
        }

        let mut redelegations = vec![];

        for (tx, tx_response) in txs.txs.iter().zip(txs.tx_responses.iter()) {
            redelegations.push(InternalRedelegation::from_tx(tx, tx_response, &self));
        }

        Ok(ListDbResult {
            data: vec![],
            pagination: Default::default(),
        })
    }

    /// Returns validator info by given validator address.
    pub async fn get_validator_info(&self, validator_addr: &str) -> Result<InternalValidator, String> {
        use crate::fetch::cosmos::staking::v1beta1::{QueryValidatorRequest, QueryValidatorResponse, query_client::QueryClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryValidatorRequest {
            validator_addr: validator_addr.to_string(),
        };

        let mut client = QueryClient::connect(endpoint)
            .await
            .unwrap();

        let (resp, bonded_height, staking_pool_resp) = join!(
            client.validator(req),
            self.get_validator_bonded_height(validator_addr),
            self.get_staking_pool()
        );

        let bonded_height = bonded_height.unwrap_or(0);
        let validator = resp.map_err(|e| format!("{}", e))?.into_inner().validator.ok_or(format!("No validator data on response: {}", validator_addr))?;
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
            convert_consensus_pubkey_to_consensus_address(utils::get_key(validator.consensus_pubkey.clone().unwrap()).map_err(|e| format!("{e}"))?.as_str(), &format!("{}valcons", self.config.base_prefix));

        let val_status_enum = self.get_validator_status(&validator, &consensus_address).await?;
        let uptime = self.get_validator_uptime(&consensus_address, Some(val_status_enum.clone())).await?;
        let status = val_status_enum.as_str().to_string();

        let commission_rates = validator.commission.unwrap().commission_rates.unwrap();
        let validator = InternalValidator {
            logo_url: validator_metadata.logo_url,
            commission: commission_rates
                .rate
                .parse()
                .map_err(|_| format!("Cannot parse commission rate, '{}'.", commission_rates.rate))?,
            max_commission: commission_rates
                .max_rate
                .parse()
                .map_err(|_| format!("Cannot parse maximum commission rate, '{}'.", commission_rates.rate))?,
            self_delegation_amount: validator_metadata.self_delegation_amount.unwrap_or(0.0),
            self_delegate_address: self
                .convert_valoper_to_self_delegate_address(&validator.operator_address)
                .ok_or_else(|| format!("Cannot parse self delegate address, {}.", validator.operator_address))?,
            operator_address: validator.operator_address,
            name: validator.description.clone().unwrap().moniker,
            website: validator.description.clone().unwrap().website,
            details: validator.description.clone().unwrap().details,
            voting_power: delegator_shares.clone().to_u64().unwrap_or(0),
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
    pub async fn get_validators_by_delegator(&self, delegator_addr: &str, pagination: PaginationData) -> Result<ListDbResult<ValidatorListValidator>, String> {
        use crate::fetch::cosmos::staking::v1beta1::{QueryDelegatorValidatorsRequest, QueryValidatorsResponse, query_client::QueryClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryDelegatorValidatorsRequest {
            delegator_addr: delegator_addr.to_string(),
            pagination: Some(pagination.into()),
        };


        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .delegator_validators(req)
            .await
            .map_err(|e| format!("{e}"))?
            .into_inner();

        let validators = resp.validators.into_iter().map(|v| v.into()).collect();

        Ok(ListDbResult {
            data: validators,
            pagination: resp.pagination.unwrap_or_default().into(),
        })
    }

    /// Returns accumulated commission of given validator.
    pub async fn get_validator_commission(&self, validator_addr: &str) -> Result<ValidatorCommisionResp, String> {
        use crate::fetch::cosmos::distribution::v1beta1::{QueryValidatorCommissionRequest, QueryValidatorCommissionResponse, query_client::QueryClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req= QueryValidatorCommissionRequest {
            validator_address: validator_addr.to_string()
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .validator_commission(req)
            .await
            .map_err(|e| format!("{e}"))?
            .into_inner();


        let commission = resp.commission.map(|c| ValidatorCommision {
            commission: c.commission.into_iter().map(|c| DenomAmount {
                denom: c.denom,
                amount: c.amount,
            }).collect(),
        }).unwrap();


        Ok(ValidatorCommisionResp {
            commission
        })
    }

    /// Returns rewards of given validator.
    pub async fn get_validator_rewards(&self, validator_addr: &str) -> Result<ValidatorRewardsResp, String> {
        use crate::fetch::cosmos::distribution::v1beta1::{QueryValidatorOutstandingRewardsRequest, QueryValidatorOutstandingRewardsResponse, query_client::QueryClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryValidatorOutstandingRewardsRequest { validator_address: validator_addr.to_string() };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .validator_outstanding_rewards(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();


        let resp = resp.rewards.map(|r| r.rewards.into_iter().map(|c| {
            DenomAmount {
                denom: c.denom,
                amount: c.amount,
            }
        }).collect()).unwrap();

        let resp = ValidatorRewardsResp { rewards: ValidatorCommision { commission: resp } };


        Ok(resp)
    }

    /// Returns the list of validators with unbonding status.
    pub async fn get_validators_unbonding(&self, pagination_config: PaginationData) -> Result<ListDbResult<ValidatorListValidator>, String> {
        use crate::fetch::cosmos::staking::v1beta1::{QueryValidatorsRequest, QueryValidatorsResponse, query_client::QueryClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryValidatorsRequest {
            status: "BOND_STATUS_UNBONDING".to_string(),
            pagination: Some(pagination_config.into()),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .validators(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let validators = resp.validators.into_iter().map(|v| v.into()).collect();

        Ok(ListDbResult {
            data: validators,
            pagination: resp.pagination.unwrap_or_default().into(),
        })
    }

    /// Returns the list of validators with unspecified status.
    pub async fn get_validators_unspecified(&self, pagination_config: PaginationData) -> Result<ListDbResult<ValidatorListValidator>, String> {
        use crate::fetch::cosmos::staking::v1beta1::{QueryValidatorsRequest, QueryValidatorsResponse, query_client::QueryClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryValidatorsRequest {
            status: "".to_string(),
            pagination: Some(pagination_config.into()),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .validators(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let validators = resp.validators.into_iter().map(|v| v.into()).collect();

        Ok(ListDbResult {
            data: validators,
            pagination: resp.pagination.unwrap_or_default().into(),
        })
    }

    /// Returns validator information by given delegator validator pair.
    pub async fn get_delegator_validator_pair_info(&self, delegator_addr: &str, validator_addr: &str) -> Result<ValidatorResp, String> {
        use crate::fetch::cosmos::staking::v1beta1::{QueryDelegatorValidatorRequest, QueryValidatorResponse, query_client::QueryClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryDelegatorValidatorRequest {
            delegator_addr: delegator_addr.to_string(),
            validator_addr: validator_addr.to_string(),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .delegator_validator(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();


        Ok(ValidatorResp {
            validator: resp.validator.unwrap().into(),
        })
    }

    /// Returns the latest validator set.
    pub async fn get_validator_set(&self) -> Result<OutRestResponse<Vec<ValidatorSetValidator>>, String> {
        let limit = 100;

        let config = PaginationData {
            cursor: None,
            offset: None,
            limit: Some(limit),
            direction: None,
        };

        let mut first_resp = self._get_validator_set(config).await?;

        let mut validator_set = vec![];

        validator_set.append(&mut first_resp.data);

        let pages_to_request = first_resp.pagination.total / limit;

        let mut jobs = vec![];

        for page in 2..=pages_to_request {
            jobs.push(self._get_validator_set(PaginationData {
                cursor: None,
                offset: Some(page * limit),
                limit: Some(limit),
                direction: None,
            }))
        }

        let rem = (first_resp.pagination.total / limit).rem(limit);

        if rem > 0 {
            jobs.push(self._get_validator_set(PaginationData {
                cursor: None,
                offset: Some((pages_to_request + 1) * limit),
                limit: Some(rem),
                direction: None
            }))
        }

        let resps = join_all(jobs).await;

        for resp in resps {
            let mut validators = resp?.data;

            validator_set.append(&mut validators)
        }

        Ok(OutRestResponse::new(validator_set, 0))
    }

    /// Returns the latest validator set.
    async fn _get_validator_set(&self, page: PaginationData) -> Result<PaginatedListResult<ValidatorSetValidator>, String> {
        use crate::fetch::cosmos::base::tendermint::v1beta1::{GetLatestValidatorSetRequest, GetLatestValidatorSetResponse, service_client::ServiceClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = GetLatestValidatorSetRequest {
            pagination: Some(page.into()),
        };

        let resp = ServiceClient::connect(endpoint)
            .await
            .unwrap()
            .get_latest_validator_set(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();



        let validators = resp
            .validators
            .into_iter()
            .map(|v| ValidatorSetValidator {
                address: v.address,
                pub_key: ValidatorSetPubKey { key: get_key(v.pub_key.unwrap()).unwrap() },
            })
            .collect();


        Ok(PaginatedListResult {
            data: validators,
            pagination: resp.pagination.unwrap_or_default().into(),
        })
    }

    /// Returns the validator set at given height.
    pub async fn get_validator_set_by_height(&self, height: i64) -> Result<OutRestResponse<Vec<ValidatorSetValidator>>, String> {
        let limit = 100;

        let config = PaginationData {
            cursor: None,
            offset: None,
            limit: Some(limit),
            direction: None,
        };


        let mut first_resp = self._get_validator_set_by_height(height, config).await?;

        let mut validator_set = vec![];

        validator_set.append(&mut first_resp.data);


        let mut pages_to_request = first_resp.pagination.total / limit;

        let mut jobs = vec![];

        // tracing!("{}", pages_to_request);

        for page in 2..=pages_to_request {
            jobs.push(self._get_validator_set_by_height(height, PaginationData {
                cursor: None,
                offset: Some(page * limit),
                limit: Some(limit),
                direction: None,
            }))
        }

        let rem = (first_resp.pagination.total / limit).rem(limit);

        if rem > 0 {
            jobs.push(self._get_validator_set_by_height(height, PaginationData {
                cursor: None,
                offset: Some((pages_to_request + 1) * limit),
                limit: Some(rem),
                direction: None
            }))
        }


        let resps = join_all(jobs).await;

        for resp in resps {
            let mut validators = resp?.data;

            validator_set.append(&mut validators)
        }

        Ok(OutRestResponse::new(validator_set, 0))
    }

    /// Returns the validator set at given height.
    pub async fn _get_validator_set_by_height(
        &self,
        height: i64,
        config: PaginationData,
    ) -> Result<PaginatedListResult<ValidatorSetValidator>, String> {
        use crate::fetch::cosmos::base::tendermint::v1beta1::{GetValidatorSetByHeightRequest, GetValidatorSetByHeightResponse, service_client::ServiceClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = GetValidatorSetByHeightRequest {
            height,
            pagination: Some(config.into()),
        };

        let resp = ServiceClient::connect(endpoint)
            .await
            .unwrap()
            .get_validator_set_by_height(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let validators = resp.validators.into_iter().map(|v| ValidatorSetValidator {
            address: v.address,
            pub_key: ValidatorSetPubKey { key: get_key(v.pub_key.unwrap()).unwrap() },
        }).collect();

        Ok(PaginatedListResult {
            data: validators,
            pagination: resp.pagination.unwrap().into(),
        })
    }

    /// Returns the validator set at given height.
    async fn get_validator_bonded_height(&self, valoper_addr: &str) -> Result<i64, String> {
        use crate::fetch::cosmos::tx::v1beta1::{GetTxsEventRequest, GetTxsEventResponse, service_client::ServiceClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = GetTxsEventRequest {
            events: vec![],
            pagination: None,
            order_by: 2,
            page: 0,
            limit: 1,
            query: format!("create_validator.validator={}", valoper_addr),
        };

        let resp = ServiceClient::connect(endpoint)
            .await
            .unwrap()
            .get_txs_event(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let default_bonded_height = 0;

        let bonded_height = match resp.tx_responses.get(0) {
            Some(tx) => tx.height,
            None => default_bonded_height,
        };

        Ok(bonded_height)
    }

    pub async fn get_validator_uptime(&self, consensus_address: &str, val_status: Option<ValidatorStatus>) -> Result<f64, String> {
        let default_uptime_value = 0.0;

        if val_status.unwrap() != ValidatorStatus::Active {
            return Ok(default_uptime_value);
        }

        let (val_signing_info, slashing_params) = join!(self.get_validator_signing_info(consensus_address), self.get_slashing_params());

        let val_signing_info = val_signing_info?;

        let slashing_params = slashing_params?.value;

        Ok(1.0 - (val_signing_info.missed_blocks_counter as f64 / slashing_params.signed_blocks_window as f64))
    }

    pub async fn get_validator_status(&self, validator: &Validator, consensus_address: &str) -> Result<ValidatorStatus, String> {
        let signing_info = self.get_validator_signing_info(consensus_address).await?;

        let status = if validator.jailed {
            ValidatorStatus::Jailed
        } else if validator.status == 1 {
            ValidatorStatus::Inactive
        } else if signing_info.tombstoned {
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
        use crate::fetch::cosmos::tx::v1beta1::{GetTxsEventRequest, GetTxsEventResponse, service_client::ServiceClient};
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
        query.push(format!("message.sender='{}'", operator_address));
        query.push(format!("message.action='{}'", "RegisterProxy")); let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let query = query.join(",");

        let req = GetTxsEventRequest {
            events: vec![],
            pagination: None,
            order_by: 0,
            page: 0,
            limit: 0,
            query,
        };

        let resp = ServiceClient::connect(endpoint)
            .await
            .unwrap()
            .get_txs_event(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        for tx in resp.txs.iter() {
            for message in tx.body.clone().unwrap().messages {
                if message.type_url.as_str() == "/axelar.snapshot.v1beta1.RegisterProxyRequest" {
                    use crate::fetch::axelar::snapshot::v1beta1::RegisterProxyRequest;
                    let msg = RegisterProxyRequest::decode(message.value.as_slice()).unwrap();

                    result = Some(STANDARD.encode(msg.proxy_addr))

                }
            }
        }

        Ok(result)
    }

    //Self delegations of validator
    pub async fn get_val_self_delegations(&self, operator_address: String) -> Result<InternalDelegation, String> {
        let val = self.database.find_validator(doc! {"operator_address": operator_address.clone()}).await?;
        let self_delegate_address = val.self_delegate_address;

        use crate::fetch::cosmos::staking::v1beta1::{QueryDelegationRequest, QueryDelegationResponse, query_client::QueryClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryDelegationRequest {
            delegator_addr: operator_address,
            validator_addr: self_delegate_address.clone(),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .delegation(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let amount = self.string_amount_parser(resp.delegation_response.unwrap().delegation.unwrap().shares, None).await?;
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
    pub unbonding_height: i64,
    /// Unbonding time. Eg: `"2022-08-21T03:48:38.952541966Z"`
    pub unbonding_time: String,
    /// Validator commission.
    pub commission: ValidatorListValidatorCommission,
    /// Minimum self delegation. Eg: `"1"`
    pub min_self_delegation: String,
}

fn validator_status(status: i32) -> String {
    let status = match status {
        1 => "Unbonded",
        2 => "Unbonding",
        3 => "Bonded",
        _ => "Unspecified",
    };

    status.to_string()
}

fn to_rfc3339(timestamp: prost_wkt_types::Timestamp) -> String {
    let datetime: DateTime<Utc> = timestamp.into();
    datetime.to_rfc3339()
}

impl From<Description> for ValidatorListValidatorDescription {
    fn from(v: Description) -> Self {
        Self {
            moniker: v.moniker,
            identity: v.identity,
            website: v.website,
            security_contact: v.security_contact,
            details: v.details,
        }
    }
}

impl From<Commission> for ValidatorListValidatorCommission {
    fn from(v: Commission) -> Self {
        let cr = v.commission_rates.unwrap();
       Self {
           commission_rates: ValidatorListValidatorCommissionRates {
               rate: cr.rate,
               max_rate: cr.max_rate,
               max_change_rate: cr.max_change_rate,
           },
           update_time: to_rfc3339(v.update_time.unwrap()),
       }
    }
}

impl From<Validator> for ValidatorListValidator {
    fn from(v: Validator) -> Self {
        let c = v.commission.unwrap();
        let pubkey = v.consensus_pubkey.unwrap();
        Self {
            operator_address: v.operator_address,
            consensus_pubkey: ConsensusPubkey {
                key_type:pubkey.type_url.clone(),
                key: utils::get_key(pubkey).unwrap(),
            },
            jailed: v.jailed,
            status: validator_status(v.status),
            tokens: v.tokens,
            delegator_shares: v.delegator_shares,
            description: v.description.unwrap().into(),
            unbonding_height: v.unbonding_height,
            unbonding_time: to_rfc3339(v.unbonding_time.unwrap()),
            commission: c.into(),
            min_self_delegation: v.min_self_delegation,
        }
    }
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
    bonded_height: i64,
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

// #[serde(rename = "")]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Redelegate {
    /// Delegator address. Eg: `"evmos1a37y062zjspzrcaxhz76lskwnvm0xlsymdfgg0"`
    delegator_address: String,
    /// Source validator address. Eg: `"evmosvaloper1v4crs2adgcu2cdm2jxq07mw7ugzx0z4x6alxeg"`
    validator_src_address: String,
    /// Destination validator address. Eg: `"evmosvaloper1sp9frqwep52chwavv3xd776myy8gyyvkv5uysl"`
    validator_dst_address: String,
    /// Amount.
    amount: DenomAmount,
}

impl InternalRedelegation {
    pub async fn from_tx(tx: &GrpcTx, tx_response: &GrpcTxResponse, chain: &Chain) -> Result<Self, String> {
        let (delegator_address, validator_dst_address, amount) = match tx.body.clone().map(|b| b.messages.get(0).cloned()).flatten() {
            Some(message) => {
                if message.type_url == "/cosmos.staking.v1beta1.MsgBeginRedelegate" {
                    use crate::fetch::cosmos::staking::v1beta1::MsgBeginRedelegate;

                    let redelegate = MsgBeginRedelegate::decode(message.value.as_slice())
                        .map_err(|e| format!("Failed to decode MsgBeginRedelegate messaage: {}", e))?;

                    (redelegate.delegator_address, redelegate.validator_dst_address, redelegate.amount)
                } else {
                    return Err(format!("Message is not the correct type, {}", tx_response.txhash));
                }
            }
            None => return Err(format!("Tx doesn't have a redelegation message, {}.", tx_response.txhash))
        };

        let validator = chain.database.find_validator_by_operator_addr(&validator_dst_address).await?;
        let amount = amount.unwrap();
        let amount = chain.string_amount_parser(amount.amount.clone(), Some(amount.denom.clone())).await?;

        // let completion_time = match txhashsh_response.logs.get(0) {
        let Some(logs) = tx_response.logs.get(0) else {
            return Err(format!("Tx doesn't have a log, {}.", tx_response.txhash));
        };

        let Some(event) = logs.events.iter().find(|event| event.r#type == "redelegate") else {
            return Err(format!(
                "Tx redelagate event log doesn't have `completion_time` attribute, {}.",
                tx_response.txhash
            ));
        };

        let Some(attr) = event.attributes.iter().find(|attr| attr.key == "completion_time") else {
            return Err(format!("Tx doesn't have a redelagate event log, {}.", tx_response.txhash));
        };

        let completion_time = match DateTime::parse_from_rfc3339(&attr.value) {
            Ok(date_time) => {
                let ts = date_time.timestamp_millis();
                if ts < 0 {
                    0
                } else {
                    ts
                }
            }
            Err(e) => return Err(format!("Cannot parse datetetime {}: {}", attr.value, e))
        };

        Ok(Self {
            amount,
            completion_time,
            delegator_address,
            validator_to_address: validator.operator_address,
            validator_to_logo_url: validator.logo_url,
            validator_to_name: validator.name,
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
            missed_blocks_counter: value.missed_blocks_counter,
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
