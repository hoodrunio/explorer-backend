use std::{
    ops::{Div, Mul},
    str::FromStr,
};

use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use tokio::join;
use tonic::transport::Endpoint;

use crate::{chain::Chain, fetch::amount_util::TnrDecimal, utils::str_to_dec};

impl Chain {
    /// Returns the APR rate of the chain.
    ///
    /// Reference: https://github.com/bro-n-bro/prometheus_exporter/blob/main/docs/APR%20calcucation.md#non-epoch-cosmos-based--blockchains
    pub async fn get_apr(&self) -> Result<f64, String> {
        const ANNUAL_PROVISION_MUL_RATIO: f64 = 365.3;
        // If the chain has epochs.
        if self.config.epoch {
            let (epoch_prevision_res, staking_pool_res) = join!(self.get_epoch_provision(), self.get_staking_pool());
            let epoch_provisions = epoch_prevision_res?;
            let annual_provisions = epoch_provisions * ANNUAL_PROVISION_MUL_RATIO;
            let bonded_tokens_amount = staking_pool_res?.value.bonded as f64;

            match self.config.name.as_str() {
                "osmosis" => {
                    let staking_rewards_factor = 0.25;
                    let bonded_tokens_amount = bonded_tokens_amount * (self.config.decimals_pow as f64 * 10000.0);
                    let apr = annual_provisions * staking_rewards_factor / bonded_tokens_amount;

                    Ok(apr)
                }
                "evmos" => {
                    use crate::fetch::evmos::inflation::v1::{query_client::QueryClient, QueryParamsRequest};

                    let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

                    let req = QueryParamsRequest {};

                    let inflation_dist = QueryClient::connect(endpoint)
                        .await
                        .unwrap()
                        .params(req)
                        .await
                        .map_err(|e| format!("{}", e))?
                        .into_inner()
                        .params
                        .ok_or_else(|| "Missing params".to_string())?
                        .inflation_distribution
                        .ok_or_else(|| "Missing inflation distribution param".to_string())?;

                    let staking_rewards_factor = TnrDecimal::from_str(str_to_dec(inflation_dist.staking_rewards.as_str()).as_str())
                        .unwrap_or_default()
                        .to_f64()
                        .unwrap_or_default();

                    let annual_provisions = epoch_provisions * ANNUAL_PROVISION_MUL_RATIO;
                    let apr = annual_provisions * staking_rewards_factor / bonded_tokens_amount;

                    Ok(apr)
                }
                "quicksilver" => {
                    let mint_params = self.get_mint_params().await?;
                    let staking_rewards_factor = mint_params
                        .params
                        .distribution_proportions
                        .staking
                        .parse::<f64>()
                        .map_err(|_| "Failed to parse staking rewards factor".to_string())?;

                    let epoch_prevision = self
                        .calc_tnr_decimal_amount(TnrDecimal::from_f64(epoch_provisions).unwrap_or_default(), None)
                        .to_f64()
                        .ok_or_else(|| "Failed to parse total supply".to_string())?;

                    let annual_provision = epoch_prevision * ANNUAL_PROVISION_MUL_RATIO;
                    let apr = annual_provision * staking_rewards_factor / bonded_tokens_amount;
                    Ok(apr)
                }
                chain_name => Err(format!("APR for {chain_name} is not implemented.")),
            }
        } else {
            let (params_all_res, staking_pool_res, inflation_rate_res) =
                join!(self.get_params_all(), self.get_staking_pool(), self.get_inflation_rate());

            let inflation = inflation_rate_res?;
            let bonded_token_amount = staking_pool_res?.value.bonded as f64;
            let community_tax = params_all_res?.distribution.community_tax;
            match self.config.name.as_str() {
                "axelar" => {
                    let bonded_token_amount = bonded_token_amount / 1000000000.0;
                    Ok((inflation * (1.0 - community_tax)) / bonded_token_amount)
                }
                "c4e" => {
                    let (total_supply_res, share_param_res) = join!(self.get_supply_by_denom(&self.config.main_denom), self.get_share_param());
                    let inflation = TnrDecimal::from_f64(inflation).unwrap_or_default();
                    let total_supply = total_supply_res?.amount;
                    let bonded_token_amount = TnrDecimal::from_f64(bonded_token_amount).unwrap_or_default();
                    let share_param = share_param_res?;

                    Ok(inflation
                        .mul(total_supply)
                        .div(bonded_token_amount)
                        .mul(share_param)
                        .to_f64()
                        .unwrap_or_default())
                }
                _ => {
                    let bonded_token_amount = bonded_token_amount * (self.config.decimals_pow as f64 * 10000.0);
                    let annual_provisions = match self.get_annual_provisions().await {
                        Ok(res) => res.value,
                        Err(error) => return Err(error),
                    };

                    let non_epoch_apr_calculator = NonEpochAprCalculator {
                        annual_provisions,
                        community_tax,
                        bonded_token_amount,
                        correction_annual_coefficient: None,
                    };

                    Ok(non_epoch_apr_calculator.get_apr())
                } // chain_name => Err(format!("APR for {chain_name} is not implemented.")),
            }
        }
    }
}

pub struct NonEpochAprCalculator {
    pub annual_provisions: f64,
    pub community_tax: f64,
    pub bonded_token_amount: f64,
    pub correction_annual_coefficient: Option<f64>,
}

impl NonEpochAprCalculator {
    pub fn get_apr(&self) -> f64 {
        (self.annual_provisions * (1.0 - self.community_tax) / self.bonded_token_amount) * self.correction_annual_coefficient.unwrap_or(1.0)
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
