use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// The chart for the price of the native coin.
#[derive(Deserialize, Serialize, Debug)]
pub struct PriceChart {
    pub inner: VecDeque<ChartItem>,
}

impl PriceChart {
    /// Creates a new price chart.
    pub fn new() -> PriceChart {
        PriceChart { inner: VecDeque::new() }
    }

    /// Adds a new price to the chart.
    pub fn add_new(&mut self, price: f64) {
        let t = chrono::offset::Utc::now().timestamp_millis() as u32;

        if let Some(last_item) = self.inner.back() {
            if t > last_item.t + 3_600_000 {
                self.inner.push_back(ChartItem { p: price, t });

                if self.inner.len() > 24 {
                    self.inner.pop_front();
                };
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ChartItem {
    pub p: f64, // price
    pub t: u32, // timestamp
}
