use crate::data::ChainData;

/// The struct that represents any Cosmos based chain.
pub struct Chain {
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
    /// The REST API URL of the chain.
    pub rest_url: &'static str,
    /// The Web Socket URL of the chain.
    pub wss_url: &'static str,
    /// The Cosmos SDK version of the chain.
    pub sdk_version: u8,
    /// The decimals of the native coin of the chain.
    pub decimals: u8,
    /// The request client.
    pub client: reqwest::Client,
    /// The chain data.
    pub data: ChainData,
}
