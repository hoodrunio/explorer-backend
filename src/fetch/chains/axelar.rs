use reqwest::Client;

use super::common::Chain;

pub struct Axelar<'a> {
    name: &'static str,
    sdk_ver: usize,
    prefix: &'static str,
    logo: &'static str,
    decimals: usize,
    rest_api_url: &'static str,
    rpc_url: &'static str,
    client: &'a Client,
}

impl<'a> Chain for Axelar<'a> {
    fn name(&self) -> &'static str {
        self.name
    }
    fn sdk_version(&self) -> usize {
        self.sdk_ver
    }
    fn client(&self) -> &Client {
        self.client
    }
    fn base_prefix(&self) -> &'static str {
        self.name
    }
    fn logo(&self) -> &'static str {
        self.logo
    }
    fn decimals(&self) -> usize {
        self.decimals
    }
    fn rest_api_url(&self) -> &'static str {
        self.rest_api_url
    }
    fn rpc_url(&self) -> &'static str {
        self.rpc_url
    }
}

//
// Mainnets.
//
impl<'a> Axelar<'a> {
    /// Creates Axelar mainnet chain.
    pub fn mainnet(client: &'a Client) -> Self {
        Axelar {
            name: "axelar",
            logo: "https://assets.coingecko.com/coins/images/24489/large/tsYr25vB_400x400.jpg",

            prefix: "axelar",

            sdk_ver: 45,
            decimals: 6,

            rpc_url: "https://rpc.cosmos.directory/axelar",
            rest_api_url: "https://axelar-api.polkachu.com",

            client,
        }
    }

    /// Creates Axelar testnet chain.
    pub fn testnet(client: &'a Client) -> Self {
        Axelar {
            name: "axelar-testnet",
            logo: "https://assets.coingecko.com/coins/images/24489/large/tsYr25vB_400x400.jpg",

            prefix: "axelar",

            sdk_ver: 45,
            decimals: 6,

            rpc_url: "",
            rest_api_url: "",

            client,
        }
    }
}
