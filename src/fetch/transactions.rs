use std::collections::HashMap;

use chrono::DateTime;
use futures::{
    future::{join_all, BoxFuture},
    FutureExt,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::others::{DenomAmount, InternalDenomAmount, Pagination, PaginationConfig, PublicKey};
use crate::{
    chain::Chain,
    routes::{calc_pages, OutRestResponse},
    utils::get_msg_name,
};
use crate::fetch::socket::EvmPollVote;

impl Chain {
    /// Returns transaction by given hash.
    pub async fn get_tx_by_hash(&self, hash: &str) -> Result<OutRestResponse<InternalTransaction>, String> {
        match self.config.name.as_str() {
            "evmos" => {
                if hash.starts_with("0x") {
                    let resp = self.get_evm_tx_by_hash(hash).await?;
                    let resp = self
                        .get_txs_by_height_detailed(Some(resp.block_number), PaginationConfig::new().limit(100))
                        .await?;
                    let tx = resp
                        .value
                        .into_iter()
                        .find(|a| {
                            a.content
                                .iter()
                                .find(|a| {
                                    if let InternalTransactionContent::Known(InternalTransactionContentKnowns::EthereumTx { hash: tx_hash }) = a {
                                        tx_hash == hash
                                    } else {
                                        false
                                    }
                                })
                                .is_some()
                        })
                        .ok_or_else(|| format!("This transaction does not exist, {hash}."))?;

                    Ok(OutRestResponse::new(tx, 0))
                } else {
                    let path = format!("/cosmos/tx/v1beta1/txs/{hash}");

                    let resp = self.rest_api_request::<TxResp>(&path, &[]).await?;

                    let tx = InternalTransaction::new(resp.tx, resp.tx_response, self).await?;

                    Ok(OutRestResponse::new(tx, 0))
                }
            }

            _ => {
                let path = format!("/cosmos/tx/v1beta1/txs/{hash}");

                let resp = self.rest_api_request::<TxResp>(&path, &[]).await?;

                let tx = InternalTransaction::new(resp.tx, resp.tx_response, self).await?;

                Ok(OutRestResponse::new(tx, 0))
            }
        }
    }

    /// Returns transactions with given sender.
    pub async fn get_txs_by_sender(&self, sender_address: &str, config: PaginationConfig) -> Result<OutRestResponse<Vec<TransactionItem>>, String> {
        let mut query = vec![];

        query.push(("events", format!("message.sender='{}'", sender_address)));
        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<TxsResp>("/cosmos/tx/v1beta1/txs", &query).await?;

        let mut txs = vec![];

        for i in 0..resp.txs.len() {
            let (tx, tx_response) = (
                resp.txs
                    .get(i)
                    .ok_or_else(|| "The count of transactions and transaction responses aren't the same.".to_string())?,
                resp.tx_responses
                    .get(i)
                    .ok_or_else(|| "The count of transactions and transaction responses aren't the same.".to_string())?,
            );

            txs.push(TransactionItem::new(tx, tx_response, self)?)
        }

        let pages = calc_pages(resp.pagination.unwrap_or(Pagination::default()), config)?;

        Ok(OutRestResponse::new(txs, pages))
    }

    pub async fn get_internal_txs_by_sender_height(&self, sender_address: &str, block_height: Option<u64>, config: PaginationConfig) -> Result<OutRestResponse<Vec<InternalTransaction>>, String> {
        let mut query = vec![];

        if let Some(block_height) = block_height {
            query.push(("events", format!("tx.height={}", block_height)));
        };

        query.push(("events", format!("message.sender='{}'", sender_address)));
        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<TxsResp>("/cosmos/tx/v1beta1/txs", &query).await?;

        let mut txs = vec![];

        for i in 0..resp.txs.len() {
            let (tx, tx_response) = (
                resp.txs
                    .get(i)
                    .cloned()
                    .ok_or_else(|| "The count of transactions and transaction responses aren't the same.".to_string())?,
                resp.tx_responses
                    .get(i)
                    .cloned()
                    .ok_or_else(|| "The count of transactions and transaction responses aren't the same.".to_string())?,
            );

            txs.push(InternalTransaction::new(tx, tx_response, self).await?)
        }

        let pages = calc_pages(resp.pagination.unwrap_or(Pagination::default()), config)?;

        Ok(OutRestResponse::new(txs, pages))
    }


    /// Returns transactions with given recipient.
    pub async fn get_txs_by_recipient(
        &self,
        recipient_address: &str,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<TransactionItem>>, String> {
        let mut query = vec![];

        query.push(("events", format!("message.recipient='{}'", recipient_address)));
        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<TxsResp>("/cosmos/tx/v1beta1/txs", &query).await?;

        let mut txs = vec![];

        for i in 0..resp.txs.len() {
            let (tx, tx_response) = (
                resp.txs
                    .get(i)
                    .ok_or_else(|| "The count of transactions and transaction responses aren't the same.".to_string())?,
                resp.tx_responses
                    .get(i)
                    .ok_or_else(|| "The count of transactions and transaction responses aren't the same.".to_string())?,
            );

            txs.push(TransactionItem::new(tx, tx_response, self)?)
        }

        let pages = calc_pages(resp.pagination.unwrap_or(Pagination::default()), config)?;

        Ok(OutRestResponse::new(txs, pages))
    }

    /// Returns detailed transactions at given height.
    pub async fn get_txs_by_height_detailed(
        &self,
        block_height: Option<u64>,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalTransaction>>, String> {
        let mut query = vec![];

        if let Some(block_height) = block_height {
            query.push(("events", format!("tx.height={}", block_height)));
        };
        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<TxsResp>("/cosmos/tx/v1beta1/txs", &query).await?;

        let mut txs = vec![];

        for i in 0..resp.txs.len() {
            let (tx, tx_response) = (
                resp.txs
                    .get(i)
                    .cloned()
                    .ok_or_else(|| "The count of transactions and transaction responses aren't the same.".to_string())?,
                resp.tx_responses
                    .get(i)
                    .cloned()
                    .ok_or_else(|| "The count of transactions and transaction responses aren't the same.".to_string())?,
            );

            txs.push(InternalTransaction::new(tx, tx_response, self).await?)
        }

        let pages = calc_pages(resp.pagination.unwrap_or(Pagination::default()), config)?;

        Ok(OutRestResponse::new(txs, pages))
    }

    /// Returns transactions at given height.
    pub async fn get_txs_by_height(
        &self,
        block_height: Option<u64>,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<TransactionItem>>, String> {
        let mut query = vec![];

        if let Some(block_height) = block_height {
            query.push(("events", format!("tx.height={}", block_height)));
        };
        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<TxsResp>("/cosmos/tx/v1beta1/txs", &query).await?;

        let mut txs = vec![];

        for i in 0..resp.txs.len() {
            let (tx, tx_response) = (
                resp.txs
                    .get(i)
                    .ok_or_else(|| "The count of transactions and transaction responses aren't the same.".to_string())?,
                resp.tx_responses
                    .get(i)
                    .ok_or_else(|| "The count of transactions and transaction responses aren't the same.".to_string())?,
            );

            txs.push(TransactionItem::new(tx, tx_response, self)?)
        }

        let pages = calc_pages(resp.pagination.unwrap_or(Pagination::default()), config)?;

        Ok(OutRestResponse::new(txs, pages))
    }

    /// Returns the EVM TX response by given hash. Only works for Evmos chain.
    ///
    /// The hash must start with `"0x..."`.
    async fn get_evm_tx_by_hash(&self, hash: &str) -> Result<InternalEvmTxResp, String> {
        self.jsonrpc_request::<EvmTxResp>(format!(
            r#"{{"method":"eth_getTransactionByHash","params":["{hash}"],"id":1,"jsonrpc":"2.0"}}"#
        ))
            .await?
            .try_into()
    }
    pub async fn get_axelar_sender_heartbeat_info(&self, val_voter_address: &String, block_height: u64) -> Result<InternalAxelarHeartbeatInfo, String> {
        match self.get_internal_txs_by_sender_height(&val_voter_address, Some(block_height), PaginationConfig::new().limit(1).page(1)).await {
            Ok(txs_res) => {
                for contents in txs_res.value {
                    match contents.extract_axelar_heartbeat_info() {
                        Some(res) => { return Ok(res); }
                        None => {}
                    };
                };
                let message = String::from("This is not an heartbeat tx");
                Err(message)
            }
            Err(e) => {
                tracing::error!("Could not fetched txs by sender");
                Err(e)
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalEvmTxResp {
    /// Block number.
    pub block_number: u64,
}

impl TryInto<InternalEvmTxResp> for EvmTxResp {
    type Error = String;
    fn try_into(self) -> Result<InternalEvmTxResp, Self::Error> {
        use hex::FromHex;

        Ok(InternalEvmTxResp {
            block_number: {
                let mut block_no: u64 = 0;
                let hex_block_no = if self.block_number.len() > 2 { &self.block_number[2..] } else { "00" };
                let mut bytes = <Vec<u8>>::from_hex(hex_block_no).map_err(|_| format!("Cannot parse HEX block number, {}.", self.block_number))?;

                let mut i: u32 = 0;

                while !bytes.is_empty() {
                    if let Some(byte) = bytes.pop() {
                        block_no += <u64>::from(byte) * 256_u64.pow(i);
                    }

                    i += 1;
                }

                block_no
            },
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EvmTxResp {
    /// HEX encoded block number. Eg: `"0x5f08d0"`
    pub block_number: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalTransaction {
    pub hash: String,
    pub r#type: String,
    pub amount: f64,
    pub height: u64,
    pub time: i64,
    pub fee: f64,
    pub gas_wanted: u64,
    pub gas_used: u64,
    pub raw: String,
    pub result: String,
    pub memo: String,
    pub signatures: Vec<String>,
    pub content: Vec<InternalTransactionContent>,
}

impl InternalTransaction {
    async fn new(tx: Tx, tx_response: TxResponse, chain: &Chain) -> Result<Self, String> {
        let mut jobs = vec![];

        let r#type = tx
            .body
            .messages
            .get(0)
            .map(|msg| msg.get_type())
            .ok_or_else(|| format!("There is no TX type, '{}'.", tx_response.txhash))?;

        let amount = tx
            .body
            .messages
            .get(0)
            .map(|msg| match msg {
                TxsTransactionMessage::Known(msg) => match msg {
                    TxsTransactionMessageKnowns::Delegate {
                        delegator_address: _,
                        validator_address: _,
                        amount,
                    } => chain._get_amount(&amount.amount),
                    TxsTransactionMessageKnowns::Redelegate {
                        delegator_address: _,
                        validator_src_address: _,
                        validator_dst_address: _,
                        amount,
                    } => chain._get_amount(&amount.amount),
                    TxsTransactionMessageKnowns::Send {
                        from_address: _,
                        to_address: _,
                        amount,
                    } => amount.get(0).map(|amount| chain._get_amount(&amount.amount)).unwrap_or(0.00),
                    TxsTransactionMessageKnowns::Undelegate {
                        delegator_address: _,
                        validator_address: _,
                        amount,
                    } => chain._get_amount(&amount.amount),
                    _ => 0.00,
                },
                _ => 0.00,
            })
            .ok_or_else(|| format!("There is no TX type, '{}'.", tx_response.txhash))?;

        for message in tx.body.messages {
            jobs.push(async move { message.to_internal(chain).await })
        }

        let resps = join_all(jobs).await;

        let mut content = vec![];

        for resp in resps {
            content.push(resp?)
        }

        Ok(Self {
            hash: tx_response.txhash,
            height: tx_response
                .height
                .parse::<u64>()
                .map_err(|_| format!("Cannot parse transaction height, '{}'.", tx_response.height))?,
            time: DateTime::parse_from_rfc3339(&tx_response.timestamp)
                .map_err(|_| format!("Cannot parse transaction timestamp, '{}'.", tx_response.timestamp))?
                .timestamp_millis(),
            fee: tx
                .auth_info
                .fee
                .amount
                .get(0)
                .map(|ad| ad.amount.to_string())
                .and_then(|amount| amount.parse::<u128>().ok())
                .map(|amount| chain.calc_amount_u128_to_f64(amount))
                .unwrap_or(0.0),
            gas_wanted: tx_response
                .gas_wanted
                .parse::<u64>()
                .map_err(|_| format!("Cannot parse transaction gas wanted, '{}'.", tx_response.gas_wanted))?,
            gas_used: tx_response
                .gas_used
                .parse::<u64>()
                .map_err(|_| format!("Cannot parse transaction gas used, '{}'.", tx_response.gas_used))?,
            result: if tx_response.raw_log.starts_with('[') || tx_response.raw_log.starts_with('{') {
                "Success".to_string()
            } else {
                "Failed".to_string()
            },
            signatures: match tx_response.tx { TxsResponseTx::Tx { signatures, .. } => { signatures } },
            memo: tx.body.memo,
            raw: tx_response.raw_log,
            content,
            amount,
            r#type,
        })
    }
    pub fn extract_axelar_heartbeat_info(&self) -> Option<InternalAxelarHeartbeatInfo> {
        let mut res = None;
        for content_item in &self.content {
            if let InternalTransactionContent::Known(InternalTransactionContentKnowns::AxelarRefundRequest { sender: _, inner_message }) = content_item {
                if let InnerMessage::Known(InnerMessageKnown::HeartBeatRequest { sender, key_ids }) = inner_message {
                    res = Some(InternalAxelarHeartbeatInfo {
                        sender: sender.clone(),
                        key_ids: key_ids.clone(),
                        signatures: self.signatures.clone(),
                        tx_hash: self.hash.clone(),
                        height: self.height.clone(),
                        timestamp: self.time.clone(),
                    });
                    break;
                }
            }
        }

        res
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InternalAxelarHeartbeatInfo {
    pub sender: String,
    pub key_ids: Vec<String>,
    pub signatures: Vec<String>,
    pub tx_hash: String,
    pub height: u64,
    pub timestamp: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TransactionItem {
    pub height: u64,
    pub r#type: String,
    pub hash: String,
    pub amount: f64,
    pub fee: f64,
    pub result: String,
    pub time: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum InternalTransactionContent {
    Known(InternalTransactionContentKnowns),
    Unknown { r#type: String, keys_values: HashMap<String, String> },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum InternalTransactionContentKnowns {
    Exec {
        grantee: String,
        msgs: Vec<InternalTransactionContent>,
    },
    Grant {
        granter: String,
        grantee: String,
        expiration: i64,
        authorization_type: String,
        authorization_data: Vec<KeyValue>,
    },
    Send {
        from_address: String,
        to_address: String,
        amounts: Vec<InternalDenomAmount>,
    },
    Delegate {
        delegator_address: String,
        validator_name: String,
        validator_address: String,
        amount: f64,
    },
    Undelegate {
        delegator_address: String,
        validator_name: String,
        validator_address: String,
        amount: f64,
    },
    #[serde(rename = "Withdraw Delegator Reward")]
    WithdrawDelegatorReward {
        delegator_address: String,
        validator_name: String,
        validator_address: String,
    },
    Redelegate {
        delegator_address: String,
        validator_from_name: String,
        validator_from_address: String,
        validator_to_name: String,
        validator_to_address: String,
        amount: f64,
    },
    Revoke {
        granter_address: String,
        grantee_address: String,
    },
    Vote {
        proposal_id: u32,
        voter_address: String,
        option: String,
    },
    #[serde(rename = "Ethereum Tx")]
    EthereumTx {
        hash: String,
    },
    RegisterProxy {
        sender: String,
        proxy_addr: String,
    },
    AxelarRefundRequest {
        sender: String,
        inner_message: InnerMessage,
    },
}

impl From<InternalTransaction> for TransactionItem {
    fn from(tx: InternalTransaction) -> Self {
        Self {
            height: tx.height,
            r#type: tx.r#type,
            hash: tx.hash,
            amount: tx.amount,
            fee: tx.fee,
            result: tx.result,
            time: tx.time,
        }
    }
}

impl TransactionItem {
    fn new(tx: &Tx, tx_response: &TxResponse, chain: &Chain) -> Result<Self, String> {
        Ok(Self {
            height: tx_response
                .height
                .parse()
                .map_err(|_| format!("Cannot parse transaction height, '{}'.", tx_response.height))?,
            r#type: tx
                .body
                .messages
                .get(0)
                .map(|msg| msg.get_type())
                .ok_or_else(|| format!("There is no TX type, '{}'.", tx_response.txhash))?,
            hash: tx_response.txhash.to_string(),
            amount: tx
                .body
                .messages
                .get(0)
                .map(|msg| match msg {
                    TxsTransactionMessage::Known(msg) => match msg {
                        TxsTransactionMessageKnowns::Delegate {
                            delegator_address: _,
                            validator_address: _,
                            amount,
                        } => chain._get_amount(&amount.amount),
                        TxsTransactionMessageKnowns::Redelegate {
                            delegator_address: _,
                            validator_src_address: _,
                            validator_dst_address: _,
                            amount,
                        } => chain._get_amount(&amount.amount),
                        TxsTransactionMessageKnowns::Send {
                            from_address: _,
                            to_address: _,
                            amount,
                        } => amount.get(0).map(|amount| chain._get_amount(&amount.amount)).unwrap_or(0.00),
                        TxsTransactionMessageKnowns::Undelegate {
                            delegator_address: _,
                            validator_address: _,
                            amount,
                        } => chain._get_amount(&amount.amount),
                        _ => 0.00,
                    },
                    _ => 0.00,
                })
                .ok_or_else(|| format!("There is no TX type, '{}'.", tx_response.txhash))?,
            fee: tx
                .auth_info
                .fee
                .amount
                .get(0)
                .map(|ad| ad.amount.to_string())
                .and_then(|amount| amount.parse::<u128>().ok())
                .map(|amount| chain.calc_amount_u128_to_f64(amount))
                .unwrap_or(0.0),
            result: if tx_response.raw_log.starts_with('[') || tx_response.raw_log.starts_with('{') {
                "Success".to_string()
            } else {
                "Failed".to_string()
            },
            time: DateTime::parse_from_rfc3339(&tx_response.timestamp)
                .map_err(|_| format!("Cannot parse transaction timestamp, '{}'.", tx_response.timestamp))?
                .timestamp_millis(),
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxsResp {
    pub txs: Vec<Tx>,
    pub tx_responses: Vec<TxResponse>,
    pub pagination: Option<Pagination>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TxsTransactionBody {
    /// Transaction messages.
    pub messages: Vec<TxsTransactionMessage>,
    /// Transaction memo. Eg: `"1891420480"`
    pub memo: String,
    /// Transaction timeout height. Eg: `"0"`
    pub timeout_height: String,
    // Non-critical transaction extension options.
    // pub non_critical_extension_options: Vec<UNKNOWN>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TxsTransactionAuthInfo {
    /// Transaction fee.
    pub fee: TxsTransactionAuthInfoFee,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GrantTxGrant {
    /// It is almost impossible to know all the variants.
    authorization: HashMap<String, Value>,
    /// Expiration datetime. Eg: `"2024-12-05T01:04:03Z"`
    expiration: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum TxsTransactionMessage {
    Known(TxsTransactionMessageKnowns),
    Unknown(HashMap<String, Value>),
}

impl TxsTransactionMessage {
    /// Creates a new Message.
    pub fn to_internal<'a>(self, chain: &'a Chain) -> BoxFuture<'a, Result<InternalTransactionContent, String>> {
        async move {
            Ok::<_, String>(match self {
                TxsTransactionMessage::Known(message) => match message {
                    TxsTransactionMessageKnowns::Delegate {
                        delegator_address,
                        validator_address,
                        amount,
                    } => InternalTransactionContent::Known(InternalTransactionContentKnowns::Delegate {
                        delegator_address,
                        validator_name: chain.database.find_validator_by_operator_addr(&validator_address.clone()).await?.name,
                        validator_address,
                        amount: chain.calc_amount_u128_to_f64(
                            amount
                                .amount
                                .parse::<u128>()
                                .map_err(|_| format!("Cannot parse delegation amount, '{}'.", amount.amount))?,
                        ),
                    }),

                    TxsTransactionMessageKnowns::Redelegate {
                        delegator_address,
                        validator_src_address,
                        validator_dst_address,
                        amount,
                    } => InternalTransactionContent::Known(InternalTransactionContentKnowns::Redelegate {
                        delegator_address,
                        validator_from_name: chain.database.find_validator_by_operator_addr(&validator_src_address.clone()).await?.name,
                        validator_from_address: validator_src_address,
                        validator_to_name: chain.database.find_validator_by_operator_addr(&validator_dst_address.clone()).await?.name,
                        validator_to_address: validator_dst_address,
                        amount: chain.calc_amount_u128_to_f64(
                            amount
                                .amount
                                .parse::<u128>()
                                .map_err(|_| format!("Cannot parse delegation amount, '{}'.", amount.amount))?,
                        ),
                    }),

                    TxsTransactionMessageKnowns::Revoke {
                        granter_address,
                        grantee_address,
                    } => InternalTransactionContent::Known(InternalTransactionContentKnowns::Revoke {
                        granter_address,
                        grantee_address,
                    }),

                    TxsTransactionMessageKnowns::Send {
                        from_address,
                        to_address,
                        amount,
                    } => {
                        let mut amounts = vec![];

                        for denom_amount in amount {
                            amounts.push(denom_amount.try_into()?)
                            // We don't work with decimals here, cuz there might be a token which is not the same with the native coin of the chain.
                            // If this situation is highly unlikely to be happen, you can just convert `amounts` to `f64` and just store the amount (in native coin, others wo't be supported).
                        }

                        InternalTransactionContent::Known(InternalTransactionContentKnowns::Send {
                            from_address,
                            to_address,
                            amounts,
                        })
                    }

                    TxsTransactionMessageKnowns::Undelegate {
                        delegator_address,
                        validator_address,
                        amount,
                    } => InternalTransactionContent::Known(InternalTransactionContentKnowns::Undelegate {
                        delegator_address,
                        validator_name: chain.database.find_validator_by_operator_addr(&validator_address.clone()).await?.name,
                        validator_address,
                        amount: chain.calc_amount_u128_to_f64(
                            amount
                                .amount
                                .parse::<u128>()
                                .map_err(|_| format!("Cannot parse undelegation amount, '{}'.", amount.amount))?,
                        ),
                    }),

                    TxsTransactionMessageKnowns::Vote { proposal_id, voter, option } => {
                        InternalTransactionContent::Known(InternalTransactionContentKnowns::Vote {
                            proposal_id: proposal_id
                                .parse::<u32>()
                                .map_err(|_| format!("Cannot parse proposal ID, '{}'.", proposal_id))?,
                            voter_address: voter,
                            option: match option.as_ref() {
                                "VOTE_OPTION_YES" => "Yes",
                                "VOTE_OPTION_NO" => "No",
                                "VOTE_OPTION_ABSTAIN" => "Abstain",
                                "VOTE_OPTION_UNSPECIFIED" => "Empty",
                                "VOTE_OPTION_NO_WITH_VETO" => "Veto",
                                _ => "Unknown",
                            }
                                .to_string(),
                        })
                    }

                    TxsTransactionMessageKnowns::WithdrawDelegatorReward {
                        delegator_address,
                        validator_address,
                    } => InternalTransactionContent::Known(InternalTransactionContentKnowns::WithdrawDelegatorReward {
                        delegator_address,
                        validator_name: chain.database.find_validator_by_operator_addr(&validator_address.clone()).await?.name,
                        validator_address,
                    }),
                    TxsTransactionMessageKnowns::EthereumTx { hash } => {
                        InternalTransactionContent::Known(InternalTransactionContentKnowns::EthereumTx { hash })
                    }
                    TxsTransactionMessageKnowns::Grant { granter, grantee, mut grant } => {
                        InternalTransactionContent::Known(InternalTransactionContentKnowns::Grant {
                            granter,
                            grantee,
                            expiration: DateTime::parse_from_rfc3339(&grant.expiration)
                                .map_err(|_| format!("Cannot parse date time, {}.", grant.expiration))?
                                .timestamp_millis(),

                            authorization_type: get_msg_name(
                                &grant
                                    .authorization
                                    .remove("@type")
                                    .map(|v| v.to_string())
                                    .unwrap_or("Unknown".to_string()),
                            ),
                            authorization_data: grant
                                .authorization
                                .into_iter()
                                .map(|(key, value)| KeyValue {
                                    key,
                                    value: value.to_string(),
                                })
                                .collect(),
                        })
                    }
                    TxsTransactionMessageKnowns::Exec { grantee, msgs } => {
                        InternalTransactionContent::Known(InternalTransactionContentKnowns::Exec {
                            grantee,
                            msgs: {
                                let resps = join_all(msgs.into_iter().map(|msg| msg.to_internal(chain))).await;
                                let mut internal_msgs = vec![];
                                for resp in resps {
                                    internal_msgs.push(resp?)
                                }

                                internal_msgs
                            },
                        })
                    }
                    TxsTransactionMessageKnowns::RegisterProxy { sender, proxy_addr } => {
                        InternalTransactionContent::Known(InternalTransactionContentKnowns::RegisterProxy { sender, proxy_addr })
                    }
                    TxsTransactionMessageKnowns::AxelarRegisterProxy { sender, proxy_addr } => {
                        InternalTransactionContent::Known(InternalTransactionContentKnowns::RegisterProxy { sender, proxy_addr })
                    }
                    TxsTransactionMessageKnowns::AxelarRefundRequest { sender, inner_message } => {
                        InternalTransactionContent::Known(InternalTransactionContentKnowns::AxelarRefundRequest {
                            sender,
                            inner_message,
                        })
                    }
                },
                TxsTransactionMessage::Unknown(mut keys_values) => {
                    let r#type = keys_values.remove("@type").map(|t| t.to_string()).unwrap_or("Unknown".to_string());
                    InternalTransactionContent::Unknown {
                        r#type,
                        keys_values: keys_values.into_iter().map(|(k, v)| (k, v.to_string())).collect(),
                    }
                }
            })
        }
            .boxed()
    }

    /// Return the type of message.
    pub fn get_type(&self) -> String {
        match self {
            TxsTransactionMessage::Known(msg) => match msg {
                TxsTransactionMessageKnowns::Delegate {
                    delegator_address: _,
                    validator_address: _,
                    amount: _,
                } => "Delegate",
                TxsTransactionMessageKnowns::Redelegate {
                    delegator_address: _,
                    validator_src_address: _,
                    validator_dst_address: _,
                    amount: _,
                } => "Redelegate",
                TxsTransactionMessageKnowns::Revoke {
                    granter_address: _,
                    grantee_address: _,
                } => "Revoke",
                TxsTransactionMessageKnowns::Send {
                    from_address: _,
                    to_address: _,
                    amount: _,
                } => "Send",
                TxsTransactionMessageKnowns::Undelegate {
                    delegator_address: _,
                    validator_address: _,
                    amount: _,
                } => "Undelegate",
                TxsTransactionMessageKnowns::Vote {
                    proposal_id: _,
                    voter: _,
                    option: _,
                } => "Vote",
                TxsTransactionMessageKnowns::WithdrawDelegatorReward {
                    delegator_address: _,
                    validator_address: _,
                } => "Withdraw Delegator Rewards",
                TxsTransactionMessageKnowns::EthereumTx { hash: _ } => "Ethereum Tx",
                TxsTransactionMessageKnowns::Grant {
                    granter: _,
                    grantee: _,
                    grant: _,
                } => "Grant",
                TxsTransactionMessageKnowns::Exec { grantee: _, msgs: _ } => "Exec",
                TxsTransactionMessageKnowns::RegisterProxy { sender: _, proxy_addr: _ } => "RegisterProxy",
                TxsTransactionMessageKnowns::AxelarRegisterProxy { sender: _, proxy_addr: _ } => "RegisterProxy",
                TxsTransactionMessageKnowns::AxelarRefundRequest { sender: _, inner_message: _ } => "AxelarRefundRequest"
            }
                .to_string(),
            TxsTransactionMessage::Unknown(keys_values) => keys_values
                .get("@type")
                .cloned()
                .map(|r#type| get_msg_name(r#type.to_string().as_ref()))
                .unwrap_or("Unknown".to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "@type")]
pub enum TxsTransactionMessageKnowns {
    #[serde(rename = "/cosmos.authz.v1beta1.MsgExec")]
    Exec {
        /// The grantee address. Eg: `"mantle1e44rluarkdw56dy2turnwjtvtg4wqvs0v0wpg0"`
        grantee: String,
        /// Transaction messages.
        msgs: Vec<TxsTransactionMessage>,
    },
    #[serde(rename = "/cosmos.authz.v1beta1.MsgGrant")]
    Grant {
        /// The granter address. Eg: `"evmos1la8cn9uhagcejvp36ftucy0569a5pg34pty8lr"`
        granter: String,
        /// The grantee address. Eg: `"evmos1fr6dylwlhaetqke95agqnyk29la9hqkxy0jplg"`
        grantee: String,
        /// Grant object.
        grant: GrantTxGrant,
    },
    #[serde(rename = "/cosmos.bank.v1beta1.MsgSend")]
    Send {
        /// The address transaction is from. Eg: `"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf"`
        from_address: String,
        /// The address transaction is to. Eg: `"cosmos1vl8xm7x04cedgh639hc9ucvf6zc754fyfewhef"`
        to_address: String,
        /// Transaction amounts.
        amount: Vec<DenomAmount>,
    },
    #[serde(rename = "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward")]
    WithdrawDelegatorReward {
        /// Delegator address. Eg: `"evmos1wl8penajxqyqarw94q00cd46nvwuduq40er8sj"`
        delegator_address: String,
        /// Validator address. Eg: `"evmosvaloper1d74wdckw5vyn6gwqt4r0ruemp9n8vmwtudw848"`
        validator_address: String,
    },
    #[serde(rename = "/cosmos.authz.v1beta1.MsgRevoke")]
    Revoke {
        /// Granter address. Eg: `"evmos1qpc5u5zzhre7zqz343kmuvk206pdzy4r7d0jev"`
        granter_address: String,
        /// Grantee address. Eg: `"evmos182d5yfc5wwaphwjm5wqj9xmsf0vmp30qw9a07p"`
        grantee_address: String,
    },
    #[serde(rename = "/cosmos.gov.v1beta1.MsgVote")]
    Vote {
        /// Proposal ID. Eg: `"78"`
        proposal_id: String,
        /// Voter address. Eg: `"evmos16arqk5g5zntx00czgqtwjjy7dz4ex3v8fuw0t2"`
        voter: String,
        /// Vote option. Eg: `"VOTE_OPTION_YES"`
        option: String,
    },
    #[serde(rename = "/cosmos.staking.v1beta1.MsgDelegate")]
    Delegate {
        /// Delegator address. Eg: `"evmos1a37y062zjspzrcaxhz76lskwnvm0xlsymdfgg0"`
        delegator_address: String,
        /// Validator address. Eg: `"evmosvaloper14zatq4jagqtm9ejgvglnv0t364d88u80futp65"`
        validator_address: String,
        /// Amount.
        amount: DenomAmount,
    },
    #[serde(rename = "/cosmos.staking.v1beta1.MsgBeginRedelegate")]
    Redelegate {
        /// Delegator address. Eg: `"evmos1a37y062zjspzrcaxhz76lskwnvm0xlsymdfgg0"`
        delegator_address: String,
        /// Source validator address. Eg: `"evmosvaloper1v4crs2adgcu2cdm2jxq07mw7ugzx0z4x6alxeg"`
        validator_src_address: String,
        /// Destination validator address. Eg: `"evmosvaloper1sp9frqwep52chwavv3xd776myy8gyyvkv5uysl"`
        validator_dst_address: String,
        /// Amount.
        amount: DenomAmount,
    },
    #[serde(rename = "/cosmos.staking.v1beta1.MsgUndelegate")]
    Undelegate {
        /// Delegator address. Eg: `"evmos1a37y062zjspzrcaxhz76lskwnvm0xlsymdfgg0"`
        delegator_address: String,
        /// Validator address. Eg: `"evmosvaloper14zatq4jagqtm9ejgvglnv0t364d88u80futp65"`
        validator_address: String,
        /// Amount.
        amount: DenomAmount,
    },
    #[serde(rename = "/ethermint.evm.v1.MsgEthereumTx")]
    EthereumTx {
        /// Ethereum transaction hash. Eg: `"0xc8137e7716e65483da73aa8d1f9f4730c253429c3d3dabce92cf63dd55027ac6"`
        hash: String,
        // Ethereum transaction data.
        // There are multiple types of this property.
        // Creating an enum for it is necessary if we need to show the data in the explorer.
        // data: UNKNOWN,
    },
    #[serde(rename = "/snapshot.v1beta1.RegisterProxyRequest")]
    RegisterProxy {
        sender: String,
        proxy_addr: String,
    },
    #[serde(rename = "/axelar.snapshot.v1beta1.RegisterProxyRequest")]
    AxelarRegisterProxy {
        sender: String,
        proxy_addr: String,
    },
    #[serde(rename = "/axelar.reward.v1beta1.RefundMsgRequest")]
    AxelarRefundRequest {
        sender: String,
        inner_message: InnerMessage,
    },
}


#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum InnerMessage {
    Known(InnerMessageKnown),
    Unknown(HashMap<String, Value>),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "@type")]
pub enum InnerMessageKnown {
    #[serde(rename = "/axelar.vote.v1beta1.VoteRequest")]
    VoteRequest {
        sender: String,
        poll_id: String,
        vote: AxelarVote,
    },
    #[serde(rename = "/axelar.tss.v1beta1.HeartBeatRequest")]
    HeartBeatRequest {
        sender: String,
        key_ids: Vec<String>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum AxelarVote {
    Known(AxelarKnownVote),
    Unknown(HashMap<String, Value>),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "@type")]
pub enum AxelarKnownVote {
    #[serde(rename = "/axelar.evm.v1beta1.VoteEvents")]
    VoteEvent {
        chain: String,
        events: Vec<HashMap<String, Value>>,
    }
}

impl AxelarKnownVote {
    pub fn evm_vote(&self) -> EvmPollVote {
        match self {
            AxelarKnownVote::VoteEvent { chain: _, events } => {
                let vote = if !events.is_empty() {
                    EvmPollVote::Yes
                } else {
                    EvmPollVote::No
                };
                vote
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IbcAcknowledgementPacket {
    /// Source channel. Eg: `"channel-0"`
    pub source_channel: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TimeoutHeight {
    /// Timeout revision number. Eg: `"1"`
    pub revision_number: String,
    /// Timout revision height. Eg: `"6789255"`
    pub revision_height: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TxsTransactionAuthInfoFee {
    /// Amount.
    pub amount: Vec<DenomAmount>,
    /// Transaction gas limit.
    pub gas_limit: String,
    /// Transaction payer. Eg: `""`
    pub payer: String,
    /// Transaction granter. Eg: `""`
    pub granter: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxsTransactionSignerInfo {
    pub public_key: PublicKey,
    pub mode_info: TxsTransactionModeInfo,
    /// Transaction signer info sequence. Eg: `"1"`
    pub sequence: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxsTransactionModeInfo {
    pub single: TxsTransactionModeInfoSingle,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxsTransactionModeInfoSingle {
    /// Mode. Eg: `"SIGN_MODE_LEGACY_AMINO_JSON"`
    pub mode: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TxResponse {
    /// Block height. Eg: `"12713829"`
    pub height: String,
    /// HEX encoded transaction hash. Eg: `"D29DEB0948ADC9B14A1758ED164A46407AF33EA2950404DB4AFFF68164B01C58"`
    pub txhash: String,
    /// Transaction codespace. Eg: `""`
    pub codespace: String,
    /// Code. Eg: `0`
    pub code: usize,
    /// HEX encoded data. Eg: `"0A1E0A1C2F636F736D6F732E62616E6B2E763162657461312E4D736753656E64"`
    pub data: String,
    /// JSON encoded raw log. Eg: `"[{\"events\":[{\"type\":\"coin_received\",\"attributes\":[{\"key\":\"receiver\",\"value\":\"cosmos1vl8xm7x04cedgh639hc9ucvf6zc754fyfewhef\"},{\"key\":\"amount\",\"value\":\"450000uatom\"}]},{\"type\":\"coin_spent\",\"attributes\":[{\"key\":\"spender\",\"value\":\"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf\"},{\"key\":\"amount\",\"value\":\"450000uatom\"}]},{\"type\":\"message\",\"attributes\":[{\"key\":\"action\",\"value\":\"/cosmos.bank.v1beta1.MsgSend\"},{\"key\":\"sender\",\"value\":\"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf\"},{\"key\":\"module\",\"value\":\"bank\"}]},{\"type\":\"transfer\",\"attributes\":[{\"key\":\"recipient\",\"value\":\"cosmos1vl8xm7x04cedgh639hc9ucvf6zc754fyfewhef\"},{\"key\":\"sender\",\"value\":\"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf\"},{\"key\":\"amount\",\"value\":\"450000uatom\"}]}]}]"`
    pub raw_log: String,
    /// Array of logs.
    pub logs: Vec<TxResponseLog>,
    /// Info. Eg: `""`
    pub info: String,
    // Gas wanted. Eg: `"80000"`
    pub gas_wanted: String,
    /// Gas used. Eg: `"74032"`
    pub gas_used: String,
    // Tx.
    pub tx: TxsResponseTx,
    // Timestamp. Eg: `"2022-07-19T05:26:26Z"`
    pub timestamp: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TxResponseLog {
    /// Array of events.
    pub events: Vec<TxResponseEvent>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TxResponseEvent {
    /// Event type. Eg: `"redelegate"`
    pub r#type: String,
    /// Array of attributes.
    pub attributes: Vec<TxResponseEventAttribute>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TxResponseEventAttribute {
    /// Event attribute key. Eg: `"completion_time"`
    pub key: String,
    /// Event attribute value. Eg: `"2022-12-18T19:20:04Z"`
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "@type")]
pub enum TxsResponseTx {
    #[serde(rename = "/cosmos.tx.v1beta1.Tx")]
    Tx {
        // Tx body.
        body: TxsTransactionBody,
        // Tx auth info.
        auth_info: TxsTransactionAuthInfo,
        /// Array of Base 64 encoded signatures.
        signatures: Vec<String>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Tx {
    // Tx body.
    pub body: TxsTransactionBody,
    // Tx auth info.
    pub auth_info: TxsTransactionAuthInfo,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum TxsResponseEvent<T> {
    CoinReceived {
        /// Coin received attributes.
        attributes: Vec<T>,
    },
    ProposalVote {
        /// Proposal attributes.
        attributes: Vec<T>,
    },
    CoinSpent {
        /// Coin spent attributes.
        attributes: Vec<T>,
    },
    IbcTransfer {
        /// Coin spent attributes.
        attributes: Vec<T>,
    },
    SendPacket {
        /// Send packet attributes.
        attributes: Vec<T>,
    },

    Message {
        /// Message attributes.
        attributes: Vec<T>,
    },
    Transfer {
        /// Transfer attributes.
        attributes: Vec<T>,
    },
    Tx {
        /// Tx attributes.
        attributes: Vec<T>,
    },
    WithdrawRewards {
        /// Withdraw rewards attributes.
        attributes: Vec<T>,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UnparsedTxEventAttribute {
    /// Unparsed event attribute key. Eg: `"cmVjaXBpZW50"`
    pub key: String,
    /// Unparsed event attribute key. Might be `None`. Eg: `"ZXZtb3MxN3hwZnZha20yYW1nOTYyeWxzNmY4NHoza2VsbDhjNWxqY2p3MzQ"`
    pub value: Option<String>,
    /// Unparsed event attribute index. Might be `None`. Eg: `true`
    pub index: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxResp {
    pub tx: Tx,
    pub tx_response: TxResponse,
}
