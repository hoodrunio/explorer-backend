use reqwest::Client;

use crate::chains::{Axelar, Celestia, Cosmos, Evmos, Kyve, Osmosis, Secret};
use crate::fetch::Chain;

/// The state of the web server.
/// Stores reusable data.
pub struct ServerState<'a> {
    /// The `reqwest::Client` to make requests.
    reqwest_client: reqwest::Client,
    /// Stores all the chains.
    chains: Chains<'a>,
}

impl<'a> ServerState<'a> {
    /// Creates a new `ServerState`.
    pub fn new() -> Self {
        todo!()
    }
}

/// Stores all the chains.
pub struct Chains<'a> {
    axelar: Axelar<'a>,
    celestia: Celestia<'a>,
    cosmos: Cosmos<'a>,
    evmos: Evmos<'a>,
    kyve: Kyve<'a>,
    osmosis: Osmosis<'a>,
    secret: Secret<'a>,
}

impl<'a> Chains<'a> {
    /// Creates a new chains object.
    pub fn new(client: &'a reqwest::Client) -> Self {
        Self {
            axelar: Axelar::new(&client),
            celestia: Celestia::new(&client),
            cosmos: Cosmos::new(&client),
            evmos: Evmos::new(&client),
            kyve: Kyve::new(&client),
            osmosis: Osmosis::new(&client),
            secret: Secret::new(&client),
        }
    }
}
