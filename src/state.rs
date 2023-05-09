use std::collections::HashMap;
use std::env::VarError;
use std::fs::File;
use std::io;
use std::io::Read;

use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use tracing_subscriber::fmt::format;

use crate::chain::{Chain, IntermediateChainConfig};
use crate::database::DatabaseTR;
use crate::events::WsEvent;

/// The state of the server.
pub struct State {
    chains: HashMap<String, Chain>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AssetsManifest {
    files: Vec<String>,
}

impl State {
    /// Creates a new `State`.
    pub async fn new() -> State {
        let chain_configs = match std::env::var("OFFLINE") {
            Ok(var) if var == "true" => {
                let mut yml = String::new();
                File::open("Chains.yml").expect("Missing Chains.yml file").read_to_string(&mut yml).unwrap();
                let chain_configs: HashMap<String, IntermediateChainConfig> = serde_yaml::from_str(yml.as_str()).expect("Invalid Chains.yml format");
                chain_configs
            }
            _ => {
                let manifest: AssetsManifest = reqwest::get(format!("{}/chains.json", std::env::var("TNR_EXPLORER_ASSETS_URI").unwrap())).await.unwrap().json().await.unwrap();
                let mut chains = HashMap::new();
                for file in manifest.files {
                    let content = reqwest::get(format!("{}/{file}", std::env::var("TNR_EXPLORER_ASSETS_URI").unwrap())).await.unwrap().text().await.unwrap();
                    let current_config: HashMap<String, IntermediateChainConfig> = serde_yaml::from_str(content.as_str()).expect("Invalid Chains.yml format");
                    current_config.into_iter().for_each(|(name, config)| {
                        chains.entry(name).or_insert(config);
                    });
                }

                chains
            }
        };

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

        State { chains }
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
                loop {
                    let tx_clone = tx.clone();
                    match chain.subscribe_events(tx_clone).await {
                        Ok(_) => tracing::info!("Stopped listening events for {name}"),
                        Err(e) => tracing::error!("Failed listening events for {name}: {e}"),
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            });
        });
    }

    pub fn get_chains(&self) -> &HashMap<String, Chain> {
        &self.chains
    }
}
