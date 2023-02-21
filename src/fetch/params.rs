use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Display};
use blockscout_display_bytes::Bytes as DisplayBytes;

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

//Historical data db struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoricalValidatorData {
    pub operator_address: String,
    pub voting_power_data: Vec<VotingPower>,
}

//Voting power db struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VotingPower {
    pub voting_power: f64,
    pub voting_power_percentage: f64,
    pub ts: i64,
}

impl VotingPower {
    pub fn init(self) -> Self {
        VotingPower {
            voting_power: self.voting_power,
            voting_power_percentage: self.voting_power_percentage,
            ts: self.ts,
        }
    }
}

impl Default for VotingPower {
    fn default() -> Self {
        Self {
            voting_power: 0.0,
            voting_power_percentage: 0.0,
            ts: Utc::now().timestamp_millis(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ContractData {
    pub contract_address: String,
    pub result: VerificationResult
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BytecodePart {
    Main { data: DisplayBytes },
    Meta { data: DisplayBytes },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct VerificationResult {
    pub file_name: String,
    pub contract_name: String,
    pub compiler_version: String,
    pub evm_version: String,
    pub constructor_arguments: Option<DisplayBytes>,
    pub optimization: Option<bool>,
    pub optimization_runs: Option<usize>,
    pub contract_libraries: BTreeMap<String, String>,
    pub abi: Option<String>,
    pub sources: BTreeMap<String, String>,
    pub compiler_settings: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_creation_input_parts: Option<Vec<BytecodePart>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_deployed_bytecode_parts: Option<Vec<BytecodePart>>,
}