use reqwest::Client;

use super::common::Chain;

pub struct Evmos<'a> {
    name: &'static str,
    sdk_ver: usize,
    prefix: &'static str,
    logo: &'static str,
    decimals: usize,
    rest_api_url: &'static str,
    rpc_url: &'static str,
    client: &'a Client,
}

impl<'a> Chain for Evmos<'a> {
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
impl<'a> Evmos<'a> {
    /// Creates Evmos mainnet chain.
    pub fn mainnet(client: &'a Client) -> Self {
        Evmos {
            name: "evmos",
            logo: "https://assets.coingecko.com/coins/images/24023/large/evmos.png",

            prefix: "evmos",

            sdk_ver: 45,
            decimals: 18,

            rpc_url: "https://rpc.cosmos.directory/evmos",
            rest_api_url: "https://evmos-api.polkachu.com",

            client,
        }
    }

    /// Creates Evmos testnet chain.
    pub fn testnet(client: &'a Client) -> Self {
        Evmos {
            name: "evmos-testnet",
            logo: "https://assets.coingecko.com/coins/images/24023/large/evmos.png",

            prefix: "evmos",

            sdk_ver: 45,
            decimals: 18,

            rpc_url: "",
            rest_api_url: "",

            client,
        }
    }
}

#[tokio::test]
async fn test_get_block_by_hash() {
    let client = reqwest::Client::new();

    let chain = Evmos::mainnet(&client);

    let block = chain
        .get_block_by_hash("14B6BB26CF30A559AE3AD18B0E3640BC3FD819B1182830D359969E02BAB0F633")
        .await
        .unwrap();

    // Test blok height.
    assert_eq!(block.block.header.height, "6764887");
}

#[tokio::test]
async fn test_get_block_by_height() {
    let client = reqwest::Client::new();

    let chain = Evmos::mainnet(&client);

    let block = chain.get_block_by_height(Some(6764887)).await.unwrap();

    // Test blok height.
    assert_eq!(
        block.block_id.hash,
        "14B6BB26CF30A559AE3AD18B0E3640BC3FD819B1182830D359969E02BAB0F633"
    );
}

#[tokio::test]
async fn test_get_blockchain() {
    let client = reqwest::Client::new();

    let chain = Evmos::mainnet(&client);

    let blockchains = chain.get_blockchain(6764887, 6764907).await.unwrap();

    // Test blok height.
    assert_eq!(
        blockchains.block_metas[0].block_id.hash,
        "A7B1485206A77EC75A535BB5EE08071671B33BDEAB6E255B06C3D599007C64C5"
    );
}
