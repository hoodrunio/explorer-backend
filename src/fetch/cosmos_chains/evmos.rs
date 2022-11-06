use reqwest::Client;

use super::common::{Chain, ChainUrls};

pub struct Evmos<'a> {
    name: &'static str,
    urls: ChainUrls,
    sdk_ver: usize,
    client: &'a Client,
}

impl<'a> Chain for Evmos<'a> {
    fn name(&self) -> &'static str {
        self.name
    }
    fn sdk_version(&self) -> usize {
        self.sdk_ver
    }
    fn urls(&self) -> &ChainUrls {
        &self.urls
    }
    fn client(&self) -> &Client {
        self.client
    }
}

impl<'a> Evmos<'a> {
    /// Creates Evmos chain.
    pub fn init(client: &'a Client) -> Self {
        Evmos {
            name: "evmos",
            client,
            urls: ChainUrls {
                rest_api: "https://evmos-api.polkachu.com",
                rpc: "https://rpc.cosmos.directory/evmos",
            },
            sdk_ver: 45,
        }
    }
}
