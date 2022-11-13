use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

/// The latest 10 transaction on the chain.
#[derive(Deserialize, Serialize, Debug)]
pub struct LatestTransactions {
    pub inner: VecDeque<TransactionItem>,
}

impl LatestTransactions {
    /// Creates new txs.
    pub fn new() -> LatestTransactions {
        LatestTransactions { inner: VecDeque::new() }
    }

    /// Adds a new tx.
    pub fn add_new(&mut self, new_tx: TransactionItem) {
        self.inner.push_back(new_tx);

        if self.inner.len() > 10 {
            self.inner.pop_front();
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TransactionItem {
    pub height: u64,
    pub r#type: String,
    pub hash: String,
    pub result: String,
    pub timestamp: u32,
    pub fee: f64,
}
