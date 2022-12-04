use super::{db::Db, last_ten_blocks::LastTenBlocks, last_ten_txs::LastTenTxs, params::ChainParams, price_chart::PriceChart};
use std::sync::Mutex;

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
    pub params: Mutex<ChainParams>,
    pub db: Db,
    pub last_ten_blocks: LastTenBlocks,
    pub last_ten_txs: LastTenTxs,
}

impl ChainData {
    /// Creates a new `ChainData`.
    pub fn new(chain: &str) -> ChainData {
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
            params: Mutex::new(ChainParams::new()),
            db: Db::new(chain),
            last_ten_blocks: LastTenBlocks::new(),
            last_ten_txs: LastTenTxs::new(),
        }
    }
}
