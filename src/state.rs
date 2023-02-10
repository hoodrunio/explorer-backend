use std::collections::HashMap;
use std::fs::File;

use futures::{stream, StreamExt};
use tokio::sync::broadcast::Sender;

use crate::chain::{Chain, IntermediateChainConfig};
use crate::database::DatabaseTR;
use crate::events::WsEvent;

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

        let chains = stream
            .then(|(name, config)| async move {
                let chain = match Chain::initialize(config, reqwest::Client::new(), DatabaseTR::new().await).await {
                    Ok(mut chain) => {
                        tracing::info!("Successfully initialized {name}");
                        chain.database = chain.database.change_name(&name);
                        Some(chain)
                    }
                    Err(e) => {
                        tracing::error!("Error initializing chain {name}: {e}");
                        None
                    }
                };
                (name, chain)
            })
            .filter(|(_name, chain)| futures::future::ready(chain.is_some()))
            .map(|(name, chain)| (name, chain.unwrap()))
            .collect::<HashMap<String, Chain>>()
            .await;

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
    pub async fn subscribe_to_events(&self, tx: Sender<(String, WsEvent)>) {
        self.chains.clone().into_iter().for_each(|(name, chain)| {
            let tx = tx.clone();

            tokio::spawn(async move {
                match chain.subscribe_to_events(tx).await {
                    Ok(_) => tracing::info!("Stopped listening events for {name}"),
                    Err(e) => tracing::error!("Failed listening events for {name}: {e}"),
                }
            });
        });
    }

    pub fn get_chains(&self) -> &HashMap<String, Chain> {
        &self.chains
    }
}
