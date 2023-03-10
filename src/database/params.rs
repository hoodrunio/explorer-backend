use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::fetch::chain::TokenMarketHistory;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Params {
    pub staking: StakingParams,
    pub slashing: SlashingParams,
    pub gov: GovParams,
    pub distribution: DistributionParams,
    pub market_price_history: Option<TokenMarketPriceHistories>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenMarketPriceHistories {
    pub daily: TokenMarketPriceHistory,
}

impl From<TokenMarketHistory> for TokenMarketPriceHistories {
    fn from(value: TokenMarketHistory) -> Self {
        let market_caps = value
            .market_caps
            .into_iter()
            .map(|v| MarketChart {
                timestamp: v.timestamp,
                value: v.value,
            })
            .collect();

        let prices = value
            .prices
            .into_iter()
            .map(|v| MarketChart {
                timestamp: v.timestamp,
                value: v.value,
            })
            .collect();

        let total_volumes = value
            .total_volumes
            .into_iter()
            .map(|v| MarketChart {
                timestamp: v.timestamp,
                value: v.value,
            })
            .collect();

        Self {
            daily: TokenMarketPriceHistory {
                parity: value.parity,
                token_id: value.token_id,
                day_period: value.day_period,
                market_caps,
                prices,
                total_volumes,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenMarketPriceHistory {
    pub parity: String,
    pub token_id: String,
    pub day_period: String,
    pub market_caps: Vec<MarketChart>,
    pub prices: Vec<MarketChart>,
    pub total_volumes: Vec<MarketChart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketChart {
    pub timestamp: u64,
    pub value: f64,
}
