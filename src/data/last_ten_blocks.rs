use std::{collections::VecDeque, sync::Mutex};

use crate::{chain::Chain, fetch::blocks::BlockItem};

/// The last 10 blocks.
pub struct LastTenBlocks {
    pub queue: Mutex<VecDeque<BlockItem>>,
    pub avg_block_time_secs: Mutex<f64>,
    pub latest_block_height: Mutex<u64>,
}

impl LastTenBlocks {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            avg_block_time_secs: Mutex::new(12.0),
            latest_block_height: Mutex::new(0),
        }
    }

    pub fn add(&self, block: BlockItem) {
        if let Ok(mut queue) = self.queue.lock() {
            if queue.len() == 10 {
                queue.pop_front();
            }

            let block_height = block.height;

            queue.push_back(block);

            let queue_clone = queue.clone();
            drop(queue);

            match self.latest_block_height.lock() {
                Ok(mut latest_block_height) => *latest_block_height = block_height,
                _ => (),
            };

            if queue_clone.len() == 10 {
                let mut difs = Vec::with_capacity(5);
                for i in 0..queue_clone.len() {
                    if i % 2 == 0 {
                        match (queue_clone.get(i), queue_clone.get(i + 1)) {
                            (Some(block_1), Some(block_2)) => {
                                let dif_in_secs = ((block_2.timestamp / 1000) - (block_1.timestamp / 1000)) as f64;
                                difs.push(dif_in_secs)
                            }
                            _ => (),
                        };
                    }
                }
                let avg = difs.into_iter().sum::<f64>() / 5.0;

                match self.avg_block_time_secs.lock() {
                    Ok(mut avg_block_time) => *avg_block_time = avg,
                    _ => (),
                }
            }
        }
    }

    pub fn get_blocks_till(&self, height: u64) -> Option<VecDeque<BlockItem>> {
        let queue_mutex = self.queue.lock().ok()?;
        let mut queue = queue_mutex.clone();
        drop(queue_mutex);

        let mut blocks = VecDeque::new();

        loop {
            let block = queue.pop_back()?;

            if block.height == height || queue.is_empty() {
                break;
            } else {
                blocks.push_front(block)
            }
        }

        Some(blocks)
    }
}

impl Chain {
    /// Stores a new block.
    pub fn store_new_block(&self, block: BlockItem) {
        self.inner.data.last_ten_blocks.add(block);
    }
}
