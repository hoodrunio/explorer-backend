use std::collections::HashMap;
use chrono::DateTime;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::database::{BlockForDb, ValidatorForDb};
use crate::utils::{Base64Convert, convert_tx_to_hex};
use crate::{chain::Chain, routes::OutRestResponse};

impl Chain {
    /// Returns the block at given height. Returns the latest block, if no height is given.
    pub async fn get_block_by_height(&self, height: Option<u64>) -> Result<OutRestResponse<InternalBlock>, String> {
        let mut query = vec![];

        let height = height.map(|height| height.to_string());

        if let Some(height) = height {
            query.push(("height", height))
        }

        let resp = self.rpc_request::<BlockResp>("/block", &query).await?;

        let block = InternalBlock::new(resp, self).await?;

        Ok(OutRestResponse::new(block, 0))
    }

    /// Returns the block with given hash.
    /// # Usage
    /// ```rs
    /// let block = chain.get_block_by_hash("14b6bb26cf30a559ae3ad18b0e3640bc3fd819b1182830d359969e02bab0f633").await;
    /// ```
    pub async fn get_block_by_hash(&self, hash: &str) -> Result<OutRestResponse<InternalBlock>, String> {
        let mut query = vec![];

        let hash = if hash.starts_with("0x") {
            hash.to_string()
        } else {
            format!("0x{}", hash)
        };

        query.push(("hash", hash));

        let resp = self.rpc_request::<BlockResp>("/block_by_hash", &query).await?;

        let block = InternalBlock::new(resp, self).await?;

        Ok(OutRestResponse::new(block, 0))
    }

    pub async fn get_latest_block(&self) -> Result<Block, String> {
        let latest_height = self.get_blockchain(None, None).await.unwrap().last_height;
        let mut query = vec![];
        query.push(("height", latest_height.to_string()));
        let latest_block = self.rpc_request::<BlockResp>("/block", &query).await?.block;
        Ok(latest_block)
    }

    pub async fn get_block_result_by_height(&self, height: Option<u64>) -> Result<OutRestResponse<InternalBlockResult>, String> {
        let mut query = vec![];

        let height = height.map(|height| height.to_string());

        if let Some(height) = height {
            query.push(("height", height))
        }

        let resp = self.rpc_request::<BlockResult>("/block_results", &query).await?;

        let block = InternalBlockResult::new(resp);

        Ok(OutRestResponse::new(block, 0))
    }

    /// Returns the block headers between `min_height` & `max_height`.
    async fn get_blockchain(&self, min_height: Option<u64>, max_height: Option<u64>) -> Result<InternalBlockchainResp, String> {
        let mut query = vec![];

        if let Some(min_height) = min_height {
            query.push(("minHeight", min_height.to_string()));
        };

        if let Some(max_height) = max_height {
            query.push(("maxHeight", max_height.to_string()));
        };

        match self.rpc_request::<BlockchainResp>("/blockchain", &query).await {
            Ok(resp) => {
                let last_height = match resp.last_height.parse() {
                    Ok(last_height) => last_height,
                    Err(_) => return Err("Blockchain parsing error.".to_string()),
                };

                let mut block_metas = vec![];

                for block_meta in resp.block_metas {
                    let block_size = match block_meta.block_size.parse() {
                        Ok(block_size) => block_size,
                        Err(_) => return Err("Blockchain parsing error.".to_string()),
                    };

                    let num_txs = match block_meta.num_txs.parse() {
                        Ok(num_txs) => num_txs,
                        Err(_) => return Err("Block tx count parsing error.".to_string()),
                    };

                    let height = match block_meta.header.height.parse() {
                        Ok(height) => height,
                        Err(_) => return Err("Block height parsing error.".to_string()),
                    };

                    let time = match chrono::DateTime::parse_from_rfc3339(&block_meta.header.time) {
                        Ok(time) => time.timestamp_millis() as u32,
                        Err(_) => return Err("Block time parsing error.".to_string()),
                    };

                    block_metas.push(InternalBlockMeta {
                        block_id: block_meta.block_id,
                        block_size,
                        header: InternalBlockHeader {
                            version: block_meta.header.version,
                            chain_id: block_meta.header.chain_id,
                            height,
                            time,
                            last_block_id: block_meta.header.last_block_id,
                            last_commit_hash: block_meta.header.last_commit_hash,
                            data_hash: block_meta.header.data_hash,
                            validators_hash: block_meta.header.validators_hash,
                            next_validators_hash: block_meta.header.next_validators_hash,
                            consensus_hash: block_meta.header.consensus_hash,
                            app_hash: block_meta.header.app_hash,
                            last_results_hash: block_meta.header.last_results_hash,
                            evidence_hash: block_meta.header.evidence_hash,
                            proposer_address: block_meta.header.proposer_address,
                        },
                        num_txs,
                    })
                }

                Ok(InternalBlockchainResp { last_height, block_metas })
            }
            Err(error) => Err(error),
        }
    }

