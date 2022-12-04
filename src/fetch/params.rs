use serde::{Deserialize, Serialize};
use tokio::join;

use super::others::DenomAmount;
use crate::{
    chain::Chain,
    data::params::{ChainParams, ChainParamsDistribution, ChainParamsGov, ChainParamsSlashing, ChainParamsStaking},
    routes::OutRestResponse,
};

impl Chain {
    /// Returns the all parameters of the chain.
    pub async fn get_params_all(&self) -> Result<OutRestResponse<ChainParams>, String> {
        let (tally, voting, deposit, distribution, staking, slashing) = join!(
            self.get_tally_params(),
            self.get_voting_params(),
            self.get_deposit_params(),
            self.get_distribution_params(),
            self.get_staking_params(),
            self.get_slashing_params()
        );

        let tally = tally?.value;
        let voting = voting?.value;
        let deposit = deposit?.value;
        let distribution = distribution?.value;
        let staking = staking?.value;
        let slashing = slashing?.value;

        let chain_params = ChainParams {
            distribution: ChainParamsDistribution {
                community_tax: distribution.community_tax,
                base_proposer_reward: distribution.base_proposer_reward,
                bonus_proposer_reward: distribution.bonus_proposer_reward,
                withdraw_addr_enabled: distribution.withdraw_addr_enabled,
            },
            gov: ChainParamsGov {
                max_deposit_period: deposit.max_deposit_period,
                min_deposit: deposit.min_deposit,
                quorum: tally.quorum,
                threshold: tally.threshold,
                voting_period: voting.voting_period,
            },
            slashing: ChainParamsSlashing {
                downtime_jail_duration: slashing.downtime_jail_duration,

                min_signed_per_window: slashing.min_signed_per_window,

                signed_blocks_window: slashing.signed_blocks_window,
                slash_fraction_double_sign: slashing.slash_fraction_double_sign,
                slash_fraction_downtime: slashing.slash_fraction_downtime,
            },
            staking: ChainParamsStaking {
                bond_denom: staking.bond_denom,
                historical_entries: staking.historical_entries,
                max_entries: staking.max_entries,
                max_validators: staking.max_validators,
                unbonding_time: staking.unbonding_time,
            },
        };

        Ok(OutRestResponse::new(chain_params, 0))
    }

    /// Returns the slashing parameters of the chain.
    async fn get_slashing_params(&self) -> Result<OutRestResponse<InternalSlashingParams>, String> {
        let resp = self
            .rest_api_request::<ParamsResp<SlashingParams>>("/cosmos/slashing/v1beta1/params", &[])
            .await?;

        let slashing_params = resp.params.try_into()?;

        Ok(OutRestResponse::new(slashing_params, 0))
    }

    /// Returns the staking parameters.
    async fn get_staking_params(&self) -> Result<OutRestResponse<InternalStakingParams>, String> {
        let resp = self
            .rest_api_request::<ParamsResp<StakingParams>>("/cosmos/staking/v1beta1/params", &[])
            .await?;

        let staking_params = resp.params.try_into()?;

        Ok(OutRestResponse::new(staking_params, 0))
    }

    /// Returns the voting parameters.
    async fn get_voting_params(&self) -> Result<OutRestResponse<InternalVotingParams>, String> {
        let resp = self
            .rest_api_request::<VotingParamsResp>("/cosmos/gov/v1beta1/params/voting", &[])
            .await?;

        let voting_params = resp.voting_params.try_into()?;

        Ok(OutRestResponse::new(voting_params, 0))
    }

    /// Returns the distribution parameters.
    async fn get_distribution_params(&self) -> Result<OutRestResponse<InternalDistributionParams>, String> {
        let resp = self
            .rest_api_request::<ParamsResp<DistributionParams>>("/cosmos/distribution/v1beta1/params", &[])
            .await?;

        let distribution_params = resp.params.try_into()?;

        Ok(OutRestResponse::new(distribution_params, 0))
    }

    /// Returns the deposit parameters.
    async fn get_deposit_params(&self) -> Result<OutRestResponse<InternalDepositParams>, String> {
        let resp = self
            .rest_api_request::<DepositParamsResp>("/cosmos/gov/v1beta1/params/deposit", &[])
            .await?;

        let deposit_params = InternalDepositParams {
            max_deposit_period: if resp.deposit_params.max_deposit_period.ends_with('s') {
                match resp.deposit_params.max_deposit_period[..resp.deposit_params.max_deposit_period.len() - 2].parse() {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(format!(
                            "Cannot parse maximum deposit period, '{}'.",
                            resp.deposit_params.max_deposit_period
                        ))
                    }
                }
            } else {
                return Err(format!(
                    "Maximum deposit params couldn't be parsed, {}.",
                    resp.deposit_params.max_deposit_period
                ));
            },
            min_deposit: match resp.deposit_params.min_deposit.get(0) {
                Some(den) => match den.amount.parse::<u128>() {
                    Ok(amount) => self.calc_amount_u128_to_f64(amount),
                    Err(_) => return Err(format!("Cannor parse amount, '{}'.", den.amount)),
                },
                None => return Err("There is no min deposit amount.".to_string()),
            },
        };

