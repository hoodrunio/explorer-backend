use serde::{Deserialize, Serialize};

use crate::chain::Chain;

impl Chain {
    /// Returns staking pool information.
    pub async fn get_staking_pool(&self) -> Result<InternalStakingPool, String> {
        match self.rest_api_request::<StakingPoolResp>("/cosmos/staking/v1beta1/pool", &[]).await {
            Ok(resp) => {
                let bonded = match resp.pool.bonded_tokens.parse::<u128>() {
                    Ok(bonded_tokens) => (bonded_tokens / 10_u128.pow(self.decimals.into())) as u64,
                    Err(_) => return Err("Tokenomics parsing error.".to_string()),
                };

                let unbonded = match resp.pool.not_bonded_tokens.parse::<u128>() {
                    Ok(not_bonded_tokens) => (not_bonded_tokens / 10_u128.pow(self.decimals.into())) as u64,
                    Err(_) => return Err("Tokenomics parsing error.".to_string()),
                };

                Ok(InternalStakingPool { unbonded, bonded })
            }
            Err(error) => Err(error),
        }
    }

    /// Returns the signing info by given cons address.
    pub async fn get_signing_info(&self, cons_addr: &str) -> Result<SigningInfoResp, String> {
        let path = format!("/cosmos/slashing/v1beta1/signing_infos/{cons_addr}");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns the native coin amount in the community pool.
    pub async fn get_community_pool(&self) -> Result<u64, String> {
        match self
            .rest_api_request::<CommunityPoolResp>("/cosmos/distribution/v1beta1/community_pool", &[])
            .await
        {
            Ok(resp) => match resp.pool.get(0) {
                Some(amount) => match amount.amount.parse::<f64>() {
                    Ok(community_pool_amount) => Ok((community_pool_amount / 10_f64.powi(self.decimals.into())) as u64),
                    _ => Err(format!("Cannot parse number, {}.", amount.amount)),
                },
                None => Err(format!("There is no community pool for '{}'", self.name)),
            },
            Err(error) => Err(error),
        }
    }
}

// Returns the mint parameters of the chain.
/* async fn get_mint_params(&self) -> Option<MintParams> {
    if self.name() == "evmos" {
        self.rest_api_request::<ParamsResp<InflationParams>>("/evmos/inflation/v1/params", &[]).await.and_then(|a|)
    } else if self.name() == "echelon" {
        self.rest_api_request::<ParamsResp<InflationParams>>("/echelon/inflation/v1/params", &[]).await
    } else {
        self.rest_api_request::<ParamsResp<MintParams>>("/cosmos/mint/v1beta1/params", &[])
            .await
            .ok()
            .and_then(|res| Some(res.params))
    }
    .unwrap_or(None)
}
*/

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

#[derive(Deserialize, Serialize, Debug)]
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
    pub amount: u128,
}

impl TryFrom<DenomAmount> for InternalDenomAmount {
    type Error = String;
    fn try_from(value: DenomAmount) -> Result<Self, Self::Error> {
        let amount: u128 = match value.amount.parse() {
            Ok(v) => v,
            Err(_) => return Err(format!("Cannot parse amount, '{}'.", value.amount)),
        };

        Ok(InternalDenomAmount { denom: value.denom, amount })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SigningInfoResp {
    /// Validator signing info.
    pub val_signing_info: SlashingSigningInfoItem,
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
pub struct SlashingSigningInfo {
    pub info: Vec<SlashingSigningInfoItem>,
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
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
    #[serde(rename = "/ethermint.crypto.v1.ethsecp256k1.PubKey")]
    Ethsecp256k1 {
        /// Base 64 encoded Ethsecp256k1 public key. Eg: `"AqrviRnJYWdC2OMM1haDI2X6oEIev8u0oqR10Elb06+1"`
        key: String,
    },
}

/// The configuration to be used while making REST API requests.
pub struct PaginationConfig {
    /// It is set to true if results are to be returned in the descending order.
    reverse: bool,
    /// It is the number of result to not to be returned in the result page
    offset: u32,
    /// It is the total number of results to be returned in the result page
    limit: u32,
}

impl PaginationConfig {
    /// Creates a new `PaginationConfig`.
    /// ## Default:
    /// ```rs
    /// PaginationConfig {
    ///     reverse: false,
    ///     offset: 0,
    ///     limit: 10,
    /// }
    /// ```
    pub const fn new() -> Self {
        Self {
            reverse: false,
            offset: 0,
            limit: 10,
        }
    }

    /// Returns `true` if `reverse` property is set to `true`.
    pub const fn is_reverse(&self) -> bool {
        self.reverse
    }

    /// Returns the value `limit` property holds.
    pub const fn get_limit(&self) -> u32 {
        self.limit
    }

    /// Returns the value `offset` property holds.
    pub const fn get_offset(&self) -> u32 {
        self.offset
    }

    /// Makes the response reversed.
    pub const fn reverse(self) -> Self {
        Self { reverse: true, ..self }
    }

    /// Sets a limit for results to be returned in the result page
    pub const fn limit(self, limit: u32) -> Self {
        Self { limit, ..self }
    }

    /// Sets an offset for padding from the first result.
    pub const fn offset(self, offset: u32) -> Self {
        Self { offset, ..self }
    }

    /// Specifies the offset by given page. \
    /// **Base index is 1/ONE.**
    pub fn page(self, page: u32) -> Self {
        Self {
            offset: if page < 2 { 0 } else { self.limit * (page - 1) },
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
