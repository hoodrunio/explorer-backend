use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Chain {
    pub name: String,
    pub annual_provisions: Option<f64>,
    pub avg_block_time_24h: f64,
    pub block_per_year: f64,
    pub bonded_tokens_amount: f64,
    pub community_tax: f64,
    pub epoch_provisions: Option<f64>,
    pub inflation_rate: f64,
    pub unbonded_tokens_amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChainDashboardInfo {
    pub inflation_rate: f64,
    pub apr: f64,
    pub total_unbonded: f64,
    pub total_bonded: f64,
    pub total_supply: String,
    pub community_pool: u64,
}

impl Default for ChainDashboardInfo {
    fn default() -> Self {
        Self {
            inflation_rate: 0.0,
            apr: 0.0,
            total_unbonded: 0.0,
            total_bonded: 0.0,
            total_supply: "0".to_string(),
            community_pool: 0,
        }
    }
}
