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
        if let Ok(mut queue) = self.queue.lock() {
            if queue.len() == 10 {
                queue.pop_front();
            }

            queue.push_back(tx)
        }
    }

    pub fn get_txs_till(&self, hash: &str) -> Option<VecDeque<TransactionItem>> {
        let queue_mutex = self.queue.lock().ok()?;
        let mut queue = queue_mutex.clone();
        drop(queue_mutex);

        let mut transactions = VecDeque::new();

        loop {
            let tx = queue.pop_back()?;

            if tx.hash == hash || queue.is_empty() {
                break;
            } else {
                transactions.push_front(tx)
            }
        }

        Some(transactions)
    }
}

impl Chain {
    /// Stores a new block.
    pub fn store_new_tx(&self, tx: TransactionItem) {
        self.inner.data.last_ten_txs.add(tx);
    }
}
