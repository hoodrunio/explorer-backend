use super::common::{Chain, ChainUrls};

pub struct Evmos {
    name: &'static str,
    urls: ChainUrls,
    sdk_ver: usize,
}

impl Chain for Evmos {
    fn name(&self) -> &'static str {
        self.name
    }
    fn sdk_version(&self) -> usize {
        self.sdk_ver
    }
    fn urls(&self) -> &ChainUrls {
        &self.urls
    }
}

impl Evmos {
    /// Creates Evmos chain.
    pub fn new() -> Evmos {
        Evmos {
            name: "evmos",
            urls: ChainUrls {
                rest_api: "https://evmos-api.polkachu.com",
                rpc: "https://rpc.cosmos.directory/evmos",
            },
            sdk_ver: 45,
        }
    }
}

#[tokio::test]
async fn test() {
    let evmos_chain = Evmos::new();
    let client = reqwest::Client::new();

    // Get latest block.
    let res = evmos_chain.get_block_by_height(&client, None).await;

    // It should be okay. Cuz there must be a latest block.
    assert!(res.is_ok());
    println!("{:#?}", res.unwrap());

    // Get the block at height `1`.
    let res = evmos_chain.get_block_by_height(&client, Some(1)).await;

    // It should be an error. Cuz of pruning.
    assert!(res.is_err());
    println!("pretty error message: {}", res.unwrap_err());
}
