use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use tokio::join;
use tonic::transport::Endpoint;

use super::{
    amount_util::TnrDecimal,
    others::{DenomAmount, Pagination},
};
use crate::database::ListDbResult;
use crate::routes::PaginationData;
use crate::utils::{bytes_to_dec, str_to_dec, val_address_to_bech32};
use crate::{chain::Chain, routes::ChainAmountItem};

impl Chain {
    /// Returns the total supply of all tokens.
    pub async fn get_supply_of_all_tokens(&self, config: PaginationData) -> Result<ListDbResult<ChainAmountItem>, String> {
        use crate::fetch::cosmos::bank::v1beta1::{query_client::QueryClient, QueryTotalSupplyRequest};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryTotalSupplyRequest {
            pagination: Some(config.into()),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .total_supply(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let mut supplies = vec![];

        for supply in resp.supply {
            supplies.push(self.string_amount_parser(supply.amount, Some(supply.denom)).await?);
        }

        Ok(ListDbResult {
            data: supplies,
            pagination: resp.pagination.unwrap_or_default().into(),
        })
    }

    /// Returns the supply of given token.
    pub async fn get_supply_by_denom(&self, denom: &str) -> Result<ChainAmountItem, String> {
        use crate::fetch::cosmos::bank::v1beta1::{query_client::QueryClient, QuerySupplyOfRequest};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let mut client = QueryClient::connect(endpoint).await.unwrap();

        let from_supply_list_chains = vec!["evmos", "umee", "quicksilver", "kyve", "c4e", "babylon-testnet"];
        if from_supply_list_chains.contains(&self.config.name.as_str()) {
            use crate::fetch::cosmos::bank::v1beta1::QueryTotalSupplyRequest;

            let req = QueryTotalSupplyRequest { pagination: None };

            let resp = client
                .total_supply(req)
                .await
                .map_err(|e| format!("{}", e))?
                .into_inner()
                .supply
                .iter()
                .find(|s| s.denom == denom)
                .cloned();

            if let Some(supply) = resp {
                let supply = self.string_amount_parser(supply.amount.clone(), Some(supply.denom.clone())).await?;

                return Ok(supply);
            };

            return Err("Token not found".to_string());
        };

        let req = QuerySupplyOfRequest { denom: denom.to_string() };

        let resp = client.supply_of(req).await.map_err(|e| format!("{}", e))?.into_inner();

        let amount = resp.amount.unwrap();
        let supply = self.string_amount_parser(amount.amount, Some(amount.denom)).await?;

        Ok(supply)
    }

    pub async fn get_evm_supported_chains(&self) -> Result<Vec<String>, String> {
        use crate::fetch::axelar::evm::v1beta1::{query_service_client::QueryServiceClient, ChainsRequest};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = ChainsRequest { status: 0 };

        let resp = QueryServiceClient::connect(endpoint)
            .await
            .unwrap()
            .chains(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        Ok(resp.chains)
    }

    pub async fn get_evm_chain_maintainers(&self, chain_name: &str) -> Result<Vec<String>, String> {
        use crate::fetch::axelar::nexus::v1beta1::{query_service_client::QueryServiceClient, ChainMaintainersRequest};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = ChainMaintainersRequest {
            chain: chain_name.to_string(),
        };

        let resp = QueryServiceClient::connect(endpoint)
            .await
            .unwrap()
            .chain_maintainers(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let mut maintainers = vec![];
        for addr in resp.maintainers {
            let bech32 = val_address_to_bech32(addr.as_slice(), format!("{}valoper", self.config.base_prefix));

            maintainers.push(bech32);
        }

        Ok(maintainers)
    }
    /// Returns the minting inflation rate of native coin of the chain.
    pub async fn get_inflation_rate(&self) -> Result<f64, String> {
        let default_return_value = 0.0;
        let chain_name = self.config.name.as_str();

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let mut inflation = if ["evmos", "echelon"].contains(&chain_name) {
            use crate::fetch::evmos::inflation::v1::{query_client::QueryClient, QueryInflationRateRequest};

            let req = QueryInflationRateRequest {};

            let resp = QueryClient::connect(endpoint.clone())
                .await
                .unwrap()
                .inflation_rate(req)
                .await
                .map_err(|e| format!("{}", e))?
                .into_inner();

            let rate = str_to_dec(resp.inflation_rate.as_str());
            rate.parse::<f64>().unwrap_or(default_return_value) / 100.0
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

            annual_provision / total_supply
        } else if ["c4e"].contains(&chain_name) {
            use crate::fetch::c4e::minter::v1beta1::{query_client::QueryClient, QueryInflationRequest};

            let req = QueryInflationRequest {};

            let resp = QueryClient::connect(endpoint.clone())
                .await
                .unwrap()
                .inflation(req)
                .await
                .map_err(|e| format!("{}", e))?
                .into_inner();

            let rate = str_to_dec(resp.inflation.as_str());
            rate.parse::<f64>().unwrap_or(default_return_value) / 100.0
        } else {
            use crate::fetch::cosmos::mint::v1beta1::{query_client::QueryClient, QueryInflationRequest};

            let req = QueryInflationRequest {};

            let resp = QueryClient::connect(endpoint.clone())
                .await
                .unwrap()
                .inflation(req)
                .await
                .map_err(|e| format!("{}", e))?
                .into_inner();

            let rate = bytes_to_dec(resp.inflation);
            rate.parse::<f64>().unwrap_or(default_return_value)
        };

        //Axelar calculation different than others. That is why we are overriding inflation variable here.
        if self.config.name == "axelar" {
            use crate::fetch::cosmos::params::v1beta1::{query_client::QueryClient, QueryParamsRequest};

            let req = QueryParamsRequest {
                subspace: "reward".to_string(),
                key: "ExternalChainVotingInflationRate".to_string(),
            };

            let rate_string = QueryClient::connect(endpoint)
                .await
                .unwrap()
                .params(req)
                .await
                .map_err(|e| format!("{}", e))?
                .into_inner()
                .param
                .ok_or_else(|| "Missing param".to_string())?
                .value
                .replace('\"', "");

            let external_chain_voting_inflation_rate = rate_string.parse::<f64>().unwrap_or(default_return_value);

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

        let epoch_provision = match chain_name.as_str() {
            "evmos" => {
                use crate::fetch::evmos::inflation::v1::{query_client::QueryClient, QueryEpochMintProvisionRequest};

                let req = QueryEpochMintProvisionRequest {};

                let resp = QueryClient::connect(endpoint)
                    .await
                    .unwrap()
                    .epoch_mint_provision(req)
                    .await
                    .map_err(|e| format!("{}", e))?
                    .into_inner();

                let epoch_mint_provision_amount = resp.epoch_mint_provision.map_or_else(|| "0".to_string(), |e| e.amount);

                let res = str_to_dec(epoch_mint_provision_amount.as_str());
                let parsed = self.string_amount_parser(res, None).await.unwrap_or_default();

                parsed.amount.to_f64().unwrap_or(default_return_value)
            }
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

    pub async fn get_distributor_param(&self) -> Result<DistributorParamResponse, String> {
        let chain_name = self.config.name.clone();
        let distributor_param = self
            .rest_api_request::<DistributorParamResponse>(&format!("/{chain_name}/distributor/v1beta1/params"), &[])
            .await?;

        Ok(distributor_param)
    }

    pub async fn get_share_param(&self) -> Result<TnrDecimal, String> {
        let distributor_param = self.get_distributor_param().await?;
        let mut share_param = TnrDecimal::from_f64(1.0).unwrap_or_default();
        if let Some(dp) = distributor_param
            .params
            .sub_distributors
            .iter()
            .find(|sd| sd.name == "inflation_and_fee_distributor")
        {
            for dest in dp.destinations.shares.iter() {
                let parsed_share = TnrDecimal::from_str_exact(&dest.share).unwrap_or_default();
                share_param = share_param.checked_sub(parsed_share).unwrap_or_default();
            }
        }

        Ok(share_param)
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
pub struct C4EInflationRateResp {
    /// Minting inflation rate. Eg: `"91.087708112747866100"`
    pub inflation: String,
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

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct DistributorParamResponse {
    pub params: DistributorParam,
}
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct DistributorParam {
    pub sub_distributors: Vec<SubDistributor>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct IdType {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
}
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct SubDistributor {
    pub name: String,
    pub sources: Vec<IdType>,
    pub destinations: DistributorDestination,
}
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct DistributorDestination {
    pub primary_share: IdType,
    pub burn_share: String,
    pub shares: Vec<DistributionShare>,
}
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct DistributionShare {
    pub name: String,
    pub share: String,
    pub destination: IdType,
}
