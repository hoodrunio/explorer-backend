use async_trait::async_trait;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize};

use crate::fetch::error::FetchError;

/// The struct that stores important URLs of a chain.
pub struct ChainUrls {
    /// The REST API URL of the chain.
    rest_api: &'static str,
    /// The RPC URL of the chain.
    rpc: &'static str,
}

/// The trait that provides methods to get common properties of chains.
pub trait Chain {
    /// Returns the name of the chain.
    fn name(&self) -> &'static str;
    /// Returns the `ChainUrls` of the chain.
    fn urls(&self) -> &ChainUrls;
    /// Returns Cosmos SDK version of the chain.
    fn sdk_version(&self) -> usize;
}

/// The trait that provides methods for common operation types.
#[async_trait]
pub trait ChainOperations
where
    Self: Chain + Sync,
{
    /// Makes an RPC request.
    async fn rpc_request<T>(&self, client: &Client, path: &str, query: &[(&'static str, String)]) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.urls().rpc, path);

        match client.get(url).query(&query).send().await {
            Ok(res) => {
                if res.status().is_success() {
                    match res.json::<RPCSuccessResponse<T>>().await {
                        Ok(res_json) => Ok(res_json.result),
                        Err(_) => Err("Cannot parse JSON response.".to_string()),
                    }
                } else {
                    match res.json::<RPCErrorResponse>().await {
                        Ok(latest_block) => Err(latest_block.error.data),
                        Err(_) => Err("Cannot parse JSON error response.".to_string()),
                    }
                }
            }
            Err(_) => Err("Unsuccessful request.".to_string()),
        }
    }
    /// Returns the block at given height. Returns the latest block, if no height is given.
    async fn get_block_by_height(&self, client: &Client, height: Option<usize>) -> Result<RPCSuccessResponse<Block>, String>;

    /// Returns the block with given hash.
    async fn get_block_by_hash(&self, client: &Client, hash: &str) -> Result<RPCSuccessResponse<Block>, String>;

    /// Returns transaction by given hash. Hash should start with `0x`.
    async fn get_tx_by_hash(&self, client: &Client, hash: &str) -> Result<Transaction, String>;
}

#[async_trait]
impl<T> ChainOperations for T
where
    T: Chain + Sync,
{
    async fn get_block_by_height(&self, client: &Client, height: Option<usize>) -> Result<RPCSuccessResponse<Block>, String> {
        let mut query = vec![];

        let height = height.and_then(|height| Some(height.to_string()));

        if let Some(height) = height {
            query.push(("height", height))
        }

        self.rpc_request(client, "/block", &query).await
    }

    async fn get_block_by_hash(&self, client: &Client, hash: &str) -> Result<RPCSuccessResponse<Block>, String> {
        let mut query = vec![];

        query.push(("hash", hash.to_string()));

        self.rpc_request(client, "/block_by_hash", &query).await
    }

    async fn get_tx_by_hash(&self, client: &Client, hash: &str) -> Result<Transaction, String> {
        let mut query = vec![];

        query.push(("hash", hash.to_string()));

        self.rpc_request(client, "/tx", &query).await
    }
}

#[derive(Deserialize)]
pub struct Transaction {
    /// HEX encoded TX hash, without leading `0x`. Eg: `"25EC6BCEA9B4A6835F5A38AB566959187F968C295EE71D015C3D907B25C5C72F"`
    hash: String,
    /// The block height TX at. Eg: `"6684890"`
    height: String,
    /// Unknown. Eg: `0`
    index: usize,
    /// The transaction result.
    tx_result: TransactionResult,
}

#[derive(Deserialize)]
pub struct TransactionResult {
    /// Unknown. Eg: `0`
    code: usize,
    /// Base64 encoded transaction data. Eg: `"CrgECh8vZXRoZXJtaW50LmV2bS52MS5Nc2dFdGhlcmV1bVR4EpQECkIweDgxNTRhOGEyYmViYzQyYzNhNmVlYTZjMTAwMDMwMzkwMzhkOTJiZGYxOWNiMmQ4NDBhYzJkN2Q2ZmI3YjBmMzISpwMKKjB4NEY0MWE5ZTJjYTc4YWQ2QjZlRmFiNTJGNjYxQjVmMEEwQzIxMUY3NRJCMHg4YzViZTFlNWViZWM3ZDViZDE0ZjcxNDI3ZDFlODRmM2RkMDMxNGMwZjdiMjI5MWU1YjIwMGFjOGM3YzNiOTI1EkIweDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDcyODYxOWNlZjE0MTEyZjFiNzc3ZTQ2ODAwYTkwNjc3ZDQ5OTI1NDQSQjB4MDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDY3ZWM4Nzg0NGZiZDczZWRhNGExMDU5ZjMwMDM5NTg0NTg2ZTA5ZBog//////////////////////////////////////////8g2oGYAypCMHg4MTU0YThhMmJlYmM0MmMzYTZlZWE2YzEwMDAzMDM5MDM4ZDkyYmRmMTljYjJkODQwYWMyZDdkNmZiN2IwZjMyOkIweDBhMjdkZDQyNDBkYzM1MjE1OWYxZTVhMzA3NjM0NDIwZmFjN2I2ZDg5YzYxYWI5NzIyNDI4MjIxZWFmYjg4NGYaIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABKKbqAg=="`
    data: String,
    /// JSON encoded transaction log. Eg: `"[{\"events\":[{\"type\":\"coin_received\",\"attributes\":[{\"key\":\"receiver\",\"value\":\"evmos1w2rpnnh3gyf0rdmhu35qp2gxwl2fjf2y4vjkhg\"},{\"key\":\"amount\",\"value\":\"199391000000000aevmos\"}]},{\"type\":\"coin_spent\",\"attributes\":[{\"key\":\"spender\",\"value\":\"evmos17xpfvakm2amg962yls6f84z3kell8c5ljcjw34\"},{\"key\":\"amount\",\"value\":\"199391000000000aevmos\"}]},{\"type\":\"ethereum_tx\",\"attributes\":[{\"key\":\"amount\",\"value\":\"0\"},{\"key\":\"ethereumTxHash\",\"value\":\"0x8154a8a2bebc42c3a6eea6c10003039038d92bdf19cb2d840ac2d7d6fb7b0f32\"},{\"key\":\"txIndex\",\"value\":\"0\"},{\"key\":\"txGasUsed\",\"value\":\"46374\"},{\"key\":\"txHash\",\"value\":\"25EC6BCEA9B4A6835F5A38AB566959187F968C295EE71D015C3D907B25C5C72F\"},{\"key\":\"recipient\",\"value\":\"0x4F41a9e2ca78ad6B6eFab52F661B5f0A0C211F75\"}]},{\"type\":\"message\",\"attributes\":[{\"key\":\"action\",\"value\":\"/ethermint.evm.v1.MsgEthereumTx\"},{\"key\":\"sender\",\"value\":\"evmos17xpfvakm2amg962yls6f84z3kell8c5ljcjw34\"},{\"key\":\"module\",\"value\":\"evm\"},{\"key\":\"sender\",\"value\":\"0x728619cEf14112F1B777E46800a90677d4992544\"},{\"key\":\"txType\",\"value\":\"2\"}]},{\"type\":\"transfer\",\"attributes\":[{\"key\":\"recipient\",\"value\":\"evmos1w2rpnnh3gyf0rdmhu35qp2gxwl2fjf2y4vjkhg\"},{\"key\":\"sender\",\"value\":\"evmos17xpfvakm2amg962yls6f84z3kell8c5ljcjw34\"},{\"key\":\"amount\",\"value\":\"199391000000000aevmos\"}]},{\"type\":\"tx_log\",\"attributes\":[{\"key\":\"txLog\",\"value\":\"{\\\"address\\\":\\\"0x4F41a9e2ca78ad6B6eFab52F661B5f0A0C211F75\\\",\\\"topics\\\":[\\\"0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925\\\",\\\"0x000000000000000000000000728619cef14112f1b777e46800a90677d4992544\\\",\\\"0x000000000000000000000000067ec87844fbd73eda4a1059f30039584586e09d\\\"],\\\"data\\\":\\\"//////////////////////////////////////////8=\\\",\\\"blockNumber\\\":6684890,\\\"transactionHash\\\":\\\"0x8154a8a2bebc42c3a6eea6c10003039038d92bdf19cb2d840ac2d7d6fb7b0f32\\\",\\\"transactionIndex\\\":0,\\\"blockHash\\\":\\\"0x0a27dd4240dc352159f1e5a307634420fac7b6d89c61ab9722428221eafb884f\\\",\\\"logIndex\\\":0}\"}]}]}]"`
    log: String,
    /// The transaction information. Eg: `""`
    info: String,
    /// Gas wanted. Eg: `"55648"`
    gas_wanted: String,
    /// Gas used. Eg: `"46374"`
    gas_used: String,
    /// Transaction events.
    events: Vec<TransactionEvent>,
    /// Transaction codespace. Eg: `""`
    codespace: String,
    // Base 64 encoded transaction. Eg: `"CqMDCu8CCh8vZXRoZXJtaW50LmV2bS52MS5Nc2dFdGhlcmV1bVR4EssCCoQCCh4vZXRoZXJtaW50LmV2bS52MS5EeW5hbWljRmVlVHgS4QEKBDkwMDEQKRoKMTUwMDAwMDAwMCILMjU1MDAwMDAwMDAo4LIDMioweDRGNDFhOWUyY2E3OGFkNkI2ZUZhYjUyRjY2MUI1ZjBBMEMyMTFGNzU6ATBCRAlep7MAAAAAAAAAAAAAAAAGfsh4RPvXPtpKEFnzADlYRYbgnf//////////////////////////////////////////UgEBWiD4q5dJAnhCoLGbgwyqtMO3GuL4kx1WmrtUyDr7hzaeYmIgL/+FNXRbiS+/RyH2p5dwQ0O8OOcFHGxUDg6AP3gPYxQaQjB4ODE1NGE4YTJiZWJjNDJjM2E2ZWVhNmMxMDAwMzAzOTAzOGQ5MmJkZjE5Y2IyZDg0MGFjMmQ3ZDZmYjdiMGYzMvo/LgosL2V0aGVybWludC5ldm0udjEuRXh0ZW5zaW9uT3B0aW9uc0V0aGVyZXVtVHgSIhIgChoKBmFldm1vcxIQMTQxOTAyNDAwMDAwMDAwMBDgsgM="`
    tx: String,
}

#[derive(Deserialize)]
pub struct TransactionEvent {
    /// Transaction event type. Eg: `"coin_spent"`
    r#type: String,
    /// Transaction event attributes.
    attributes: Vec<TransactionEventAttribute>,
}

#[derive(Deserialize)]
pub struct TransactionEventAttribute {
    /// Base 64 encoded transaction event attribute key. Eg: `"c3BlbmRlcg=="`
    key: String,
    /// Base 64 encoded transaction event attribute value. Eg: `"ZXZtb3MxdzJycG5uaDNneWYwcmRtaHUzNXFwMmd4d2wyZmpmMnk0dmpraGc="`
    value: String,
    /// Transaction event attribute index. Eg: `true`
    index: bool,
}

#[derive(Deserialize)]
pub struct Pagination {
    /// Pagination next key. Might be `None`. Eg: `"FGxWOxzuw4bZozVHta3qYgdKOuRC"`
    next_key: Option<String>,
    /// Total. Eg: `"0"`
    total: String,
}

#[derive(Deserialize)]
pub struct SlashingSigningInfo {
    info: Vec<SlashingSigningInfoItem>,
    pagination: Pagination,
}

#[derive(Deserialize)]
pub struct SlashingSigningInfoItem {
    /// Validator address. Eg: `"evmosvalcons1qx4hehfny66jfzymzn6d5t38m0ely3cvw6zn06"`
    address: String,
    /// The block height slashing is started at. Eg: `"0"`
    start_height: String,
    /// Unknown. Eg: `"5888077"`
    index_offset: String,
    /// The time jailed until. Eg: `"2022-05-14T04:31:49.705643236Z"`
    jailed_until: String,
    /// Tombstoned state. Eg: `false`
    tombstoned: bool,
    /// The count of missed blocks. Eg: `"16433"`
    missed_blocks_counter: String,
}

#[derive(Deserialize)]
pub struct RPCSuccessResponse<T> {
    jsonrpc: String,
    id: isize,
    result: T,
}

#[derive(Deserialize)]
pub struct RPCErrorResponse {
    jsonrpc: String,
    id: isize,
    error: RpcErrorResponseError,
}

#[derive(Deserialize)]
pub struct RpcErrorResponseError {
    /// The error code.
    code: isize,
    /// The message about error type.
    message: String,
    /// Description about error.
    data: String,
}

#[derive(Deserialize)]
pub struct Block {
    block_id: BlockId,
    block: BlockBlock,
}

#[derive(Deserialize)]
pub struct BlockId {
    /// HEX encoded transaction hash.
    hash: String,
    parts: BlockIdParts,
}

#[derive(Deserialize)]
pub struct BlockBlock {
    header: BlockHeader,
    data: BlockData,
    evidence: BlockEvidence,
    last_commit: BlockLastCommit,
}

#[derive(Deserialize)]
pub struct BlockIdParts {
    /// Unknown. Eg: `1`
    total: usize,
    /// HEX encoded transaction hash.
    hash: String,
}

#[derive(Deserialize)]
pub struct BlockHeader {
    version: BlockHeaderVersion,
    /// The ID of the chain. Eg: `"axelar-dojo-1"`
    chain_id: String,
    /// The current block height. Eg: `"4611328"`
    height: String,
    /// The current block time. Eg: `"2022-11-03T17:45:14.115240656Z"`
    time: String,
    last_block_id: BlockId,

    /// HEX encoded transaction hash.
    last_commit_hash: String,
    /// HEX encoded transaction hash.
    data_hash: String,
    /// HEX encoded transaction hash.
    validators_hash: String,
    /// HEX encoded transaction hash.
    next_validators_hash: String,
    /// HEX encoded transaction hash.
    consensus_hash: String,
    /// HEX encoded transaction hash.
    app_hash: String,
    /// HEX encoded transaction hash.
    last_results_hash: String,
    /// HEX encoded transaction hash.
    evidence_hash: String,
    /// HEX encoded address.
    proposer_address: String,
}

#[derive(Deserialize)]
pub struct BlockData {
    /// Array of very long Base64 encoded transactions. Eg: `["CoYBCoMBCiUvYXhlbGFyLmF4ZWxhcm5ldC52MWJldGExLkxpbmtSZXF1ZXN0EloKFAfFBMRZ8AeNGGkWVAcX+idm5UutEioweDM1NzkyNTRmNTgwNWQxNjZiNjhhNTg3MzIwNzA0NDQ4MjBmYTRiZjEaCGV0aGVyZXVtIgx3YnRjLXNhdG9zaGkSlQEKUQpGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQPUmMSQ2WoB0eD589u7pruIZt2gbHT2DO3QSIPX0z8WXBIECgIIARiuCBJACgsKBHVheGwSAzY3NRDh8AUiLWF4ZWxhcjFwdTJzd2MwbjB0cmZ0bGRoejU3cHlxa3c2ZDg3aGFobjdnNjk3YxpANmM1rQE1P3hbVtuFoaQEpGpnBnlygbotxEA0qR/rmAwVRB+acJ6idoF1V0Qul5eSCpi1Z0TLLwQEMya4nMdl3g=="]`
    txs: Vec<String>,
}

#[derive(Deserialize)]
pub struct BlockEvidence {
    // Property below is an unknown array. TODO!
    // evidence: Vec<UNKNOWN>
}
#[derive(Deserialize)]
pub struct BlockLastCommit {
    /// The block height of the latest commit. Eg: `"4611327"`
    height: String,
    /// Unknown. Eg: `0`
    round: usize,
    block_id: BlockId,
    /// Array of signatures.
    signatures: Vec<BlockLastCommitSignatures>,
}

#[derive(Deserialize)]
pub struct BlockHeaderVersion {
    /// Unknown. Eg: `"11"`
    block: String,
}

#[derive(Deserialize)]
pub struct BlockLastCommitSignatures {
    /// Unknown. Eg: `2`
    block_id_flag: usize,
    /// HEX encoded address of a validator.
    validator_address: String,
    /// The time of the unix timestamp. Eg: `"2022-11-03T17:45:14.193617481Z"`
    timestamp: String,
    /// Base 64 encoded signature. It might be `None`, so unsigned. Eg: `"rum2poquBDmHkGLGxjtjrlNBP5bV52m6ckexmNHdln85WRii4tCaVqAmxAKR+fP+hzoxEDuhOGwQ/xlgMVFrAA=="`
    signature: Option<String>,
}
