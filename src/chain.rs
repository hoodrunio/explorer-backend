use serde::{Deserialize, Serialize};
use serde_json::Value;
use versions::SemVer;

use crate::database::DatabaseTR;

/// The struct that represents any Cosmos based chain.
#[derive(Clone)]
pub struct Chain {
    /// The request client.
    pub client: reqwest::Client,
    /// The request client.
    pub database: DatabaseTR,
    pub config: ChainConfig,
}

async fn get_sdk_ver(rest_url: &str, client: reqwest::Client) -> Result<SemVer, String> {
    let value: Value = client
        .get(&format!("{rest_url}/cosmos/base/tendermint/v1beta1/node_info"))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch the node_info {e}"))?
        .json()
        .await
        .map_err(|e| format!("Failed to deserialize json {e}"))?;

    value["application_version"]["cosmos_sdk_version"]
        .as_str()
        .map(|s| SemVer::parse(&s[1..]).map(|(_, v)| v))
        .transpose()
        .map_err(|e| format!("manually set the version for '{rest_url}: {e}'"))?
        .ok_or(format!("version info not found"))
}

async fn get_main_denom(rest_url: &str, client: reqwest::Client) -> Result<String, String> {
    let value: Value = client
        .get(&format!("{rest_url}/cosmos/staking/v1beta1/params"))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch the staking params {e}"))?
        .json()
        .await
        .map_err(|e| format!("Failed to fetch the node_info {e}"))?;

    value["params"]["bond_denom"]
        .as_str()
        .ok_or(format!("bond_demon not found"))
        .map(str::to_string)
}

impl Chain {
    /// Creates a new chain.
    pub async fn initialize(ic: IntermediateChainConfig, client: reqwest::Client, database: DatabaseTR) -> Result<Self, String> {
        let decimals: u8 = ic.decimals.unwrap_or(6);
        let decimals_pow = 10_u64.pow(decimals as u32 - 4);

        let (sdk_version, manual_versioning) = match ic.sdk_version {
            Some(version) => (version, true),
            None => (get_sdk_ver(&ic.rest_url, client.clone()).await?, false),
        };

        let main_denom = match ic.main_denom {
            Some(denom) => denom.to_string(),
            None => get_main_denom(&ic.rest_url, client.clone()).await?,
        };

        let chain_config = ChainConfig {
            name: ic.name.clone(),
            logo: ic.logo,
            epoch: ic.epoch.unwrap_or(false),
            gecko: ic.gecko,
            base_prefix: ic.prefix.unwrap_or_else(|| ic.name),
            main_symbol: ic.symbol,
            rpc_url: ic.rpc_url,
            jsonrpc_url: ic.jsonrpc_url,
            rest_url: ic.rest_url,
            wss_url: ic.wss_url,
            archive_url: ic.archive_url,
            main_denom,
            sdk_version,
            manual_versioning,
            decimals,
            decimals_pow,
        };

        Ok(Self {
            client: Default::default(),
            database,
            config: chain_config,
        })
    }

    pub fn new(chain_config: ChainConfig, client: reqwest::Client, database: DatabaseTR) -> Chain {
        Chain {
            config: chain_config,
            client,
            database,
        }
    }
}

/// The configuration of a chain.
#[derive(Clone)]
pub struct ChainConfig {
    /// The name of the chain.
    pub name: String,
    /// Logo url
    pub logo: String,
    /// The name of the chain.
    pub epoch: bool,
    /// The optional Coin Gecko ID.
    pub gecko: Option<String>,
    /// The base prefix of the chain.
    pub base_prefix: String,
    ///The Symbol of the chain token
    pub main_symbol: String,
    /// The valoper prefix of the chain.
    // pub valoper_prefix: String,
    /// The cons prefix of the chain.
    // pub cons_prefix: String,
    /// The denom of the native coin.
    pub main_denom: String,
    /// The RPC URL of the chain.
    pub rpc_url: String,
    /// The JSON RPC URL of the chain.
    pub jsonrpc_url: Option<String>,
    /// The REST API URL of the chain.
    pub rest_url: String,
    /// The Web Socket URL of the chain.
    pub wss_url: String,
    /// The REST API Archive Node URL of the chain.
    pub archive_url: String,
    /// The Cosmos SDK version of the chain.
    pub sdk_version: SemVer,
    /// Is the sdk version from the config or the actual chain
    pub manual_versioning: bool,
    /// decimals
    pub decimals: u8,
    /// The decimals of the native coin of the chain.
    pub decimals_pow: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IntermediateChainConfig {
    pub name: String,
    pub logo: String,
    pub epoch: Option<bool>,
    pub gecko: Option<String>,
    pub prefix: Option<String>,
    pub rpc_url: String,
    pub rest_url: String,
    pub wss_url: String,
    pub archive_url: String,
    pub decimals: Option<u8>,
    pub sdk_version: Option<SemVer>,
    pub jsonrpc_url: Option<String>,
    pub symbol: String,
    pub main_denom: Option<String>,
}
