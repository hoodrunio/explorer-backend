use std::collections::HashMap;

use chrono::DateTime;
use futures::{
    future::{join_all, BoxFuture},
    FutureExt,
};
use mongodb::bson::doc;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::fetch::socket::EvmPollVote;
use crate::{
    chain::Chain,
    routes::{calc_pages, OutRestResponse},
    utils::{get_msg_name, Base64Convert},
};
use crate::{database::TransactionForDb, routes::ChainAmountItem};

use super::{
    blocks::CosmosEvent,
    others::{DenomAmount, Pagination, PaginationConfig, PublicKey},
};

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
                            a.content.iter().any(|a| {
                                if let InternalTransactionContent::Known(InternalTransactionContentKnowns::EthereumTx { hash: tx_hash, data: _ }) = a
                                {
                                    tx_hash == hash
                                } else {
                                    false
                                }
                            })
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

            txs.push(TransactionItem::new(tx, tx_response, self).await?)
        }

        let pages = calc_pages(resp.pagination.unwrap_or(Pagination::default()), config)?;

        Ok(OutRestResponse::new(txs, pages))
    }

    pub async fn get_internal_txs_by_sender_height(
        &self,
        sender_address: &str,
        block_height: Option<u64>,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalTransaction>>, String> {
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

            txs.push(TransactionItem::new(tx, tx_response, self).await?)
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

            txs.push(TransactionItem::new(tx, tx_response, self).await?)
        }

        let pages = calc_pages(resp.pagination.unwrap_or(Pagination::default()), config)?;

        Ok(OutRestResponse::new(txs, pages))
    }

    /// Returns transactions from db.
    pub async fn get_last_txs_from_db(&self, count: u16) -> Result<Vec<TransactionForDb>, String> {
        let txs = self.database.find_last_count_transactions(None, count).await?;

        Ok(txs)
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
    pub async fn get_axelar_sender_heartbeat_info(
        &self,
        val_voter_address: &String,
        block_height: u64,
    ) -> Result<InternalAxelarHeartbeatInfo, String> {
        match self
            .get_internal_txs_by_sender_height(val_voter_address, Some(block_height), PaginationConfig::new().limit(1).page(1))
            .await
        {
            Ok(txs_res) => {
                for contents in txs_res.value {
                    if let Some(res) = contents.extract_axelar_heartbeat_info() {
                        return Ok(res);
                    };
                }
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
    pub amount: ChainAmountItem,
    pub height: u64,
    pub time: i64,
    pub fee: ChainAmountItem,
    pub gas_wanted: u64,
    pub gas_used: u64,
    pub result: String,
    pub memo: String,
    pub signatures: Vec<String>,
    pub content: Vec<InternalTransactionContent>,
    pub logs: Vec<TxResponseLog>,
    pub raw: String,
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

        let denom_amount = tx
            .body
            .messages
            .get(0)
            .map(|msg| match msg {
                TxsTransactionMessage::Known(msg) => match msg {
                    TxsTransactionMessageKnowns::Delegate {
                        delegator_address: _,
                        validator_address: _,
                        amount,
                    } => Some(amount),
                    TxsTransactionMessageKnowns::Redelegate {
                        delegator_address: _,
                        validator_src_address: _,
                        validator_dst_address: _,
                        amount,
                    } => Some(amount),
                    TxsTransactionMessageKnowns::Send {
                        from_address: _,
                        to_address: _,
                        amount,
                    } => amount.get(0),
                    TxsTransactionMessageKnowns::Undelegate {
                        delegator_address: _,
                        validator_address: _,
                        amount,
                    } => Some(amount),
                    _ => None,
                },
                _ => None,
            })
            .ok_or_else(|| format!("There is no TX type, '{}'.", tx_response.txhash))?;

        let amount = match denom_amount {
            Some(amount_denom) => match chain
                .string_amount_parser(amount_denom.amount.clone(), Some(amount_denom.denom.clone()))
                .await
            {
                Ok(res) => res,
                Err(err) => return Err(format!("Cannot parse transaction amount, '{}'.", err)),
            },
            None => ChainAmountItem::default(),
        };

        for message in tx.body.messages {
            let logs = tx_response.logs.clone();
            jobs.push(async move { message.to_internal(chain, &Some(logs.clone())).await })
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
            fee: chain
                .string_amount_parser(
                    tx.auth_info
                        .fee
                        .amount
                        .get(0)
                        .map(|ad| ad.amount.to_string())
                        .unwrap_or(String::from("0.0")),
                    None,
                )
                .await?,
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
            signatures: match tx_response.tx {
                TxsResponseTx::Tx { signatures, .. } => signatures,
            },
            memo: tx.body.memo,
            raw: tx_response.raw_log,
            content,
            amount,
            r#type,
            logs: tx_response.logs,
        })
    }
    pub fn extract_axelar_heartbeat_info(&self) -> Option<InternalAxelarHeartbeatInfo> {
        let mut res = None;
        for content_item in &self.content {
            if let InternalTransactionContent::Known(InternalTransactionContentKnowns::AxelarRefundRequest { sender: _, inner_message }) =
                content_item
            {
                if let InnerMessage::Known(InnerMessageKnown::HeartBeatRequest { sender, key_ids }) = inner_message {
                    res = Some(InternalAxelarHeartbeatInfo {
                        sender: sender.clone(),
                        key_ids: key_ids.clone(),
                        signatures: self.signatures.clone(),
                        tx_hash: self.hash.clone(),
                        height: self.height,
                        timestamp: self.time,
                    });
                    break;
                }
            }
        }

        res
    }
    pub fn is_evm_poll_failed(&self) -> bool {
        let logs = &self.logs.clone();
        match logs.into_iter().find(|log| log.log == "failed" && log.log != "already confirmed") {
            None => {}
            Some(_) => {
                return true;
            }
        };

        for log in logs {
            match log.events.clone().into_iter().find(|event| event.r#type == "EVMEventFailed") {
                None => {}
                Some(_) => {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_evm_poll_confirmation_tx(&self) -> bool {
        let evm_confirmation_event_types = [
            String::from("axelar.evm.v1beta1.EVMEventConfirmed"),
            String::from("depositConfirmation"),
            String::from("eventConfirmation"),
            String::from("transferKeyConfirmation"),
            String::from("tokenConfirmation"),
            String::from("TokenSent"),
            String::from("ContractCall"),
        ];
        let logs = &self.logs.clone();
        for log in logs {
            for event in &log.events {
                let event_type = &event.r#type.clone();
                if evm_confirmation_event_types.contains(event_type) {
                    return true;
                };
            }
        }

        false
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
    pub amount: ChainAmountItem,
    pub fee: ChainAmountItem,
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
        amounts: Vec<ChainAmountItem>,
    },
    Delegate {
        delegator_address: String,
        validator_name: String,
        validator_address: String,
        amount: ChainAmountItem,
    },
    Undelegate {
        delegator_address: String,
        validator_name: String,
        validator_address: String,
        amount: ChainAmountItem,
    },
    #[serde(rename = "Withdraw Delegator Reward")]
    WithdrawDelegatorReward {
        amount: ChainAmountItem,
        delegator_address: String,
        validator_name: String,
        validator_address: String,
    },
    #[serde(rename = "Withdraw Validator Commission")]
    WithdrawValidatorCommission {
        amount: ChainAmountItem,
        validator_address: String,
    },
    Redelegate {
        delegator_address: String,
        validator_from_name: String,
        validator_from_address: String,
        validator_to_name: String,
        validator_to_address: String,
        amount: ChainAmountItem,
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
        data: EthereumTxData,
    },
    SwapExactAmountIn {
        sender: String,
        pool_ids: Vec<String>,
        token_in: ChainAmountItem,
        token_out: ChainAmountItem,
    },
    IBCUpdateClient {
        signer: String,
        client_id: String,
        block: String,
        app: String,
        chain_id: String,
        height: String,
        time: String,
        hash: String,
        total: i64,
        last_commit_hash: String,
        data_hash: String,
        validators_hash: String,
        next_validators_hash: String,
        consensus_hash: String,
        app_hash: String,
        last_results_hash: String,
        evidence_hash: String,
        proposer_address: String,
    },
    IBCReceived {
        sequence: String,
        source_port: String,
        source_channel: String,
        destination_port: String,
        destination_channel: String,
        signer: String,
        amount: ChainAmountItem,
        origin_amount: String,
        origin_denom: String,
        sender: String,
        receiver: String,
    },
    IBCAcknowledgement {
        sequence: String,
        source_port: String,
        source_channel: String,
        destination_port: String,
        destination_channel: String,
        signer: String,
        amount: ChainAmountItem,
        origin_amount: String,
        origin_denom: String,
        sender: String,
        receiver: String,
    },
    IBCTransfer {
        sender: String,
        receiver: String,
        source_channel: String,
        source_port: String,
        sequence: String,
        amount: ChainAmountItem,
        origin_amount: String,
        origin_denom: String,
    },
    RegisterProxy {
        sender: String,
        proxy_addr: String,
    },
    AxelarRefundRequest {
        sender: String,
        inner_message: InnerMessage,
    },
    AxelarLinkRequest {
        sender: String,
        recipient_addr: String,
        recipient_chain: String,
        asset: String,
        source_chain: String,
        deposit_address: String,
    },
    AxelarConfirmDepositRequest {
        asset: String,
        sender: String,
        destination_chain: String,
        destination_address: String,
        amount: String,
        transfer_id: String,
        deposit_address: String,
        source_chain: String,
    },
    AxelarCreatePendingTransfersRequest {
        chain: String,
        sender: String,
        amount: ChainAmountItem,
        destination_address: String,
        destination_chain: String,
        transfer_id: String,
        command_id: String,
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
    async fn new(tx: &Tx, tx_response: &TxResponse, chain: &Chain) -> Result<Self, String> {
        let fee_amount = tx
            .auth_info
            .fee
            .amount
            .get(0)
            .map(|ad| ad.amount.to_string())
            .unwrap_or(String::from("0.0"));

        let fee = chain.string_amount_parser(fee_amount, None).await?;
        let denom_amount = tx
            .body
            .messages
            .get(0)
            .map(|msg| match msg {
                TxsTransactionMessage::Known(msg) => match msg {
                    TxsTransactionMessageKnowns::Delegate {
                        delegator_address: _,
                        validator_address: _,
                        amount,
                    } => Some(amount),
                    TxsTransactionMessageKnowns::Redelegate {
                        delegator_address: _,
                        validator_src_address: _,
                        validator_dst_address: _,
                        amount,
                    } => Some(amount),
                    TxsTransactionMessageKnowns::Send {
                        from_address: _,
                        to_address: _,
                        amount,
                    } => amount.get(0),
                    TxsTransactionMessageKnowns::Undelegate {
                        delegator_address: _,
                        validator_address: _,
                        amount,
                    } => Some(amount),
                    _ => None,
                },
                _ => None,
            })
            .ok_or_else(|| format!("There is no TX type, '{}'.", tx_response.txhash))?;

        let amount = match denom_amount {
            Some(amount_denom) => match chain
                .string_amount_parser(amount_denom.amount.clone(), Some(amount_denom.denom.clone()))
                .await
            {
                Ok(res) => res,
                Err(err) => return Err(format!("Cannot parse transaction amount, '{}'.", err)),
            },
            None => ChainAmountItem::default(),
        };

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
            amount,
            fee,
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
    pub fn to_internal<'a>(
        self,
        chain: &'a Chain,
        logs: &'a Option<Vec<TxResponseLog>>,
    ) -> BoxFuture<'a, Result<InternalTransactionContent, String>> {
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
                        amount: chain.string_amount_parser(amount.amount.clone(), Some(amount.denom.clone())).await?,
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
                        amount: chain.string_amount_parser(amount.amount.clone(), Some(amount.denom.clone())).await?,
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
                            amounts.push(
                                chain
                                    .string_amount_parser(denom_amount.amount.clone(), Some(denom_amount.denom.clone()))
                                    .await?,
                            )
                            //TODO check if it is native token if it is we can convert with decimal pow if not get related token decimal count.
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
                        amount: chain.string_amount_parser(amount.amount.clone(), Some(amount.denom.clone())).await?,
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
                    } => {
                        let logs = logs.clone().unwrap_or(vec![]);
                        let mut amount_string_denom = String::default();
                        for log in logs {
                            if let Some(event) = log.events.iter().find(|event| event.r#type == "withdraw_rewards") {
                                let is_validator_attr = event
                                    .attributes
                                    .iter()
                                    .find(|attr| attr.key == "validator" && attr.value == validator_address);
                                if is_validator_attr.is_some() {
                                    amount_string_denom = event
                                        .attributes
                                        .iter()
                                        .find(|attr| attr.key == "amount")
                                        .map(|attr| attr.value.replace(chain.config.main_denom.as_str(), ""))
                                        .unwrap_or(String::default());
                                }
                            };
                        }

                        let amount = chain.string_amount_parser(amount_string_denom, None).await?;

                        InternalTransactionContent::Known(InternalTransactionContentKnowns::WithdrawDelegatorReward {
                            amount,
                            delegator_address,
                            validator_name: chain.database.find_validator_by_operator_addr(&validator_address.clone()).await?.name,
                            validator_address,
                        })
                    }
                    TxsTransactionMessageKnowns::WithdrawValidatorCommission { validator_address } => {
                        let logs = logs.clone().unwrap_or(vec![]);
                        let mut amount_string_denom = String::default();
                        for log in logs {
                            if let Some(event) = log.events.iter().find(|event| event.r#type == "withdraw_commission") {
                                amount_string_denom = event
                                    .attributes
                                    .iter()
                                    .find(|attr| attr.key == "amount")
                                    .map(|attr| attr.value.replace(chain.config.main_denom.as_str(), ""))
                                    .unwrap_or(String::default());
                            };
                        }

                        let amount = chain.string_amount_parser(amount_string_denom, None).await?;

                        InternalTransactionContent::Known(InternalTransactionContentKnowns::WithdrawValidatorCommission { amount, validator_address })
                    }
                    TxsTransactionMessageKnowns::EthereumTx { hash, data } => {
                        InternalTransactionContent::Known(InternalTransactionContentKnowns::EthereumTx { hash, data })
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
                                let resps = join_all(msgs.into_iter().map(|msg| msg.to_internal(chain, &None))).await;
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
                        InternalTransactionContent::Known(InternalTransactionContentKnowns::AxelarRefundRequest { sender, inner_message })
                    }
                    TxsTransactionMessageKnowns::IBCUpdateClient { signer, client_id, header } => {
                        InternalTransactionContent::Known(InternalTransactionContentKnowns::IBCUpdateClient {
                            signer,
                            client_id,
                            block: header.signed_header.header.version.block,
                            app: header.signed_header.header.version.app,
                            chain_id: header.signed_header.header.chain_id,
                            height: header.signed_header.header.height,
                            time: header.signed_header.header.time,
                            hash: header.signed_header.header.last_block_id.part_set_header.hash,
                            total: header.signed_header.header.last_block_id.part_set_header.total,
                            last_commit_hash: header.signed_header.header.last_commit_hash,
                            data_hash: header.signed_header.header.data_hash,
                            validators_hash: header.signed_header.header.validators_hash,
                            next_validators_hash: header.signed_header.header.next_validators_hash,
                            consensus_hash: header.signed_header.header.consensus_hash,
                            app_hash: header.signed_header.header.app_hash,
                            last_results_hash: header.signed_header.header.last_results_hash,
                            evidence_hash: header.signed_header.header.evidence_hash,
                            proposer_address: header.signed_header.header.proposer_address,
                        })
                    }
                    TxsTransactionMessageKnowns::IBCReceived { packet, signer, .. } => {
                        let amount_data = serde_json::from_str::<TransactionMessagePacketAmount>(&packet.data)
                            .map_err(|e| format!("Cannot parse packet data, {}. Error {}.", packet.data, e))?;
                        let amount = chain
                            .string_amount_parser(amount_data.amount.clone(), Some(amount_data.denom.clone()))
                            .await?;

                        InternalTransactionContent::Known(InternalTransactionContentKnowns::IBCReceived {
                            sequence: packet.sequence,
                            source_port: packet.source_port,
                            source_channel: packet.source_channel,
                            destination_port: packet.destination_port,
                            destination_channel: packet.destination_channel,
                            origin_amount: amount_data.amount,
                            origin_denom: amount_data.denom,
                            sender: amount_data.sender,
                            receiver: amount_data.receiver,
                            signer,
                            amount,
                        })
                    }
                    TxsTransactionMessageKnowns::IBCAcknowledgement { signer, packet, .. } => {
                        let amount_data = serde_json::from_str::<TransactionMessagePacketAmount>(&packet.data)
                            .map_err(|e| format!("Cannot parse packet data, {}. Error {}.", packet.data, e))?;
                        let amount = chain
                            .string_amount_parser(amount_data.amount.clone(), Some(amount_data.denom.clone()))
                            .await?;

                        InternalTransactionContent::Known(InternalTransactionContentKnowns::IBCAcknowledgement {
                            sequence: packet.sequence,
                            source_port: packet.source_port,
                            source_channel: packet.source_channel,
                            destination_port: packet.destination_port,
                            destination_channel: packet.destination_channel,
                            origin_amount: amount_data.amount,
                            origin_denom: amount_data.denom,
                            sender: amount_data.sender,
                            receiver: amount_data.receiver,
                            signer,
                            amount,
                        })
                    }
                    TxsTransactionMessageKnowns::IBCTransfer {
                        source_port,
                        source_channel,
                        token,
                        sender,
                        receiver,
                        ..
                    } => {
                        let amount = chain.string_amount_parser(token.amount.clone(), Some(token.denom.clone())).await?;
                        InternalTransactionContent::Known(InternalTransactionContentKnowns::IBCTransfer {
                            sender,
                            receiver,
                            source_channel,
                            source_port,
                            //TODO: get the sequence from the transaction
                            sequence: String::from("Unknown"),
                            origin_amount: token.amount,
                            origin_denom: token.denom,
                            amount,
                        })
                    }
                    TxsTransactionMessageKnowns::SwapExactAmountIn {
                        sender,
                        routes,
                        token_in,
                        token_out_min_amount,
                    } => {
                        let token_in_amount = chain.string_amount_parser(token_in.amount.clone(), Some(token_in.denom.clone())).await?;

                        let token_out_amount = {
                            let token_out_denom = routes.iter().last().map(|r| r.token_out_denom.clone());

                            chain.string_amount_parser(token_out_min_amount, token_out_denom).await?
                        };

                        let pool_ids = routes.iter().map(|r| r.pool_id.clone()).collect();

                        InternalTransactionContent::Known(InternalTransactionContentKnowns::SwapExactAmountIn {
                            sender,
                            pool_ids,
                            token_in: token_in_amount,
                            token_out: token_out_amount,
                        })
                    }
                    TxsTransactionMessageKnowns::AxelarLinkRequest {
                        sender,
                        recipient_addr,
                        recipient_chain,
                        asset,
                    } => {
                        let (source_chain, deposit_address) = {
                            let mut source_chain = String::from("");
                            let mut deposit_address = String::from("");
                            let logs = logs.clone().unwrap_or_default();
                            for log in logs {
                                for event in &log.events {
                                    if event.r#type == "link" {
                                        for attribute in &event.attributes {
                                            if attribute.key == "sourceChain" {
                                                source_chain = attribute.value.clone();
                                            }
                                            if attribute.key == "depositAddress" {
                                                deposit_address = attribute.value.clone();
                                            }
                                        }
                                    }
                                }
                            }

                            (source_chain, deposit_address)
                        };

                        InternalTransactionContent::Known(InternalTransactionContentKnowns::AxelarLinkRequest {
                            sender,
                            recipient_addr,
                            recipient_chain,
                            asset,
                            source_chain,
                            deposit_address,
                        })
                    }
                    TxsTransactionMessageKnowns::AxelarConfirmDepositRequest {
                        denom,
                        deposit_address,
                        sender,
                    } => {
                        let mut amount = String::from("");
                        let mut destination_chain = String::from("");
                        let mut destination_address = String::from("");
                        let mut transfer_id = String::from("");
                        let mut source_chain = String::from("");

                        let logs = logs.clone().unwrap_or_default();
                        for log in logs {
                            for event in &log.events {
                                if event.r#type == "depositConfirmation" {
                                    for attribute in &event.attributes {
                                        if attribute.key == "sourceChain" {
                                            source_chain = attribute.value.clone();
                                        }
                                        if attribute.key == "destinationAddress" {
                                            destination_address = attribute.value.clone();
                                        }
                                        if attribute.key == "destinationChain" {
                                            destination_chain = attribute.value.clone();
                                        }
                                        if attribute.key == "amount" {
                                            amount = attribute.value.clone();
                                        }
                                        if attribute.key == "transferID" {
                                            transfer_id = attribute.value.clone();
                                        }
                                    }
                                }
                            }
                        }

                        let amount = amount.replace(&denom, "");

                        InternalTransactionContent::Known(InternalTransactionContentKnowns::AxelarConfirmDepositRequest {
                            asset: denom,
                            deposit_address,
                            sender,
                            destination_chain,
                            destination_address,
                            amount,
                            transfer_id,
                            source_chain,
                        })
                    }
                    TxsTransactionMessageKnowns::AxelarCreatePendingTransfersRequest { chain: tx_chain, sender } => {
                        let mut amount = ChainAmountItem::default();
                        let mut destination_chain = String::from("");
                        let mut destination_address = String::from("");
                        let mut transfer_id = String::from("");
                        let mut command_id = String::from("");

                        let logs = logs.clone().unwrap_or_default();
                        for log in logs {
                            for event in &log.events {
                                if event.r#type == "axelar.evm.v1beta1.MintCommand" {
                                    for attribute in &event.attributes {
                                        if attribute.key == "asset" {
                                            if let Ok(value) = serde_json::from_str::<DenomAmount>(&attribute.value) {
                                                amount = chain.string_amount_parser(value.amount, Some(value.denom)).await?;
                                            }
                                        }
                                        if attribute.key == "chain" {
                                            destination_address = attribute.value.clone();
                                        }
                                        if attribute.key == "destination_chain" {
                                            destination_chain = attribute.value.replace('\"', "").clone();
                                        }
                                        if attribute.key == "destination_address" {
                                            destination_address = attribute.value.replace('\"', "").clone();
                                        }
                                        if attribute.key == "transfer_id" {
                                            transfer_id = attribute.value.replace('\"', "").clone();
                                        }
                                        if attribute.key == "command_id" {
                                            command_id = attribute.value.clone();
                                        }
                                    }
                                }
                            }
                        }

                        InternalTransactionContent::Known(InternalTransactionContentKnowns::AxelarCreatePendingTransfersRequest {
                            chain: tx_chain,
                            sender,
                            amount,
                            destination_address,
                            destination_chain,
                            transfer_id,
                            command_id,
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
                TxsTransactionMessageKnowns::Delegate { .. } => "Delegate",
                TxsTransactionMessageKnowns::Redelegate { .. } => "Redelegate",
                TxsTransactionMessageKnowns::Revoke { .. } => "Revoke",
                TxsTransactionMessageKnowns::Send { .. } => "Send",
                TxsTransactionMessageKnowns::Undelegate { .. } => "Undelegate",
                TxsTransactionMessageKnowns::Vote { .. } => "Vote",
                TxsTransactionMessageKnowns::WithdrawDelegatorReward { .. } => "Withdraw Delegator Rewards",
                TxsTransactionMessageKnowns::WithdrawValidatorCommission { .. } => "Withdraw Validator Commission",
                TxsTransactionMessageKnowns::EthereumTx { .. } => "Ethereum Tx",
                TxsTransactionMessageKnowns::Grant { .. } => "Grant",
                TxsTransactionMessageKnowns::Exec { .. } => "Exec",
                TxsTransactionMessageKnowns::RegisterProxy { .. } => "RegisterProxy",
                TxsTransactionMessageKnowns::IBCUpdateClient { .. } => "IBCUpdateClient",
                TxsTransactionMessageKnowns::IBCReceived { .. } => "IBCReceived",
                TxsTransactionMessageKnowns::IBCAcknowledgement { .. } => "IBCAcknowledgement",
                TxsTransactionMessageKnowns::IBCTransfer { .. } => "IBCTransfer",
                TxsTransactionMessageKnowns::SwapExactAmountIn { .. } => "SwapExactAmountIn",
                TxsTransactionMessageKnowns::AxelarRegisterProxy { .. } => "RegisterProxy",
                TxsTransactionMessageKnowns::AxelarRefundRequest { .. } => "AxelarRefundRequest",
                TxsTransactionMessageKnowns::AxelarLinkRequest { .. } => "LinkRequest",
                TxsTransactionMessageKnowns::AxelarConfirmDepositRequest { .. } => "ConfirmDepositRequest",
                TxsTransactionMessageKnowns::AxelarCreatePendingTransfersRequest { .. } => "CreatePendingTransfersRequest",
            }
            .to_string(),
            TxsTransactionMessage::Unknown(keys_values) => keys_values
                .get("@type")
                .cloned()
                .map(|r#type| get_msg_name(r#type.as_str().unwrap_or_else(|| "Unknown")))
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
    #[serde(rename = "/cosmos.distribution.v1beta1.MsgWithdrawValidatorCommission")]
    WithdrawValidatorCommission {
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
        data: EthereumTxData,
        // Ethereum transaction data.
        // There are multiple types of this property.
        // Creating an enum for it is necessary if we need to show the data in the explorer.
        // data: UNKNOWN,
    },
    #[serde(rename = "/snapshot.v1beta1.RegisterProxyRequest")]
    RegisterProxy { sender: String, proxy_addr: String },

    //Swap Exact Amount In
    #[serde(rename = "/osmosis.gamm.v1beta1.MsgSwapExactAmountIn")]
    SwapExactAmountIn {
        sender: String,
        routes: Vec<SwapRoute>,
        token_in: DenomAmount,
        token_out_min_amount: String,
    },

    //IBC Messages
    #[serde(rename = "/ibc.core.client.v1.MsgUpdateClient")]
    IBCUpdateClient {
        signer: String,
        client_id: String,
        header: IBCMessageHeader,
    },

    #[serde(rename = "/ibc.core.channel.v1.MsgRecvPacket")]
    IBCReceived {
        packet: TxsTransactionMessagePacket,
        proof_commitment: String,
        proof_height: RevisionHeight,
        signer: String,
    },

    #[serde(rename = "/ibc.core.channel.v1.MsgAcknowledgement")]
    IBCAcknowledgement {
        packet: TxsTransactionMessagePacket,
        proof_acked: String,
        acknowledgement: String,
        proof_height: RevisionHeight,
        signer: String,
    },

    #[serde(rename = "/ibc.applications.transfer.v1.MsgTransfer")]
    IBCTransfer {
        source_port: String,
        source_channel: String,
        token: DenomAmount,
        sender: String,
        receiver: String,
        timeout_height: RevisionHeight,
        timeout_timestamp: String,
        memo: Option<String>,
    },

    //Axelar Messages
    #[serde(rename = "/axelar.snapshot.v1beta1.RegisterProxyRequest")]
    AxelarRegisterProxy { sender: String, proxy_addr: String },
    #[serde(rename = "/axelar.reward.v1beta1.RefundMsgRequest")]
    AxelarRefundRequest { sender: String, inner_message: InnerMessage },
    #[serde(rename = "/axelar.axelarnet.v1beta1.LinkRequest")]
    AxelarLinkRequest {
        sender: String,
        recipient_addr: String,
        recipient_chain: String,
        asset: String,
    },
    #[serde(rename = "/axelar.axelarnet.v1beta1.ConfirmDepositRequest")]
    AxelarConfirmDepositRequest {
        denom: String,
        deposit_address: String,
        sender: String,
    },
    #[serde(rename = "/axelar.evm.v1beta1.CreatePendingTransfersRequest")]
    AxelarCreatePendingTransfersRequest { chain: String, sender: String },
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
    VoteRequest { sender: String, poll_id: String, vote: AxelarVote },
    #[serde(rename = "/axelar.tss.v1beta1.HeartBeatRequest")]
    HeartBeatRequest { sender: String, key_ids: Vec<String> },
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
    VoteEvent { chain: String, events: Vec<HashMap<String, Value>> },
}

impl AxelarKnownVote {
    pub fn evm_vote(&self) -> EvmPollVote {
        match self {
            AxelarKnownVote::VoteEvent { chain: _, events } => {
                if !events.is_empty() {
                    EvmPollVote::Yes
                } else {
                    EvmPollVote::No
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IbcAcknowledgementPacket {
    /// Source channel. Eg: `"channel-0"`
    pub source_channel: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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
    // Transaction events.
    pub events: Option<Vec<CosmosEvent>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TxResponseLog {
    /// Array of events.
    pub log: String,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TxsTransactionMessagePacket {
    #[serde(rename = "data", deserialize_with = "from_base64")]
    pub data: String,

    #[serde(rename = "source_port")]
    pub source_port: String,

    #[serde(rename = "destination_channel")]
    pub destination_channel: String,

    #[serde(rename = "destination_port")]
    pub destination_port: String,

    #[serde(rename = "timeout_timestamp")]
    pub timeout_timestamp: String,

    #[serde(rename = "timeout_height")]
    pub timeout_height: TimeoutHeight,

    #[serde(rename = "source_channel")]
    pub source_channel: String,

    #[serde(rename = "sequence")]
    pub sequence: String,
}

pub fn from_base64<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    Ok(String::base64_to_string(&String::from(s)))
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RevisionHeight {
    #[serde(rename = "revision_number")]
    pub revision_number: String,

    #[serde(rename = "revision_height")]
    pub revision_height: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionMessagePacketAmount {
    pub amount: String,
    pub denom: String,
    pub receiver: String,
    pub sender: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IBCMessageHeader {
    pub signed_header: IBCMessageSignedHeader,
    pub trusted_height: RevisionHeight,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IBCMessageSignedHeader {
    pub header: IBCTxMessageHeader,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IBCTxMessageHeader {
    pub version: IBCTxMessageHeaderVersion,
    pub chain_id: String,
    pub height: String,
    pub time: String,
    pub last_commit_hash: String,
    pub last_block_id: IBCTxMessageHeaderLastBlockId,
    pub data_hash: String,
    pub validators_hash: String,
    pub next_validators_hash: String,
    pub consensus_hash: String,
    pub app_hash: String,
    pub last_results_hash: String,
    pub evidence_hash: String,
    pub proposer_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IBCTxMessageHeaderVersion {
    pub block: String,
    pub app: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IBCTxMessageHeaderLastBlockId {
    pub hash: String,
    pub part_set_header: IBCTxMessageHeaderLastBlockIdPartSetHeader,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IBCTxMessageHeaderLastBlockIdPartSetHeader {
    pub hash: String,
    pub total: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SwapRoute {
    pub pool_id: String,
    pub token_out_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EthereumTxData {
    #[serde(rename = "@type")]
    pub r#type: String,
    pub nonce: String,
    pub gas_price: Option<String>,
    pub gas_tip_cap: Option<String>,
    pub gas_fee_cap: Option<String>,
    pub gas: String,
    pub to: String,
    pub value: String,
    pub data: Option<String>,
    pub accesses: Option<Vec<HashMap<String, Value>>>,
}
