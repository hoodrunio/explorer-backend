use serde::{Deserialize, Serialize};

/// The chain params.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChainParams {
    pub staking: ChainParamsStaking,
    pub slashing: ChainParamsSlashing,
    pub gov: ChainParamsGov,
    pub distribution: ChainParamsDistribution, // TODO! Where to get data?
}

/// The staking params.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainParamsStaking {
    pub unbonding_time: u32,
    pub max_validators: u32,
    pub max_entries: u32,
    pub historical_entries: u32,
    pub bond_denom: String,
}

/// The slashing params.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainParamsSlashing {
    pub signed_blocks_window: u32,
    pub min_signed_per_window: f64,
    pub downtime_jail_duration: u32,
    pub slash_fraction_double_sign: f64,
    pub slash_fraction_downtime: f64,
}

/// The governance params.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainParamsGov {
    pub quorum: f64,
    pub threshold: f64,
    pub min_deposit: f64,
    pub voting_period: u32,
    pub max_deposit_period: u32,
}

/// The governance params.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainParamsDistribution {
    pub community_tax: f64,
    pub base_proposer_reward: f64,
    pub bonus_proposer_reward: f64,
    pub withdraw_addr_enabled: bool,
}

impl ChainParams {
    pub fn new() -> ChainParams {
        ChainParams {
            distribution: ChainParamsDistribution {
                community_tax: 0.0,
                base_proposer_reward: 0.0,
                bonus_proposer_reward: 0.0,
                withdraw_addr_enabled: false,
            },
            gov: ChainParamsGov {
                quorum: 0.0,
                threshold: 0.0,
                min_deposit: 0.0,
                voting_period: 0,
                max_deposit_period: 0,
            },
            slashing: ChainParamsSlashing {
                signed_blocks_window: 0,
                min_signed_per_window: 0.0,
                downtime_jail_duration: 0,
                slash_fraction_double_sign: 0.0,
                slash_fraction_downtime: 0.0,
            },
            staking: ChainParamsStaking {
                unbonding_time: 0,
                max_validators: 0,
                max_entries: 0,
                historical_entries: 0,
                bond_denom: "unknown".to_string(),
            },
        }
    }
}
