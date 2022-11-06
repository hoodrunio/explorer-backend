use reqwest::Client;

use super::common::Chain;

pub struct Celestia<'a> {
    name: &'static str,
    sdk_ver: usize,
    prefix: &'static str,
    logo: &'static str,
    decimals: usize,
    rest_api_url: &'static str,
    rpc_url: &'static str,
    client: &'a Client,
}

impl<'a> Chain for Celestia<'a> {
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
impl<'a> Celestia<'a> {
    /// Creates Celestia mainnet chain.
    pub fn celestia(client: &'a Client) -> Self {
        Celestia {
            name: "celestia",
            logo: "",

            prefix: "celestia",

            sdk_ver: 45,
            decimals: 6,

            rpc_url: "https://rpc.celestia.testnet.run",
            rest_api_url: "https://api.celestia.testnet.run",

            client,
        }
    }

    /// Creates Celestia testnet chain.
    pub fn testnet(client: &'a Client) -> Self {
        Celestia {
            name: "celestia-testnet",
            logo: "",

            prefix: "celestia",

            sdk_ver: 45,
            decimals: 6,

            rpc_url: "",
            rest_api_url: "",

            client,
        }
    }
}
