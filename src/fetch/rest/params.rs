use serde::{Deserialize, Serialize};

use super::others::DenomAmount;
use crate::chain::Chain;

impl Chain {
    /// Returns the slashing parameters of the chain.
    pub async fn get_slashing_params(&self) -> Option<InternalSlashingParams> {
        self.rest_api_request::<ParamsResp<SlashingParams>>("/cosmos/slashing/v1beta1/params", &[])
            .await
            .ok()
            .and_then(|res| res.params.try_into().ok())
    }

    /// Returns the staking parameters.
    pub async fn get_staking_params(&self) -> Result<InternalStakingParams, String> {
        self.rest_api_request::<ParamsResp<StakingParams>>("/cosmos/staking/v1beta1/params", &[])
            .await
            .and_then(|res| res.params.try_into())
    }

    /// Returns the voting parameters.
    pub async fn get_voting_params(&self) -> Result<InternalVotingParams, String> {
        self.rest_api_request::<VotingParamsResp>("/cosmos/gov/v1beta1/params/voting", &[])
            .await
            .and_then(|res| res.voting_params.try_into())
    }

    /// Returns the deposit parameters.
    pub async fn get_deposit_params(&self) -> Result<InternalDepositParams, String> {
        self.rest_api_request::<DepositParamsResp>("/cosmos/gov/v1beta1/params/deposit", &[])
            .await
            .and_then(|res| InternalDepositParams::try_from(res.deposit_params, self.decimals))
    }

