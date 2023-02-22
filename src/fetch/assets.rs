use std::sync::Arc;

use chrono::{Duration, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::chain::Chain;

struct AssetsPool {
    list: Mutex<Vec<ChainAsset>>,
    last_timestamp: Mutex<usize>,
}
thread_local! {
    static ASSETS_MUTEX: Arc<AssetsPool> = Arc::new(AssetsPool{
        list: Mutex::new(vec![]),
        last_timestamp: Mutex::new(0),
    });
}

impl AssetsPool {
    pub fn get_instance() -> Arc<AssetsPool> {
        ASSETS_MUTEX.with(|assets_pool| assets_pool.clone())
    }
}

impl Chain {
    pub async fn cosmos_assets(&self) -> Result<Assets, String> {
        let base_assets_url = std::env::var("TNR_EXPLORER_ASSETS_URI").expect("TNR_EXPLORER_ASSETS_URI must be set in .env file");
        let full_cosmos_assets_url = format!("{}/cosmos/chain_assets.json", base_assets_url);
        let current_timestamp = Utc::now().timestamp() as usize;

        let mutex_poll = AssetsPool::get_instance();
        let mut last_fetched_timestamp = mutex_poll.last_timestamp.lock().await;
        let mut last_fetched_list = mutex_poll.list.lock().await;

        let upper_timestamp_constraint = *last_fetched_timestamp + Duration::minutes(5).num_seconds() as usize;
        let should_fetch = current_timestamp > upper_timestamp_constraint;

        if should_fetch {
            let assets = self
                .external_rest_api_req::<Assets>(&self.client, Method::GET, &full_cosmos_assets_url, &[])
                .await?;
            *last_fetched_timestamp = current_timestamp;
            *last_fetched_list = assets.assets;
        }

        Ok(Assets {
            assets: last_fetched_list.clone(),
        })
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CounterParty {
    #[serde(rename = "channel")]
    pub channel: String,

    #[serde(rename = "port")]
    pub port: String,

    #[serde(rename = "denom")]
    pub denom: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Port {
    #[serde(rename = "transfer")]
    Transfer,
}
