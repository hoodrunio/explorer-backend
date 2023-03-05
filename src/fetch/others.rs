use chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::fetch::blocks::BlockResp;
use crate::fetch::params::ParamsResp;
use crate::{chain::Chain, routes::OutRestResponse};

impl Chain {
    /// Returns staking pool information.
    pub async fn get_staking_pool(&self) -> Result<OutRestResponse<InternalStakingPool>, String> {
        let resp = self.rest_api_request::<StakingPoolResp>("/cosmos/staking/v1beta1/pool", &[]).await?;
        let staking_pool = InternalStakingPool {
            unbonded: self.calc_amount_u128_to_u64(
                resp.pool
                    .not_bonded_tokens
                    .parse::<u128>()
                    .map_err(|_| format!("Cannot parse unbonded tokens, {}.", resp.pool.not_bonded_tokens))?,
            ),
            bonded: self.calc_amount_u128_to_u64(
                resp.pool
                    .bonded_tokens
                    .parse::<u128>()
                    .map_err(|_| format!("Cannot parse bonded tokens, {}.", resp.pool.bonded_tokens))?,
            ),
        };

        Ok(OutRestResponse::new(staking_pool, 0))
    }

    /// Returns the native coin amount in the community pool.
    pub async fn get_community_pool(&self) -> Result<OutRestResponse<u64>, String> {
        let resp = self
            .rest_api_request::<CommunityPoolResp>("/cosmos/distribution/v1beta1/community_pool", &[])
            .await?;

        let pool = resp
            .pool
            .get(0)
            .ok_or_else(|| format!("There is no community pool for '{}' chain.", self.config.name))?;

        let community_pool_amount = pool
            .amount
            .split_once('.')
            .map(|s| s.0)
            .unwrap_or("0")
            .parse::<u128>()
            .map_err(|_| format!("Cannot parse community pool coin amount, '{}'.", pool.amount))?;

        let community_pool_amount = self.calc_amount_u128_to_u64(community_pool_amount);

        Ok(OutRestResponse::new(community_pool_amount, 0))
    }

