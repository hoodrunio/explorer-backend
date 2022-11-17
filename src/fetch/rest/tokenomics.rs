use serde::{Deserialize, Serialize};

use super::others::{DenomAmount, InternalDenomAmount, Pagination, PaginationConfig};
use crate::chain::Chain;

impl Chain {
    /// Returns the total supply of all tokens.
    pub async fn get_supply_of_all_tokens(&self, pagination_config: PaginationConfig) -> Result<SupplyOfAllTokensResp, String> {
        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.is_reverse())));
        query.push(("pagination.limit", format!("{}", pagination_config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.get_offset())));

        self.rest_api_request("/cosmos/bank/v1beta1/supply", &query).await
    }

    /// Returns the supply of given token.
    pub async fn get_supply_by_denom(&self, denom: &str) -> Result<InternalDenomAmount, String> {
        let path = format!("/cosmos/bank/v1beta1/supply/{denom}");
        match self.rest_api_request::<SupplyByDenomResp>(&path, &[]).await {
            Ok(resp) => match resp.amount.amount.parse::<u128>() {
                Ok(amount) => Ok(InternalDenomAmount {
                    amount,
                    denom: resp.amount.denom,
                }),
                Err(_) => Err(format!("Cannot parse number, '{}'.", resp.amount.amount)),
            },
            Err(error) => Err(error),
        }
    }

    /// Returns the supply of the native coin.
    pub async fn get_supply_of_native_coin(&self) -> Result<InternalDenomAmount, String> {
        self.get_supply_by_denom(self.inner.main_denom).await
    }

    /// Returns the minting inflation rate of native coin of the chain.
    pub async fn get_inflation_rate(&self) -> f64 {
        if self.inner.name == "evmos" {
            self.rest_api_request::<MintingInflationRateResp>("/evmos/inflation/v1/inflation_rate", &[])
                .await
                .and_then(|res| Ok(res.inflation_rate.parse::<f64>().unwrap_or(0.0)))
        } else if self.inner.name == "echelon" {
            self.rest_api_request::<MintingInflationRateResp>("/echelon/inflation/v1/inflation_rate", &[])
                .await
                .and_then(|res| Ok(res.inflation_rate.parse::<f64>().unwrap_or(0.0)))
        } else {
            self.rest_api_request::<MintingInflationResp>("/cosmos/mint/v1beta1/inflation", &[])
                .await
                .and_then(|res| Ok(res.inflation.parse::<f64>().unwrap_or(0.0) * 100.0))
        }
        .unwrap_or(0.0)
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