    /// Returns the last 20 block headers.
    pub async fn get_block_headers_last_20(&self) -> Result<InternalBlockchainResp, String> {
        self.get_blockchain(None, None).await
    }

    /// Returns the block headers between max and min height.
    pub async fn get_block_headers(&self, min_height: u64, max_height: u64) -> Result<InternalBlockchainResp, String> {
        self.get_blockchain(Some(min_height), Some(max_height)).await
    }

    /// Returns the block headers between max and min height.
    pub async fn get_last_count_block(&self, count: u64) -> Result<Vec<BlockForDb>, String> {
        self.database.find_last_count_blocks(count).await
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockItem {
    pub proposer_address: String,
    pub proposer_name: String,
    pub proposer_logo_url: String,
    pub height: u64,
    pub hash: String,
    pub tx_count: u64,
    pub timestamp: i64,
}

#[derive(Serialize, Debug)]
pub struct InternalBlock {
    pub height: u64,
    pub hash: String,
    pub proposer_name: String,
    pub proposer_logo_url: String,
    pub proposer_address: String,
    pub time: i64,
    pub txs: Vec<String>,
    pub tx_count: u32,
    pub signatures: Vec<InternalBlockSignature>,
}

impl InternalBlock {
    async fn new(block_resp: BlockResp, chain: &Chain) -> Result<Self, String> {
        let mut proposer = None;

        let mut signatures = vec![];

        for signature in block_resp.block.last_commit.signatures {
            //TODO there is no certainty to get validator from db check if db does not have find a way to get info from smw.
            if let Ok(validator_metadata) = chain.database.find_validator_by_hex_addr(&signature.validator_address).await {
                if block_resp.block.header.proposer_address == signature.validator_address {
                    proposer = Some(validator_metadata.clone());
                    signatures.push(validator_metadata.into())
                } else {
                    signatures.push(validator_metadata.into())
                }
            } else {
                //Else needed because of if let pattern, what is gonna happen if let not OK? Kinda infinite loop
            }
        }

        let proposer = proposer.ok_or_else(|| "Proposer is not found found in the database.".to_string())?;

        let mut txs: Vec<String> = vec![];

        for tx_base64 in block_resp.block.data.txs {
            if let Some(tx_hex) = convert_tx_to_hex(&tx_base64) {
                txs.push(tx_hex.clone());
            } else {
                println!("Could not convert tx base to tx hex {:?} ", tx_base64);
            }
        }

        Ok(Self {
            height: block_resp
                .block
                .header
                .height
                .parse::<u64>()
                .map_err(|_| format!("Cannot parse block height, '{}'.", block_resp.block.header.height))?,
            hash: block_resp.block_id.hash,
            proposer_name: proposer.name,
            proposer_address: proposer.operator_address,
            proposer_logo_url: proposer.logo_url,
            time: DateTime::parse_from_rfc3339(&block_resp.block.header.time)
                .map_err(|_| format!("Cannot parse block datetime, '{}'.", block_resp.block.header.time))?
                .timestamp_millis(),
            tx_count: txs.len() as u32,
            txs,
            signatures,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalBlockSignature {
    /// Validator name. `heisenbug`
    pub name: String,
    /// Validator logo URL. `example.com`
    pub logo_url: String,
    /// Validator valoper prefixed address. `cosmosvaloper156gqf9837u7d4c4678yt3rl4ls9c5vuursrrzf`
    pub address: String,
}

impl From<ValidatorForDb> for InternalBlockSignature {
    fn from(validator_for_db: ValidatorForDb) -> Self {
        Self {
            name: validator_for_db.name,
            logo_url: validator_for_db.logo_url,
            address: validator_for_db.operator_address,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalBlockchainResp {
    /// Last block height. `12733014`
    pub last_height: u64,
    /// Array of internal block metas.
    pub block_metas: Vec<InternalBlockMeta>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalBlockMeta {
    /// Block ID.
    pub block_id: BlockId,
    /// Block size. Eg: `13971`
    pub block_size: u64,
    /// Block header.
    pub header: InternalBlockHeader,
    /// Number of transactions. Eg: `3`
    pub num_txs: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BlockchainResp {
    /// Last block height. `"12733014"`
    pub last_height: String,
    /// Array of block metas.
    pub block_metas: Vec<BlockMeta>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BlockMeta {
    /// Last block height. `"12733014"`
    pub block_id: BlockId,
    /// Block size. Eg: `"13971"`
    pub block_size: String,
    /// Block header.
    pub header: BlockHeader,
    /// Number of transactions. Eg: `"3"`
    pub num_txs: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BlockResp {
    pub block_id: BlockId,
    pub block: Block,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InternalBlockResult {
    pub height: String,
    pub txs_results: Vec<InternalBlockResultTxsResult>,
    pub begin_block_events: Vec<ResultBlockEvent>,
    pub end_block_events: Vec<ResultBlockEvent>,
}

impl InternalBlockResult {
    fn new(block_result: BlockResult) -> Self {
        let txs_results = block_result.txs_results.clone().unwrap_or(vec![]);
        Self {
            height: block_result.height.clone(),
            txs_results: txs_results.into_iter().map(|res| { InternalBlockResultTxsResult::new(res) }).collect(),
            begin_block_events: block_result.begin_block_events.clone().unwrap_or(vec![]),
            end_block_events: block_result.end_block_events.clone().unwrap_or(vec![]),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InternalBlockResultTxsResult {
    pub code: i64,
    pub data: String,
    pub log: String,
    pub info: String,
    pub gas_wanted: String,
    pub gas_used: String,
    pub events: Vec<ResultBlockEvent>,
    pub codespace: String,
}

impl InternalBlockResultTxsResult {
    fn new(block_result_txs_result: BlockResultTxResult) -> Self {
        Self {
            code: block_result_txs_result.code.clone(),
            data: block_result_txs_result.data.clone(),
            log: block_result_txs_result.log.clone(),
            info: block_result_txs_result.info.clone(),
            gas_wanted: block_result_txs_result.gas_wanted.clone(),
            gas_used: block_result_txs_result.gas_used.clone(),
            events: block_result_txs_result.events.clone(),
            codespace: block_result_txs_result.codespace.clone(),
        }
    }

    pub fn get_sender_address(&self) -> Option<String> {
        for res_block_event in self.events.clone() {
            match res_block_event.attributes.into_iter().find(|attr_item| { attr_item.key == "sender" }) {
                None => {}
                Some(item) => {
                    return Some(item.value.clone());
                }
            }
        };

        None
    }
}


#[derive(Deserialize, Serialize, Debug)]
pub struct BlockResult {
    pub height: String,
    pub txs_results: Option<Vec<BlockResultTxResult>>,
    pub begin_block_events: Option<Vec<ResultBlockEvent>>,
    pub end_block_events: Option<Vec<ResultBlockEvent>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockId {
    /// HEX encoded hash.
    pub hash: String,
    pub parts: BlockIdParts,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub data: BlockData,
    pub last_commit: BlockLastCommit,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResultEndBlock {
    pub events: Vec<ResultBlockEvent>,
    pub consensus_param_updates: HashMap<String, Value>,
    // pub validator_updates: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResultBeginBlock {
    pub events: Vec<ResultBlockEvent>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResultBlockEvent {
    pub attributes: Vec<ResultBlockEventAttribute>,
    pub r#type: String,
}

impl ResultBlockEvent {
    pub fn is_heartbeat_event(&self) -> bool {
        self.r#type == "heartbeat"
    }
}

impl ResultEndBlock {
    pub fn is_heartbeat_begin(&self) -> bool {
        for event in &self.events {
            let res = event.is_heartbeat_event();
            if res {
                return res;
            }
        };

        false
    }
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResultBlockEventAttribute {
    #[serde(deserialize_with = "from_base64")]
    pub key: String,
    #[serde(deserialize_with = "from_base64")]
    pub value: String,
    pub index: bool,
}

pub fn from_base64<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    Ok(String::base64_to_string(&String::from(s)))
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockIdParts {
    /// Unknown. Eg: `1`
    pub total: u32,
    /// HEX encoded transaction hash.
    pub hash: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalBlockHeader {
    /// Block header version.
    pub version: BlockHeaderVersion,
    /// The ID of the chain. Eg: `"axelar-dojo-1"`
    pub chain_id: String,
    /// The current block height. Eg: `4611328`
    pub height: u64,
    /// The current block timestamp. Eg: `12344654`
    pub time: u32,
    /// Last block ID.
    pub last_block_id: BlockId,
    /// HEX encoded last commit hash. Eg: `"9AB6C12C713C21A0AFB95D86443370EA7B0DC383685888B7A724824DFD3F8CB5"`
    pub last_commit_hash: String,
    /// HEX encoded data hash. Eg: `"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"`
    pub data_hash: String,
    /// HEX encoded validators hash. Eg: `"0CF77EC655E0FBB4D4D639320955CB1E7921B43709C85254F57005EF6C7BA66A"`
    pub validators_hash: String,
    /// HEX encoded next validators hash. Eg: `"0CF77EC655E0FBB4D4D639320955CB1E7921B43709C85254F57005EF6C7BA66A"`
    pub next_validators_hash: String,
    /// HEX encoded consensus hash. Eg: `"048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F"`
    pub consensus_hash: String,
    /// HEX encoded app hash. Eg: `"75EDB053429518CA3AF1026A7E00CEA8597588CB2D361A078E8B6969C216D74F"`
    pub app_hash: String,
    /// HEX encoded results hash. Eg: `"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"`
    pub last_results_hash: String,
    /// HEX encoded evidence hash. Eg: `"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"`
    pub evidence_hash: String,
    /// HEX encoded address. Eg: `"FF33E637849DF84209F802494C70B4B193E9C644"`
    pub proposer_address: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockHeader {
    /// Block header version.
    pub version: BlockHeaderVersion,
    /// The ID of the chain. Eg: `"axelar-dojo-1"`
    pub chain_id: String,
    /// The current block height. Eg: `"4611328"`
    pub height: String,
    /// The current block time. Eg: `"2022-11-03T17:45:14.115240656Z"`
    pub time: String,
    /// Last block ID.
    pub last_block_id: BlockId,
    /// HEX encoded transaction hash.
    pub last_commit_hash: String,
    /// HEX encoded transaction hash.
    pub data_hash: String,
    /// HEX encoded transaction hash.
    pub validators_hash: String,
    /// HEX encoded transaction hash.
    pub next_validators_hash: String,
    /// HEX encoded transaction hash.
    pub consensus_hash: String,
    /// HEX encoded transaction hash.
    pub app_hash: String,
    /// HEX encoded transaction hash.
    pub last_results_hash: String,
    /// HEX encoded transaction hash.
    pub evidence_hash: String,
    /// HEX encoded address.
    pub proposer_address: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockData {
    /// Array of very long Base64 encoded transactions. Eg: `["CoYBCoMBCiUvYXhlbGFyLmF4ZWxhcm5ldC52MWJldGExLkxpbmtSZXF1ZXN0EloKFAfFBMRZ8AeNGGkWVAcX+idm5UutEioweDM1NzkyNTRmNTgwNWQxNjZiNjhhNTg3MzIwNzA0NDQ4MjBmYTRiZjEaCGV0aGVyZXVtIgx3YnRjLXNhdG9zaGkSlQEKUQpGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQPUmMSQ2WoB0eD589u7pruIZt2gbHT2DO3QSIPX0z8WXBIECgIIARiuCBJACgsKBHVheGwSAzY3NRDh8AUiLWF4ZWxhcjFwdTJzd2MwbjB0cmZ0bGRoejU3cHlxa3c2ZDg3aGFobjdnNjk3YxpANmM1rQE1P3hbVtuFoaQEpGpnBnlygbotxEA0qR/rmAwVRB+acJ6idoF1V0Qul5eSCpi1Z0TLLwQEMya4nMdl3g=="]`
    pub txs: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockHeaderVersion {
    /// Unknown. Eg: `"11"`
    pub block: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BlockLastCommitSignatures {
    /// Unknown. Eg: `2`
    pub block_id_flag: usize,
    /// HEX encoded address of a validator. Eg: `"E42125451E65AC3931726936026F295677DB5D07"`
    pub validator_address: String,
    /// The time of the unix timestamp. Eg: `"2022-11-03T17:45:14.193617481Z"`
    pub timestamp: String,
    /// Base 64 encoded signature. It might be `None`, so unsigned. Eg: `"rum2poquBDmHkGLGxjtjrlNBP5bV52m6ckexmNHdln85WRii4tCaVqAmxAKR+fP+hzoxEDuhOGwQ/xlgMVFrAA=="`
    pub signature: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockLastCommit {
    /// The block height of the latest commit. Eg: `"4611327"`
    pub height: String,
    /// Unknown. Eg: `0`
    pub round: usize,
    /// Block ID.
    pub block_id: BlockId,
    /// Array of signatures.
    pub signatures: Vec<BlockLastCommitSignatures>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockResultTxResult {
    pub code: i64,
    pub data: String,
    pub log: String,
    pub info: String,
    pub gas_wanted: String,
    pub gas_used: String,
    pub events: Vec<ResultBlockEvent>,
    pub codespace: String,
}