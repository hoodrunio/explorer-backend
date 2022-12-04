use futures::future::join_all;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{read_to_string, write};
use std::path::Path;
use std::time::Duration;

#[derive(Debug)]
pub struct Chain<'a> {
    pub name: &'a str,
    pub logo: &'a str,
    pub gecko: Option<&'a str>,
    pub prefix: &'a str,
    pub main_denom: String,
    pub rpc_url: &'a str,
    pub rest_url: &'a str,
    pub wss_url: &'a str,
    pub decimals: u8,
    pub decimals_pow: u64,
    pub sdk_version: u8,
    pub manual_versioning: bool,
    pub json_rpc: Option<&'a str>,
}

#[tokio::main]
async fn main() {
    // Create `Client`.
    let client = Client::builder().timeout(Duration::from_secs(10)).build().unwrap();

    // Read `Chains.yml` file.
    let chains_yml_content = read_to_string("Chains.yml").unwrap();

    // Make iterable hash maps for chains.
    let content_parts: Vec<_> = chains_yml_content
        .split("name")
        .filter(|part| !part.is_empty())
        .map(|part| String::from("name") + part.trim())
        .collect();

    let chain_maps: Vec<_> = content_parts
        .iter()
        .map(|part| {
            let mut chain_map = HashMap::new();
            for line in part.lines() {
                if !line.starts_with('#') {
                    let (key, value) = line.split_once(": ").unwrap();
                    chain_map.insert(key.trim(), value.trim());
                }
            }
            chain_map
        })
        .collect();

    let jobs: Vec<_> = chain_maps.iter().map(|chain_map| create_chain(chain_map, client.clone())).collect();

    let chains = join_all(jobs).await;

    update_chains_yml(&chains);
    update_state_rs(&chains);
}

async fn create_chain<'a>(chain_map: &HashMap<&'a str, &'a str>, client: Client) -> Chain<'a> {
    let name = chain_map.get("name").unwrap();
    let logo = chain_map.get("logo").unwrap();
    let gecko = chain_map.get("gecko").map(|a| *a);
    let prefix = chain_map.get("prefix").unwrap_or(name);
    let rpc_url = chain_map.get("rpc").unwrap();
    let rest_url = chain_map.get("rest").unwrap();
    let wss_url = chain_map.get("wss").unwrap();
    let decimals: u8 = chain_map.get("decimals").unwrap_or(&"6").parse().unwrap();
    let decimals_pow = 10_u64.pow(decimals as u32 - 4);

    let (sdk_version, manual_versioning) = match chain_map.get("version") {
        Some(version) => (version[2..4].parse().unwrap(), true),
        None => (get_sdk_ver(rest_url, client.clone()).await, false),
    };

    let main_denom = match chain_map.get("denom") {
        Some(denom) => denom.to_string(),
        None => get_main_denom(rest_url, client.clone()).await,
    };

    let json_rpc = chain_map.get("jsonrpc").map(|a| *a);

    Chain {
        name,
        logo,
        gecko,
        prefix,
        main_denom,
        rpc_url,
        rest_url,
        wss_url,
        decimals,
        decimals_pow,
        sdk_version,
        manual_versioning,
        json_rpc,
    }
}

async fn get_sdk_ver(rest_url: &str, client: Client) -> u8 {
    let value: Value = client.get(&format!("{rest_url}/node_info")).send().await.unwrap().json().await.unwrap();

    value["application_version"]["cosmos_sdk_version"].as_str().unwrap()[3..5]
        .parse()
        .map_err(|_| format!("manually set the version for '{rest_url}'"))
        .unwrap()
}

