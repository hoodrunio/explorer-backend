use reqwest::Client;

use super::common::Chain;

pub struct Secret<'a> {
    name: &'static str,
    sdk_ver: usize,
    prefix: &'static str,
    logo: &'static str,
    decimals: usize,
    rest_api_url: &'static str,
    rpc_url: &'static str,
    client: &'a Client,
}

impl<'a> Chain for Secret<'a> {
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
impl<'a> Secret<'a> {
    /// Creates Secret mainnet chain.
    pub fn mainnet(client: &'a Client) -> Self {
        Secret {
            name: "secret",
            logo: "https://raw.githubusercontent.com/cosmos/chain-registry/master/secretnetwork/images/scrt.svg",

            prefix: "secret",

            sdk_ver: 45,
            decimals: 6,

            rpc_url: "https://rpc.cosmos.directory/secretnetwork",
            rest_api_url: "https://rest.cosmos.directory/secretnetwork",

            client,
        }
    }

    /// Creates Evmos testnet chain.
    pub fn testnet(client: &'a Client) -> Self {
        Secret {
            name: "secret-testnet",
            logo: "https://raw.githubusercontent.com/cosmos/chain-registry/master/secretnetwork/images/scrt.svg",

            prefix: "secret",

            sdk_ver: 45,
            decimals: 6,

            rpc_url: "",
            rest_api_url: "",

            client,
        }
    }
}
