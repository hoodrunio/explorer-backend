use std::{collections::VecDeque, sync::Mutex};

use crate::{chain::Chain, fetch::transactions::TransactionItem};

/// The last 10 Txs.
pub struct LastTenTxs {
    pub queue: Mutex<VecDeque<TransactionItem>>,
}

impl LastTenTxs {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
        }
    }

    pub fn add(&self, tx: TransactionItem) {
        match self.queue.lock() {
            Ok(mut queue) => {
                if queue.len() == 10 {
                    queue.pop_front();
                }

                queue.push_back(tx)
            }

            _ => (),
        }
    }
}

impl Chain {
    /// Stores a new block.
    pub fn store_new_tx(&self, tx: TransactionItem) {
        self.inner.data.last_ten_txs.add(tx);
    }
}
