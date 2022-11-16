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
            _ => (),
        },
        _ => (),
    }

    HashMap::new()
}

#[derive(Deserialize)]
pub struct CoinGeckoPrice {
    pub usd: f64,
}

/// Returns the logo url of the given validator.
pub async fn get_validator_logo(client: Client, validator_identity: &str) -> Result<String, String> {
    let url = format!("https://keybase.io/_/api/1.0/user/lookup.json?key_suffix={validator_identity}&fields=pictures");

    match client.get(url).send().await {
        Ok(resp) => match resp.json::<LogoResp>().await {
            Ok(resp) => match resp.them.get(0) {
                Some(picture) => Ok(picture.pictures.primary.url.clone()),
                None => Err(format!("There is no logo of the validator with identity, {validator_identity}")),
            },
            _z => Err(format!("Cannot parse the logo of the validator with identity, '{validator_identity}'.")),
        },
        _ => Err(format!("Cannot request the logo of the validator with identity, '{validator_identity}'.")),
    }
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
