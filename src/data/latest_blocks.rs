use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

/// The latest 10 blocks on the chain.
#[derive(Deserialize, Serialize, Debug)]
pub struct LatestBlocks {
    pub inner: VecDeque<BlockItem>,
}

impl LatestBlocks {
    /// Creates new blocks.
    pub fn new() -> LatestBlocks {
        LatestBlocks { inner: VecDeque::new() }
    }

    /// Adds a new block.
    pub fn add_new(&mut self, new_block: BlockItem) {
        self.inner.push_back(new_block);

        if self.inner.len() > 10 {
            self.inner.pop_front();
        }
    }

    /// Returns the average block time in milliseconds.
    pub fn get_avg_block_time(&self) -> i64 {
        let mut diffs = vec![];

        if self.inner.len() != 0 {
            for i in 0..self.inner.len() - 1 {
                if let (Some(cur_block), Some(next_block)) = (self.inner.get(i), self.inner.get(i + 1)) {
                    diffs.push(next_block.timestamp - cur_block.timestamp)
                }
            }
        }

        diffs.iter().sum()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockItem {
    pub proposer_name: String,
    pub proposer_logo: String,
    pub height: u64,
    pub hash: String,
    pub tx_count: u64,
    pub timestamp: i64,
}