        Ok(OutRestResponse::new(deposit_params, 0))
    }

    /// Returns the tallying parameters.
    async fn get_tally_params(&self) -> Result<OutRestResponse<InternalTallyParams>, String> {
        let resp = self
            .rest_api_request::<TallyingParamsResp>("/cosmos/gov/v1beta1/params/tallying", &[])
            .await?;

        let tally_params = resp.tally_params.try_into()?;

        Ok(OutRestResponse::new(tally_params, 0))
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
        let voting_period: u32 = if value.voting_period.ends_with('s') {
            match value.voting_period[..value.voting_period.len() - 2].parse() {
                Ok(v) => v,
                Err(_) => return Err(format!("Cannot parse voting period, '{}'.", value.voting_period)),
            }
        } else {
            return Err(format!("Voting period couldn't be parsed, {}.", value.voting_period));
        };
        Ok(Self { voting_period })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DistributionParams {
    /// Community tax. Eg: `"0.000000000000000000"`
    community_tax: String,
    /// Base proposer reward. Eg: `"0.000000000000000000"`
    base_proposer_reward: String,
    /// Bonus proposer reward. Eg: `"0.000000000000000000"`
    bonus_proposer_reward: String,
    /// Withdraw addrress enabled. Eg: `true`
    withdraw_addr_enabled: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalDistributionParams {
    /// Community tax. Eg: `0.000000000000000000`
    community_tax: f64,
    /// Base proposer reward. Eg: `0.000000000000000000`
    base_proposer_reward: f64,
    /// Bonus proposer reward. Eg: `0.000000000000000000`
    bonus_proposer_reward: f64,
    /// Withdraw addrress enabled. Eg: `true`
    withdraw_addr_enabled: bool,
}

impl TryFrom<DistributionParams> for InternalDistributionParams {
    type Error = String;
    fn try_from(params: DistributionParams) -> Result<Self, Self::Error> {
        Ok(Self {
            community_tax: params
                .community_tax
                .parse()
                .map_err(|_| format!("Cannot parse community tax, '{}'", params.community_tax))?,
            base_proposer_reward: params
                .base_proposer_reward
                .parse()
                .map_err(|_| format!("Cannot parse community tax, '{}'", params.base_proposer_reward))?,
            bonus_proposer_reward: params
                .bonus_proposer_reward
                .parse()
                .map_err(|_| format!("Cannot parse community tax, '{}'", params.bonus_proposer_reward))?,
            withdraw_addr_enabled: params.withdraw_addr_enabled,
        })
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
        let unbonding_time: u32 = if value.unbonding_time.ends_with('s') {
            match value.unbonding_time[..value.unbonding_time.len() - 2].parse() {
                Ok(v) => v,
                Err(_) => return Err(format!("Cannot parse unbonding time, '{}'.", value.unbonding_time)),
            }
        } else {
            return Err(format!("Unbonding time couldn't be parsed, {}.", value.unbonding_time));
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
        let downtime_jail_duration: u32 = if value.downtime_jail_duration.ends_with('s') {
            match value.downtime_jail_duration[..value.downtime_jail_duration.len() - 2].parse() {
                Ok(v) => v,
                Err(_) => return Err(format!("Cannot parse downtime jail time, '{}'.", value.downtime_jail_duration)),
            }
        } else {
            return Err(format!("Downtime jail couldn't be parsed, {}.", value.downtime_jail_duration));
        };

        let signed_blocks_window = match value.signed_blocks_window.parse() {
            Ok(v) => v,
            Err(_) => return Err(format!("Cannot parse signed blocks window, '{}'.", value.signed_blocks_window)),
        };

        let min_signed_per_window = match value.min_signed_per_window.parse() {
            Ok(v) => v,
            Err(_) => return Err(format!("Cannot parse minimum signed per window, '{}'.", value.min_signed_per_window)),
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
            Err(_) => return Err(format!("Cannot parse slash fraction downtime, '{}'.", value.slash_fraction_downtime)),
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
