use std::collections::HashMap;
use std::fs::File;
use std::future::Future;
use futures::future::{join, join_all};
use futures::{stream, StreamExt};
use crate::chain::{Chain, ChainConfig, IntermediateChainConfig};
use crate::database::DatabaseTR;
use crate::init_chain;
use tokio::join;

/// The state of the server.
pub struct State {
    chains: HashMap<String, Chain>,
    reqwest_client: reqwest::Client,
    database: DatabaseTR,
}

impl State {
    /// Creates a new `State`.
    pub async fn new() -> State {
        let client = reqwest::Client::new();
        let database = DatabaseTR::new().await;
        let file = File::open("Chains.yml").expect("Missing Chains.yml file");
        let chain_configs: HashMap<String, IntermediateChainConfig> = serde_yaml::from_reader(file).expect("Invalid Chains.yml format");

        let stream = stream::iter(chain_configs.into_iter());

        let chains = stream.then(|(name, config)| async move {
            let chain = Chain::initialize(config, reqwest::Client::new(), DatabaseTR::new().await).await.unwrap();
            (name, chain)
        }).collect::<HashMap<String, Chain>>().await;

        State {
            chains,
            reqwest_client: client,
            database,
        }
    }

    /// Returns the matched chain.
    pub fn get(&self, name: &str) -> Result<Chain, String> {
        self.chains.get(name).cloned().ok_or_else(|| format!("{name} is not a supported chain"))
    }

    /// Updates all the validator databases of chain.
    pub fn run_cron_jobs(&self) {
        for chain in self.chains.values() {
            chain.cron_jobs_all();
        }
    }

    /// Subscribes to all the events for all the chains.
    pub async fn subscribe_to_events(&self) {
        join_all(self.chains.iter().map(|(_, c)| c.subscribe_to_events())).await;
    }
}
