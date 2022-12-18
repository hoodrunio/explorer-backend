use std::fmt::format;
use std::num::ParseFloatError;

use chrono::DateTime;
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use sha2::digest::typenum::private::IsGreaterOrEqualPrivate;

use crate::chain::Chain;
use crate::fetch::blocks::{Block, BlockResp};
use crate::fetch::others::{MintParams, StakingPoolResp};
use crate::fetch::params::ChainParams;
use crate::routes::OutRestResponse;

impl Chain {
    /// Returns the APR rate of the chain.
    ///
    /// Reference: https://github.com/bro-n-bro/prometheus_exporter/blob/main/docs/APR%20calcucation.md#non-epoch-cosmos-based--blockchains
    pub async fn get_apr(&self) -> Result<f64, String> {
        let client = Client::new();
        const ANNUAL_PROVISION_MUL_RATIO: f64 = 365.3;

        // If the chain has epochs.
        if self.inner.epoch {
            match self.inner.name {
                "osmosis" => {
                    let epoch_provisions_response = match self.external_rest_api_req::<OsmosisEpochProvisionResponse>
                    (&client, Method::GET, "https://lcd.osmosis-1.bronbro.io/osmosis/mint/v1beta1/epoch_provisions", &[]).await {
                        Ok(parsed_response) => parsed_response,
                        Err(error) => return Err(error)
                    };
                    let epoch_provisions = match epoch_provisions_response.epoch_provisions.parse::<f64>() {
                        Ok(value) => value,
                        Err(_) => return Err("FLOAT_PARSING_ERROR".to_string())
                    };

                    let osmosis_staking_pool_tokens_response = match self.external_rest_api_req::<OsmosisStakingPoolTokensResponse>
                    (&client, Method::GET, "https://lcd.osmosis-1.bronbro.io/cosmos/staking/v1beta1/pool", &[]).await
                    {
                        Ok(parsed_response) => parsed_response,
                        Err(error) => return Err(error)
                    };

                    let bonded_tokens_amount = match osmosis_staking_pool_tokens_response.pool.bonded_tokens.parse::<f64>() {
                        Ok(val) => val,
                        Err(_) => return Err("FLOAT_PARSING_ERROR".to_string())
                    };

                    let annual_provisions = epoch_provisions * ANNUAL_PROVISION_MUL_RATIO;
                    let staking_rewards_factor = 0.25;
                    let apr = annual_provisions * staking_rewards_factor / bonded_tokens_amount;

                    Ok(apr)
                }
                "evmos" => {
                    let evmos_decimal = (self.inner.decimals_pow as f64);
                    let evmos_inflation_params_response = match self.external_rest_api_req::<EvmosInflationParamsResponse>
                    (&client, Method::GET, "https://lcd.evmos-9001-2.bronbro.io/evmos/inflation/v1/params", &[]).await
                    {
                        Ok(parsed_response) => parsed_response,
                        Err(error) => return Err(error)
                    };

                    let staking_rewards_factor = match evmos_inflation_params_response.params.inflation_distribution.staking_rewards.parse::<f64>() {
                        Ok(value) => value,
                        Err(_) => return Err("FLOAT_PARSING_ERROR".to_string())
                    };

                    let evmos_inflation_epoch_prevision_response = match self.external_rest_api_req::<EvmosInflationEpochProvisionResponse>
                    (&client, Method::GET, "https://lcd.evmos-9001-2.bronbro.io/evmos/inflation/v1/epoch_mint_provision", &[]).await
                    {
                        Ok(parsed_response) => parsed_response,
                        Err(error) => return Err(error)
                    };

                    let epoch_provisions = match evmos_inflation_epoch_prevision_response.epoch_mint_provision.amount.parse::<f64>() {
                        Ok(value) => value / evmos_decimal,
                        Err(_) => return Err("FLOAT_PARSING_ERROR".to_string())
                    };

                    let evmos_staking_pool_response = match self.external_rest_api_req::<EvmosStakingPoolResponse>
                    (&client, Method::GET, "https://lcd.evmos-9001-2.bronbro.io/cosmos/staking/v1beta1/pool", &[]).await
                    {
                        Ok(parsed_response) => parsed_response,
                        Err(error) => return Err(error)
                    };

                    let bonded_tokens_amount = match evmos_staking_pool_response.pool.bonded_tokens.parse::<f64>() {
                        Ok(value) => value / evmos_decimal,
                        Err(_) => return Err("FLOAT_PARSING_ERROR".to_string())
                    };
                    let annual_provisions = epoch_provisions * ANNUAL_PROVISION_MUL_RATIO;
                    let apr = annual_provisions * staking_rewards_factor / bonded_tokens_amount;

                    Ok(apr)
                }
                chain_name => Err(format!("APR for {chain_name} is not implemented.")),
            }
        } else {
            match self.inner.name {
                "axelar" => {
                    let axelar_inflation_rate = match self.get_inflation_rate().await {
                        Ok(value) => value.value * 2.0,
                        Err(error) => return Err(error)
                    };

                    let external_chain_voting_inflation_rate =
                        match self.rest_api_request::<AxelarExternalChainVotingInflationRateResponse>(
                            "/cosmos/params/v1beta1/params?subspace=reward&key=ExternalChainVotingInflationRate",
                            &[]).await {
                            Ok(response) => match response.param.get_parsed_value() {
                                Ok(value) => value,
                                Err(error) => return Err(error),
                            },
                            Err(error) => return Err(error),
                        };

                    let external_chain_inflation =
                        match self.rest_api_request::<AxelarSupportedEvmChainsResponse>(
                            "/axelar/evm/v1beta1/chains",
                            &[]).await {
                            Ok(response) => response.get_supported_evm_chains_length() * external_chain_voting_inflation_rate,
                            Err(error) => return Err(error),
                        };


                    let chain_params = match self.get_params_all().await {
                        Ok(res) => res.value,
                        Err(error) => return Err(error)
                    };

                    let staking_pool = match self.get_staking_pool().await {
                        Ok(res) => res.value,
                        Err(error) => return Err(error)
                    };

                    //TODO Get total supply from remote
                    let bonded_tokens_amount = self.calc_amount_u128_to_u64(staking_pool.bonded as u128);
                    let bonded_token_ratio = (bonded_tokens_amount as f64) / (1000000000.0);
                    let inflation = external_chain_inflation + axelar_inflation_rate;
                    let community_tax = chain_params.distribution.community_tax as f64;


                    return Ok((inflation * (1.0 - community_tax)) / bonded_token_ratio);
                }
                _ => {
                    let community_tax = match self.get_params_all().await {
                        Ok(res) => res.value.distribution.community_tax,
                        Err(error) => return Err(error)
                    };
                    let bonded_tokens_amount = match self.get_staking_pool().await {
                        Ok(res) => res.value.bonded as f64,
                        Err(error) => return Err(error)
                    };
                    let annual_provisions = match self.get_annual_provisions().await {
                        Ok(res) => res.value,
                        Err(error) => return Err(error)
                    };

                    //If needed call self.get_correction_annual_coefficient

                    let non_epoch_apr_calculator = NonEpochAprCalculator {
                        annual_provisions,
                        community_tax,
                        bonded_tokens_amount,
                        correction_annual_coefficient: None,
                    };

                    let result = match non_epoch_apr_calculator.get_apr() {
                        Ok(apr) => apr,
                        Err(error) => return Err(error)
                    };

                    Ok(result)
                }
                // chain_name => Err(format!("APR for {chain_name} is not implemented.")),
            }
        }
    }

