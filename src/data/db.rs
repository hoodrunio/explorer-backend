use std::fs;
use std::sync::Mutex;
use std::{collections::BTreeMap, sync::Arc};

use futures::future::join_all;
use serde::{Deserialize, Serialize};

use crate::{
    chain::Chain,
    fetch::others::PaginationConfig,
    state::PATH,
    utils::{convert_consensus_pub_key_to_hex_address, get_validator_logo},
};

/// Validator name and the URL of its logo.
#[derive(Clone, Serialize, Deserialize)]
pub struct ValidatorMetadataRaw {
    pub name: String,
    pub logo_url: String,
    pub cons_address: String,
}

/// Validator name and the URL of its logo.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ValidatorMetadata {
    pub name: String,
    pub logo_url: String,
    pub valoper_address: String,
    pub cons_address: String,
}

/// Validator name and the URL of its logo.
#[derive(Clone, Debug)]
pub struct ValidatorMetadataFull {
    pub name: String,
    pub logo_url: String,
    pub hex: String,
    pub valoper_address: String,
    pub cons_address: String,
}

/// The in-memory database implementation for validator names and monikers.
pub struct Db {
    pub hex_to_valoper_map: Mutex<BTreeMap<String, String>>,
    pub valoper_to_metadata_map: Mutex<BTreeMap<String, ValidatorMetadataRaw>>,
}

