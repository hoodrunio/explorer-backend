use std::{collections::BTreeMap, num::ParseIntError};

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::{chain::Chain, fetch::transactions::TransactionItem};

use super::{
    ConfirmDepositStarted, ConfirmGatewayTxStartedEvents, ConfirmKeyTransferStartedEvents, NewPollEvent, NewProposalVoteEvent, PollVoteEvent,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExtraTxEventData {
    NewPoll(NewPollEvent),
    PollVote(PollVoteEvent),
    NewProposalVote(NewProposalVoteEvent),
}

#[derive(Debug, Clone)]
pub enum ParseError {
    ParseIntError(ParseIntError),
    MissingData,
}

impl From<ParseIntError> for ParseError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ParseIntError(e) => {
                write!(f, "Failed to parse number: {}", e)
            }
            ParseError::MissingData => {
                write!(f, "Some data is missing")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseTransaction {
    /// `[ "F0E26D70191E27C8AB6249DE9C088B8C2812443CDF0DF04D7C83AE76A117C083" ]`
    // #[serde(rename = "tx.hash")]
    pub hash: String,
    /// `[ "2931697000000000aevmos" ]`
    // #[serde(rename = "tx.fee")]
    pub fee: String,
    /// `[ "8076531" ]`
    // #[serde(rename = "tx.height")]
    pub height: String,
    /// `[ "/ethermint.evm.v1.MsgEthereumTx" ]`
    // #[serde(rename = "message.action")]
    pub message_action: String,
    /// `[ "1535902500000000aevmos" ]`
    // #[serde(rename = "transfer.amount")]
    pub transfer_amount: String,
}

pub type TXMap = BTreeMap<String, Vec<String>>;

impl BaseTransaction {
    pub fn from_tx_events(ev: TXMap) -> Option<Self> {
        let tx_fee_denom = ev.get("tx.fee")?.get(0)?.to_string();
        let transfer_amount = ev
            .get("transfer.amount")?
            .iter()
            .filter(|str| str.to_string() != tx_fee_denom)
            .map(String::from)
            .collect::<Vec<String>>()
            .get(0)
            .unwrap_or(&String::from("0.00"))
            .clone();

        Some(Self {
            hash: ev.get("tx.hash")?.get(0)?.to_string(),
            fee: tx_fee_denom,
            height: ev.get("tx.height")?.get(0)?.to_string(),
            message_action: ev.get("message.action")?.get(0)?.to_string(),
            transfer_amount,
        })
    }
    pub async fn as_tx_item(&self, chain: &Chain) -> Result<TransactionItem, String> {
        let tx_fee_denom = self.fee.clone();
        let amount = chain
            .string_amount_parser(self.transfer_amount.replace(chain.config.main_denom.as_str(), "").clone(), None)
            .await?;

        let fee = chain
            .string_amount_parser(tx_fee_denom.replace(chain.config.main_denom.as_str(), "").clone(), None)
            .await?;

        Ok(TransactionItem {
            amount,
            fee,
            hash: self.hash.clone(),
            height: self
                .height
                .parse::<u64>()
                .map_err(|e| format!("Cannot parse tx height {}: {e}", self.height))?,
            time: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64,
            result: "Success".to_string(),
            tx_type: self
                .message_action
                .split_once("Msg")
                .map(|(_, r)| r)
                .unwrap_or(self.message_action.split('.').last().unwrap_or("Unknown"))
                .to_string(),
        })
    }
}

pub fn parse_transaction(events: TXMap) -> Result<(BaseTransaction, Option<ExtraTxEventData>), ParseError> {
    let tx = BaseTransaction::from_tx_events(events.clone()).ok_or(ParseError::MissingData)?;

    match tx.message_action.as_str() {
        "ConfirmERC20Deposit" | "ConfirmDeposit" => {
            if events.contains_key("axelar.evm.v1beta1.ConfirmDepositStarted.participants") {
                let sp_tx = ConfirmDepositStarted::from_tx_events(events);
                return Ok((tx, Some(ExtraTxEventData::NewPoll(sp_tx.into()))));
            }
        }
        "ConfirmGatewayTx" => {
            if events.contains_key("axelar.evm.v1beta1.ConfirmGatewayTxStarted.participants") {
                let sp_tx = ConfirmGatewayTxStartedEvents::from_tx_events(events);
                return Ok((tx, Some(ExtraTxEventData::NewPoll(sp_tx.into()))));
            }
        }
        "ConfirmTransferKey" => {
            if events.contains_key("axelar.evm.v1beta1.ConfirmKeyTransferStarted.participants") {
                let sp_tx = ConfirmKeyTransferStartedEvents::from_tx_events(events);
                return Ok((tx, Some(ExtraTxEventData::NewPoll(sp_tx.into()))));
            }
        }
        "/cosmos.gov.v1beta1.MsgVote" => {
            if events.contains_key("proposal_vote.option") {
                let sp_tx = NewProposalVoteEvent::from_tx_events(events);
                return Ok((tx, Some(ExtraTxEventData::NewProposalVote(sp_tx))));
            };
        }
        _other => {
            if events.contains_key("axelar.vote.v1beta1.Voted.state") {
                let sp_tx = PollVoteEvent::from_tx_events(events);
                return Ok((tx, Some(ExtraTxEventData::PollVote(sp_tx))));
            }
        } // m => { if m != "RefundMsgRequest" { dbg!(m); } }
    }
    Ok((tx, None))
}