    /// Returns the tallying parameters.
    pub async fn get_tally_params(&self) -> Result<InternalTallyParams, String> {
        self.rest_api_request::<TallyingParamsResp>("/cosmos/gov/v1beta1/params/tallying", &[])
            .await
            .and_then(|res| res.tally_params.try_into())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TallyingParamsResp {
    /// Tally parameters.
    pub tally_params: TallyParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TallyParams {
    /// Quorum. Eg: `"0.400000000000000000"`
    pub quorum: String,
    /// Threshold. Eg: `"0.500000000000000000"`
    pub threshold: String,
    /// Veto threshold. Eg: `"0.334000000000000000"`
    pub veto_threshold: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalTallyParams {
    /// Quorum. Eg: `0.400000000000000000`
    pub quorum: f64,
    /// Threshold. Eg: `0.500000000000000000`
    pub threshold: f64,
    /// Veto threshold. Eg: `0.334000000000000000`
    pub veto_threshold: f64,
}

impl TryFrom<TallyParams> for InternalTallyParams {
    type Error = String;
    fn try_from(value: TallyParams) -> Result<Self, Self::Error> {
        let quorum: f64 = match value.quorum.parse() {
            Ok(v) => v,
            Err(_) => return Err(format!("Cannot parse quorum, '{}'.", value.quorum)),
        };
        let threshold: f64 = match value.threshold.parse() {
            Ok(v) => v,
            Err(_) => return Err(format!("Cannot parse threshold, '{}'.", value.threshold)),
        };
        let veto_threshold: f64 = match value.veto_threshold.parse() {
            Ok(v) => v,
            Err(_) => return Err(format!("Cannot parse veto threshold, '{}'.", value.veto_threshold)),
        };

        Ok(Self {
            quorum,
            threshold,
            veto_threshold,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DepositParamsResp {
    /// Deposit parameters.
    pub deposit_params: DepositParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DepositParams {
    /// Array of denoms and amounts.
    pub min_deposit: Vec<DenomAmount>,
    /// Maximum deposit period. Eg: `"0s"`
    pub max_deposit_period: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalDepositParams {
    /// Min deposit amount.
    pub min_deposit: f64,
    /// Maximum deposit period in seconds. Eg: `0`
    pub max_deposit_period: u32,
}

impl InternalDepositParams {
    fn try_from(value: DepositParams, decimals: u8) -> Result<Self, String> {
        let max_deposit_period: u32 = if value.max_deposit_period.ends_with("s") {
            match value.max_deposit_period[..value.max_deposit_period.len() - 2].parse() {
                Ok(v) => v,
                Err(_) => {
                    return Err(format!(
                        "Cannot parse maximum deposit period, '{}'.",
                        value.max_deposit_period
                    ))
                }
            }
        } else {
            return Err(format!("Maximum deposit params couldn't be parsed!"));
        };
        let min_deposit = match value.min_deposit.get(0) {
            Some(den) => match den.amount.parse::<u128>() {
                Ok(v) => (v / 10_u128.pow(decimals.into())) as f64,
                Err(_) => return Err(format!("Cannor parse amount, '{}'.", den.amount)),
            },
            None => return Err(format!("There is no min deposit amount.")),
        };

        Ok(Self {
            max_deposit_period,
            min_deposit,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct VotingParamsResp {
    /// Voting parameters.
    pub voting_params: VotingParams,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct VotingParams {
    /// Voting period. Eg: `"1209600s"`
    pub voting_period: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalVotingParams {
    /// Voting period in seconds. Eg: `1209600`
    pub voting_period: u32,
}

impl TryFrom<VotingParams> for InternalVotingParams {
    type Error = String;
    fn try_from(value: VotingParams) -> Result<Self, Self::Error> {
        let voting_period: u32 = if value.voting_period.ends_with("s") {
            match value.voting_period[..value.voting_period.len() - 2].parse() {
                Ok(v) => v,
                Err(_) => return Err(format!("Cannot parse voting period, '{}'.", value.voting_period)),
            }
        } else {
            return Err(format!("Voting period couldn't be parsed!"));
        };
        Ok(Self { voting_period })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StakingParams {
    /// Unbonding time. Eg: `"1814400s"`
    pub unbonding_time: String,
    /// Maximum number of validators. Eg: `175`
    pub max_validators: u32,
    /// Maximum number of entries. Eg: `7`
    pub max_entries: u32,
    /// Historical number of entries. Eg: `10000`
    pub historical_entries: u32,
    /// Bonding denom. Eg: `"uatom"`
    pub bond_denom: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalStakingParams {
    /// Unbonding time in seconds. Eg: `1814400`
    pub unbonding_time: u32,
    /// Maximum number of validators. Eg: `175`
    pub max_validators: u32,
    /// Maximum number of entries. Eg: `7`
    pub max_entries: u32,
    /// Historical number of entries. Eg: `10000`
    pub historical_entries: u32,
    /// Bonding denom. Eg: `"uatom"`
    pub bond_denom: String,
}

impl TryFrom<StakingParams> for InternalStakingParams {
    type Error = String;
    fn try_from(value: StakingParams) -> Result<Self, Self::Error> {
        let unbonding_time: u32 = if value.unbonding_time.ends_with("s") {
            match value.unbonding_time[..value.unbonding_time.len() - 2].parse() {
                Ok(v) => v,
                Err(_) => return Err(format!("Cannot parse unbonding time, '{}'.", value.unbonding_time)),
            }
        } else {
            return Err(format!("Unbonding time couldn't be parsed!"));
        };

        Ok(Self {
            unbonding_time,
            max_validators: value.max_validators,
            max_entries: value.max_entries,
            historical_entries: value.historical_entries,
            bond_denom: value.bond_denom,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SlashingParams {
    /// Slashing, signed blocks window. Eg: `"10000"`
    pub signed_blocks_window: String,
    /// Slashing, minimum signed per window. Eg: `"0.050000000000000000"`
    pub min_signed_per_window: String,
    /// Slashing, downtime jail duration. Eg: `"600s"`
    pub downtime_jail_duration: String,
    /// Slash fraction double sign. Eg: `"0.050000000000000000"`
    pub slash_fraction_double_sign: String,
    /// Slash fraction downtime. Eg: `"0.000100000000000000"`
    pub slash_fraction_downtime: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalSlashingParams {
    /// Slashing, signed blocks window. Eg: `10000`
    pub signed_blocks_window: u32,
    /// Slashing, minimum signed per window. Eg: `0.050000000000000000`
    pub min_signed_per_window: f64,
    /// Slashing, downtime jail duration in seconds. Eg: `600`
    pub downtime_jail_duration: u32,
    /// Slash fraction double sign. Eg: `0.050000000000000000`
    pub slash_fraction_double_sign: f64,
    /// Slash fraction downtime. Eg: `.000100000000000000`
    pub slash_fraction_downtime: f64,
}

impl TryFrom<SlashingParams> for InternalSlashingParams {
    type Error = String;
    fn try_from(value: SlashingParams) -> Result<Self, Self::Error> {
        let downtime_jail_duration: u32 = if value.downtime_jail_duration.ends_with("s") {
            match value.downtime_jail_duration[..value.downtime_jail_duration.len() - 2].parse() {
                Ok(v) => v,
                Err(_) => {
                    return Err(format!(
                        "Cannot parse downtime jail time, '{}'.",
                        value.downtime_jail_duration
                    ))
                }
            }
        } else {
            return Err(format!("Downtime jail couldn't be parsed!"));
        };

        let signed_blocks_window = match value.signed_blocks_window.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(format!(
                    "Cannot parse signed blocks window, '{}'.",
                    value.signed_blocks_window
                ))
            }
        };

        let min_signed_per_window = match value.min_signed_per_window.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(format!(
                    "Cannot parse minimum signed per window, '{}'.",
                    value.min_signed_per_window
                ))
            }
        };

        let slash_fraction_double_sign = match value.slash_fraction_double_sign.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(format!(
                    "Cannot parse slash fraction double sign, '{}'.",
                    value.slash_fraction_double_sign
                ))
            }
        };

        let slash_fraction_downtime = match value.slash_fraction_downtime.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(format!(
                    "Cannot parse slash fraction downtime, '{}'.",
                    value.slash_fraction_downtime
                ))
            }
        };

        Ok(Self {
            signed_blocks_window,
            min_signed_per_window,
            downtime_jail_duration,
            slash_fraction_double_sign,
            slash_fraction_downtime,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ParamsResp<T> {
    /// The parameters.
    pub params: T,
}
