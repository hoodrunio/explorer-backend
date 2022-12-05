use chrono::DateTime;
use serde::{Deserialize, Serialize};

use super::{
    others::{DenomAmount, Pagination, PaginationConfig},
    transactions::{Tx, TxResponse, TxsResp, TxsTransactionMessage, TxsTransactionMessageKnowns},
};
use crate::{
    chain::Chain,
    routes::{calc_pages, OutRestResponse},
};

impl Chain {
    /// Returns validator by given validator address.
    pub async fn get_validator(&self, validator_addr: &str) -> Result<OutRestResponse<ValidatorListValidator>, String> {
        let path = format!("/cosmos/staking/v1beta1/validators/{validator_addr}");

        match self.rest_api_request::<ValidatorResp>(&path, &[]).await {
            Ok(res) => Ok(OutRestResponse::new(res.validator, 0)),
            Err(error) => Err(error),
        }
    }

    /// Returns the delegations to given validator address.
    pub async fn get_validator_delegations(
        &self,
        validator_addr: &str,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalDelegation>>, String> {
        let path = format!("/cosmos/staking/v1beta1/validators/{validator_addr}/delegations");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<ValidatorDelegationsResp>(&path, &query).await?;

        let mut delegations = vec![];

        for delegation in resp.delegation_responses {
            delegations.push(InternalDelegation {
                address: delegation.delegation.delegator_address,
                amount: self.calc_amount_u128_to_f64(
                    delegation
                        .balance
                        .amount
                        .parse::<u128>()
                        .map_err(|_| format!("Cannot parse delegation balance, '{}'.", delegation.balance.amount))?,
                ),
            })
        }

        let pages = calc_pages(resp.pagination, config)?;

        Ok(OutRestResponse::new(delegations, pages))
    }

    /// Returns the unbonding delegations to given validator address.
    pub async fn get_validator_unbondings(
        &self,
        validator_addr: &str,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalUnbonding>>, String> {
        let path = format!("/cosmos/staking/v1beta1/validators/{validator_addr}/unbonding_delegations");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<ValidatorUnbondingsResp>(&path, &query).await?;

        let mut unbondings = vec![];

        for unbonding in &resp.unbonding_responses {
            for entry in &unbonding.entries {
                unbondings.push(InternalUnbonding {
                    address: unbonding.delegator_address.to_string(),
                    balance: self.calc_amount_u128_to_f64(
                        entry
                            .balance
                            .parse::<u128>()
                            .map_err(|_| format!("Cannot parse unbonding delegation balance, '{}'.", entry.balance))?,
                    ),
                    completion_time: DateTime::parse_from_rfc3339(&entry.completion_time)
                        .map_err(|_| format!("Cannot parse unbonding delegation completion datetime, '{}'.", entry.completion_time))?
                        .timestamp_millis(),
                })
            }
        }

        let pages = calc_pages(resp.pagination, config)?;

        Ok(OutRestResponse::new(unbondings, pages))
    }

    /// Returns the redelegations to given validator address.
    pub async fn get_validator_redelegations(
        &self,
        validator_addr: &str,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalRedelegation>>, String> {
        let mut query = vec![];
        query.push(("events", format!("redelegate.source_validator='{}'", validator_addr)));
        query.push(("message.action", "'/cosmos.staking.v1beta1.MsgBeginRedelegate'".to_string()));
        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));
        query.push(("order_by", "ORDER_BY_DESC".to_string()));

        let resp = self.rest_api_request::<TxsResp>("/cosmos/tx/v1beta1/txs", &query).await?;

        let mut redelegations = vec![];

        for i in 0..resp.txs.len() {
            let (tx, tx_response) = (
                resp.txs
                    .get(i)
                    .ok_or_else(|| "The count of transactions and transaction responses aren't the same.".to_string())?,
                resp.tx_responses
                    .get(i)
                    .ok_or_else(|| "The count of transactions and transaction responses aren't the same.".to_string())?,
            );

            redelegations.push(InternalRedelegation::new(tx, tx_response, self).await?)
        }

        let pages = calc_pages(resp.pagination, config)?;

        Ok(OutRestResponse::new(redelegations, pages))
    }

    /// Returns validator info by given validator address.
    pub async fn get_validator_info(&self, validator_addr: &str) -> Result<OutRestResponse<InternalValidator>, String> {
        let path = format!("/cosmos/staking/v1beta1/validators/{validator_addr}");

        let resp = self.rest_api_request::<ValidatorResp>(&path, &[]).await?;

        let validator = resp.validator;

        let validator = InternalValidator {
            logo_url: self
                .get_validator_metadata_by_valoper_addr(validator.operator_address.clone())
                .await?
                .logo_url,
            commission: validator
                .commission
                .commission_rates
                .rate
                .parse()
                .map_err(|_| format!("Cannot parse commission rate, '{}'.", validator.commission.commission_rates.rate))?,
            uptime: 0.0, // TODO!
            max_commission: validator
                .commission
                .commission_rates
                .max_rate
                .parse()
                .map_err(|_| format!("Cannot parse maximum commission rate, '{}'.", validator.commission.commission_rates.rate))?,
            operator_address: validator.operator_address,
            name: validator.description.moniker,
            website: validator.description.website,
            self_delegate_address: String::new(), // TODO!
            details: validator.description.details,
            voting_power_percentage: 0.0, // TODO!
            voting_power: 0,              // TODO!
            bonded_height: 0,             // TODO!
            change: 0.0,                  // TODO!
        };

        Ok(OutRestResponse::new(validator, 0))
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

    /// Returns the validator set at given height.
    pub async fn _get_validator_set(&self, height: u64) -> Result<ValidatorSetResp, String> {
        let path = format!("/validatorsets/{height}");
        self.rest_api_request::<ValidatorSetResp>(&path, &[]).await
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorSetResp {
    pub result: ValidatorSetRespResult,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorSetRespResult {
    /// Array of validators.
    pub validators: Vec<ValidatorSetValidator>,
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
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalUnbonding {
    pub address: String,
    pub balance: f64,
    pub completion_time: i64,
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
    address: String,
    amount: f64,
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
pub struct ValidatorListResp {
    /// Array of validators.
    pub validators: Vec<ValidatorListValidator>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ConsensusPubkey {
    // Consensus public key. Eg: `"zy/GxGwk1Pm3HiG67iani1u+MUieM98ZvSIrXC8mISE="`
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
    name: String,
    website: String,
    self_delegate_address: String,
    details: String,
    voting_power_percentage: f64,
    voting_power: u64,
    bonded_height: u64,
    change: f64,
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

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalRedelegation {
    pub amount: f64,
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
                validator_src_address,
                validator_dst_address,
                amount,
            })) => (delegator_address.clone(), validator_dst_address.clone(), amount),
            _ => return Err(format!("Tx doesn't have a redelegation message, {}.", tx_response.txhash)),
        };

        let validator_to_metadata = chain.get_validator_metadata_by_valoper_addr(validator_dst_address).await?;

        Ok(Self {
            amount: chain.calc_amount_u128_to_f64(
                amount
                    .amount
                    .parse()
                    .map_err(|_| format!("Cannot parse redelegation amount, {}.", amount.amount))?,
            ),
            completion_time: match tx_response.logs.get(0) {
                Some(log) => match log.events.iter().find(|event| event.r#type == "redelegate") {
                    Some(event) => match event.attributes.iter().find(|attr| attr.key == "completion_time") {
                        Some(attr) => match DateTime::parse_from_rfc3339(&attr.value) {
                            Ok(date_time) => date_time.timestamp_millis(),
                            _ => return Err(format!("Cannot parse datetime, {}.", attr.value)),
                        },
                        _ => {
                            return Err(format!(
                                "Tx redelagate event log doesn't have `completion_time` attribute, {}.",
                                tx_response.txhash
                            ))
                        }
                    },
                    _ => return Err(format!("Tx doesn't have a redelagate event log, {}.", tx_response.txhash)),
                },
                _ => return Err(format!("Tx doesn't have a log, {}.", tx_response.txhash)),
            },
            validator_to_address: validator_to_metadata.address,
            validator_to_logo_url: validator_to_metadata.logo_url,
            validator_to_name: validator_to_metadata.name,
            delegator_address,
        })
    }
}
