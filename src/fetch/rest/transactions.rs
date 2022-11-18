use chrono::DateTime;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tokio::join;

use super::others::{DenomAmount, InternalDenomAmount, Pagination, PaginationConfig, PublicKey};
use crate::{
    chain::Chain,
    routes::rest::{calc_pages, OutRestResponse},
};

impl Chain {
    /// Returns transaction by given hash.
    pub async fn get_tx_by_hash(&self, hash: &str) -> Result<OutRestResponse<InternalTransaction>, String> {
        let path = format!("/cosmos/tx/v1beta1/txs/{hash}");

        let resp = self.rest_api_request::<TxResp>(&path, &[]).await?;

        let tx = InternalTransaction::new(resp.tx, resp.tx_response, self).await?;

        OutRestResponse::new(tx, 0)
    }

    /// Returns transactions with given sender.
    pub async fn get_txs_by_sender(
        &self,
        sender_address: &str,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalTransactionSimple>>, String> {
        let mut query = vec![];

        query.push(("events", format!("message.sender='{}'", sender_address)));
        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));
        query.push(("order_by", "ORDER_BY_DESC".to_string()));

        let resp = self.rest_api_request::<TxsResp>("/cosmos/tx/v1beta1/txs", &query).await?;

        let mut txs = vec![];

        for i in 0..resp.txs.len() {
            let (tx, tx_response) = (
                resp.txs
                    .get(i)
                    .ok_or_else(|| format!("The count of transactions and transaction responses aren't the same."))?,
                resp.tx_responses
                    .get(i)
                    .ok_or_else(|| format!("The count of transactions and transaction responses aren't the same."))?,
            );

            txs.push(InternalTransactionSimple::new(tx, tx_response, self)?)
        }

        let pages = calc_pages(resp.pagination, config)?;

        OutRestResponse::new(txs, pages)
    }

    /// Returns transactions with given recipient.
    pub async fn get_txs_by_recipient(
        &self,
        recipient_address: &str,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalTransactionSimple>>, String> {
        let mut query = vec![];

        query.push(("events", format!("message.recipient='{}'", recipient_address)));
        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));
        query.push(("order_by", "ORDER_BY_DESC".to_string()));

        let resp = self.rest_api_request::<TxsResp>("/cosmos/tx/v1beta1/txs", &query).await?;

        let mut txs = vec![];

        for i in 0..resp.txs.len() {
            let (tx, tx_response) = (
                resp.txs
                    .get(i)
                    .ok_or_else(|| format!("The count of transactions and transaction responses aren't the same."))?,
                resp.tx_responses
                    .get(i)
                    .ok_or_else(|| format!("The count of transactions and transaction responses aren't the same."))?,
            );

            txs.push(InternalTransactionSimple::new(tx, tx_response, self)?)
        }

        let pages = calc_pages(resp.pagination, config)?;

        OutRestResponse::new(txs, pages)
    }

    /// Returns transactions at given height.
    pub async fn get_txs_by_height(
        &self,
        block_height: Option<u64>,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalTransactionSimple>>, String> {
        let mut query = vec![];

        if let Some(block_height) = block_height {
            query.push(("events", format!("tx.height={}", block_height)));
        };
        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));
        query.push(("order_by", "ORDER_BY_DESC".to_string()));

        let resp = self.rest_api_request::<TxsResp>("/cosmos/tx/v1beta1/txs", &query).await?;

        let mut txs = vec![];

        for i in 0..resp.txs.len() {
            let (tx, tx_response) = (
                resp.txs
                    .get(i)
                    .ok_or_else(|| format!("The count of transactions and transaction responses aren't the same."))?,
                resp.tx_responses
                    .get(i)
                    .ok_or_else(|| format!("The count of transactions and transaction responses aren't the same."))?,
            );

            txs.push(InternalTransactionSimple::new(tx, tx_response, self)?)
        }

        let pages = calc_pages(resp.pagination, config)?;

        OutRestResponse::new(txs, pages)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalTransaction {
    pub hash: String,
    pub height: u64,
    pub time: i64,
    pub fee: f64,
    pub gas_wanted: u64,
    pub gas_used: u64,
    pub raw: String,
    pub result: String,
    pub memo: String,
    pub content: Vec<InternalTransactionContent>,
}

