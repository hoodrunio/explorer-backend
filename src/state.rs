use crate::chain::Chain;
use crate::data::ChainData;
use crate::utils::get_prices;
use std::sync::Arc;
use tokio::join; 
use crate::init_chain;

/// The state of the server.
pub struct State {
    axelar: Chain,
    evmos: Chain,
    kyve: Chain,
    osmosis: Chain,
    secret: Chain,
    reqwest_client: reqwest::Client,
}

impl State {
    /// Creates a new `State`.
    pub fn new() -> State {
        let client = reqwest::Client::new();

        State {
            axelar: init_chain!{
                name: "axelar",
                gecko: Some("axelar"),
                base_prefix: "axelar",
                valoper_prefix: "axelarvaloper",
                cons_prefix: "axelarvalcons",
                main_denom: "uaxl",
                rpc_url: "https://rpc.cosmos.directory/axelar",
                rest_url: "https://axelar-api.polkachu.com",
                wss_url: "wss://axelar-rpc.chainode.tech/websocket",
                sdk_version: 45,
                decimals_pow: 10000,
                client: client.clone(),
            },
            evmos: init_chain!{
                name: "evmos",
                gecko: Some("evmos"),
                base_prefix: "evmos",
                valoper_prefix: "evmosvaloper",
                cons_prefix: "evmosvalcons",
                main_denom: "aevmos",
                rpc_url: "https://rpc.cosmos.directory/evmos",
                rest_url: "https://evmos-api.polkachu.com",
                wss_url: "wss://rpc-evmos.ecostake.com/websocket",
                sdk_version: 45,
                decimals_pow: 10000000000000000,
                client: client.clone(),
            },
            kyve: init_chain!{
                name: "kyve",
                gecko: None,
                base_prefix: "kyve",
                valoper_prefix: "kyvevaloper",
                cons_prefix: "kyvevalcons",
                main_denom: "tkyve",
                rpc_url: "https://rpc.beta.kyve.network",
                rest_url: "https://api.beta.kyve.network",
                wss_url: "wss://rpc.beta.kyve.network/websocket",
                sdk_version: 45,
                decimals_pow: 10000,
                client: client.clone(),
            },
            osmosis: init_chain!{
                name: "osmosis",
                gecko: Some("osmosis"),
                base_prefix: "osmo",
                valoper_prefix: "osmovaloper",
                cons_prefix: "osmovalcons",
                main_denom: "uosmo",
                rpc_url: "https://rpc.cosmos.directory/osmosis",
                rest_url: "https://rest.cosmos.directory/osmosis",
                wss_url: "wss://rpc.osmosis.interbloc.org/websocket",
                sdk_version: 46,
                decimals_pow: 10000,
                client: client.clone(),
            },
            secret: init_chain!{
                name: "secret",
                gecko: Some("secret"),
                base_prefix: "secret",
                valoper_prefix: "secretvaloper",
                cons_prefix: "secretvalcons",
                main_denom: "uscrt",
                rpc_url: "https://rpc.cosmos.directory/secretnetwork",
                rest_url: "https://rest.cosmos.directory/secretnetwork",
                wss_url: "wss://scrt-rpc.blockpane.com/websocket",
                sdk_version: 45,
                decimals_pow: 10000,
                client: client.clone(),
            },
            reqwest_client: client,
        }
    }
    
    /// Returns the matched chain.
    pub fn get(&self, name: &str) -> Result<Chain, String> {
        match name {
            "axelar" => Ok(self.axelar.clone()),
            "evmos" => Ok(self.evmos.clone()),
            "kyve" => Ok(self.kyve.clone()),
            "osmosis" => Ok(self.osmosis.clone()),
            "secret" => Ok(self.secret.clone()),
            _ => Err(format!("{name} is not a supported chain.")),
        }
    }

    /// Updates all the chains' data.
    pub async fn update_data(&self) {
        join!(
            self.axelar.update_data(),
            self.evmos.update_data(),
            self.kyve.update_data(),
            self.osmosis.update_data(),
            self.secret.update_data(),
        );
    }

    /// Updates all the chains' data.
    pub fn subscribe_data(&self) {
        self.axelar.subscribe_data();
        self.evmos.subscribe_data();
        self.kyve.subscribe_data();
        self.osmosis.subscribe_data();
        self.secret.subscribe_data();
    }

    /// Updates all the prices' of chains.
    pub async fn update_prices(&self) {
        let prices = get_prices(self.reqwest_client.clone(), &["axelar", "evmos", "osmosis", "secret", ]).await;

        join!(
            self.axelar.update_price(prices.get("axelar")),
            self.evmos.update_price(prices.get("evmos")),
            self.osmosis.update_price(prices.get("osmosis")),
            self.secret.update_price(prices.get("secret")),
        );
    }
}
