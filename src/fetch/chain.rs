use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serde_json::Number;

use crate::{chain::Chain, routes::TNRAppError};

use super::amount_util::TnrDecimal;

impl Chain {
    pub async fn get_dashboard_info(&self) -> Result<ChainDashboardInfoResponse, TNRAppError> {
        let market_cap = 0.0;
        let inflation_rate = self.get_inflation_rate().await?.value;
        let apr = self.get_apr().await.unwrap_or(0.0);

        let mut total_unbonded = 0.0;
        let mut total_bonded = 0.0;
        if let Ok(result) = self.get_staking_pool().await {
            total_unbonded = result.value.unbonded as f64;
            total_bonded = result.value.bonded as f64;
        };

        let total_supply = self
            .get_supply_by_denom(&self.config.main_denom)
            .await
            .map(|res| res.value.amount)
            .unwrap_or(TnrDecimal::ZERO);

        let community_pool = self.get_community_pool().await.map(|res| res.value).unwrap_or(0);

        Ok(ChainDashboardInfoResponse {
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

    pub async fn get_stats(&self) -> Result<ChainStatsInfoResponse, TNRAppError> {
        let latest_block_height: u64 = self
            .database
            .find_last_count_blocks(None, 1)
            .await
            .map(|blocks| blocks.first().map(|block| block.height).unwrap_or(0))
            .unwrap_or(0);

        let average_block_time = 0.0;
        let price = 0.0;
        let active_validators = self
            .database
            .find_validators(Some(doc! {"$match":{"is_active":true}}))
            .await
            .map(|res| res.len() as u16)
            .unwrap_or(0);

        Ok(ChainStatsInfoResponse {
            latest_block_height,
            average_block_time,
            price,
            active_validator_count: active_validators,
        })
    }

    pub async fn get_token_market_chart(
        &self,
        token_id: String,
        parity: Option<String>,
        day_period: Option<String>,
    ) -> Result<TokenMarketHistory, String> {
        let parity = parity.unwrap_or("usd".to_string());
        let day_period = day_period.unwrap_or("1".to_string());
        let query = vec![("vs_currency", parity.clone()), ("days", day_period.clone())];

        let url = format!("/coins/{token_id}/market_chart");
        let result = self.coingecko_rest_client::<GeckoTokenMarketChartResponse>(url, &query).await?;

        Ok(TokenMarketHistory::new().gecko_response_from(result, parity, token_id, day_period))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainDashboardInfoResponse {
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
pub struct ChainStatsInfoResponse {
    pub latest_block_height: u64,
    pub average_block_time: f32,
    pub price: f32,
    pub active_validator_count: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketHistory {
    pub price: f64,
    pub latest_update: String,
    pub volume: f64,
    pub market_cap: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeckoTokenMarketChartResponse {
    pub market_caps: GeckoMarketChartValue,
    pub prices: GeckoMarketChartValue,
    pub total_volumes: GeckoMarketChartValue,
}

type GeckoMarketChartValue = Vec<Vec<Number>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InternalMarketChart {
    pub timestamp: u64,
    pub value: f64,
}

type TokenMarketHistoryValue = Vec<InternalMarketChart>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenMarketHistory {
    pub parity: String,
    pub token_id: String,
    pub day_period: String,
    pub market_caps: TokenMarketHistoryValue,
    pub prices: TokenMarketHistoryValue,
    pub total_volumes: TokenMarketHistoryValue,
}

impl TokenMarketHistory {
    pub fn new() -> Self {
        Self {
            market_caps: vec![],
            prices: vec![],
            total_volumes: vec![],
            parity: "".to_string(),
            token_id: "".to_string(),
            day_period: "".to_string(),
        }
    }

    pub fn gecko_response_from(&self, value: GeckoTokenMarketChartResponse, parity: String, token_id: String, day_period: String) -> Self {
        let market_caps = self.gecko_chart_mapper(&value.market_caps);
        let prices = self.gecko_chart_mapper(&value.prices);
        let total_volumes = self.gecko_chart_mapper(&value.total_volumes);

        Self {
            market_caps,
            prices,
            total_volumes,
            parity,
            token_id,
            day_period,
        }
    }

    fn gecko_chart_mapper(&self, gecko_chart_value: &GeckoMarketChartValue) -> TokenMarketHistoryValue {
        let mut internal_market_charts: TokenMarketHistoryValue = vec![];
        for gecko_chart in gecko_chart_value {
            let timestamp = gecko_chart[0].as_u64().unwrap_or(0);
            let value = gecko_chart[1].as_f64().unwrap_or(0.0);
            internal_market_charts.push(InternalMarketChart { timestamp, value });
        }
        internal_market_charts
    }
}
