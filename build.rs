use std::collections::HashMap;
use std::fs::{read_to_string, write};
use std::path::Path;
use std::time::Duration;

use futures::future::join_all;
use reqwest::Client;
use serde_json::Value;

#[derive(Debug)]
pub struct Chain<'a> {
    pub name: &'a str,
    pub logo: &'a str,
    pub gecko: Option<&'a str>,
    pub epoch: bool,
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
fn main() {
    println!("bypassing build script");
}

// #[tokio::main]
// async fn main() {
//     Create `Client`.
// let client = Client::builder().timeout(Duration::from_secs(10)).build().unwrap();
//
// Read `Chains.yml` file.
// let chains_yml_content = read_to_string("Chains.yml").unwrap();
//
// Make iterable hash maps for chains.
// let content_parts: Vec<_> = chains_yml_content
//     .split("name")
//     .filter(|part| !part.is_empty())
//     .map(|part| String::from("name") + part.trim())
//     .collect();
//
// let chain_maps: Vec<_> = content_parts
//     .iter()
//     .map(|part| {
//         let mut chain_map = HashMap::new();
//         for line in part.lines() {
//             if !line.starts_with('#') {
//                 let (key, value) = line.split_once(": ").unwrap();
//                 chain_map.insert(key.trim(), value.trim());
//             }
//         }
//         chain_map
//     })
//     .collect();
//
// let jobs: Vec<_> = chain_maps.iter().map(|chain_map| create_chain(chain_map, client.clone())).collect();
//
// let chains = join_all(jobs).await;
//
// update_chains_yml(&chains);
// update_state_rs(&chains);
// }

async fn create_chain<'a>(chain_map: &HashMap<&'a str, &'a str>, client: Client) -> Chain<'a> {
    let name = chain_map.get("name").unwrap();
    let logo = chain_map.get("logo").unwrap();
    let epoch = chain_map.get("epoch").unwrap_or(&"false") == &"true";
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
        epoch,
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
    let value: Value = client
        .get(&format!("{rest_url}/cosmos/base/tendermint/v1beta1/node_info"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    value["application_version"]["cosmos_sdk_version"]
        .as_str()
        .expect(&*format!("{rest_url} api is unresponsive please try again"))[3..5]
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
        content += &format!("epoch: {}\n", chain.epoch);
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
    let mut run_cron_jobs_fn = String::new();
    let mut subscribe_to_events_fn = String::new();

    for chain in chains {
        state_props += &format!("\n    {}: Chain,", chain.name);

        new_fn += &format!(
            r#"
            {name}: init_chain! {{
                name: "{name}",
                epoch: {epoch},
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
                database: database.clone().change_name("{name}"),
            }},"#,
            name = chain.name,
            epoch = chain.epoch,
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

        run_cron_jobs_fn += &format!("\n        self.{chain}.cron_jobs_all();", chain = chain.name);

        subscribe_to_events_fn += &format!("\n            self.{chain}.subscribe_to_events(),", chain = chain.name);
    }

    let content = format!(
        "\
use crate::chain::Chain;
use crate::database::DatabaseTR;
use crate::init_chain;
use tokio::join;

/// The state of the server.
pub struct State {{{state_props}
    reqwest_client: reqwest::Client,
    database: DatabaseTR,
}}

impl State {{
    /// Creates a new `State`.
    pub async fn new() -> State {{
        let client = reqwest::Client::new();
        let database = DatabaseTR::new().await;

        State {{{new_fn}
            reqwest_client: client,
            database,
        }}
    }}

    /// Returns the matched chain.
    pub fn get(&self, name: &str) -> Result<Chain, String> {{
        match name {{{get_fn}
            _ => Err(format!(\"{{name}} is not a supported chain.\")),
        }}
    }}

    /// Updates all the validator databases of chain.
    pub fn run_cron_jobs(&self) {{{run_cron_jobs_fn}
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
