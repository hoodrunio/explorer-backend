use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Params {
    pub staking: StakingParams,
    pub slashing: SlashingParams,
    pub gov: GovParams,
    pub distribution: DistributionParams,
}

/// The staking params.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StakingParams {
    pub unbonding_time: u32,
    pub max_validators: u32,
    pub max_entries: u32,
    pub historical_entries: u32,
    pub bond_denom: String,
}

/// The slashing params.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlashingParams {
    pub signed_blocks_window: u32,
    pub min_signed_per_window: f64,
    pub downtime_jail_duration: u32,
    pub slash_fraction_double_sign: f64,
    pub slash_fraction_downtime: f64,
}

/// The governance params.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GovParams {
    pub quorum: f64,
    pub threshold: f64,
    pub min_deposit: f64,
    pub voting_period: u32,
    pub max_deposit_period: u32,
}

/// The governance params.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DistributionParams {
    pub community_tax: f64,
    pub base_proposer_reward: f64,
    pub bonus_proposer_reward: f64,
    pub withdraw_addr_enabled: bool,
}