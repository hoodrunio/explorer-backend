use async_trait::async_trait;
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
    pub fn new(client: &'a Client) -> Self {
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

#[tokio::test]
async fn test_blocks() {
    let client = reqwest::Client::new();
    let evmos_chain = Evmos::new(&client);

    // Get latest block.
    let res = evmos_chain.get_block_by_height(None).await;

    // It should be okay. Cuz there must be a latest block.
    assert!(res.is_ok());
    println!("{:#?}", res.unwrap());

    // Get the block at height `1`.
    let res = evmos_chain.get_block_by_height(Some(1)).await;

    // It should be an error. Cuz of pruning.
    assert!(res.is_err());
    println!("pretty error message: {}", res.unwrap_err());
}