    pub async fn get_block_time(&self) -> Result<f64, String> {
        let block_time = 100.4;
        Ok(block_time)
    }
}

pub struct NonEpochAprCalculator {
    pub annual_provisions: f64,
    pub community_tax: f64,
    pub bonded_tokens_amount: f64,
    pub correction_annual_coefficient: Option<f64>,
}

impl NonEpochAprCalculator {
    pub fn get_apr(&self) -> Result<f64, String> {
        Ok((self.annual_provisions * (1.0 - self.community_tax)
            / self.bonded_tokens_amount) * self.correction_annual_coefficient.unwrap_or(1.0))
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct AxelarExternalChainVotingInflationRateResponse {
    param: AxelarExternalChainVotingInflationRateParam,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct AxelarSupportedEvmChainsResponse {
    chains: Vec<String>,
}

impl AxelarSupportedEvmChainsResponse {
    pub fn get_supported_evm_chains_length(&self) -> f64 {
        self.chains.len() as f64
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct AxelarExternalChainVotingInflationRateParam {
    pub subspace: String,
    pub key: String,
    pub value: String,
}

impl AxelarExternalChainVotingInflationRateParam {
    pub fn get_parsed_value(&self) -> Result<f64, String> {
        match self.value.replace("\"", "").parse::<f64>() {
            Ok(parsed_value) => Ok(parsed_value),
            Err(_) => Err(format!("Parsed value error on AxelarExternalChainVotingInflationRateParam"))
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CosmosDistributionParams {
    pub community_tax: String,
    pub base_proposer_reward: String,
    pub bonus_proposer_reward: String,
    pub withdraw_addr_enabled: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CosmosDistributionParamsResponse {
    pub params: CosmosDistributionParams,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct OsmosisStakingPoolTokensResponse {
    pub pool: OsmosisStakingPoolTokens,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct OsmosisStakingPoolTokens {
    pub not_bonded_tokens: String,
    pub bonded_tokens: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct OsmosisEpochProvisionResponse {
    pub epoch_provisions: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EvmosInflationParamsResponse {
    pub params: EvmosInflationParams,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EvmosInflationParams {
    pub mint_denom: String,
    pub exponential_calculation: EvmosInflationExponentialCalcParam,
    pub inflation_distribution: EvmosInflationDistributionParam,
    pub enable_inflation: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EvmosInflationExponentialCalcParam {
    a: String,
    r: String,
    c: String,
    bonding_target: String,
    max_variance: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EvmosInflationDistributionParam {
    staking_rewards: String,
    usage_incentives: String,
    community_pool: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EvmosInflationEpochProvisionResponse {
    pub epoch_mint_provision: EvmosInflationEpochProvision,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EvmosInflationEpochProvision {
    pub denom: String,
    pub amount: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EvmosStakingPoolResponse {
    pub pool: EvmosStakingPool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EvmosStakingPool {
    not_bonded_tokens: String,
    bonded_tokens: String,
}
