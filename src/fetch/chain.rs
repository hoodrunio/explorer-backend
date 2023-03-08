use serde::{Deserialize, Serialize};

use crate::{chain::Chain, routes::TNRAppError};

use super::amount_util::TnrDecimal;

impl Chain {
    pub async fn get_dashboard_info(&self) -> Result<ChainDashboardInfo, TNRAppError> {
        let market_cap = 0.0;
        let inflation_rate = self.get_inflation_rate().await?.value;
        let apr = self.get_apr().await?;
        let staking_poll = self.get_staking_pool().await?.value;
        let total_unbonded = staking_poll.unbonded as f64;
        let total_bonded = staking_poll.bonded as f64;
        let total_supply = self.get_supply_by_denom(&self.config.main_denom).await?.value.amount;
        let community_pool = self.get_community_pool().await?.value;

        Ok(ChainDashboardInfo {
            inflation_rate,
            apr,
            total_unbonded,
            total_bonded,
            total_supply,
            community_pool,
            market_cap,
            market_history: vec![],
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainDashboardInfo {
    pub market_cap: f64,
    pub inflation_rate: f64,
    pub apr: f64,
    pub total_unbonded: f64,
    pub total_bonded: f64,
    pub total_supply: TnrDecimal,
    pub community_pool: u64,
    pub market_history: Vec<MarketHistory>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketHistory {
    pub price: f64,
    pub latest_update: String,
    pub volume: f64,
    pub market_cap: f64,
}
