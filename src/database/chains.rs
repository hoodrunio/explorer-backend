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