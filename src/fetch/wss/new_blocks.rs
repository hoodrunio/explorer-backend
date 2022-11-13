use crate::{
    chain::Chain,
    data::latest_blocks::BlockItem,
    fetch::rest::{
        blocks::Block,
        requests::{RPCResponse, RPCSuccessResponse},
        validators::ValidatorListValidator,
    },
    utils::get_validator_logo,
};
use serde::{Deserialize, Serialize};
use tungstenite::{connect, Message};

use super::others::{Event, SocketResponse, SubscribeResult};

pub type NewBlockResponse = RPCSuccessResponse<SubscribeResult<NewBlock>>;

impl Chain {
    /// Subscribes to new blocks.
    pub async fn subscribe_new_blocks(&self) {
        match connect(self.wss_url) {
            Ok((mut socket, _)) => {
                if let Ok(()) = socket.write_message(Message::Text(
                    r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='NewBlock'"], "id": 2 }"#.into(),
                )) {
                    loop {
                        if let Ok(Message::Text(msg)) = socket.read_message() {
                            match serde_json::from_str::<NewBlockResponse>(&msg) {
                                Ok(resp) => {
                                    if let Some(resp) = resp.result.data {
                                        if let Ok(proposer) = self
                                            .get_validator(&resp.value.block.header.proposer_address)
                                            .await
                                        {
                                            let logo =
                                                get_validator_logo(self.client.clone(), &proposer.description.identity).await;

                                            let new_block = (|value: NewBlock, proposer: ValidatorListValidator| {
                                                Some(BlockItem {
                                                    proposer_name: proposer.description.moniker,
                                                    proposer_logo: logo.ok()?,
                                                    height: value.block.header.height.parse::<u64>().ok()?,
                                                    hash: value.block.header.data_hash,
                                                    tx_count: 0, // TODO.
                                                    timestamp: chrono::DateTime::parse_from_rfc3339(&value.block.header.time)
                                                        .ok()?
                                                        .timestamp_millis()
                                                        as u32,
                                                })
                                            })(
                                                resp.value, proposer
                                            );

                                            self.update_latest_block(new_block);
                                        }
                                    }
                                }
                                Err(error) => println!("{} msg: {}", error, msg),
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
    pub validator_updates: Vec<u8>,
    /// Consensus param updates.
    pub consensus_param_updates: ConsensusParamUpdates,
    /// Array of events.
    pub events: Vec<u8>,
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
