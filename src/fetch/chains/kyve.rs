use reqwest::Client;

use super::common::Chain;

pub struct Kyve<'a> {
    name: &'static str,
    sdk_ver: usize,
    prefix: &'static str,
    logo: &'static str,
    decimals: usize,
    rest_api_url: &'static str,
    rpc_url: &'static str,
    client: &'a Client,
}

impl<'a> Chain for Kyve<'a> {
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
impl<'a> Kyve<'a> {
    /// Creates Kyve mainnet chain.
    pub fn mainnet(client: &'a Client) -> Self {
        Kyve {
            name: "kyve",
            logo: "https://assets.coingecko.com/coins/images/26229/large/78351592.png",

            prefix: "kyve",

            sdk_ver: 45,
            decimals: 6,

            rpc_url: "https://rpc.beta.kyve.network",
            rest_api_url: "https://api.beta.kyve.network",

            client,
        }
    }

    /// Creates Kyve testnet chain.
    pub fn testnet(client: &'a Client) -> Self {
        Kyve {
            name: "kyve-testnet",
            logo: "https://assets.coingecko.com/coins/images/24023/large/evmos.png",

            prefix: "kyve",

            sdk_ver: 45,
            decimals: 6,

            rpc_url: "",
            rest_api_url: "",

            client,
        }
    }
}
