use std::num::ParseFloatError;

use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};

use crate::chain::Chain;

impl Chain {
    /// Returns the APR rate of the chain.
    ///
    /// Reference: https://github.com/bro-n-bro/prometheus_exporter/blob/main/docs/APR%20calcucation.md#non-epoch-cosmos-based--blockchains
    pub async fn get_apr(&self) -> Result<f64, String> {
        let client = Client::new();
        const ANNUAL_PROVISION_MUL_RATIO: f64 = 365.3;

        // Constant declarations
        const SECS_IN_YEAR: f64 = 31561920.0;
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
            // We will get those below from the database.
            let annual_provisions = 0.0;
            let community_tax = 0.0;
            let bonded_tokens_amount = 0.0;
            let block_per_year = 0.0;
            let avg_block_time_24h = 0.0;

            // Calculate how many blocks will be created in a year with the speed same as last 24h.
            let current_block_per_year = SECS_IN_YEAR / avg_block_time_24h;

            // Calculate correction.
            let correction_annual_coefficient = current_block_per_year / block_per_year;

            let apr = (annual_provisions * (1.0 - community_tax) / bonded_tokens_amount) * correction_annual_coefficient;

            Ok(apr)
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct OsmosisDistributionParams {
    pub community_tax: String,
    pub base_proposer_reward: String,
    pub bonus_proposer_reward: String,
    pub withdraw_addr_enabled: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct OsmosisDistributionResponse {
    pub params: OsmosisDistributionParams,
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
