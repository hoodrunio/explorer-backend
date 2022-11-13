use std::collections::VecDeque;

use tokio::join;

use crate::chain::Chain;

use super::{latest_blocks::BlockItem, latest_txs::TransactionItem};

impl Chain {
    /// Returns the latest blocks.
    pub fn data_blocks(&self) -> Result<VecDeque<BlockItem>, &str> {
        match self.data.blocks.lock() {
            Ok(blocks) => Ok(blocks.inner.clone()),
            Err(error) => Err("Cannot send the latest blocks."),
        }
    }

    /// Returns the latest transactions.
    pub fn data_txs(&self) -> Result<VecDeque<TransactionItem>, &str> {
        match self.data.transactions.lock() {
            Ok(transactions) => Ok(transactions.inner.clone()),
            Err(_) => Err("Cannot send the latest transactions."),
        }
    }

    /// Updates general data.
    pub async fn update_data(&self) {
        join!(
            self.update_community_pool(),
            self.update_inflation(),
            self.update_supply(),
            self.update_tokenomics(),
        );
    }

    /// Subscribes to WebSocket.
    pub async fn subscribe_data(&self) {
        join!(self.subscribe_new_blocks(), self.subscribe_tx(),);
    }

    /// Updates the native coin price and the chart.
    pub async fn update_price(&self, new_price: Option<&f64>) {
        match new_price {
            Some(new_price) => {
                if let Ok(mut price) = self.data.price.lock() {
                    *price = *new_price;
                }
                if let Ok(mut chart) = self.data.chart.lock() {
                    chart.add_new(*new_price);
                }
            }
            _ => (),
        }
    }

    /// Updates the latest blocks, latest height, and the average block time.
    pub fn update_latest_block(&self, new_block: Option<BlockItem>) {
        if let Some(new_block) = new_block {
            let mut new_avg_block_time = None;

            // Update the latest blocks.
            if let Ok(mut blocks) = self.data.blocks.lock() {
                blocks.add_new(new_block.clone());
                new_avg_block_time = Some(blocks.get_avg_block_time());
            }

            // Update the latest block height.
            if let Ok(mut latest_height) = self.data.latest_height.lock() {
                *latest_height = new_block.height;
            }

            // Update the latest block time.
            if let Some(new_avg_block_time) = new_avg_block_time {
                if let Ok(mut avg_block_time) = self.data.avg_block_time.lock() {
                    *avg_block_time = new_avg_block_time;
                }
            }
        }
    }

    /// Updates the latest transactions.
    pub fn update_latest_txs(&self, new_tx: Option<TransactionItem>) {
        if let Some(new_tx) = new_tx {
            if let Ok(mut transactions) = self.data.transactions.lock() {
                transactions.add_new(new_tx);
            }
        }
    }

    /// Updates the supply and the market cap of the native coin.
    async fn update_supply(&self) {
        match self.get_supply_of_native_coin().await {
            Ok(resp) => {
                let new_supply = (resp.amount / 10_u128.pow(self.decimals.into())) as u64;

                if let Ok(mut supply) = self.data.supply.lock() {
                    *supply = new_supply;
                }

                if let Ok(price) = self.data.price.lock() {
                    let new_mcap = (*price * new_supply as f64) as u64;

                    if let Ok(mut mcap) = self.data.mcap.lock() {
                        *mcap = new_mcap;
                    }
                }
            }
            Err(error) => println!("{}", error),
        }
    }

    /// Updates the inflation rate.
    async fn update_inflation(&self) {
        let new_inflation_rate = self.get_inflation_rate().await;

        if new_inflation_rate != 0.0 {
            if let Ok(mut inflation) = self.data.inflation.lock() {
                *inflation = new_inflation_rate
            }
        }
    }

    /// Updates the community pool supply.
    async fn update_community_pool(&self) {
        let new_community_pool = self.get_community_pool().await;

        if let Ok(new_community_pool) = new_community_pool {
            if let Ok(mut pool) = self.data.pool.lock() {
                *pool = new_community_pool
            }
        }
    }

    /// Updates the bonded and unbonded coin supply.
    async fn update_tokenomics(&self) {
        let staking_pool = self.get_staking_pool().await;

        if let Ok(staking_pool) = staking_pool {
            let new_bonded = staking_pool.bonded;
            let new_unbonded = staking_pool.unbonded;

            if let Ok(mut bonded) = self.data.bonded.lock() {
                *bonded = new_bonded;
            }

            if let Ok(mut unbonded) = self.data.unbonded.lock() {
                *unbonded = new_unbonded;
            }
        }
    }
}