impl InternalTransaction {
    async fn new(tx: Tx, tx_response: TxResponse, chain: &Chain) -> Result<Self, String> {
        let mut jobs = vec![];

        for message in tx.body.messages {
            jobs.push(async move {
                Ok::<InternalTransactionContent, String>(match message {
                    TxsTransactionMessage::Known(message) => match message {
                        TxsTransactionMessageKnowns::Delegate {
                            delegator_address,
                            validator_address,
                            amount,
                        } => {
                            let validator_name = chain.get_validator(&validator_address).await?.description.moniker;
                            let amount = chain.calc_amount_u128_to_f64(
                                amount
                                    .amount
                                    .parse::<u128>()
                                    .or_else(|_| Err(format!("Cannot parse delegation amount, '{}'.", amount.amount)))?,
                            );

                            InternalTransactionContent::Known(InternalTransactionContentKnowns::Delegate {
                                delegator_address,
                                validator_name,
                                validator_address,
                                amount,
                            })
                        }

                        TxsTransactionMessageKnowns::Redelegate {
                            delegator_address,
                            validator_src_address,
                            validator_dst_address,
                            amount,
                        } => {
                            let amount = chain.calc_amount_u128_to_f64(
                                amount
                                    .amount
                                    .parse::<u128>()
                                    .or_else(|_| Err(format!("Cannot parse delegation amount, '{}'.", amount.amount)))?,
                            );

                            let (validator_from_resp, validator_to_resp) =
                                join!(chain.get_validator(&validator_src_address), chain.get_validator(&validator_dst_address));

                            let validator_from = validator_from_resp?;
                            let validator_to = validator_to_resp?;

                            InternalTransactionContent::Known(InternalTransactionContentKnowns::Redelegate {
                                delegator_address,
                                validator_from_name: validator_from.description.moniker,
                                validator_from_address: validator_src_address,
                                validator_to_name: validator_to.description.moniker,
                                validator_to_address: validator_dst_address,
                                amount,
                            })
                        }

                        TxsTransactionMessageKnowns::Revoke {
                            granter_address,
                            grantee_address,
                        } => {
                            let grantee_valoper_addr = chain.base_to_valoper(&grantee_address)?;
                            let grantee_name = chain.get_validator(&grantee_valoper_addr).await?.description.moniker;

                            InternalTransactionContent::Known(InternalTransactionContentKnowns::Revoke {
                                granter_address,
                                grantee_address,
                                grantee_name,
                            })
                        }

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
                        } => {
                            let amount = chain.calc_amount_u128_to_f64(
                                amount
                                    .amount
                                    .parse::<u128>()
                                    .or_else(|_| Err(format!("Cannot parse undelegation amount, '{}'.", amount.amount)))?,
                            );

                            let validator_name = chain.get_validator(&validator_address).await?.description.moniker;

                            InternalTransactionContent::Known(InternalTransactionContentKnowns::Undelegate {
                                delegator_address,
                                validator_name,
                                validator_address,
                                amount,
                            })
                        }

                        TxsTransactionMessageKnowns::Vote { proposal_id, voter, option } => {
                            let proposal_id = proposal_id
                                .parse::<u32>()
                                .or_else(|_| Err(format!("Cannot parse proposal ID, '{}'.", proposal_id)))?;

                            let option = match option.as_ref() {
                                "VOTE_OPTION_YES" => "Yes",
                                "VOTE_OPTION_NO" => "No",
                                // Other VOTE options gonna be added in preferred format.
                                // TODO!
                                _ => "Unknown",
                            }
                            .to_string();

                            InternalTransactionContent::Known(InternalTransactionContentKnowns::Vote {
                                proposal_id,
                                voter_address: voter,
                                option,
                            })
                        }

                        TxsTransactionMessageKnowns::WithdrawDelegatorReward {
                            delegator_address,
                            validator_address,
                        } => {
                            let validator_name = chain.get_validator(&validator_address).await?.description.moniker;

                            InternalTransactionContent::Known(InternalTransactionContentKnowns::WithdrawDelegatorReward {
                                delegator_address,
                                validator_name,
                                validator_address,
                            })
                        }
                        TxsTransactionMessageKnowns::EthereumTx { hash } => {
                            InternalTransactionContent::Known(InternalTransactionContentKnowns::EthereumTx { hash })
                        }
                    },
                    TxsTransactionMessage::Unknown { r#type } => InternalTransactionContent::Unknown { r#type },
                })
            })
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
                .or_else(|_| Err(format!("Cannot parse transaction height, '{}'.", tx_response.height)))?,
            time: DateTime::parse_from_rfc3339(&tx_response.timestamp)
                .or_else(|_| Err(format!("Cannot parse transaction timestamp, '{}'.", tx_response.timestamp)))?
                .timestamp_millis(),
            fee: tx
                .auth_info
                .fee
                .amount
                .get(0)
                .and_then(|ad| Some(ad.amount.to_string()))
                .and_then(|amount| amount.parse::<u128>().ok())
                .and_then(|amount| Some(chain.calc_amount_u128_to_f64(amount)))
                .unwrap_or(0.0),
            gas_wanted: tx_response
                .gas_wanted
                .parse::<u64>()
                .or_else(|_| Err(format!("Cannot parse transaction gas wanted, '{}'.", tx_response.gas_wanted)))?,
            gas_used: tx_response
                .gas_used
                .parse::<u64>()
                .or_else(|_| Err(format!("Cannot parse transaction gas used, '{}'.", tx_response.gas_used)))?,
            result: "Success".to_string(),
            memo: tx.body.memo,
            raw: tx_response.raw_log,
            content,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum InternalTransactionContent {
    Known(InternalTransactionContentKnowns),
    Unknown { r#type: String },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum InternalTransactionContentKnowns {
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
        grantee_name: String,
    },
    Vote {
        proposal_id: u32,
        voter_address: String,
        option: String,
    },
    #[serde(rename = "Ethereum Tx")]
    EthereumTx { hash: String },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalTransactionSimple {
    height: u64,
    r#type: String,
    hash: String,
    amount: f64,
    fee: f64,
    result: String,
    time: i64,
}

impl InternalTransactionSimple {
    fn new(tx: &TxTransaction, tx_response: &TxResponse, chain: &Chain) -> Result<Self, String> {
        Ok(Self {
            height: tx_response
                .height
                .parse()
                .or_else(|_| Err(format!("Cannot parse transaction height, '{}'.", tx_response.height)))?,
            r#type: tx
                .body
                .messages
                .get(0)
                .and_then(|msg| {
                    Some(match msg {
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
                        }
                        .to_string(),
                        TxsTransactionMessage::Unknown { r#type } => r#type.to_string(),
                    })
                })
                .ok_or_else(|| format!("There is no TX type, '{}'.", tx_response.txhash))?,
            hash: tx_response.txhash.to_string(),
            // How to find the amount in different types of TXs. TODO!
            amount: 0.0,
            fee: tx
                .auth_info
                .fee
                .amount
                .get(0)
                .and_then(|ad| Some(ad.amount.to_string()))
                .and_then(|amount| amount.parse::<u128>().ok())
                .and_then(|amount| Some(chain.calc_amount_u128_to_f64(amount)))
                .unwrap_or(0.0),
            // How to define the result here? TODO!,
            result: "Success".to_string(),
            time: DateTime::parse_from_rfc3339(&tx_response.timestamp)
                .or_else(|_| Err(format!("Cannot parse transaction timestamp, '{}'.", tx_response.timestamp)))?
                .timestamp_millis(),
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxsResp {
    pub txs: Vec<TxTransaction>,
    pub tx_responses: Vec<TxResponse>,
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxTransaction {
    /// Transaction body.
    pub body: TxsTransactionBody,
    /// Transaction auth information.
    pub auth_info: TxsTransactionAuthInfo,
    /// Array of Base 64 encoded transaction signatures.
    pub signatures: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
pub struct TxsTransactionAuthInfo {
    /// Transaction fee.
    pub fee: TxsTransactionAuthInfoFee,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum TxsTransactionMessage {
    Known(TxsTransactionMessageKnowns),
    Unknown {
        #[serde(rename = "@type")]
        r#type: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "@type")]
pub enum TxsTransactionMessageKnowns {
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

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
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

pub struct InternalTxResponse {
    /// Block height. Eg: `12713829`
    pub height: u64,
    /// HEX encoded transaction hash. Eg: `D29DEB0948ADC9B14A1758ED164A46407AF33EA2950404DB4AFFF68164B01C58`
    pub txhash: String,
    /// Transaction codespace. Eg: `""`
    pub codespace: String,
    /// Code. Eg: `0`
    pub code: u32,
    /// HEX encoded data. Eg: `"0A1E0A1C2F636F736D6F732E62616E6B2E763162657461312E4D736753656E64"`
    pub data: String,
    /// JSON encoded raw log. Eg: `"[{\"events\":[{\"type\":\"coin_received\",\"attributes\":[{\"key\":\"receiver\",\"value\":\"cosmos1vl8xm7x04cedgh639hc9ucvf6zc754fyfewhef\"},{\"key\":\"amount\",\"value\":\"450000uatom\"}]},{\"type\":\"coin_spent\",\"attributes\":[{\"key\":\"spender\",\"value\":\"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf\"},{\"key\":\"amount\",\"value\":\"450000uatom\"}]},{\"type\":\"message\",\"attributes\":[{\"key\":\"action\",\"value\":\"/cosmos.bank.v1beta1.MsgSend\"},{\"key\":\"sender\",\"value\":\"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf\"},{\"key\":\"module\",\"value\":\"bank\"}]},{\"type\":\"transfer\",\"attributes\":[{\"key\":\"recipient\",\"value\":\"cosmos1vl8xm7x04cedgh639hc9ucvf6zc754fyfewhef\"},{\"key\":\"sender\",\"value\":\"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf\"},{\"key\":\"amount\",\"value\":\"450000uatom\"}]}]}]"`
    pub raw_log: String,
    /// Info. Eg: `""`
    pub info: String,
    // Gas wanted. Eg: `80000`
    pub gas_wanted: u64,
    /// Gas used. Eg: `74032`
    pub gas_used: u64,
    // Tx.
    pub tx: TxsResponseTx,
    // Timestamp in milliseconds.
    pub timestamp: i64,
    // Events.
    pub events: Vec<TxsResponseEvent<UnparsedTxEventAttribute>>,
}

#[derive(Deserialize, Serialize, Debug)]
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
#[derive(Deserialize, Serialize, Debug)]
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
