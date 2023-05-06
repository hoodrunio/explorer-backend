use std::fmt::{Display, Formatter};
use base64::{
    engine::general_purpose::STANDARD,
    Engine
};
use bech32::ToBase32;
use chrono::DateTime;
use hex::encode as to_hex;
use prost::Message;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};

/// Returns the prices of coins with given Coin Gecko IDs.
// pub async fn get_prices(client: Client, coin_ids: &[&'static str]) -> HashMap<String, f64> {
//     const URL: &str = "https://api.coingecko.com/api/v3/simple/price";

//     let query = &[("ids", coin_ids.join("%2C")), ("vs_currencies", "usd".to_string())];

//     match client.get(URL).query(query).send().await {
//         Ok(resp) => match resp.json::<HashMap<String, CoinGeckoPrice>>().await {
//             Ok(price_map) => return price_map.iter().map(|(name, cgp)| (name.clone(), cgp.usd)).collect(),
//             _ => HashMap::new(),
//         },
//         _ => HashMap::new(),
//     }
// }

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
    String::from("https://raw.githubusercontent.com/testnetrunn/explorer-assets/main/validators/default/validator-default.webp")
}

#[derive(Deserialize, Debug)]
pub struct LogoResp {
    pub them: Vec<Picture>,
}

#[derive(Deserialize, Debug)]
pub struct Picture {
    pub pictures: Pictures,
}

#[derive(Deserialize, Debug)]
pub struct Pictures {
    pub primary: Primary,
}

#[derive(Deserialize, Debug)]
pub struct Primary {
    pub url: String,
}

#[derive(Debug, Clone)]
pub enum PubKeyParseError {
    UnknownType(String),
    DecodeError(prost::DecodeError)
}

impl Display for PubKeyParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use PubKeyParseError::*;
        match self {
            UnknownType(ty) => write!(f, "Unknown type: {ty}"),
            DecodeError(e) => write!(f, "Failed to decode: {e}")
        }
    }
}
pub fn get_key(data: prost_wkt_types::Any) -> Result<String, PubKeyParseError> {
    match data.type_url.as_str() {
        "cosmos.crypto.ed25519.PubKey" => {
            use crate::fetch::cosmos::crypto::ed25519::PubKey;
            let key = PubKey::decode(data.value.as_slice()).map_err(|e| PubKeyParseError::DecodeError(e))?;
            Ok(STANDARD.encode(key.key.as_slice()))
        }
        "cosmos.crypto.secp256k1.Pubkey" => {
            use crate::fetch::cosmos::crypto::secp256k1::PubKey;
            let key = PubKey::decode(data.value.as_slice()).map_err(|e| PubKeyParseError::DecodeError(e))?;
            Ok(STANDARD.encode(key.key.as_slice()))
        }
        "cosmos.crypto.secp256r1.Pubkey" => {
            use crate::fetch::cosmos::crypto::secp256r1::PubKey;
            let key = PubKey::decode(data.value.as_slice()).map_err(|e| PubKeyParseError::DecodeError(e))?;
            Ok(STANDARD.encode(key.key.as_slice()))
        }
        ty => Err(PubKeyParseError::UnknownType(ty.to_string()))
    }
}

/// Converts consensus public key to hex address for finding the associated operator address.
pub fn convert_consensus_pubkey_to_hex_address(consensus_pubkey: &str) -> Option<String> {
    let hex = base64_to_hex(consensus_pubkey).unwrap();
    if hex.len() < 40 {
        None
    } else {
        Some(hex[..40].parse().unwrap())
    }
}

/// Converts tx base64 to hex address.
pub fn convert_tx_to_hex(tx_base64: &str) -> Option<String> {
    base64_to_hex(tx_base64)
}

fn base64_to_hex(base64: &str) -> Option<String> {
    let mut hasher = Sha256::new();

    hasher.update(STANDARD.decode(base64.as_bytes()).ok()?);

    let hash = hasher.finalize();
    let hex = to_hex(hash);

    Some(hex.to_uppercase())
}

/// From "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward" to "Withdraw Delegator Reward".
pub fn get_msg_name(msg: &str) -> String {
    let name = match msg.split_once("Msg") {
        Some((_, name)) => name,
        _ => match msg.split('.').last() {
            Some(name) => name,
            _ => msg,
        },
    };

    name.chars()
        .map(|ch| if ch.is_uppercase() { format!(" {ch}") } else { ch.to_string() })
        .collect::<Vec<_>>()
        .join("")
        .trim_start()
        .to_string()
}

/// Converts consensus pubkey to consensus address.
pub fn convert_consensus_pubkey_to_consensus_address(address: &str, prefix: &str) -> String {
    bech32::encode(
        prefix,
        hex::decode(convert_consensus_pubkey_to_hex_address(address).unwrap())
            .unwrap()
            .to_base32(),
        bech32::Variant::Bech32,
    )
    .unwrap()
}

pub trait Base64Convert {
    fn base64_to_string(&self) -> String;
}

impl Base64Convert for String {
    fn base64_to_string(&self) -> Self {
        let default_res = String::from("");
        match STANDARD.decode(self) {
            Ok(decode) => String::from_utf8(decode).unwrap_or(default_res),
            Err(_) => default_res,
        }
    }
}

pub fn ts_to_ms(ts: &String) -> Result<i64, String> {
    let date_time = DateTime::parse_from_rfc3339(ts).map_err(|_| format!("Cannot parse timestamp, '{}'.", ts))?;

    Ok(date_time.timestamp_millis())
}
