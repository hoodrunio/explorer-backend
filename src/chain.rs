use std::sync::Arc;

use crate::{data::ChainData, state::PATH};

/// The struct that represents any Cosmos based chain.
#[derive(Clone)]
pub struct Chain {
    pub inner: Arc<ChainConfig>,
}

impl Chain {
    /// Creates a new chain.
    pub fn new(chainfig: ChainConfig) -> Chain {
        #[allow(unused_must_use)]
        {
            // Create a directory for the chain.
            std::fs::create_dir_all(format!("{PATH}/{}", chainfig.name));
        }

        Chain { inner: Arc::new(chainfig) }
    }
}

/// The configuration of a chain.
pub struct ChainConfig {
    /// The name of the chain.
    pub name: &'static str,
    /// The optional Coin Gecko ID.
    pub gecko: Option<&'static str>,
    /// The base prefix of the chain.
    pub base_prefix: &'static str,
    /// The valoper prefix of the chain.
    pub valoper_prefix: &'static str,
    /// The cons prefix of the chain.
    pub cons_prefix: &'static str,
    /// The denom of the native coin.
    pub main_denom: &'static str,
    /// The RPC URL of the chain.
    pub rpc_url: &'static str,
    /// The JSON RPC URL of the chain.
    pub jsonrpc_url: Option<&'static str>,
    /// The REST API URL of the chain.
    pub rest_url: &'static str,
    /// The Web Socket URL of the chain.
    pub wss_url: &'static str,
    /// The Cosmos SDK version of the chain.
    pub sdk_version: u8,
    /// The decimals of the native coin of the chain.
    pub decimals_pow: u64,
    /// The request client.
    pub client: reqwest::Client,
    /// The chain data.
    pub data: ChainData,
}
