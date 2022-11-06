use reqwest::Client;

use super::common::Chain;

pub struct Osmosis<'a> {
    name: &'static str,
    sdk_ver: usize,
    prefix: &'static str,
    logo: &'static str,
    decimals: usize,
    rest_api_url: &'static str,
    rpc_url: &'static str,
    client: &'a Client,
}

impl<'a> Chain for Osmosis<'a> {
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
impl<'a> Osmosis<'a> {
    /// Creates Osmosis mainnet chain.
    pub fn mainnet(client: &'a Client) -> Self {
        Osmosis {
            name: "osmosis",
            logo: "https://assets.coingecko.com/coins/images/16724/large/osmo.png",

            prefix: "osmo",

            sdk_ver: 45,
            decimals: 6,

            rpc_url: "https://rpc.cosmos.directory/osmosis",
            rest_api_url: "https://rest.cosmos.directory/osmosis",

            client,
        }
    }

    /// Creates Osmosis testnet chain.
    pub fn testnet(client: &'a Client) -> Self {
        Osmosis {
            name: "osmosis-testnet",
            logo: "https://assets.coingecko.com/coins/images/16724/large/osmo.png",

            prefix: "osmo",

            sdk_ver: 45,
            decimals: 6,

            rpc_url: "",
            rest_api_url: "",

            client,
        }
    }
}
