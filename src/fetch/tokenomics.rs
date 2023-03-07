use serde::{Deserialize, Serialize};

use super::others::{DenomAmount, Pagination, PaginationConfig};
use crate::{
    chain::Chain,
    routes::{calc_pages, ChainAmountItem, OutRestResponse},
};

impl Chain {
    /// Returns the total supply of all tokens.
    pub async fn get_supply_of_all_tokens(&self, config: PaginationConfig) -> Result<OutRestResponse<Vec<ChainAmountItem>>, String> {
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
            supplies.push(self.string_amount_parser(supply.amount, Some(supply.denom)).await?);
        }

        let pages = calc_pages(resp.pagination, config)?;

        Ok(OutRestResponse::new(supplies, pages))
    }

    /// Returns the supply of given token.
    pub async fn get_supply_by_denom(&self, denom: &str) -> Result<OutRestResponse<ChainAmountItem>, String> {
        if self.config.name == "evmos" {
            let query = vec![];

            let resp = self
                .rest_api_request::<SupplyOfAllTokensResp>("/cosmos/bank/v1beta1/supply", &query)
                .await?
                .supply
                .iter()
                .find(|supply| supply.denom == denom)
                .cloned();

            if let Some(supply) = resp {
                let supply = self.string_amount_parser(supply.amount.clone(), Some(supply.denom.clone())).await?;

                return Ok(OutRestResponse::new(supply, 0));
            };

            return Err("Token not found".to_string());
        };

        let path = format!("/cosmos/bank/v1beta1/supply/{denom}");

        let resp = self.rest_api_request::<SupplyByDenomResp>(&path, &[]).await?;

        let supply = self.string_amount_parser(resp.amount.amount, Some(resp.amount.denom)).await?;

        Ok(OutRestResponse::new(supply, 0))
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

    pub async fn get_account_balances(
        &self,
        account_address: &String,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalBalance>>, String> {
        let path = format!("/cosmos/bank/v1beta1/balances/{account_address}");
        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<AccountBalances>(&path, &[]).await?;

        let mut balances: Vec<InternalBalance> = vec![];

        let pages = calc_pages(resp.pagination, config)?;

        for balance in resp.balances {
            let amount = self.string_amount_parser(balance.amount, Some(balance.denom)).await?;
            balances.push(InternalBalance { amount });
        }

        Ok(OutRestResponse::new(balances, pages))
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
        match self.value.replace('\"', "").parse::<f64>() {
            Ok(parsed_value) => Ok(parsed_value),
            Err(_) => Err("Parsed value error on AxelarExternalChainVotingInflationRateParam".to_string()),
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

#[derive(Deserialize, Serialize, Debug)]
pub struct AccountBalances {
    pub balances: Vec<Balance>,
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Balance {
    denom: String,
    amount: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct InternalBalance {
    amount: ChainAmountItem,
}
