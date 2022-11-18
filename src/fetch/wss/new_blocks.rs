use super::others::Event;
use crate::{
    chain::Chain,
    data::latest_blocks::BlockItem,
    fetch::rest::{blocks::Block, others::PublicKey, requests::RPCSuccessResponse},
    utils::get_validator_logo,
};
use bech32::ToBase32;
use chrono::DateTime;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};

impl Chain {
    /// Subscribes to specified subscription method.
    pub async fn subscribe_to_new_blocks(&self) {
        // We make a connection to Web Socket endpoint of the chain.
        // Then we send the message and start listening incoming messages.
        // We store a reference to the previous response.
        // Because the hash of a block is given on the next response.

        // Create the message to be sent.
        let msg_to_send = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='NewBlock'"], "id": 2 }"#;

        let url = self.inner.wss_url;

        let (ws_stream, _response) = connect_async(url).await.expect(&format!("Failed to connect to {}", url));

        let (mut write, read) = ws_stream.split();

        // Write the message via socket.
        // Match the connection.
        match write.send(msg_to_send.into()).await {
            Ok(()) => {
                // Read incoming messages.
                read.for_each(|message| async {
                    let mut old_resp: Option<NewBlock> = None;

                    match message {
                        // Handle message.
                        Ok(Message::Text(message)) => {
                            // Create return type.
                            type Response = RPCSuccessResponse<NewBlocksSocketResult>;

                            // Parse JSON.
                            match serde_json::from_str::<Response>(&message) {
                                Ok(resp) => {
                                    if let Some(data) = resp.result.data {
                                        match old_resp {
                                            Some(old_block) => {
                                                let new_block = data.value;

                                                let hash = &new_block.block.header.last_block_id.hash;

                                                // Add the block from the old response.
                                                match async move {
                                                    let proposer_address_b32 = bech32::encode(
                                                        "bech32",
                                                        &old_block.block.header.proposer_address.to_base32(),
                                                        bech32::Variant::Bech32,
                                                    )
                                                    .or_else(|_| {
                                                        Err(format!(
                                                            "Cannot convert HEX proposer address to bech32, '{}'.",
                                                            &old_block.block.header.proposer_address
                                                        ))
                                                    })?;

                                                    let validator_addr = self.valoper_addr(&proposer_address_b32);

                                                    // Get validator description.
                                                    let validator_description = self.get_validator(&validator_addr).await?.description;

                                                    // Get validator logo.
                                                    let proposer_logo =
                                                        get_validator_logo(self.inner.client.clone(), &validator_description.identity).await;

                                                    Ok::<BlockItem, String>(BlockItem {
                                                        proposer_name: validator_description.moniker,
                                                        proposer_logo,
                                                        height: old_block.block.header.height.parse().or_else(|_| {
                                                            Err(format!("Cannot parse block height, '{}'.", old_block.block.header.height))
                                                        })?,
                                                        hash: hash.to_string(),
                                                        tx_count: 0,
                                                        timestamp: DateTime::parse_from_rfc3339(&old_block.block.header.time)
                                                            .or_else(|_| {
                                                                Err(format!("Cannot parse block datetime, '{}'.", old_block.block.header.time))
                                                            })?
                                                            .timestamp_millis(),
                                                    })
                                                }
                                                .await
                                                {
                                                    Ok(block_item) => self.update_latest_block(block_item),
                                                    Err(error) => eprintln!("{}", error),
                                                }
                                                old_resp = Some(new_block);
                                            }
                                            None => old_resp = Some(data.value),
                                        };
                                    };
                                }

                                Err(parse_error) => eprintln!("WS-PARSE-ERROR(src = {}): {}", self.inner.wss_url, parse_error),
                            }
                        }
                        // Leave the messages not text.
                        Ok(_) => (),
                        // Print the error message.
                        Err(read_error) => eprintln!("WS-READING-ERROR(src = {}): {}", self.inner.wss_url, read_error),
                    }
                })
                .await;
            }
            Err(send_error) => eprintln!("WS-SENDING-ERROR(src = {}): {}", self.inner.wss_url, send_error),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NewBlocksSocketResult {
    /// Data.
    pub data: Option<NewBlocksSocketData>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NewBlocksSocketData {
    /// Value.
    pub value: NewBlock,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NewBlock {
    /// Block.
    pub block: Block,
    /// Result end block.
    pub result_begin_block: ResultBeginBlock,
    /// Result end block.
    pub result_end_block: ResultEndBlock,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResultBeginBlock {
    /// Array of events.
    pub events: Vec<Event>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResultEndBlock {
    /// Validator updates.
    pub validator_updates: Vec<ValidatorUpdate>,
    /// Consensus param updates.
    pub consensus_param_updates: ConsensusParamUpdates,
    /// Array of events.
    pub events: Vec<Event>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorUpdate {
    /// Public key.
    pub_key: PublicKeySum,

    /// Validator power. Eg: `"26544215"`
    power: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PublicKeySum {
    /// Public key sum.
    #[serde(rename = "Sum")]
    sum: PublicKey,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UnparsedEventAttribute {
    /// Unparsed event attribute key. Eg: `"cmVjaXBpZW50"`
    pub key: String,
    /// Unparsed event attribute key. Might be `None`. Eg: `"ZXZtb3MxN3hwZnZha20yYW1nOTYyeWxzNmY4NHoza2VsbDhjNWxqY2p3MzQ"`
    pub value: String,
    /// Unparsed event attribute index. Might be `None`. Eg: `true`
    pub index: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConsensusParamUpdates {
    pub block: ParamUpdatesBlock,
    pub evidence: ConsensusParamEvidence,
    pub validator: ConsensusParamValidator,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ParamUpdatesBlock {
    /// Maximum bytes. Eg: `"22020096"`
    pub max_bytes: String,
    /// Maximum gas. Eg: `"-1"`
    pub max_gas: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConsensusParamEvidence {
    /// Unknown. Eg: `"100000"`,
    pub max_age_num_blocks: String,
    /// Maximum age duration. Eg: `"172800000000000"`,
    pub max_age_duration: String,
    /// Maximum bytes. Eg: `"1048576s"`
    pub max_bytes: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConsensusParamValidator {
    /// Array of public key types. Eg: `["ed25519"]`
    pub pub_key_types: Vec<String>,
}
