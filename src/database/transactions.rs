use serde::{Deserialize, Serialize};

use crate::{fetch::transactions::TransactionItem, routes::ChainAmountItem};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Transaction {
    pub height: u64,
    pub r#type: String,
    pub hash: String,
    pub amount: ChainAmountItem,
    pub fee: ChainAmountItem,
    pub result: String,
    pub time: i64,
}

impl From<TransactionItem> for Transaction {
    fn from(value: TransactionItem) -> Self {
        Self {
            height: value.height,
            r#type: value.r#type.clone(),
            hash: value.hash.clone(),
            amount: value.amount.clone(),
            fee: value.fee.clone(),
            result: value.result.clone(),
            time: value.time,
        }
    }
}
