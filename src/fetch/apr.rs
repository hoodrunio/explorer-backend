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

        let inflation = 0.0;
        let community_tax = 0.0;
        let bonded_tokens_ratio = 0.0;

        // Constant declarations
        const secs_in_year: f64 = 31561920.0;
        // If the chain has epochs.
        if !self.inner.epoch {
            // We will get those below from the database.
            let epoch_provisions = 0.0;
            let community_tax = 0.0;
            let bonded_tokens_amount = 0.0;

            // Calculate annual provisions.
            let annual_provisions = epoch_provisions * 365.3;

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

                    let distribution_params = match self.external_rest_api_req::<OsmosisDistributionResponse>
                    (&client, Method::GET, "https://lcd.osmosis-1.bronbro.io/cosmos/distribution/v1beta1/params", &[]).await
                    {
                        Ok(parsed_response) => parsed_response,
                        Err(error) => return Err(error)
                    };

                    let annual_provisions = epoch_provisions * ANNUAL_PROVISION_MUL_RATIO;
                    let staking_rewards_factor = 0.25;
                    let apr = annual_provisions * staking_rewards_factor / bonded_tokens_amount;

                    Ok(apr)
                }
                "evmos" => {
                    let staking_rewards_factor = 0.0; // will be fetched. https://lcd.evmos-9001-2.bronbro.io/evmos/inflation/v1/params

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
            let current_block_per_year = secs_in_year / avg_block_time_24h;

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
