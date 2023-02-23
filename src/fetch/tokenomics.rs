use serde::{Deserialize, Serialize};

use super::others::{DenomAmount, InternalDenomAmount, Pagination, PaginationConfig};
use crate::{
    chain::Chain,
    routes::{calc_pages, OutRestResponse},
};

impl Chain {
    /// Returns the total supply of all tokens.
    pub async fn get_supply_of_all_tokens(&self, config: PaginationConfig) -> Result<OutRestResponse<Vec<InternalDenomAmount>>, String> {
        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self
            .rest_api_request::<SupplyOfAllTokensResp>("/cosmos/bank/v1beta1/supply", &query)
            .await?;

        let mut supplies = vec![];

        for supply in resp.supply {
            supplies.push(supply.try_into()?)
        }

        let pages = calc_pages(resp.pagination, config)?;

        Ok(OutRestResponse::new(supplies, pages))
    }

    /// Returns the supply of given token.
    pub async fn get_supply_by_denom(&self, denom: &str) -> Result<OutRestResponse<InternalDenomAmount>, String> {
        let path = format!("/cosmos/bank/v1beta1/supply/{denom}");

        let resp = self.rest_api_request::<SupplyByDenomResp>(&path, &[]).await?;

        let supply = resp.amount.try_into()?;

        Ok(OutRestResponse::new(supply, 0))
    }

    /// Returns the supply of the native coin.
    pub async fn get_supply_of_native_coin(&self) -> Result<OutRestResponse<InternalDenomAmount>, String> {
        self.get_supply_by_denom(&self.config.main_denom).await
    }

    pub async fn get_evm_supported_chains(&self) -> Result<Vec<String>, String> {
        let resp = self
            .rest_api_request::<AxelarSupportedEvmChainsResponse>("/axelar/evm/v1beta1/chains", &[])
            .await?;

        Ok(resp.chains)
    }

    pub async fn get_evm_chain_maintainers(&self, chain_name: &String) -> Result<Vec<String>, String> {
        let path = format!("/axelar/nexus/v1beta1/chain_maintainers/{chain_name}");
        let resp = self.archive_api_request::<AxelarEvmChainMaintainersResponse>(&path, &[]).await?;

        Ok(resp.maintainers)
    }

    /// Returns the minting inflation rate of native coin of the chain.
    pub async fn get_inflation_rate(&self) -> Result<OutRestResponse<f64>, String> {
        let default_return_value = 0.0;
        let mut inflation = if self.config.name == "evmos" {
            self.rest_api_request::<MintingInflationRateResp>("/evmos/inflation/v1/inflation_rate", &[])
                .await
                .map(|res| res.inflation_rate.parse::<f64>().unwrap_or(0.0) / 100.0)
        } else if self.config.name == "echelon" {
            self.rest_api_request::<MintingInflationRateResp>("/echelon/inflation/v1/inflation_rate", &[])
                .await
                .map(|res| res.inflation_rate.parse::<f64>().unwrap_or(default_return_value) / 100.0)
        } else {
            self.rest_api_request::<MintingInflationResp>("/cosmos/mint/v1beta1/inflation", &[])
                .await
                .map(|res| res.inflation.parse::<f64>().unwrap_or(default_return_value))
        }
        .unwrap_or(default_return_value);

        //Axelar calculation different than others. That is why we are overriding inflation variable here.
        if self.config.name == "axelar" {
            let external_chain_voting_inflation_rate = self
                .rest_api_request::<AxelarExternalChainVotingInflationRateResponse>(
                    "/cosmos/params/v1beta1/params?subspace=reward&key=ExternalChainVotingInflationRate",
                    &[],
                )
                .await
                .map(|res| res.param.get_parsed_value().unwrap_or(default_return_value))
                .unwrap_or(default_return_value);

            let external_chain_inflation = self
                .get_evm_supported_chains()
                .await
                .map(|res| res.len() as f64 * external_chain_voting_inflation_rate)
                .unwrap_or(default_return_value);

            inflation = external_chain_inflation + (inflation * 2.0);
        }

        Ok(OutRestResponse::new(inflation, 0))
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MintingInflationResp {
    /// Minting inflation rate. Eg: `"0.131020685388983473"`
    pub inflation: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MintingInflationRateResp {
    /// Minting inflation rate. Eg: `"91.087708112747866100"`
    pub inflation_rate: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SupplyByDenomResp {
    /// Amount and denom.
    pub amount: DenomAmount,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SupplyOfAllTokensResp {
    /// Array of amounts and denoms.
    pub supply: Vec<DenomAmount>,
    /// Paginations
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct AxelarExternalChainVotingInflationRateResponse {
    param: AxelarExternalChainVotingInflationRateParam,
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
            Err(_) => Err(format!("Parsed value error on AxelarExternalChainVotingInflationRateParam")),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct AxelarSupportedEvmChainsResponse {
    chains: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct AxelarEvmChainMaintainersResponse {
    maintainers: Vec<String>,
}
