use reqwest::Client;

use crate::chains::{Axelar, Celestia, Cosmos, Evmos, Kyve, Osmosis, Secret};
use crate::fetch::Chain;

/// The state of the web server.
/// Stores reusable data.
pub struct ServerState {
    /// Stores all the chains.
    pub chains: Chains,
}

impl ServerState {
    /// Creates a new `ServerState`.
    pub fn new() -> Self {
        Self { chains: Chains::new() }
    }
}

/// Stores all the chains.
pub struct Chains {
    pub axelar: Axelar,
    pub celestia: Celestia,
    pub cosmos: Cosmos,
    pub evmos: Evmos,
    pub kyve: Kyve,
    pub osmosis: Osmosis,
    pub secret: Secret,
}

impl Chains {
    /// Creates a new chains object.
    pub fn new() -> Self {
        let client = Client::new();

        Self {
            axelar: Axelar::new(client.clone()),
            celestia: Celestia::new(client.clone()),
            cosmos: Cosmos::new(client.clone()),
            evmos: Evmos::new(client.clone()),
            kyve: Kyve::new(client.clone()),
            osmosis: Osmosis::new(client.clone()),
            secret: Secret::new(client.clone()),
        }
    }
}