use std::sync::Mutex;

use super::{latest_blocks::LatestBlocks, latest_txs::LatestTransactions, price_chart::PriceChart};

/// The struct representing chain data.
pub struct ChainData {
    pub inflation: Mutex<f64>,
    pub apr: Mutex<f64>,
    pub price: Mutex<f64>,
    pub supply: Mutex<u64>,
    pub mcap: Mutex<u64>,
    pub pool: Mutex<u64>,
    pub chart: Mutex<PriceChart>,
    pub bonded: Mutex<u64>,
    pub unbonded: Mutex<u64>,
    pub blocks: Mutex<LatestBlocks>,
    pub transactions: Mutex<LatestTransactions>,
    pub latest_height: Mutex<u64>,
    pub avg_block_time: Mutex<u32>,
}

impl ChainData {
    /// Creates a new `ChainData`.
    pub fn new() -> ChainData {
        ChainData {
            price: Mutex::new(0.0),
            inflation: Mutex::new(0.0),
            apr: Mutex::new(0.0),
            supply: Mutex::new(0),
            mcap: Mutex::new(0),
            pool: Mutex::new(0),
            chart: Mutex::new(PriceChart::new()),
            bonded: Mutex::new(0),
            unbonded: Mutex::new(0),
            blocks: Mutex::new(LatestBlocks::new()),
            transactions: Mutex::new(LatestTransactions::new()),
            latest_height: Mutex::new(0),
            avg_block_time: Mutex::new(0),
        }
    }
}
