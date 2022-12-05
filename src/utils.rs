use base64::decode as from_base_64;
use hex::encode as to_hex;
use sha2::{Digest, Sha256};

use std::collections::HashMap;

use reqwest::Client;
use serde::Deserialize;

/// Returns the prices of coins with given Coin Gecko IDs.
pub async fn get_prices(client: Client, coin_ids: &[&'static str]) -> HashMap<String, f64> {
    const URL: &str = "https://api.coingecko.com/api/v3/simple/price";

    let query = &[("ids", coin_ids.join("%2C")), ("vs_currencies", "usd".to_string())];

    match client.get(URL).query(query).send().await {
        Ok(resp) => match resp.json::<HashMap<String, CoinGeckoPrice>>().await {
            Ok(price_map) => return price_map.iter().map(|(name, cgp)| (name.clone(), cgp.usd)).collect(),
            _ => HashMap::new(),
        },
        _ => HashMap::new(),
    }
}

#[derive(Deserialize)]
pub struct CoinGeckoPrice {
    pub usd: f64,
}

/// Returns the logo url of the given validator.
pub async fn get_validator_logo(client: Client, validator_identity: &str) -> String {
    let url = format!("https://keybase.io/_/api/1.0/user/lookup.json?key_suffix={validator_identity}&fields=pictures");

    if let Ok(resp) = client.get(url).send().await {
        if let Ok(json) = resp.json::<LogoResp>().await {
            if let Some(picture) = json.them.get(0) {
                return picture.pictures.primary.url.to_string();
            }
        }
    }

    // Here, we will set a URL as the default logo.
    String::from("example.com")
}

#[derive(Deserialize)]
pub struct LogoResp {
    pub them: Vec<Picture>,
}

#[derive(Deserialize)]
pub struct Picture {
    pub pictures: Pictures,
}

#[derive(Deserialize)]
pub struct Pictures {
    pub primary: Primary,
}

#[derive(Deserialize)]
pub struct Primary {
    pub url: String,
}

/// Converts consensus public key to hex address for finding the associated operator address.
pub fn convert_consensus_pub_key_to_hex_address(consensus_pubkey: &str) -> Option<String> {
    let mut hasher = Sha256::new();

    hasher.update(from_base_64(consensus_pubkey.as_bytes()).ok()?);

    let hash = hasher.finalize();

    let hex = to_hex(hash);

    if hex.len() < 40 {
        None
    } else {
        Some(hex[..40].to_uppercase())
    }
}
