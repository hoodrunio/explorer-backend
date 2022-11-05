use async_trait::async_trait;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize};
/// The struct that stores important URLs of a chain.
pub struct ChainUrls {
    /// The REST API URL of the chain.
    pub rest_api: &'static str,
    /// The RPC URL of the chain.
    pub rpc: &'static str,
}

/// The trait that provides methods for common operation types.
#[async_trait]
pub trait Chain
where
    Self: Sync,
{
    /// Returns the name of the chain.
    fn name(&self) -> &'static str;

    /// Returns the `ChainUrls` of the chain.
    fn urls(&self) -> &ChainUrls;

    /// Returns Cosmos SDK version of the chain.
    fn sdk_version(&self) -> usize;

    /// Returns `reqwest::Client` of the chain.
    fn client(&self) -> &Client;

    /// Makes an RPC request.
    async fn rpc_request<T>(&self, path: &str, query: &[(&'static str, String)]) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.urls().rpc, path);

        match self.client().get(url).query(query).send().await {
            Ok(res) => match res.json::<RPCResponse<T>>().await {
                Ok(res_json) => match res_json {
                    RPCResponse::Success(res) => Ok(res.result),
                    RPCResponse::Error(res) => Err(res.error.data),
                },
                Err(_) => Err("Cannot parse JSON.".to_string()),
            },
            Err(_) => Err("Unsuccessful request.".to_string()),
        }
    }

    /// Makes REST API request.
    async fn rest_api_request<T>(&self, path: &str, query: &[(&'static str, String)]) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.urls().rest_api, path);

        match self.client().get(url).query(query).send().await {
            Ok(res) => {
                if res.status().is_success() {
                    match res.json().await {
                        Ok(res_json) => Ok(res_json),
                        Err(_) => Err("Cannot parse JSON error response.".to_string()),
                    }
                } else {
                    match res.json().await {
                        Ok(res_json) => Err(res_json),
                        Err(_) => Err("Cannot parse JSON error response.".to_string()),
                    }
                }
            }
            Err(_) => Err("Unsuccessful request.".to_string()),
        }
    }

    /// Returns the block at given height. Returns the latest block, if no height is given.
    async fn get_block_by_height(&self, height: Option<usize>) -> Result<BlockResp, String> {
        let mut query = vec![];

        let height = height.and_then(|height| Some(height.to_string()));

        if let Some(height) = height {
            query.push(("height", height))
        }

        self.rpc_request("/block", &query).await
    }

    /// Returns the block with given hash.
    async fn get_block_by_hash(&self, hash: &str) -> Result<RPCSuccessResponse<BlockResp>, String> {
        let mut query = vec![];

        query.push(("hash", hash.to_string()));

        self.rpc_request("/block_by_hash", &query).await
    }

    /// Returns transaction by given hash. Hash should start with `0x`.
    async fn get_tx_by_hash(&self, hash: &str) -> Result<TransactionResp, String> {
        let mut query = vec![];

        query.push(("hash", hash.to_string()));

        self.rpc_request("/tx", &query).await
    }

    /// Returns transactions with given sender.
    async fn get_txs_by_sender(&self, sender_address: &str, pagination_config: PaginationConfig) -> Result<TxsResp, String> {
        let mut query = vec![];

        query.push(("events", format!("message.sender='{}'", sender_address)));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));
        query.push(("order_by", "ORDER_BY_DESC".to_string()));

        self.rest_api_request("/cosmos/tx/v1beta1/txs", &query).await
    }

    /// Returns transactions with given recipient.
    async fn get_txs_by_recipient(
        &self,
        recipient_address: &str,
        pagination_config: &PaginationConfig,
    ) -> Result<TxsResp, String> {
        let mut query = vec![];

        query.push(("events", format!("message.recipient='{}'", recipient_address)));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));
        query.push(("order_by", "ORDER_BY_DESC".to_string()));

        self.rest_api_request("/cosmos/tx/v1beta1/txs", &query).await
    }

    /// Returns transactions at given height.
    async fn get_txs_by_height(&self, block_height: u64, pagination_config: &PaginationConfig) -> Result<TxsResp, String> {
        let mut query = vec![];

        query.push(("events", format!("tx.height={}", block_height)));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));
        query.push(("order_by", "ORDER_BY_DESC".to_string()));

        self.rest_api_request("/cosmos/tx/v1beta1/txs", &query).await
    }

    /// Returns accumulated commission of given validator.
    async fn get_validator_commission(&self, validator_addr: &str) -> Result<ValidatorCommisionResp, String> {
        let path = format!("/cosmos/distribution/v1beta1/validators/{validator_addr}/commission");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns rewards of given validator.
    async fn get_validator_rewards(&self, validator_addr: &str) -> Result<ValidatorRewardsResp, String> {
        let path = format!("/cosmos/distribution/v1beta1/validators/{validator_addr}/outstanding_rewards");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns the total supply of all tokens.
    async fn get_supply_of_all_tokens(&self, pagination_config: &PaginationConfig) -> Result<SupplyOfAllTokensResp, String> {
        let path = if self.sdk_version() < 40 {
            "/supply/total"
        } else {
            "/cosmos/bank/v1beta1/supply"
        };

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request(path, &query).await
    }

    /// Returns the supply of given token.
    async fn get_supply_by_denom(&self, denom: &str) -> Result<SupplyByDenomResp, String> {
        let path = if self.sdk_version() < 40 {
            format!("/supply/total/{denom}")
        } else {
            format!("/cosmos/bank/v1beta1/supply/{denom}")
        };

        self.rest_api_request(&path, &[]).await
    }

    /// Returns staking pool information.
    async fn get_staking_pool(&self) -> Result<StakingPoolResp, String> {
        self.rest_api_request("/cosmos/staking/v1beta1/pool", &[]).await
    }

    /// Returns the minting inflation rate of native coin of the chain.
    async fn get_minting_inflation_rate(&self) -> f64 {
        if self.name() == "evmos" {
            self.rest_api_request::<MintingInflationRateResp>("/evmos/inflation/v1/inflation_rate", &[])
                .await
                .and_then(|res| Ok(res.inflation_rate.parse::<f64>().unwrap_or(0.0) / 100.0))
        } else if self.name() == "echelon" {
            self.rest_api_request::<MintingInflationRateResp>("/echelon/inflation/v1/inflation_rate", &[])
                .await
                .and_then(|res| Ok(res.inflation_rate.parse::<f64>().unwrap_or(0.0) / 100.0))
        } else {
            self.rest_api_request::<MintingInflationResp>("/cosmos/mint/v1beta1/inflation", &[])
                .await
                .and_then(|res| Ok(res.inflation.parse::<f64>().unwrap_or(0.0)))
        }
        .unwrap_or(0.0)
    }

    /// Returns the staking parameters.
    async fn get_staking_params(&self) -> Result<StakingParamsResp, String> {
        self.rest_api_request("/cosmos/staking/v1beta1/params", &[]).await
    }

    /// Returns the list of validators with bonded status.
    async fn get_validator_list_bonded(&self, pagination_config: &PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_BONDED".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }

    /// Returns the list of validators with unbonded status.
    async fn get_validator_list_unbonded(&self, pagination_config: &PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_UNBONDED".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }

    /// Returns the list of validators with unbonding status.
    async fn get_validator_list_unbonding(&self, pagination_config: &PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_UNBONDING".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }
    /// Returns the list of validators with unspecified status.
    async fn get_validator_list_unspecified(&self, pagination_config: &PaginationConfig) -> Result<ValidatorListResp, String> {
        let mut query = vec![];

        query.push(("status", "BOND_STATUS_UNSPECIFIED".to_string()));
        query.push(("pagination.reverse", format!("{}", pagination_config.reverse)));
        query.push(("pagination.limit", format!("{}", pagination_config.limit)));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", pagination_config.offset)));

        self.rest_api_request("/cosmos/staking/v1beta1/validators", &query).await
    }
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListResp {
    /// Array of validators.
    validators: ValidatorListValidator,
    /// Pagination.
    pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListValidator {
    /// Operator address. Eg: `"evmosvaloper1qq95x6dhrdnrfunlth5uh24tkrfphzl9crd3xr"`
    operator_address: String,
    /// Consensus public key.
    consensus_pubkey: PublicKey,
    /// Jailed state. Eg: `false`
    jailed: bool,
    /// Status. Eg: `"BOND_STATUS_BONDED"`
    status: String,
    /// Tokens. Eg: `"145722654634775400576772"`
    tokens: String,
    /// Delegator shares. Eg: `"146454922655204548581706.446790192014497216"`
    delegator_shares: String,
    /// Description.
    description: ValidatorListValidatorDescription,
    /// Unbonding height. Eg: `"2580496"`
    unbonding_height: String,
    /// Unbonding time. Eg: `"2022-08-21T03:48:38.952541966Z"`
    unbonding_time: String,
    commission: ValidatorListValidatorCommission,
    /// Minimum self delegation. Eg: `"1"`
    min_self_delegation: String,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListValidatorCommission {
    /// Validator commission rates.
    commission_rates: ValidatorListValidatorCommissionRates,
    /// Validator commission update time. Eg: `"2022-03-02T19:00:00Z"`
    update_time: String,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListValidatorCommissionRates {
    /// Validator commission rate. Eg: `"0.050000000000000000"`
    rate: String,
    /// Validator maximum commission rate. Eg: `"0.200000000000000000"`
    max_rate: String,
    /// Validator maximum commission change rate. Eg: `"0.010000000000000000"`
    max_change_rate: String,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListValidatorDescription {
    /// Validator moniker. Eg: `"heisenbug"`
    moniker: String,
    /// Validator identity. Eg: `"367960C067E253A4"`
    identity: String,
    /// Validator website. Eg: `"https://heisenbug.one"`
    website: String,
    /// Validator security contact. Eg: `"@heisenbug_evmos"`
    security_contact: String,
    /// Validator details. Eg: `"reliable \u0026\u0026 secure staking"`
    details: String,
}

#[derive(Deserialize, Debug)]
pub struct StakingParamsResp {
    /// The staking parameters.
    params: StakingParams,
}

#[derive(Deserialize, Debug)]
pub struct StakingParams {
    /// Unbonding time. Eg: `"1814400s"`
    unbonding_time: String,
    /// Maximum number of validators. Eg: `175`
    max_validators: usize,
    /// Maximum number of entries. Eg: `7`
    max_entries: usize,
    /// Historical number of entries. Eg: `10000`
    historical_entries: usize,
    /// Bonding denom. Eg: `"uatom"`
    bond_denom: String,
}

#[derive(Deserialize, Debug)]
pub struct MintingInflationResp {
    /// Minting inflation rate. Eg: `"0.131020685388983473"`
    inflation: String,
}

#[derive(Deserialize, Debug)]
pub struct MintingInflationRateResp {
    /// Minting inflation rate. Eg: `"91.087708112747866100"`
    inflation_rate: String,
}

#[derive(Deserialize, Debug)]
pub struct StakingPoolResp {
    /// Staking pool information.
    pool: StakingPool,
}

#[derive(Deserialize, Debug)]
pub struct StakingPool {
    /// Tokens not bonded. Eg: `"15241580330282"`
    not_bonded_tokens: String,
    /// Tokens bonded. Eg: `"203496656637783"`
    bonded_tokens: String,
}

#[derive(Deserialize, Debug)]
pub struct SupplyByDenomResp {
    /// Amount and denom.
    amount: TxsDenomAmount,
}

#[derive(Deserialize, Debug)]
pub struct SupplyOfAllTokensResp {
    /// Array of amounts and denoms.
    supply: Vec<TxsDenomAmount>,
    /// Paginations
    pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorCommisionResp {
    /// Validator commission.
    commission: ValidatorCommision,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorCommision {
    /// Array of amounts and demons.
    commission: Vec<TxsDenomAmount>,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorRewardsResp {
    /// Validator rewards.
    rewards: ValidatorCommision,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorRewards {
    /// Array of amounts and denoms.
    rewards: Vec<TxsDenomAmount>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum RPCResponse<T> {
    Success(RPCSuccessResponse<T>),
    Error(RPCErrorResponse),
}

#[derive(Deserialize, Debug)]
pub struct TxsResp {
    pub txs: Vec<TxsTransaction>,
    pub tx_responses: Vec<TxResponse>,
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
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
    /// Logs.
    pub logs: Vec<TxsResponseLog>,
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
    // Events.
    pub events: Vec<TxsResponseEvent<TransactionEventAttribute>>,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct TxsResponseLog {
    /// Message index. Eg: `0`
    pub msg_index: usize,
    /// Log. Eg: `""`
    pub log: String,
    /// Events.
    pub events: Vec<TxsResponseEvent<TxsResponseLogEventAttribute>>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum TxsResponseEvent<T> {
    CoinReceived {
        /// Coin received attributes.
        attributes: Vec<T>,
    },
    CoinSpent {
        /// Coin spent attributes.
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
}

#[derive(Deserialize, Debug)]
#[serde(tag = "key")]
#[serde(rename_all = "lowercase")]
pub enum TxsResponseLogEventAttribute {
    Receiver {
        /// Receiver address. Eg: `"cosmos1vl8xm7x04cedgh639hc9ucvf6zc754fyfewhef"`
        #[serde(rename = "value")]
        receiver_address: String,
    },
    Amount {
        /// Received amount. Eg: `"450000uatom"`
        #[serde(rename = "value")]
        amount: String,
    },
    Spender {
        /// Spender address. Eg: `"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf"`
        #[serde(rename = "value")]
        spender_address: String,
    },
    Action {
        /// Action method. Eg: `"/cosmos.bank.v1beta1.MsgSend"`
        #[serde(rename = "value")]
        action_method: String,
    },
    Sender {
        /// Sender address. Eg: `"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf"`
        #[serde(rename = "value")]
        sender_address: String,
    },
    Module {
        /// Module type. Eg: `"bank"`
        #[serde(rename = "value")]
        module_type: String,
    },
    Recipient {
        /// Recipient address. Eg: `"cosmos1vl8xm7x04cedgh639hc9ucvf6zc754fyfewhef"`
        #[serde(rename = "value")]
        recipient_address: String,
    },
}

#[derive(Deserialize, Debug)]
pub struct TxsTransaction {
    /// Transaction body.
    pub body: TxsTransactionBody,
    /// Transaction auth information.
    pub auth_info: TxsTransactionAuthInfo,
    /// Array of Base 64 encoded transaction signatures.
    pub signatures: Vec<String>,
}
#[derive(Deserialize, Debug)]
pub struct TxsTransactionAuthInfo {
    /// Transaction signer informations.
    pub signer_infos: Vec<TxsTransactionSignerInfo>,
    /// Transaction fee.
    pub fee: TxsTransactionAuthInfoFee,
    /// Transaction gas limit.
    pub gas_limit: String,
    /// Transaction payer. Eg: `""`
    pub payer: String,
    /// Transaction granter. Eg: `""`
    pub granter: String,
}

#[derive(Deserialize, Debug)]
pub struct TxsTransactionAuthInfoFee {
    /// Amount.
    pub amount: Vec<TxsDenomAmount>,
}

#[derive(Deserialize, Debug)]
pub struct TxsTransactionBody {
    /// Transaction messages.
    pub messages: Vec<TxsTransactionMessage>,
    /// Transaction memo. Eg: `"1891420480"`
    pub memo: String,
    /// Transaction timeout height. Eg: `"0"`
    pub timeout_height: String,
    /// Transaction extension options.
    pub extension_options: Vec<u8>,
    /// Non-critical transaction extension options.
    pub extension_optionsnon_critical_extension_options: Vec<u8>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "@type")]
pub enum TxsTransactionMessage {
    #[serde(rename = "/cosmos.bank.v1beta1.MsgSend")]
    MsgSend {
        /// The address transaction is from. Eg: `"cosmos1h4qpl2fxlcatp495pn8wjqcfkq3h66r9vk4hxf"`
        from_address: String,
        /// The address transaction is to. Eg: `"cosmos1vl8xm7x04cedgh639hc9ucvf6zc754fyfewhef"`
        to_address: String,
        /// Transaction amounts.
        amount: Vec<TxsDenomAmount>,
    },
}

#[derive(Deserialize, Debug)]
pub struct TxsTransactionSignerInfo {
    pub public_key: PublicKey,
    pub mode_info: TxsTransactionModeInfo,
    /// Transaction signer info sequence. Eg: `"1"`
    pub sequence: String,
}

#[derive(Deserialize, Debug)]
pub struct TxsTransactionModeInfo {
    pub single: TxsTransactionModeInfoSingle,
}

#[derive(Deserialize, Debug)]
pub struct TxsTransactionModeInfoSingle {
    /// Mode. Eg: `"SIGN_MODE_LEGACY_AMINO_JSON"`
    pub mode: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "@type")]
pub enum PublicKey {
    #[serde(rename = "/cosmos.crypto.secp256k1.PubKey")]
    Secp256K1 {
        /// Base 64 encoded Secp256K1 public key. Eg: `"Ap9xAyS21AGuRY4W7+Mi3JzbmULJjGATAzVeIxc98t07"`
        key: String,
    },
    #[serde(rename = "/cosmos.crypto.ed25519.PubKey")]
    Ed25519 {
        /// Base 64 encoded Ed25519 public key. Eg: `"zy/GxGwk1Pm3HiG67iani1u+MUieM98ZvSIrXC8mISE="`
        key: String,
    },
}

#[derive(Deserialize, Debug)]
pub struct TxsDenomAmount {
    /// The name of the token. Eg: `"uatom"`
    pub denom: String,
    /// The amount of the token. Eg: `"450000"`
    pub amount: String,
}

#[derive(Deserialize, Debug)]
pub struct TransactionResp {
    /// HEX encoded TX hash, without leading `0x`. Eg: `"25EC6BCEA9B4A6835F5A38AB566959187F968C295EE71D015C3D907B25C5C72F"`
    pub hash: String,
    /// The block height TX at. Eg: `"6684890"`
    pub height: String,
    /// Unknown. Eg: `0`
    pub index: usize,
    /// The transaction result.
    pub tx_result: TransactionResult,
}

#[derive(Deserialize, Debug)]
pub struct TransactionResult {
    /// Unknown. Eg: `0`
    pub code: usize,
    /// Base64 encoded transaction data. Eg: `"CrgECh8vZXRoZXJtaW50LmV2bS52MS5Nc2dFdGhlcmV1bVR4EpQECkIweDgxNTRhOGEyYmViYzQyYzNhNmVlYTZjMTAwMDMwMzkwMzhkOTJiZGYxOWNiMmQ4NDBhYzJkN2Q2ZmI3YjBmMzISpwMKKjB4NEY0MWE5ZTJjYTc4YWQ2QjZlRmFiNTJGNjYxQjVmMEEwQzIxMUY3NRJCMHg4YzViZTFlNWViZWM3ZDViZDE0ZjcxNDI3ZDFlODRmM2RkMDMxNGMwZjdiMjI5MWU1YjIwMGFjOGM3YzNiOTI1EkIweDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDcyODYxOWNlZjE0MTEyZjFiNzc3ZTQ2ODAwYTkwNjc3ZDQ5OTI1NDQSQjB4MDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDY3ZWM4Nzg0NGZiZDczZWRhNGExMDU5ZjMwMDM5NTg0NTg2ZTA5ZBog//////////////////////////////////////////8g2oGYAypCMHg4MTU0YThhMmJlYmM0MmMzYTZlZWE2YzEwMDAzMDM5MDM4ZDkyYmRmMTljYjJkODQwYWMyZDdkNmZiN2IwZjMyOkIweDBhMjdkZDQyNDBkYzM1MjE1OWYxZTVhMzA3NjM0NDIwZmFjN2I2ZDg5YzYxYWI5NzIyNDI4MjIxZWFmYjg4NGYaIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABKKbqAg=="`
    pub data: String,
    /// JSON encoded transaction log. Eg: `"[{\"events\":[{\"type\":\"coin_received\",\"attributes\":[{\"key\":\"receiver\",\"value\":\"evmos1w2rpnnh3gyf0rdmhu35qp2gxwl2fjf2y4vjkhg\"},{\"key\":\"amount\",\"value\":\"199391000000000aevmos\"}]},{\"type\":\"coin_spent\",\"attributes\":[{\"key\":\"spender\",\"value\":\"evmos17xpfvakm2amg962yls6f84z3kell8c5ljcjw34\"},{\"key\":\"amount\",\"value\":\"199391000000000aevmos\"}]},{\"type\":\"ethereum_tx\",\"attributes\":[{\"key\":\"amount\",\"value\":\"0\"},{\"key\":\"ethereumTxHash\",\"value\":\"0x8154a8a2bebc42c3a6eea6c10003039038d92bdf19cb2d840ac2d7d6fb7b0f32\"},{\"key\":\"txIndex\",\"value\":\"0\"},{\"key\":\"txGasUsed\",\"value\":\"46374\"},{\"key\":\"txHash\",\"value\":\"25EC6BCEA9B4A6835F5A38AB566959187F968C295EE71D015C3D907B25C5C72F\"},{\"key\":\"recipient\",\"value\":\"0x4F41a9e2ca78ad6B6eFab52F661B5f0A0C211F75\"}]},{\"type\":\"message\",\"attributes\":[{\"key\":\"action\",\"value\":\"/ethermint.evm.v1.MsgEthereumTx\"},{\"key\":\"sender\",\"value\":\"evmos17xpfvakm2amg962yls6f84z3kell8c5ljcjw34\"},{\"key\":\"module\",\"value\":\"evm\"},{\"key\":\"sender\",\"value\":\"0x728619cEf14112F1B777E46800a90677d4992544\"},{\"key\":\"txType\",\"value\":\"2\"}]},{\"type\":\"transfer\",\"attributes\":[{\"key\":\"recipient\",\"value\":\"evmos1w2rpnnh3gyf0rdmhu35qp2gxwl2fjf2y4vjkhg\"},{\"key\":\"sender\",\"value\":\"evmos17xpfvakm2amg962yls6f84z3kell8c5ljcjw34\"},{\"key\":\"amount\",\"value\":\"199391000000000aevmos\"}]},{\"type\":\"tx_log\",\"attributes\":[{\"key\":\"txLog\",\"value\":\"{\\\"address\\\":\\\"0x4F41a9e2ca78ad6B6eFab52F661B5f0A0C211F75\\\",\\\"topics\\\":[\\\"0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925\\\",\\\"0x000000000000000000000000728619cef14112f1b777e46800a90677d4992544\\\",\\\"0x000000000000000000000000067ec87844fbd73eda4a1059f30039584586e09d\\\"],\\\"data\\\":\\\"//////////////////////////////////////////8=\\\",\\\"blockNumber\\\":6684890,\\\"transactionHash\\\":\\\"0x8154a8a2bebc42c3a6eea6c10003039038d92bdf19cb2d840ac2d7d6fb7b0f32\\\",\\\"transactionIndex\\\":0,\\\"blockHash\\\":\\\"0x0a27dd4240dc352159f1e5a307634420fac7b6d89c61ab9722428221eafb884f\\\",\\\"logIndex\\\":0}\"}]}]}]"`
    pub log: String,
    /// The transaction information. Eg: `""`
    pub info: String,
    /// Gas wanted. Eg: `"55648"`
    pub gas_wanted: String,
    /// Gas used. Eg: `"46374"`
    pub gas_used: String,
    /// Transaction events.
    pub events: Vec<TransactionEvent>,
    /// Transaction codespace. Eg: `""`
    pub codespace: String,
    // Base 64 encoded transaction. Eg: `"CqMDCu8CCh8vZXRoZXJtaW50LmV2bS52MS5Nc2dFdGhlcmV1bVR4EssCCoQCCh4vZXRoZXJtaW50LmV2bS52MS5EeW5hbWljRmVlVHgS4QEKBDkwMDEQKRoKMTUwMDAwMDAwMCILMjU1MDAwMDAwMDAo4LIDMioweDRGNDFhOWUyY2E3OGFkNkI2ZUZhYjUyRjY2MUI1ZjBBMEMyMTFGNzU6ATBCRAlep7MAAAAAAAAAAAAAAAAGfsh4RPvXPtpKEFnzADlYRYbgnf//////////////////////////////////////////UgEBWiD4q5dJAnhCoLGbgwyqtMO3GuL4kx1WmrtUyDr7hzaeYmIgL/+FNXRbiS+/RyH2p5dwQ0O8OOcFHGxUDg6AP3gPYxQaQjB4ODE1NGE4YTJiZWJjNDJjM2E2ZWVhNmMxMDAwMzAzOTAzOGQ5MmJkZjE5Y2IyZDg0MGFjMmQ3ZDZmYjdiMGYzMvo/LgosL2V0aGVybWludC5ldm0udjEuRXh0ZW5zaW9uT3B0aW9uc0V0aGVyZXVtVHgSIhIgChoKBmFldm1vcxIQMTQxOTAyNDAwMDAwMDAwMBDgsgM="`
    pub tx: String,
}

#[derive(Deserialize, Debug)]
pub struct TransactionEvent {
    /// Transaction event type. Eg: `"coin_spent"`
    pub r#type: String,
    /// Transaction event attributes.
    pub attributes: Vec<TransactionEventAttribute>,
}

#[derive(Deserialize, Debug)]
pub struct TransactionEventAttribute {
    /// Base 64 encoded transaction event attribute key. Eg: `"c3BlbmRlcg=="`
    pub key: String,
    /// Base 64 encoded transaction event attribute value. Eg: `"ZXZtb3MxdzJycG5uaDNneWYwcmRtaHUzNXFwMmd4d2wyZmpmMnk0dmpraGc="`
    pub value: String,
    /// Transaction event attribute index. Eg: `true`
    pub index: bool,
}

/// The configuration to be used while making REST API requests.
pub struct PaginationConfig {
    /// It is set to true if results are to be returned in the descending order.
    pub reverse: bool,
    pub offset: u64,
    /// It is the total number of results to be returned in the result page
    pub limit: u64,
}

#[derive(Deserialize, Debug)]
pub struct Pagination {
    /// Pagination next key. Might be `None`. Eg: `"FGxWOxzuw4bZozVHta3qYgdKOuRC"`
    pub next_key: Option<String>,
    /// Total. Eg: `"0"`
    pub total: String,
}

#[derive(Deserialize, Debug)]
pub struct SlashingSigningInfo {
    pub info: Vec<SlashingSigningInfoItem>,
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct SlashingSigningInfoItem {
    /// Validator address. Eg: `"evmosvalcons1qx4hehfny66jfzymzn6d5t38m0ely3cvw6zn06"`
    pub address: String,
    /// The block height slashing is started at. Eg: `"0"`
    pub start_height: String,
    /// Unknown. Eg: `"5888077"`
    pub index_offset: String,
    /// The time jailed until. Eg: `"2022-05-14T04:31:49.705643236Z"`
    pub jailed_until: String,
    /// Tombstoned state. Eg: `false`
    pub tombstoned: bool,
    /// The count of missed blocks. Eg: `"16433"`
    pub missed_blocks_counter: String,
}

#[derive(Deserialize, Debug)]
pub struct RPCSuccessResponse<T> {
    pub jsonrpc: String,
    pub id: isize,
    pub result: T,
}

#[derive(Deserialize, Debug)]
pub struct RPCErrorResponse {
    pub jsonrpc: String,
    pub id: isize,
    pub error: RpcErrorResponseError,
}

#[derive(Deserialize, Debug)]
pub struct RpcErrorResponseError {
    /// The error code.
    pub code: isize,
    /// The message about error type.
    pub message: String,
    /// Description about error.
    pub data: String,
}

#[derive(Deserialize, Debug)]
pub struct BlockResp {
    pub block_id: BlockId,
    pub block: BlockBlock,
}

#[derive(Deserialize, Debug)]
pub struct BlockId {
    /// HEX encoded transaction hash.
    pub hash: String,
    pub parts: BlockIdParts,
}

#[derive(Deserialize, Debug)]
pub struct BlockBlock {
    pub header: BlockHeader,
    pub data: BlockData,
    pub evidence: BlockEvidence,
    pub last_commit: BlockLastCommit,
}

#[derive(Deserialize, Debug)]
pub struct BlockIdParts {
    /// Unknown. Eg: `1`
    pub total: usize,
    /// HEX encoded transaction hash.
    pub hash: String,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct BlockData {
    /// Array of very long Base64 encoded transactions. Eg: `["CoYBCoMBCiUvYXhlbGFyLmF4ZWxhcm5ldC52MWJldGExLkxpbmtSZXF1ZXN0EloKFAfFBMRZ8AeNGGkWVAcX+idm5UutEioweDM1NzkyNTRmNTgwNWQxNjZiNjhhNTg3MzIwNzA0NDQ4MjBmYTRiZjEaCGV0aGVyZXVtIgx3YnRjLXNhdG9zaGkSlQEKUQpGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQPUmMSQ2WoB0eD589u7pruIZt2gbHT2DO3QSIPX0z8WXBIECgIIARiuCBJACgsKBHVheGwSAzY3NRDh8AUiLWF4ZWxhcjFwdTJzd2MwbjB0cmZ0bGRoejU3cHlxa3c2ZDg3aGFobjdnNjk3YxpANmM1rQE1P3hbVtuFoaQEpGpnBnlygbotxEA0qR/rmAwVRB+acJ6idoF1V0Qul5eSCpi1Z0TLLwQEMya4nMdl3g=="]`
    pub txs: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct BlockEvidence {
    // Property below is an unknown array. TODO!
    // evidence: Vec<UNKNOWN>
}
#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct BlockHeaderVersion {
    /// Unknown. Eg: `"11"`
    pub block: String,
}

#[derive(Deserialize, Debug)]
pub struct BlockLastCommitSignatures {
    /// Unknown. Eg: `2`
    pub block_id_flag: usize,
    /// HEX encoded address of a validator.
    pub validator_address: String,
    /// The time of the unix timestamp. Eg: `"2022-11-03T17:45:14.193617481Z"`
    pub timestamp: String,
    /// Base 64 encoded signature. It might be `None`, so unsigned. Eg: `"rum2poquBDmHkGLGxjtjrlNBP5bV52m6ckexmNHdln85WRii4tCaVqAmxAKR+fP+hzoxEDuhOGwQ/xlgMVFrAA=="`
    pub signature: Option<String>,
}
