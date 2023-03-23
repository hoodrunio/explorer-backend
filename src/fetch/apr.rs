use reqwest::Method;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use tokio::join;

use crate::{chain::Chain, fetch::amount_util::TnrDecimal};

impl Chain {
    /// Returns the APR rate of the chain.
    ///
    /// Reference: https://github.com/bro-n-bro/prometheus_exporter/blob/main/docs/APR%20calcucation.md#non-epoch-cosmos-based--blockchains
    pub async fn get_apr(&self) -> Result<f64, String> {
        const ANNUAL_PROVISION_MUL_RATIO: f64 = 365.3;

        // If the chain has epochs.
        if self.config.epoch {
            match self.config.name.as_str() {
                "osmosis" => {
                    let epoch_provisions = self.get_epoch_provision().await?;

                    let osmosis_staking_pool_tokens_response = match self
                        .external_rest_api_req::<OsmosisStakingPoolTokensResponse>(
                            &self.client,
                            Method::GET,
                            "https://lcd.osmosis-1.bronbro.io/cosmos/staking/v1beta1/pool",
                            &[],
                        )
                        .await
                    {
                        Ok(parsed_response) => parsed_response,
                        Err(error) => return Err(error),
                    };

                    let bonded_tokens_amount = match osmosis_staking_pool_tokens_response.pool.bonded_tokens.parse::<f64>() {
                        Ok(val) => val,
                        Err(_) => return Err("FLOAT_PARSING_ERROR".to_string()),
                    };

                    let annual_provisions = epoch_provisions * ANNUAL_PROVISION_MUL_RATIO;
                    let staking_rewards_factor = 0.25;

                    let apr = annual_provisions * staking_rewards_factor / bonded_tokens_amount;

                    Ok(apr)
                }
                "evmos" => {
                    let evmos_decimal = self.config.decimals_pow as f64;
                    let evmos_inflation_params_response = match self
                        .external_rest_api_req::<EvmosInflationParamsResponse>(
                            &self.client,
                            Method::GET,
                            "https://lcd.evmos-9001-2.bronbro.io/evmos/inflation/v1/params",
                            &[],
                        )
                        .await
                    {
                        Ok(parsed_response) => parsed_response,
                        Err(error) => return Err(error),
                    };

                    let staking_rewards_factor = match evmos_inflation_params_response
                        .params
                        .inflation_distribution
                        .staking_rewards
                        .parse::<f64>()
                    {
                        Ok(value) => value,
                        Err(_) => return Err("FLOAT_PARSING_ERROR".to_string()),
                    };

                    let epoch_provisions = self.get_epoch_provision().await.map(|res| res / evmos_decimal)?;

                    let evmos_staking_pool_response = match self
                        .external_rest_api_req::<EvmosStakingPoolResponse>(
                            &self.client,
                            Method::GET,
                            "https://lcd.evmos-9001-2.bronbro.io/cosmos/staking/v1beta1/pool",
                            &[],
                        )
                        .await
                    {
                        Ok(parsed_response) => parsed_response,
                        Err(error) => return Err(error),
                    };

                    let bonded_tokens_amount = match evmos_staking_pool_response.pool.bonded_tokens.parse::<f64>() {
                        Ok(value) => value / evmos_decimal,
                        Err(_) => return Err("FLOAT_PARSING_ERROR".to_string()),
                    };
                    let annual_provisions = epoch_provisions * ANNUAL_PROVISION_MUL_RATIO;
                    let apr = annual_provisions * staking_rewards_factor / bonded_tokens_amount;

                    Ok(apr)
                }
                "quicksilver" => {
                    let (epoch_prevision_res, staking_pool_res, mint_params_res) =
                        join!(self.get_epoch_provision(), self.get_staking_pool(), self.get_mint_params());
                    let epoch_prevision_number = epoch_prevision_res?;
                    let bonded_tokens = staking_pool_res?.value.bonded;
                    let mint_params = mint_params_res?;
                    let staking_rewards_factor = mint_params
                        .params
                        .distribution_proportions
                        .staking
                        .parse::<f64>()
                        .map_err(|_| "Failed to parse staking rewards factor".to_string())?;

                    let epoch_prevision = self
                        .calc_tnr_decimal_amount(TnrDecimal::from_f64(epoch_prevision_number).unwrap_or_default(), None)
                        .to_f64()
                        .ok_or_else(|| "Failed to parse total supply".to_string())?;

                    let annual_provision = epoch_prevision * 365.0;
                    let apr = annual_provision * staking_rewards_factor / bonded_tokens as f64;
                    Ok(apr)
                }
                chain_name => Err(format!("APR for {chain_name} is not implemented.")),
            }
        } else {
            match self.config.name.as_str() {
                "axelar" => {
                    let chain_params = match self.get_params_all().await {
                        Ok(res) => res.value,
                        Err(error) => return Err(error),
                    };

                    let staking_pool = match self.get_staking_pool().await {
                        Ok(res) => res.value,
                        Err(error) => return Err(error),
                    };

                    //TODO Get total supply from remote
                    let bonded_tokens_amount = staking_pool.bonded;
                    let bonded_token_ratio = (bonded_tokens_amount as f64) / (1000000000.0);
                    let inflation = match self.get_inflation_rate().await {
                        Ok(res) => res,
                        Err(error) => return Err(error),
                    };
                    let community_tax = chain_params.distribution.community_tax;

                    Ok((inflation * (1.0 - community_tax)) / bonded_token_ratio)
                }
                _ => {
                    let community_tax = match self.get_params_all().await {
                        Ok(res) => res.value.distribution.community_tax,
                        Err(error) => return Err(error),
                    };
                    let bonded_tokens_amount = match self.get_staking_pool().await {
                        Ok(res) => (res.value.bonded * (self.config.decimals_pow * 10000)) as f64,
                        Err(error) => return Err(error),
                    };
                    let annual_provisions = match self.get_annual_provisions().await {
                        Ok(res) => res.value,
                        Err(error) => return Err(error),
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
                        Err(error) => return Err(error),
                    };

                    Ok(result)
                } // chain_name => Err(format!("APR for {chain_name} is not implemented.")),
            }
        }
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
        Ok((self.annual_provisions * (1.0 - self.community_tax) / self.bonded_tokens_amount) * self.correction_annual_coefficient.unwrap_or(1.0))
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
pub struct EpochProvisionResponse {
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
