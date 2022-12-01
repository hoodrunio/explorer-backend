use chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::{chain::Chain, routes::OutRestResponse};

impl Chain {
    /// Returns staking pool information.
    pub async fn get_staking_pool(&self) -> Result<OutRestResponse<InternalStakingPool>, String> {
        let resp = self.rest_api_request::<StakingPoolResp>("/cosmos/staking/v1beta1/pool", &[]).await?;

        let staking_pool = InternalStakingPool {
            unbonded: self.calc_amount_u128_to_u64(resp.pool.not_bonded_tokens.parse::<u128>().or_else(|_| Err(format!("")))?),
            bonded: self.calc_amount_u128_to_u64(resp.pool.bonded_tokens.parse::<u128>().or_else(|_| Err(format!("")))?),
        };

        OutRestResponse::new(staking_pool, 0)
    }

    /// Returns the signing info by given cons address.
    pub async fn get_signing_info(&self, cons_addr: &str) -> Result<OutRestResponse<InternalSlashingSigningInfoItem>, String> {
        let path = format!("/cosmos/slashing/v1beta1/signing_infos/{cons_addr}");

        let resp = self.rest_api_request::<SigningInfoResp>(&path, &[]).await?;

        let signing_info = resp.val_signing_info.try_into()?;

        OutRestResponse::new(signing_info, 0)
    }

    /// Returns the native coin amount in the community pool.
    pub async fn get_community_pool(&self) -> Result<OutRestResponse<u64>, String> {
        let resp = self
            .rest_api_request::<CommunityPoolResp>("/cosmos/distribution/v1beta1/community_pool", &[])
            .await?;

        let pool = resp
            .pool
            .get(0)
            .ok_or_else(|| format!("There is no community pool for '{}' chain.", self.inner.name))?;

        let community_pool_amount = pool
            .amount
            .split_once(".")
            .and_then(|s| Some(s.0))
            .unwrap_or("0")
            .parse::<u128>()
            .or_else(|_| Err(format!("Cannot parse community pool coin amount, '{}'.", pool.amount)))?;

        let community_pool_amount = self.calc_amount_u128_to_u64(community_pool_amount);

        OutRestResponse::new(community_pool_amount, 0)
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
pub struct InternalSlashingSigningInfoItem {
    /// Validator address. Eg: `"evmosvalcons1qx4hehfny66jfzymzn6d5t38m0ely3cvw6zn06"`
    pub address: String,
    /// The block height slashing is started at. Eg: `0`
    pub start_height: u64,
    /// Unknown. Eg: `5888077`
    pub index_offset: u64,
    /// The timestamp in milliseconds jailed until.
    pub jailed_until: i64,
    /// Tombstoned state. Eg: `false`
    pub tombstoned: bool,
    /// The count of missed blocks. Eg: `16433`
    pub missed_blocks_counter: u64,
}

impl TryFrom<SlashingSigningInfoItem> for InternalSlashingSigningInfoItem {
    type Error = String;
    fn try_from(value: SlashingSigningInfoItem) -> Result<Self, Self::Error> {
        Ok(Self {
            address: value.address,
            start_height: value
                .start_height
                .parse()
                .or_else(|_| Err(format!("Cannot parse slashing start height, `{}`.", value.start_height)))?,
            index_offset: value
                .start_height
                .parse()
                .or_else(|_| Err(format!("Cannot parse slashing index offset, `{}`.", value.index_offset)))?,
            jailed_until: DateTime::parse_from_rfc3339(&value.jailed_until)
                .or_else(|_| Err(format!("Cannot parse jailed untile datetime, '{}'", value.jailed_until)))?
                .timestamp_millis(),
            tombstoned: value.tombstoned,
            missed_blocks_counter: value
                .missed_blocks_counter
                .parse()
                .or_else(|_| Err(format!("Cannot parse missed blocks counter, `{}`.", value.missed_blocks_counter)))?,
        })
    }
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
#[derive(Clone, Copy)]
pub struct PaginationConfig {
    /// It is set to true if results are to be returned in the descending order.
    reverse: bool,
    /// It is the number of result to not to be returned in the result page
    offset: u32,
    /// It is the total number of results to be returned in the result page
    limit: u16,
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
    pub const fn get_limit(&self) -> u16 {
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
    pub const fn limit(self, limit: u16) -> Self {
        Self { limit, ..self }
    }

    /// Specifies the offset by given page. \
    /// **Base index is 1/ONE.**
    pub fn page(self, page: u8) -> Self {
        Self {
            offset: if page < 2 { 0 } else { (self.limit * (page as u16 - 1)).into() },
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