impl Db {
    pub fn new(chain: &str) -> Self {
        Self {
            hex_to_valoper_map: match fs::read_to_string(format!("{PATH}/{chain}/db_hex_to_valoper.json")) {
                Ok(db) => match serde_json::from_str::<BTreeMap<String, String>>(&db) {
                    Ok(db) => Mutex::new(db),
                    Err(_) => {
                        eprintln!("`{PATH}/{chain}/db_hex_to_valoper.json` is mistaken. Please don't modify it manually.");
                        Mutex::new(BTreeMap::new())
                    }
                },
                Err(_) => {
                    eprintln!("`{PATH}/{chain}/db_hex_to_valoper.json` is not found. Starting from scratch.");
                    Mutex::new(BTreeMap::new())
                }
            },
            valoper_to_metadata_map: match fs::read_to_string(format!("{PATH}/{chain}/db_valoper_to_metadata.json")) {
                Ok(db) => match serde_json::from_str::<BTreeMap<String, ValidatorMetadataRaw>>(&db) {
                    Ok(db) => Mutex::new(db),
                    Err(_) => {
                        eprintln!("`{PATH}/{chain}/db_valoper_to_metadata.json` is mistaken. Please don't modify it manually.");
                        Mutex::new(BTreeMap::new())
                    }
                },
                Err(_) => {
                    eprintln!("{PATH}/{chain}/db_valoper_to_metadata.json` is not found. Starting from scratch.");
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
    pub async fn get_validator_metadata_by_hex_addr(&self, hex_addr: String) -> Option<ValidatorMetadata> {
        let valoper_addr = self.inner.data.db.hex_to_valoper_map.lock().ok()?.get(&hex_addr)?.clone();
        self.get_validator_metadata_by_valoper_addr(valoper_addr).await.ok()
    }

    /// Returns the validator name, logo, and address by given HEX validator address.
    ///
    /// Example HEX address: `"7ADB6183B66D6D60C88692578A7E722269066F74"`
    pub fn get_validator_metadata_by_hex_addr_blocking(&self, hex_addr: String) -> Option<ValidatorMetadata> {
        let valoper_addr = self.inner.data.db.hex_to_valoper_map.lock().ok()?.get(&hex_addr)?.clone();
        self.get_validator_metadata_by_valoper_addr_blocking(valoper_addr)
    }

    /// Returns the validator name, logo, and address by given valoper prefixed validator address.
    ///
    /// Example valoper prefixed address: `"cosmosvaloper14l0fp639yudfl46zauvv8rkzjgd4u0zk2aseys"`
    pub async fn get_validator_metadata_by_valoper_addr(&self, valoper_addr: String) -> Result<ValidatorMetadata, String> {
        match self
            .inner
            .data
            .db
            .valoper_to_metadata_map
            .lock()
            .map_err(|_| "Cannot access to validator database.".to_string())?
            .get(&valoper_addr)
            .cloned()
        {
            Some(validator_metadata_raw) => Ok(ValidatorMetadata {
                valoper_address: valoper_addr,
                logo_url: validator_metadata_raw.logo_url,
                name: validator_metadata_raw.name,
                cons_address: validator_metadata_raw.cons_address,
            }),
            None => Err(format!("Cannot find the validator, {}.", valoper_addr)),
        }
    }

    /// Returns the validator name, logo, and address by given valoper prefixed validator address.
    ///
    /// Example valoper prefixed address: `"cosmosvaloper14l0fp639yudfl46zauvv8rkzjgd4u0zk2aseys"`
    pub fn get_validator_metadata_by_valoper_addr_blocking(&self, valoper_addr: String) -> Option<ValidatorMetadata> {
        match self.inner.data.db.valoper_to_metadata_map.lock().ok()?.get(&valoper_addr).cloned() {
            Some(validator_metadata_raw) => Some(ValidatorMetadata {
                valoper_address: valoper_addr,
                logo_url: validator_metadata_raw.logo_url,
                name: validator_metadata_raw.name,
                cons_address: validator_metadata_raw.cons_address,
            }),
            None => None,
        }
    }

    /// Returns validator metadatas by given block height.
    pub async fn _get_validator_metadatas_by_height(&self, _height: u64, _proposer_hex_addr: &str) -> Result<Vec<ValidatorMetadata>, String> {
        todo!();

        /*
        let _resp = self.get_validator_set(height).await?;
        let jobs: Vec<_> = _resp
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
        */
    }

    /// Saves given validator to the validator database.
    pub fn save_validator_to_database(&self, validator: ValidatorMetadata) {
        if let Ok(mut map) = self.inner.data.db.valoper_to_metadata_map.lock() {
            map.insert(
                validator.valoper_address,
                ValidatorMetadataRaw {
                    name: validator.name,
                    logo_url: validator.logo_url,
                    cons_address: validator.cons_address,
                },
            );
        }
    }

    /// Saves validators to the validator database.
    pub fn save_validators_to_database(&self, validators: Vec<ValidatorMetadataFull>) {
        let mut hex_to_valoper_map = match self.inner.data.db.hex_to_valoper_map.lock() {
            Ok(map) => map,
            Err(_) => return,
        };

        let mut valoper_to_metadata_map = match self.inner.data.db.valoper_to_metadata_map.lock() {
            Ok(map) => map,
            Err(_) => return,
        };

        // Save validators to the database.
        for validator in validators {
            hex_to_valoper_map.insert(validator.hex.clone(), validator.valoper_address.clone());

            valoper_to_metadata_map.insert(
                validator.valoper_address,
                ValidatorMetadataRaw {
                    name: validator.name,
                    logo_url: validator.logo_url,
                    cons_address: validator.cons_address,
                },
            );
        }

        // Save hex-to-valoper database to a JSON file.
        match serde_json::to_string::<BTreeMap<_, _>>(&hex_to_valoper_map) {
            Ok(contents) => {
                let path = format!("{PATH}/{chain}/db_hex_to_valoper.json", chain = self.inner.name);
                if let Err(error) = std::fs::write(path, contents) {
                    eprintln!("Database saving error\n{error}")
                };
            }
            Err(error) => eprintln!("Database saving error\n{error}"),
        }
        drop(hex_to_valoper_map);

        // Save valoper-to-metadata database to a JSON file.
        match serde_json::to_string::<BTreeMap<_, _>>(&valoper_to_metadata_map) {
            Ok(contents) => {
                let path = format!("{PATH}/{chain}/db_valoper_to_metadata.json", chain = self.inner.name);
                if let Err(error) = std::fs::write(path, contents) {
                    eprintln!("Database saving error\n{error}")
                };
            }
            Err(error) => eprintln!("Database saving error\n{error}"),
        }

        drop(valoper_to_metadata_map);
    }

    /// Adds new validators to the database.
    pub async fn update_validator_database(&self) {
        let resp = self.get_validators_unspecified(PaginationConfig::new().limit(10000)).await;

        let cons_address_map = match self.get_validator_set().await {
            Ok(resp) => Arc::new(
                resp.value
                    .into_iter()
                    .map(|validator| (validator.pub_key.key, validator.address))
                    .collect::<BTreeMap<String, String>>(),
            ),
            Err(err) => return eprintln!("Database job:\n{}", err),
        };

        let mut validators_future = vec![];

        match resp {
            Ok(resp) => {
                for validator in resp.validators {
                    let cons_address_map = cons_address_map.clone();
                    validators_future.push(async move {
                        let hex_addr = convert_consensus_pub_key_to_hex_address(&validator.consensus_pubkey.key);

                        match hex_addr {
                            Some(hex_addr) => Some(match self.get_validator_metadata_by_hex_addr_blocking(hex_addr.clone()) {
                                Some(metadata) => ValidatorMetadataFull {
                                    valoper_address: validator.operator_address.clone(),
                                    hex: hex_addr,
                                    logo_url: metadata.logo_url,
                                    name: metadata.name,
                                    cons_address: metadata.cons_address,
                                },
                                None => ValidatorMetadataFull {
                                    name: validator.description.moniker,
                                    valoper_address: validator.operator_address.clone(),
                                    hex: convert_consensus_pub_key_to_hex_address(&validator.consensus_pubkey.key)
                                        .unwrap_or(format!("Error, {}", validator.operator_address)),
                                    logo_url: get_validator_logo(self.inner.client.clone(), &validator.description.identity).await,
                                    cons_address: cons_address_map.get(&validator.consensus_pubkey.key).cloned()?,
                                },
                            }),
                            _ => None,
                        }
                    });
                }
            }
            Err(error) => return eprintln!("Cannot update validator database. {}", error),
        };

        let validators = join_all(validators_future).await;

        let validators = validators.into_iter().flatten().collect();

        self.save_validators_to_database(validators);
    }
}
