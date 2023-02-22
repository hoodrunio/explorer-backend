use std::{fs, path::Path};

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::chain::Chain;

impl Chain {
    pub async fn cosmos_assets(&self) -> Result<Assets, String> {
        let base_assets_url = std::env::var("TNR_EXPLORER_ASSETS_URI").expect("TNR_EXPLORER_ASSETS_URI must be set in .env file");
        let full_cosmos_assets_url = format!("{}/cosmos/chain_assets.json", base_assets_url);

        let assets = self
            .external_rest_api_req::<Assets>(&self.client, Method::GET, &full_cosmos_assets_url, &[])
            .await?;

        Ok(assets)
    }

    pub async fn cosmos_chain_assets(&self) -> Result<Vec<ChainAsset>, String> {
        let assets = self.cosmos_assets().await?;

        Ok(assets.assets.into_iter().filter(|asset| asset.chain == self.config.name).collect())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Assets {
    #[serde(rename = "assets")]
    pub assets: Vec<ChainAsset>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChainAsset {
    #[serde(rename = "chain")]
    pub chain: String,

    #[serde(rename = "denom")]
    pub denom: String,

    #[serde(rename = "type")]
    pub asset_type: Type,

    #[serde(rename = "origin_chain")]
    pub origin_chain: String,

    #[serde(rename = "origin_denom")]
    pub origin_denom: String,

    #[serde(rename = "origin_type")]
    pub origin_type: Type,

    #[serde(rename = "symbol")]
    pub symbol: String,

    #[serde(rename = "decimals")]
    pub decimals: i64,

    #[serde(rename = "description")]
    pub description: Option<String>,

    #[serde(rename = "image")]
    pub image: String,

    #[serde(rename = "coinGeckoId")]
    pub coin_gecko_id: Option<String>,

    #[serde(rename = "enable")]
    pub enable: Option<bool>,

    #[serde(rename = "path")]
    pub path: Option<String>,

    #[serde(rename = "channel")]
    pub channel: Option<String>,

    #[serde(rename = "port")]
    pub port: Option<Port>,

    #[serde(rename = "counter_party")]
    pub counter_party: Option<CounterParty>,

    #[serde(rename = "contract")]
    pub contract: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CounterParty {
    #[serde(rename = "channel")]
    pub channel: String,

    #[serde(rename = "port")]
    pub port: String,

    #[serde(rename = "denom")]
    pub denom: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Type {
    #[serde(rename = "bep2")]
    Bep2,

    #[serde(rename = "bridge")]
    Bridge,

    #[serde(rename = "cw20")]
    Cw20,

    #[serde(rename = "erc20")]
    Erc20,

    #[serde(rename = "ibc")]
    Ibc,

    #[serde(rename = "native")]
    Native,

    #[serde(rename = "pool")]
    Pool,

    #[serde(rename = "staking")]
    Staking,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Port {
    #[serde(rename = "transfer")]
    Transfer,
}
