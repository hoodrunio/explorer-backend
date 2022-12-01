use std::collections::BTreeMap;
use std::fs;
use std::sync::Mutex;

use cosmrs::AccountId;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tokio::join;

use crate::{chain::Chain, utils::get_validator_logo};

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

    /// Returns validator metadatas by given block height.
    pub async fn get_validator_metadatas_by_height(&self, height: u64, proposer_hex_addr: &str) -> Result<Vec<ValidatorMetadata>, String> {

        todo!();
        
        let resp = self.get_validator_set(height).await?;
        let jobs: Vec<_> = resp
            .result
            .validators
            .into_iter()
            .map(|validator| async {
                match self.get_validator_metadata_by_valoper_addr(validator.address.clone()) {
                    Some(metadata) => Ok::<_, String>(metadata),
                    None => {
                        let resp = self.get_validator(&validator.address).await?;
                        let logo_url = get_validator_logo(self.inner.client.clone(), &resp.description.identity).await;

                        let metadata_raw = ValidatorMetadataRaw {
                            logo_url,
                            name: resp.description.moniker,
                        };

                        // Save the validator to the database.
                        match self.inner.data.db.valoper_to_metadata_map.lock() {
                            Ok(mut db) => {
                                db.insert(validator.address.clone(), metadata_raw.clone());
                            }
                            Err(_) => {
                                eprintln!("Cannot save data to database.")
                            }
                        };

                        let metadata = ValidatorMetadata {
                            address: validator.address,
                            logo_url: metadata_raw.logo_url,
                            name: metadata_raw.name,
                        };

                        Ok(metadata)
                    }
                }
            })
            .collect();

        let jobs_done = join_all(jobs).await;

        let mut validator_metadatas: Vec<ValidatorMetadata> = vec![];

        for job_done in jobs_done {
            validator_metadatas.push(job_done?)
        }

        Ok(validator_metadatas)
    }
}