    // Returns the mint parameters of the chain.
    pub async fn get_mint_params(&self) -> Result<OutRestResponse<MintParams>, String> {
        match self.config.name.as_str() {
            "example_chain_name" => {
                //TODO If Needed Fill this block scopr with related chain
                Err("Chain Mint Params Not Implemented Yet".to_string())
            }
            _ => {
                let mint_params = match self.rest_api_request::<ParamsResp<MintParams>>("/cosmos/mint/v1beta1/params", &[]).await {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                };
                Ok(OutRestResponse::new(mint_params.params, 0))
            }
        }
    }
    pub async fn get_annual_provisions(&self) -> Result<OutRestResponse<f64>, String> {
        match self.config.name.as_str() {
            "example_chain_name" => {
                //TODO If Needed Fill this block scope with related chain
                Err("Chain Mint Params Not Implemented Yet".to_string())
            }
            _ => {
                let annual_provisions = match self
                    .rest_api_request::<AnnualProvisions>("/cosmos/mint/v1beta1/annual_provisions", &[])
                    .await
                {
                    Ok(value) => match value.annual_provisions.parse::<f64>() {
                        Ok(res) => res,
                        Err(_) => return Err("Parsing Error".to_string()),
                    },
                    Err(error) => return Err(error),
                };
                Ok(OutRestResponse::new(annual_provisions, 0))
            }
        }
    }
    pub async fn get_correction_annual_coefficient(&self) -> Result<Option<f64>, String> {
        const SECS_IN_YEAR: f64 = 31561920.0;
        let block_per_year = match self.get_mint_params().await {
            Ok(res) => match res.value.blocks_per_year.parse::<f64>() {
                Ok(value) => value,
                Err(_) => return Err("Parse Error".to_string()),
            },
            Err(error) => return Err(error),
        };
        let latest_block = match self.get_latest_block().await {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let block_window_size = 1000.0;
        let latest_block_date_time = latest_block.header.time;
        let lower_block_height = match latest_block.header.height.parse::<f64>() {
            Ok(value) => value - block_window_size,
            Err(_) => return Err("Parse Error".to_string()),
        };
        let query = vec![("height", lower_block_height.to_string())];
        let lower_block_date_time = match self.rpc_request::<BlockResp>("/block", &query).await {
            Ok(res) => res.block.header.time,
            Err(error) => return Err(error),
        };
        let latest_block_time_sec = DateTime::parse_from_rfc3339(&latest_block_date_time).unwrap().timestamp() as f64;
        let lower_block_time_sec = DateTime::parse_from_rfc3339(&lower_block_date_time).unwrap().timestamp() as f64;
        let avg_block_time_24h = (latest_block_time_sec - lower_block_time_sec) / block_window_size;
        // Calculate how many blocks will be created in a year with the speed same as last 24h.
        let current_real_block_per_year = SECS_IN_YEAR / avg_block_time_24h;
        // Calculate correction.
        let correction_annual_coefficient = current_real_block_per_year / block_per_year;

        Ok(Some(correction_annual_coefficient))
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CommunityPoolResp {
    /// Community pool.
    pub pool: Vec<DenomAmount>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Pagination {
    /// Pagination next key. Might be `None`. Eg: `"FGxWOxzuw4bZozVHta3qYgdKOuRC"`
    pub next_key: Option<String>,
    /// Total. Eg: `"0"`
    pub total: String,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            next_key: None,
            total: "0".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DenomAmount {
    /// The name of the token. Eg: `"uatom"`
    pub denom: String,
    /// The amount of the token. Eg: `"450000"`
    pub amount: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalDenomAmount {
    /// The name of the token. Eg: `"uatom"`
    pub denom: String,
    /// The amount of the token. Eg: `450000`
    pub amount: f64,
}

impl InternalDenomAmount {
    pub async fn from_chain_denom_amount(chain: &Chain, denom_amount: &DenomAmount) -> Result<InternalDenomAmount, String> {
        let amount = chain.calc_amount_f64_to_f64(
            denom_amount
                .amount
                .parse::<f64>()
                .map_err(|_| format!("Cannot parse amount, '{}'.", &denom_amount.amount))?,
        );

        Ok(InternalDenomAmount {
            denom: denom_amount.denom.clone(),
            amount,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AnnualProvisions {
    pub annual_provisions: String,
}

impl TryFrom<DenomAmount> for InternalDenomAmount {
    type Error = String;
    fn try_from(value: DenomAmount) -> Result<Self, Self::Error> {
        let amount: f64 = match value.amount.parse() {
            Ok(v) => v,
            Err(_) => return Err(format!("Cannot parse amount, '{}'.", value.amount)),
        };

        Ok(InternalDenomAmount { denom: value.denom, amount })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InflationParams {
    /// Mint denom. Eg: `"aevmos"`
    pub mint_denom: String,
    /// Exponential calculation.
    pub exponential_calculation: InflationParamsExponentialCalculation,
    /// Inflation distribution.
    pub inflation_distribution: InflationParamsInflationDistribution,
    /// Enable inflation. Eg: `true`
    pub enable_inflation: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InflationParamsInflationDistribution {
    /// Staking rewards inflation. Eg: `"0.533333334000000000"`
    pub staking_rewards: String,
    /// Usage incentives inflation. Eg: `"0.333333333000000000"`
    pub usage_incentives: String,
    /// Community pool inflation. Eg: `"0.133333333000000000"`
    pub community_pool: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InflationParamsExponentialCalculation {
    /// Unknown. Eg: `"300000000.000000000000000000"`
    pub a: String,
    /// Unknown. Eg: `"0.500000000000000000"`
    pub r: String,
    /// Unknown. Eg: `"9375000.000000000000000000"`
    pub c: String,
    /// Bonding target. Eg: `"0.660000000000000000"`
    pub bonding_target: String,
    /// Maximum variance. Eg: `"0.000000000000000000"`
    pub max_variance: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MintParams {
    /// Mint denom. Eg: `"uatom"`
    pub mint_denom: String,
    /// Mint inflation rate change. Eg: ` "1.000000000000000000"`
    pub inflation_rate_change: String,
    /// Maximum mint inflation. Eg: `"0.200000000000000000"`
    pub inflation_max: String,
    /// Minimum mint inflation. Eg: `"0.070000000000000000"`
    pub inflation_min: String,
    /// Goal bonded. Eg: `"0.670000000000000000"`
    pub goal_bonded: String,
    /// Blocks per year. Eg: `"4360000"`
    pub blocks_per_year: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StakingPoolResp {
    /// Staking pool information.
    pub pool: StakingPool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StakingPool {
    /// Tokens not bonded. Eg: `"15241580330282"`
    pub not_bonded_tokens: String,
    /// Tokens bonded. Eg: `"203496656637783"`
    pub bonded_tokens: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalStakingPool {
    /// Tokens unbonded. Eg: `15241580330282`
    pub unbonded: u64,
    /// Tokens bonded. Eg: `203496656637783`
    pub bonded: u64,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum PublicKey {
    Known(PublicKeyKnown),
    Unknown(),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "@type")]
pub enum PublicKeyKnown {
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
    #[serde(rename = "/ethermint.crypto.v1.ethsecp256k1.PubKey")]
    Ethsecp256k1 {
        /// Base 64 encoded Ethsecp256k1 public key. Eg: `"AqrviRnJYWdC2OMM1haDI2X6oEIev8u0oqR10Elb06+1"`
        key: String,
    },
    #[serde(rename = "/cosmos.crypto.multisig.LegacyAminoPubKey")]
    LegacyAminoPubKey {
        /// Multisig threshold.
        threshold: u32,
        /// Public keys which comprise the multisig key.
        public_keys: Vec<PublicKey>,
    },
}

/// The configuration to be used while making REST API requests.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PaginationConfig {
    /// It is set to true if results are to be returned in the descending order.
    pub reverse: bool,
    /// It is the number of result to not to be returned in the result page
    pub offset: u32,
    /// It is the total number of results to be returned in the result page
    pub limit: u16,
    /// It is the current page count which will include with target result
    pub page: u8,
}

impl PaginationConfig {
    /// Creates a new `PaginationConfig`.
    /// ## Default:
    /// ```rs
    /// PaginationConfig {
    ///     reverse: false,
    ///     offset: 0,
    ///     limit: 10,
    ///     page: 1
    /// }
    /// ```
    pub const fn new() -> Self {
        Self {
            reverse: false,
            offset: 0,
            limit: 10,
            page: 1,
        }
    }

    /// Returns `true` if `reverse` property is set to `true`.
    pub const fn is_reverse(&self) -> bool {
        self.reverse
    }

    /// Returns the value `limit` property holds.
    pub const fn get_limit(&self) -> u16 {
        self.limit
    }

    /// Returns the value `offset` property holds.
    pub const fn get_offset(&self) -> u32 {
        self.offset
    }

    /// Returns the value `offset` property holds.
    pub const fn get_page(&self) -> u8 {
        self.page
    }

    /// Makes the response reversed.
    pub const fn reverse(self) -> Self {
        Self { reverse: true, ..self }
    }

    /// Sets a limit for results to be returned in the result page
    pub const fn limit(self, limit: u16) -> Self {
        Self { limit, ..self }
    }

    /// Specifies the offset by given page. \
    /// **Base index is 1/ONE.**
    pub fn page(self, page: u8) -> Self {
        Self {
            offset: if page < 2 { 0 } else { (self.limit * (page as u16 - 1)).into() },
            page,
            ..self
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Response<S> {
    Success(S),
    Error(String),
}

impl<T> From<Result<T, String>> for Response<T> {
    fn from(res: Result<T, String>) -> Self {
        match res {
            Ok(value) => Response::Success(value),
            Err(error) => Response::Error(error),
        }
    }
}