async fn get_main_denom(rest_url: &str, client: Client) -> String {
    let value: Value = client
        .get(&format!("{rest_url}/cosmos/staking/v1beta1/params"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    value["params"]["bond_denom"].as_str().unwrap().to_string()
}

fn update_chains_yml(chains: &[Chain]) {
    let mut content = String::new();

    for chain in chains {
        content += &format!("name: {}\n", chain.name);
        if chain.gecko.is_some() {
            content += &format!("gecko: {}\n", chain.gecko.unwrap());
        } else {
            content += "# gecko: no token\n";
        };
        content += &format!("denom: {}\n", chain.main_denom);
        if chain.prefix != chain.name {
            content += &format!("prefix: {}\n", chain.prefix);
        } else {
            content += &format!("# prefix: {}\n", chain.prefix);
        };
        content += &format!("logo: {}\n", chain.logo);
        if chain.decimals != 6 {
            content += &format!("decimals: {}\n", chain.decimals);
        } else {
            content += &format!("# decimals: {}\n", chain.decimals);
        }
        if chain.manual_versioning {
            content += &format!("version: 0.{}\n", chain.sdk_version);
        } else {
            content += &format!("# version: 0.{}\n", chain.sdk_version);
        };
        if let Some(jsonrpc) = chain.json_rpc {
            content += &format!("jsonrpc: {}\n", jsonrpc);
        };
        content += &format!("rpc: {}\n", chain.rpc_url);
        content += &format!("rest: {}\n", chain.rest_url);
        content += &format!("wss: {}\n", chain.wss_url);
        content += "\n\n";
    }

    write("Chains.yml", content).unwrap();
}

fn update_state_rs(chains: &[Chain]) {
    let mut state_props = String::new();
    let mut new_fn = String::new();
    let mut get_fn = String::new();
    let mut update_data_fn = String::new();
    let mut update_prices_fn = String::new();
    let mut update_database_fn = String::new();
    let mut subscribe_to_events_fn = String::new();
    let mut get_prices_props = String::new();
    let path = format!(
        "{home}/.backend",
        home = std::env::var("HOME").expect("$HOME environment variable must be specified."),
    );

    for chain in chains {
        state_props += &format!("\n    {}: Chain,", chain.name);

        new_fn += &format!(
            r#"
            {name}: init_chain! {{
                name: "{name}",
                gecko: {gecko},
                base_prefix: "{fix}",
                valoper_prefix: "{fix}valoper",
                cons_prefix: "{fix}valcons",
                main_denom: "{main_denom}",
                rpc_url: "{rpc}",
                jsonrpc_url: {jsonrpc},
                rest_url: "{rest}",
                wss_url: "{wss}",
                sdk_version: {ver},
                decimals_pow: {dec_pow},
                client: client.clone(),
            }},"#,
            name = chain.name,
            gecko = chain
                .gecko
                .map(|gecko| format!("Some(\"{gecko}\")"))
                .unwrap_or_else(|| "None".to_string()),
            fix = chain.prefix,
            main_denom = chain.main_denom,
            rpc = chain.rpc_url,
            jsonrpc = chain
                .json_rpc
                .map(|json_rpc| format!("Some(\"{json_rpc}\")"))
                .unwrap_or_else(|| "None".to_string()),
            rest = chain.rest_url,
            wss = chain.wss_url,
            ver = chain.sdk_version,
            dec_pow = chain.decimals_pow,
        );

        get_fn += &format!("\n            \"{chain}\" => Ok(self.{chain}.clone()),", chain = chain.name);

        update_data_fn += &format!("\n            self.{chain}.update_data(),", chain = chain.name);

        update_database_fn += &format!("\n            self.{chain}.update_validator_database(),", chain = chain.name);

        subscribe_to_events_fn += &format!("\n            self.{chain}.subscribe_to_events(),", chain = chain.name);

        match chain.gecko {
            Some(gecko) => {
                update_prices_fn += &format!("\n            self.{chain}.update_price(prices.get(\"{gecko}\")),", chain = chain.name);
                get_prices_props += &format!("\"{gecko}\", ");
            }
            _ => (),
        }
    }

    let content = format!(
        "\
use crate::chain::Chain;
use crate::data::ChainData;
use crate::init_chain;
use crate::utils::get_prices;
use tokio::join;

pub const PATH: &str = \"{path}\";

/// The state of the server.
pub struct State {{{state_props}
    reqwest_client: reqwest::Client,
}}

impl State {{
    /// Creates a new `State`.
    pub fn new() -> State {{
        let client = reqwest::Client::new();

        State {{{new_fn}
            reqwest_client: client,
        }}
    }}

    /// Returns the matched chain.
    pub fn get(&self, name: &str) -> Result<Chain, String> {{
        match name {{{get_fn}
            _ => Err(format!(\"{{name}} is not a supported chain.\")),
        }}
    }}

    /// Updates all the chains' data.
    pub async fn update_data(&self) {{
        join!({update_data_fn}
        );
    }}

    /// Updates all the prices' of chains.
    pub async fn update_prices(&self) {{
        let prices = get_prices(self.reqwest_client.clone(), &[{get_prices_props}]).await;

        join!({update_prices_fn}
        );
    }}

    /// Updates all the validator databases of chain.
    pub async fn update_database(&self) {{
        join!({update_database_fn}
        );
    }}

    /// Subscribes to all the events for all the chains.
    pub async fn subscribe_to_events(&self) {{
        join!({subscribe_to_events_fn}
        );
    }}
}}
"
    );

    write(Path::new("src/state.rs"), content).unwrap();
}
