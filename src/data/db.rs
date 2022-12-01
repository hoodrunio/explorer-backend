use std::collections::BTreeMap;
use std::fs;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::chain::Chain;

/// Validator name and the URL of its logo.
#[derive(Clone, Serialize, Deserialize)]
pub struct ValidatorMetadataRaw {
    pub name: String,
    pub logo_url: String,
}

/// Validator name and the URL of its logo.
#[derive(Clone)]
pub struct ValidatorMetadata {
    pub address: String,
    pub name: String,
    pub logo_url: String,
}

/// The in-memory database implementation for validator names and monikers.
pub struct Db {
    pub hex_to_valoper_map: Mutex<BTreeMap<String, String>>,
    pub valoper_to_metadata_map: Mutex<BTreeMap<String, ValidatorMetadataRaw>>,
}

impl Db {
    pub fn new(chain: &str) -> Self {
        Self {
            hex_to_valoper_map: match fs::read_to_string(format!("~/.backend/{chain}/db-hex-to-valoper.json")) {
                Ok(db) => match serde_json::from_str::<BTreeMap<String, String>>(&db) {
                    Ok(db) => Mutex::new(db),
                    Err(_) => {
                        eprintln!("`~/.backend/{chain}/db-hex-to-valoper.json` is mistaken. Please don't modify it manually.");
                        Mutex::new(BTreeMap::new())
                    }
                },
                Err(_) => {
                    eprintln!("`~/.backend/{chain}/db-hex-to-valoper.json` is not found. Starting from scratch.");
                    Mutex::new(BTreeMap::new())
                }
            },
            valoper_to_metadata_map: match fs::read_to_string(format!("~/.backend/{chain}/db-valoper-to-metadata.json")) {
                Ok(db) => match serde_json::from_str::<BTreeMap<String, ValidatorMetadataRaw>>(&db) {
                    Ok(db) => Mutex::new(db),
                    Err(_) => {
                        eprintln!("`~/.backend/{chain}/db-valoper-to-metadata.json` is mistaken. Please don't modify it manually.");
                        Mutex::new(BTreeMap::new())
                    }
                },
                Err(_) => {
                    eprintln!("`~/.backend/{chain}/db-valoper-to-metadata.json` is not found. Starting from scratch.");
                    Mutex::new(BTreeMap::new())
                }
            },
        }
    }
}

impl Chain {
    /// Returns the validator name, logo, and address by given HEX validator address.
    ///
    /// Example HEX address: `"7ADB6183B66D6D60C88692578A7E722269066F74"`
    pub fn get_validator_metadata_by_hex_addr(&self, hex_addr: String) -> Option<ValidatorMetadata> {
        let valoper_addr = self.inner.data.db.hex_to_valoper_map.lock().ok()?.get(&hex_addr)?.clone();
        self.get_validator_metadata_by_valoper_addr(valoper_addr)
    }

    /// Returns the validator name, logo, and address by given valoper prefixed validator address.
    ///
    /// Example valoper prefixed address: `"cosmosvaloper14l0fp639yudfl46zauvv8rkzjgd4u0zk2aseys"`
    pub fn get_validator_metadata_by_valoper_addr(&self, valoper_addr: String) -> Option<ValidatorMetadata> {
        let validator_metadata_raw = self.inner.data.db.valoper_to_metadata_map.lock().ok()?.get(&valoper_addr)?.clone();
        Some(ValidatorMetadata {
            address: valoper_addr,
            name: validator_metadata_raw.name,
            logo_url: validator_metadata_raw.logo_url,
        })
    }
}
