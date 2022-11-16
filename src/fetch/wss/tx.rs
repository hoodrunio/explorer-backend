use std::fs::write;

use crate::{
    chain::Chain,
    data::latest_txs::TransactionItem,
    fetch::rest::requests::{RPCResponse, RPCSuccessResponse},
};
use serde::{Deserialize, Serialize};
use strum_macros::IntoStaticStr;
use tungstenite::{connect, Message};

use super::others::{Event, SocketResponse, SubscribeData};

pub type TxSocketResp = RPCSuccessResponse<TxSocketResult>;

impl Chain {
    /// Subscribes to new blocks.
    pub async fn subscribe_to_tx(&self) {
        // We make a connection to Web Socket endpoint of the chain.
        // Then we send the message and start listening incoming messages.
        // We store a reference to the previous response.
        // Because the hash of a block is given on the next response.

        // Make a new connection.
        let connection = connect(self.wss_url);

        // Match the connection.
        match connection {
            Ok((mut socket, _)) => {
                // Create the message to be sent.
                let msg = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='Tx'"], "id": 1 }"#;

                // Write the message via socket.
                if socket.write_message(msg.into()).is_ok() {
                    // Start the loop
                    loop {
                        // Read incoming messages.
                        if let Ok(Message::Text(msg)) = socket.read_message() {
                            type Response = RPCResponse<TxSocketResult>;

                            // Parse JSON.
                            match serde_json::from_str::<Response>(&msg) {
                                Ok(RPCResponse::Success(resp)) => {
                                    if let (Some(data), Some(events)) = (resp.result.data, resp.result.events) {
                                        // Add the block from the old response.

                                        let tx = data.value.tx_result;
                                        println!("tx {}", self.name);

                                        let r#type: String = tx.result.events.get(0).and_then(|e| Some(e.into())).unwrap_or("Unknown").to_string();

                                        self.update_latest_txs(
                                            async move {
                                                Some(TransactionItem {
                                                    height: tx.height.parse().ok()?,
                                                    r#type,
                                                    hash: events.tx_hash.get(0)?.to_string(),
                                                    result: "".into(), // TODO
                                                    timestamp: 0,      // TODO
                                                    fee: events.tx_fee.get(0).and_then(|fee| {
                                                        if fee.len() > self.main_denom.len() {
                                                            Some(fee[..fee.len() - 1 - self.main_denom.len()].parse().ok()?)
                                                        } else {
                                                            None
                                                        }
                                                    })?,
                                                })
                                            }
                                            .await,
                                        );
                                    };
                                }
                                Ok(RPCResponse::Error(resp_err)) => eprintln!("{}", resp_err.error.data),
                                Err(ser_error) => eprintln!("{ser_error}"),
                            }
                        }
                    }
                }
            }
            Err(_) => {
                eprintln!("Couldn't connect to {}", self.wss_url);
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TxSocketResult {
    data: Option<SubscribeData<Tx>>,
    events: Option<RespResultEvents>,
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
