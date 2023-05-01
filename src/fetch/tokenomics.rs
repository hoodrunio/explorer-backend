use bech32::ToBase32;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use tokio::join;
use tonic::transport::Endpoint;

use super::{
    amount_util::TnrDecimal,
    others::{DenomAmount, Pagination, PaginationConfig},
};
use crate::{
    chain::Chain,
    routes::{calc_pages, ChainAmountItem, OutRestResponse},
};
use crate::database::ListDbResult;
use crate::fetch::cosmos::bank::v1beta1::QuerySupplyOfRequest;
use crate::routes::PaginationData;
use crate::fetch::cosmos::base::query::v1beta1::{PageRequest, PageResponse};

impl Chain {
    /// Returns the total supply of all tokens.
    pub async fn get_supply_of_all_tokens(&self, config: PaginationData) -> Result<ListDbResult<ChainAmountItem>, String> {
        use crate::fetch::cosmos::bank::v1beta1::{query_client::QueryClient, QueryTotalSupplyRequest, QueryTotalSupplyResponse};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let pagination = PageRequest {
            key: config.cursor.map(|c| c.as_bytes().to_vec()).unwrap_or_else(|| Vec::new()),
            limit: config.limit.unwrap_or_else(|| 50),
            count_total: true,
            ..Default::default()
        };

        let supply_request = QueryTotalSupplyRequest {
            pagination: Some(pagination),
        };

        let resp = QueryClient::connect(endpoint.clone())
            .await
            .unwrap()
            .total_supply(supply_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let total_supply = resp.into_inner();



        let mut supplies = vec![];

        for supply in total_supply.supply {
            supplies.push(self.string_amount_parser(supply.amount, Some(supply.denom)).await?);
        }



        let pagination = total_supply.pagination.map(|ts| ts.into()).unwrap_or_default();
        Ok(ListDbResult {
            data: supplies,
            pagination,
        })
    }

    /// Returns the supply of given token.
    pub async fn get_supply_by_denom(&self, denom: &str) -> Result<ChainAmountItem, String> {
        use crate::fetch::cosmos::bank::v1beta1::{query_client::QueryClient, QueryTotalSupplyRequest, QueryTotalSupplyResponse, QuerySupplyOfRequest, QuerySupplyOfResponse};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let mut client = QueryClient::connect(endpoint.clone())
            .await
            .unwrap();

        let from_supply_list_chains = vec!["evmos", "umee", "quicksilver", "kyve", "c4e", "babylon-testnet"];
        if from_supply_list_chains.contains(&self.config.name.as_str()) {


            let supply_request = QueryTotalSupplyRequest {
                pagination: None,
            };

            let resp = client
                .total_supply(supply_request)
                .await
                .map_err(|e| format!("{}", e))?
                .into_inner()
                .supply
                .iter()
                .find(|supply| supply.denom == denom)
                .cloned();

            if let Some(supply) = resp {
                let supply = self.string_amount_parser(supply.amount.clone(), Some(supply.denom.clone())).await?;

                return Ok(supply);
            };

            return Err("Token not found".to_string());
        };

        let supply_of_request = QuerySupplyOfRequest {
            denom: denom.to_string(),
        };

        let resp = client
            .supply_of(supply_of_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let supply = resp.into_inner();

        let supply = self.string_amount_parser(supply.amount.clone().unwrap().amount, Some(supply.amount.unwrap().denom)).await?;

        Ok(supply)
    }

    pub async fn get_evm_supported_chains(&self) -> Result<Vec<String>, String> {
        use crate::fetch::axelar::evm::v1beta1::{ChainsResponse, ChainsRequest, query_service_client::QueryServiceClient};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let chains_request = ChainsRequest {
            status: 0
        };

        let resp = QueryServiceClient::connect(endpoint)
            .await
            .unwrap()
            .chains(chains_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let chains = resp.into_inner();

        Ok(chains.chains)
    }

    pub async fn get_evm_chain_maintainers(&self, chain_name: &str) -> Result<Vec<String>, String> {
        use crate::fetch::axelar::nexus::v1beta1::{ ChainMaintainersRequest, ChainMaintainersResponse, query_service_client::QueryServiceClient };

        let endpoint = Endpoint::from_shared((self.config.grpc_url.clone().unwrap())).unwrap();

        let maintainers_request = ChainMaintainersRequest {
            chain: chain_name.to_string()
        };

        let resp = QueryServiceClient::connect(endpoint)
            .await
            .unwrap()
            .chain_maintainers(maintainers_request)
            .await
            .map_err(|e| format!("{}", e))?;

        // dbg!(resp);
        let maintainers = resp
            .into_inner()
            .maintainers
            .into_iter().map(|m| {

            bech32::encode(format!("{}valoper", &self.config.base_prefix).as_str(), m.to_base32(), bech32::Variant::Bech32).unwrap()
        })
            .collect();

        Ok(maintainers)
    }
    /// Returns the minting inflation rate of native coin of the chain.
    pub async fn get_inflation_rate(&self) -> Result<f64, String> {
        let default_return_value = 0.0;
        let chain_name = self.config.name.as_str();

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let mut inflation = if ["evmos", "echelon"].contains(&chain_name) {
            use crate::fetch::evmos::inflation::v1::{QueryInflationRateRequest, QueryInflationRateResponse, query_client::QueryClient};

            let inf_req = QueryInflationRateRequest {

            };

            let resp = QueryClient::connect(endpoint.clone())
                .await
                .unwrap()
                .inflation_rate(inf_req)
                .await
                .map_err(|e| format!("{}", e))?;

            let inflation_rate = resp.into_inner();

            inflation_rate.inflation_rate.parse::<f64>().map(|r| r / 100.0)
        } else if ["quicksilver", "osmosis"].contains(&chain_name) {
            let (epoch_provision_res, total_supply_res) = join!(self.get_epoch_provision(), self.get_supply_by_denom(&self.config.main_denom));
            let epoch_provision_number = epoch_provision_res?;
            let epoch_provision = self
                .calc_tnr_decimal_amount(TnrDecimal::from_f64(epoch_provision_number).unwrap_or_default(), None)
                .to_f64()
                .ok_or_else(|| "Failed to parse total supply".to_string())?;

            let annual_provision = epoch_provision * 365.0;

            let total_supply = total_supply_res?
                .amount
                .to_f64()
                .ok_or_else(|| "Failed to parse total supply".to_string())?;

            Ok(annual_provision / total_supply)
        } else {
            use crate::fetch::cosmos::mint::v1beta1::{QueryInflationRequest, QueryInflationResponse, query_client::QueryClient};

            let inf_req = QueryInflationRequest {};

            let resp = QueryClient::connect(endpoint.clone())
                .await
                .unwrap()
                .inflation(inf_req)
                .await
                .map_err(|e| format!("{}", e))?;

            let inflation_rate = resp.into_inner();

            dbg!(String::from_utf8(inflation_rate.inflation.clone()));

            String::from_utf8(inflation_rate.inflation).unwrap().parse::<f64>().map(|r| r / 100.0)
        }
        .unwrap_or(default_return_value);

        //Axelar calculation different than others. That is why we are overriding inflation variable here.
        if self.config.name == "axelar" {
            use crate::fetch::cosmos::params::v1beta1::{query_client::QueryClient, QueryParamsRequest, QueryParamsResponse};

            let req = QueryParamsRequest {
                key: "reward".to_string(),
                subspace: "ExternalChainVotingInflationRate".to_string()
            };


            let resp = QueryClient::connect(endpoint.clone())
                .await
                .unwrap()
                .params(req)
                .await
                .map_err(|e| format!("{}", e))?;

            let external_chain_voting_inflation_rate = resp.into_inner().param.unwrap().value.parse::<f64>().unwrap();

            let external_chain_inflation = self
                .get_evm_supported_chains()
                .await
                .map(|res| res.len() as f64 * external_chain_voting_inflation_rate)
                .unwrap_or(default_return_value);

            inflation = external_chain_inflation + (inflation * 2.0);
        }

        Ok(inflation)
    }

    //Returns epoch provision
    pub async fn get_epoch_provision(&self) -> Result<f64, String> {

        let default_return_value = 0.0;
        let chain_name = self.config.name.clone();

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        if &chain_name == "evmos" {

        } else {

        }
        let epoch_provision = match chain_name.as_str() {
            "evmos" => self
                .rest_api_request::<EvmosInflationEpochProvisionResponse>("/evmos/inflation/v1/epoch_mint_provision", &[])
                .await?
                .epoch_mint_provision
                .amount
                .parse::<f64>()
                .unwrap_or(default_return_value),
            _ => self
                .rest_api_request::<EpochProvisionResponse>(&format!("/{chain_name}/mint/v1beta1/epoch_provisions"), &[])
                .await?
                .epoch_provisions
                .parse::<f64>()
                .map_err(|e| e.to_string())?,
        };

        Ok(epoch_provision)
    }

    pub async fn get_mint_params(&self) -> Result<MintParamsResponse, String> {
        let chain_name = self.config.name.clone();
        let mint_params = self
            .rest_api_request::<MintParamsResponse>(&format!("/{chain_name}/mint/v1beta1/params"), &[])
            .await?;

        Ok(mint_params)
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

#[derive(Serialize, Deserialize)]
pub struct MintParamsResponse {
    #[serde(rename = "params")]
    pub params: Params,
}

#[derive(Serialize, Deserialize)]
pub struct Params {
    #[serde(rename = "mint_denom")]
    pub mint_denom: String,

    #[serde(rename = "genesis_epoch_provisions")]
    pub genesis_epoch_provisions: String,

    #[serde(rename = "epoch_identifier")]
    pub epoch_identifier: String,

    #[serde(rename = "reduction_period_in_epochs")]
    pub reduction_period_in_epochs: String,

    #[serde(rename = "reduction_factor")]
    pub reduction_factor: String,

    #[serde(rename = "distribution_proportions")]
    pub distribution_proportions: DistributionProportions,

    #[serde(rename = "minting_rewards_distribution_start_epoch")]
    pub minting_rewards_distribution_start_epoch: String,
}

#[derive(Serialize, Deserialize)]
pub struct DistributionProportions {
    #[serde(rename = "staking")]
    pub staking: String,

    #[serde(rename = "pool_incentives")]
    pub pool_incentives: String,

    #[serde(rename = "participation_rewards")]
    pub participation_rewards: String,

    #[serde(rename = "community_pool")]
    pub community_pool: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EpochProvisionResponse {
    pub epoch_provisions: String,
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
