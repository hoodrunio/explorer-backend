use serde::{Deserialize, Serialize};

/// The chain params.
#[derive(Debug, Serialize, Deserialize)]
pub struct Params {
    pub unbonding_time: u32,
    pub max_validators: u32,
    pub max_entries: u32,
    pub historical_entries: u32,
    pub bond_denom: String,
    pub signed_blocks_window: u32,
    pub min_signed_per_window: f64,
    pub downtime_jail_duration: u32, // seconds
    pub slash_fraction_double_sign: f64,
    pub slash_fraction_downtime: f64,
    pub quorum: f64,
    pub threshold: f64,
    pub veto_threshold: f64,
}

impl Params {
    pub fn new() -> Params {
        Params {
            unbonding_time: 0,
            max_validators: 0,
            max_entries: 0,
            historical_entries: 0,
            bond_denom: "unknown".to_string(),
            signed_blocks_window: 0,
            min_signed_per_window: 0.0,
            downtime_jail_duration: 0,
            slash_fraction_double_sign: 0.0,
            slash_fraction_downtime: 0.0,
            quorum: 0.0,
            threshold: 0.0,
            veto_threshold: 0.0,
        }
    }
}
