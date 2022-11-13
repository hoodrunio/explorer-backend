use std::fs::write;

use crate::{chain::Chain, data::latest_txs::TransactionItem, fetch::rest::requests::RPCSuccessResponse};
use serde::{Deserialize, Serialize};
use tungstenite::{connect, Message};

use super::others::{Event, SocketResponse, SubscribeData};

pub type TxSocketResp = RPCSuccessResponse<RespResult>;

impl Chain {
    /// Subscribes to new blocks.
    pub async fn subscribe_tx(&self) {
        match connect(self.wss_url) {
            Ok((mut socket, _)) => {
                if let Ok(()) = socket.write_message(Message::Text(
                    r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='Tx'"], "id": 1 }"#.into(),
                )) {
                    loop {
                        if let Ok(Message::Text(msg)) = socket.read_message() {
                            match serde_json::from_str::<TxSocketResp>(&msg) {
                                Ok(resp) => {
                                    let new_tx = (|resp: TxSocketResp| {
                                        let fee_str = resp.result.events.tx_fee.get(0)?;

                                        let fee = if fee_str.len() > self.main_denom.len() {
                                            fee_str[..fee_str.len() - self.main_denom.len()].parse::<f64>().ok()?
                                        } else {
                                            0.0
                                        };

                                        Some(TransactionItem {
                                            height: resp.result.data.value.tx_result.height.parse::<u64>().ok()?,
                                            r#type: "todo".to_string(),
                                            hash: resp.result.events.tx_hash.get(0)?.to_string(),
                                            result: "todo".to_string(),
                                            timestamp: 0, // TODO
                                            fee,
                                        })
                                    })(resp);

                                    self.update_latest_txs(new_tx);
                                }
                                Err(error) => {
                                    write("hex.log", format!("{}\n{}", error, msg));
                                }
                            }
                        }
                    }
                }
            }
            Err(error) => {
                println!("Websocket error: {}", error)
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RespResult {
    data: SubscribeData<Tx>,
    events: RespResultEvents,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RespResultEvents {
    #[serde(rename = "tx.hash")]
    tx_hash: [String; 1],
    #[serde(rename = "tx.fee")]
    tx_fee: [String; 1],
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Tx {
    #[serde(rename = "TxResult")]
    pub tx_result: TxResult,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxResult {
    /// Block height. Eg: `"4752701"`
    pub height: String,
    /// Index. Might be none. Eg: `50`
    pub index: Option<u64>,
    /// Tx. Eg: `"CqYGCqMGCicvYXhlbGFyLnJld2FyZC52MWJldGExLlJlZnVuZE1zZ1JlcXVlc3QS9wUKFCp5GhaH0InznAHfEuJjnZ1EnHD7Et4FCiQvYXhlbGFyLnRzcy52MWJldGExLkhlYXJ0QmVhdFJlcXVlc3QStQUKFCp5GhaH0InznAHfEuJjnZ1EnHD7EhhhdXJvcmEtc2Vjb25kYXJ5LWdlbmVzaXMSFWV2bS1hdmFsYW5jaGUtNDY3ODk4OBIVZXZtLWF2YWxhbmNoZS00NjA0NzY4EhVldm0tYXZhbGFuY2hlLTQ1NzQyNTUSFWV2bS1hdmFsYW5jaGUtNDUxNDgwNhIVZXZtLWF2YWxhbmNoZS00NDYwNTUyEhNldm0tYmluYW5jZS00Njc4OTg4EhNldm0tYmluYW5jZS00NjA0NzY4EhNldm0tYmluYW5jZS00NTc0MjU1EhNldm0tYmluYW5jZS00NTE0ODA2EhNldm0tYmluYW5jZS00NDYwNTUyEhRldm0tZXRoZXJldW0tNDY3ODk4ORIUZXZtLWV0aGVyZXVtLTQ2MDQ3NjkSFGV2bS1ldGhlcmV1bS00NTc0MzA0EhRldm0tZXRoZXJldW0tNDUxNDgwNhIUZXZtLWV0aGVyZXVtLTQ0NjA1NTISEmV2bS1mYW50b20tNDY3ODk4ORISZXZtLWZhbnRvbS00NjA0NzY5EhJldm0tZmFudG9tLTQ1NzQyNTYSEmV2bS1mYW50b20tNDUxNDgwNhISZXZtLWZhbnRvbS00NDYwNTUyEhRldm0tbW9vbmJlYW0tNDY3ODk4ORIUZXZtLW1vb25iZWFtLTQ2MDQ3NjkSFGV2bS1tb29uYmVhbS00NTc0MjU2EhRldm0tbW9vbmJlYW0tNDUxNDgwNhIUZXZtLW1vb25iZWFtLTQ0NjA1NTMSE2V2bS1wb2x5Z29uLTQ2Nzg5ODkSE2V2bS1wb2x5Z29uLTQ2MDQ3NjkSE2V2bS1wb2x5Z29uLTQ1NzQyNTYSE2V2bS1wb2x5Z29uLTQ1MTQ4MDcSE2V2bS1wb2x5Z29uLTQ0NjA1NTMSagpSCkYKHy9jb3Ntb3MuY3J5cHRvLnNlY3AyNTZrMS5QdWJLZXkSIwohAq"`
    pub tx: String,
    /// Result.
    pub result: TxResultResult,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxResultResult {
    /// Tx data. Might be `None`. Eg: `"CikKJy9heGVsYXIucmV3YXJkLnYxYmV0YTEuUmVmdW5kTXNnUmVxdWVzdA"`
    pub data: Option<String>,
    /// JSON encoded Tx log. Eg: `"[{\"events\":[{\"type\":\"coin_received\",\"attributes\":[{\"key\":\"receiver\",\"value\":\"axelar19fu359586zyl88qpmufwycuan4zfcu8m55n577\"},{\"key\":\"amount\",\"value\":\"26745uaxl\"}]},{\"type\":\"coin_spent\",\"attributes\":[{\"key\":\"spender\",\"value\":\"axelar17xpfvakm2amg962yls6f84z3kell8c5l5h4gqu\"},{\"key\":\"amount\",\"value\":\"26745uaxl\"}]},{\"type\":\"message\",\"attributes\":[{\"key\":\"action\",\"value\":\"RefundMsgRequest\"},{\"key\":\"sender\",\"value\":\"axelar17xpfvakm2amg962yls6f84z3kell8c5l5h4gqu\"}]},{\"type\":\"transfer\",\"attributes\":[{\"key\":\"recipient\",\"value\":\"axelar19fu359586zyl88qpmufwycuan4zfcu8m55n577\"},{\"key\":\"sender\",\"value\":\"axelar17xpfvakm2amg962yls6f84z3kell8c5l5h4gqu\"},{\"key\":\"amount\",\"value\":\"26745uaxl\"}]}]}]"`
    pub log: String,
    /// Gas wanted. Eg: `"3820608"`
    pub gas_wanted: String,
    /// Gas used. Eg: `"953199"`
    pub gas_used: String,
    /// Tx events.
    pub events: Vec<Event>,
}
