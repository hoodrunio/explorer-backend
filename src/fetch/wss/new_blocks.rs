use crate::{
    chain::Chain,
    data::latest_blocks::BlockItem,
    fetch::rest::{
        blocks::Block,
        others::PublicKey,
        requests::{RPCResponse, RPCSuccessResponse},
    },
    utils::get_validator_logo,
};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use tungstenite::{connect, Message};

use super::others::Event;

impl Chain {
    /// Subscribes to specified subscription method.
    pub async fn subscribe_to_new_blocks(&self) {
        // We make a connection to Web Socket endpoint of the chain.
        // Then we send the message and start listening incoming messages.
        // We store a reference to the previous response.
        // Because the hash of a block is given on the next response.

        // Make a new connection.
        let connection = connect(self.inner.wss_url);

        // Match the connection.
        match connection {
            Ok((mut socket, _)) => {
                // Create the message to be sent.
                let msg = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='NewBlock'"], "id": 2 }"#;

                // Write the message via socket.
                if socket.write_message(msg.into()).is_ok() {
                    let mut old_resp: Option<NewBlock> = None;

                    // Start the loop
                    loop {
                        // Read incoming messages.
                        if let Ok(Message::Text(msg)) = socket.read_message() {
                            type Response = RPCSuccessResponse<NewBlocksSocketResult>;

                            // Parse JSON.
                            match serde_json::from_str::<Response>(&msg) {
                                Ok(resp) => {
                                    if let Some(data) = resp.result.data {
                                        println!("block {}", self.inner.name);
                                        match old_resp {
                                            Some(old_block) => {
                                                let new_block = data.value;

                                                let hash = &new_block.block.header.last_block_id.hash;

                                                // Add the block from the old response.
                                                self.update_latest_block(
                                                    async move {
                                                        // Get validator description.
                                                        let validator_description =
                                                            self.get_validator(&old_block.block.header.proposer_address).await.ok()?.description;

                                                        // Get validator logo.
                                                        let proposer_logo =
                                                            get_validator_logo(self.inner.client.clone(), &validator_description.identity).await;

                                                        Some(BlockItem {
                                                            proposer_name: validator_description.moniker,
                                                            proposer_logo,
                                                            height: old_block.block.header.height.parse().ok()?,
                                                            hash: hash.to_string(),
                                                            tx_count: 0,
                                                            timestamp: DateTime::parse_from_rfc3339(&old_block.block.header.time)
                                                                .ok()?
                                                                .timestamp_millis()
                                                                as u32,
                                                        })
                                                    }
                                                    .await,
                                                );
                                                old_resp = Some(new_block);
                                            }
                                            None => old_resp = Some(data.value),
                                        };
                                    };
                                }

                                Err(ser_error) => eprintln!("{ser_error}"),
                            }
                        }
                    }
                }
            }
            Err(_) => {
                eprintln!("Couldn't connect to {}", self.inner.wss_url);
            }
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
