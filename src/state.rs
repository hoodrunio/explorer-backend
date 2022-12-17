use crate::chain::Chain;
use crate::database::DatabaseTR;
use crate::init_chain;
use tokio::join;

/// The state of the server.
pub struct State {
    axelar: Chain,
    evmos: Chain,
    kyve: Chain,
    osmosis: Chain,
    umee: Chain,
    reqwest_client: reqwest::Client,
    database: DatabaseTR,
}

impl State {
    /// Creates a new `State`.
    pub async fn new() -> State {
        let client = reqwest::Client::new();
        let database = DatabaseTR::new().await;

        State {
            axelar: init_chain! {
                name: "axelar",
                epoch: false,
                gecko: Some("axelar"),
                base_prefix: "axelar",
                valoper_prefix: "axelarvaloper",
                cons_prefix: "axelarvalcons",
                main_denom: "uaxl",
                rpc_url: "https://rpc.cosmos.directory/axelar",
                jsonrpc_url: None,
                rest_url: "https://axelar-api.polkachu.com",
                wss_url: "wss://axelar-rpc.chainode.tech/websocket",
                sdk_version: 45,
                decimals_pow: 100,
                client: client.clone(),
                database: database.clone().change_name("axelar"),
            },
            evmos: init_chain! {
                name: "evmos",
                epoch: true,
                gecko: Some("evmos"),
                base_prefix: "evmos",
                valoper_prefix: "evmosvaloper",
                cons_prefix: "evmosvalcons",
                main_denom: "aevmos",
                rpc_url: "https://rpc.cosmos.directory/evmos",
                jsonrpc_url: Some("https://eth.bd.evmos.org:8545/"),
                rest_url: "https://evmos-api.polkachu.com",
                wss_url: "wss://rpc-evmos.ecostake.com/websocket",
                sdk_version: 45,
                decimals_pow: 100000000000000,
                client: client.clone(),
                database: database.clone().change_name("evmos"),
            },
            kyve: init_chain! {
                name: "kyve",
                epoch: false,
                gecko: None,
                base_prefix: "kyve",
                valoper_prefix: "kyvevaloper",
                cons_prefix: "kyvevalcons",
                main_denom: "tkyve",
                rpc_url: "https://rpc.beta.kyve.network",
                jsonrpc_url: None,
                rest_url: "https://api.beta.kyve.network",
                wss_url: "wss://rpc.beta.kyve.network/websocket",
                sdk_version: 45,
                decimals_pow: 100,
                client: client.clone(),
                database: database.clone().change_name("kyve"),
            },
            osmosis: init_chain! {
                name: "osmosis",
                epoch: true,
                gecko: Some("osmosis"),
                base_prefix: "osmo",
                valoper_prefix: "osmovaloper",
                cons_prefix: "osmovalcons",
                main_denom: "uosmo",
                rpc_url: "https://rpc.cosmos.directory/osmosis",
                jsonrpc_url: None,
                rest_url: "https://rest.cosmos.directory/osmosis",
                wss_url: "wss://rpc.osmosis.interbloc.org/websocket",
                sdk_version: 45,
                decimals_pow: 100,
                client: client.clone(),
                database: database.clone().change_name("osmosis"),
            },
            umee: init_chain! {
                name: "umee",
                epoch: false,
                gecko: Some("umee"),
                base_prefix: "umee",
                valoper_prefix: "umeevaloper",
                cons_prefix: "umeevalcons",
                main_denom: "uumee",
                rpc_url: "https://rpc-umee.huginn.tech",
                jsonrpc_url: None,
                rest_url: "https://api.umee.huginn.tech",
                wss_url: "wss://rpc-umee.huginn.tech/websocket",
                sdk_version: 46,
                decimals_pow: 100,
                client: client.clone(),
                database: database.clone().change_name("umee"),
            },
            reqwest_client: client,
            database,
        }
    }

    /// Returns the matched chain.
    pub fn get(&self, name: &str) -> Result<Chain, String> {
        match name {
            "axelar" => Ok(self.axelar.clone()),
            "evmos" => Ok(self.evmos.clone()),
            "kyve" => Ok(self.kyve.clone()),
            "osmosis" => Ok(self.osmosis.clone()),
            "umee" => Ok(self.umee.clone()),
            _ => Err(format!("{name} is not a supported chain.")),
        }
    }

    /// Updates all the validator databases of chain.
    pub fn run_cron_jobs(&self) {
        self.axelar.cron_jobs_all();
        self.evmos.cron_jobs_all();
        self.kyve.cron_jobs_all();
        self.osmosis.cron_jobs_all();
        self.umee.cron_jobs_all();        
    }

    /// Subscribes to all the events for all the chains.
    pub async fn subscribe_to_events(&self) {
        join!(
            self.axelar.subscribe_to_events(),
            self.evmos.subscribe_to_events(),
            self.kyve.subscribe_to_events(),
            self.osmosis.subscribe_to_events(),
            self.umee.subscribe_to_events(),
        );
    }
}
