use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use tokio::join;

use crate::{
    chain::Chain,
    database::{ChainDashboardInfoForDb, TokenMarketPriceHistoriesForDb},
    routes::TNRAppError,
};

use super::amount_util::TnrDecimal;

impl Chain {
    pub async fn get_dashboard_info(&self) -> Result<ChainDashboardInfoResponse, TNRAppError> {
        let (dashboard_info_res, market_history) = join!(self.database.find_chain_dashboard_info(), self.get_chain_market_chart_history());

        let mut market_cap = 0.0;
        let mut price = 0.0;

        let ChainDashboardInfoForDb {
            inflation_rate,
            apr,
            total_unbonded,
            total_bonded,
            total_supply,
            community_pool,
        } = dashboard_info_res?;

        let market_history = match market_history {
            Ok(res) => {
                market_cap = res.market_caps.last().cloned().unwrap_or_default().value;
                price = res.prices.last().cloned().unwrap_or_default().value;
                Some(res)
            }
            Err(_) => None,
        };

        Ok(ChainDashboardInfoResponse {
            inflation_rate,
            apr,
            total_unbonded,
            total_bonded,
            total_supply,
            price,
            community_pool,
            market_cap,
            market_history,
        })
    }

    pub async fn get_chain_market_chart_history(&self) -> Result<TokenMarketHistory, String> {
        let result = match self.database.find_market_history(self.config.name.clone()).await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Error while fetching market history: {}", e);
                return Err("Error while fetching market history".to_string());
            }
        };

        Ok(result.into())
    }

    pub async fn get_stats(&self) -> Result<ChainStatsInfoResponse, TNRAppError> {
        let (latest_block_height_resp, avg_block_time_resp, chain_market_history, active_validators_query) = join!(
            self.database.find_last_count_blocks(None, 1),
            self.get_avg_block_time(),
            self.get_chain_market_chart_history(),
            self.database.find_validators(Some(doc! {"$match":{"is_active":true}}))
        );
        let latest_block_height: u64 = latest_block_height_resp
            .map(|blocks| blocks.first().map(|block| block.height).unwrap_or(0))
            .unwrap_or(0);

        let mut average_block_time_ms = 0.0;
        let mut price = 0.0;

        if let Ok(avg_block_time_ms) = avg_block_time_resp {
            average_block_time_ms = avg_block_time_ms;
        };

        if let Ok(res) = chain_market_history {
            price = res.prices.last().cloned().unwrap_or_default().value;
        };

        let active_validator_count: u16 = active_validators_query.map(|res| res.len() as u16).unwrap_or(0);

        Ok(ChainStatsInfoResponse {
            latest_block_height,
            average_block_time_ms,
            price,
            active_validator_count,
        })
    }

    pub async fn gecko_token_market_chart(
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
    pub price: f64,
    pub market_cap: f64,
    pub inflation_rate: f64,
    pub apr: f64,
    pub total_unbonded: f64,
    pub total_bonded: f64,
    pub total_supply: String,
    pub community_pool: u64,
    pub market_history: Option<TokenMarketHistory>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainStatsInfoResponse {
    pub latest_block_height: u64,
    pub average_block_time_ms: f64,
    pub price: f64,
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

pub type GeckoMarketChartValue = Vec<Vec<Number>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InternalMarketChart {
    pub timestamp: u64,
    pub value: f64,
}

impl Default for InternalMarketChart {
    fn default() -> Self {
        Self { timestamp: 0, value: 0.0 }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenMarketHistory {
    pub parity: String,
    pub token_id: String,
    pub day_period: String,
    pub market_caps: Vec<InternalMarketChart>,
    pub prices: Vec<InternalMarketChart>,
    pub total_volumes: Vec<InternalMarketChart>,
}

impl From<TokenMarketPriceHistoriesForDb> for TokenMarketHistory {
    fn from(value: TokenMarketPriceHistoriesForDb) -> Self {
        let daily = value.daily;
        let market_caps = daily
            .market_caps
            .into_iter()
            .map(|cap| InternalMarketChart {
                timestamp: cap.timestamp,
                value: cap.value,
            })
            .collect();

        let prices = daily
            .prices
            .into_iter()
            .map(|price| InternalMarketChart {
                timestamp: price.timestamp,
                value: price.value,
            })
            .collect();

        let total_volumes = daily
            .total_volumes
            .into_iter()
            .map(|volume| InternalMarketChart {
                timestamp: volume.timestamp,
                value: volume.value,
            })
            .collect();

        Self {
            market_caps,
            prices,
            total_volumes,
            parity: daily.parity,
            token_id: daily.token_id,
            day_period: daily.day_period,
        }
    }
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

    fn gecko_chart_mapper(&self, gecko_chart_value: &GeckoMarketChartValue) -> Vec<InternalMarketChart> {
        let mut internal_market_charts: Vec<InternalMarketChart> = vec![];
        for gecko_chart in gecko_chart_value {
            let timestamp = gecko_chart[0].as_u64().unwrap_or(0);
            let value = gecko_chart[1].as_f64().unwrap_or(0.0);
            internal_market_charts.push(InternalMarketChart { timestamp, value });
        }
        internal_market_charts
    }
}
