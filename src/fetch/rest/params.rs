use serde::{Deserialize, Serialize};

use super::others::{DenomAmount, Pagination, PaginationConfig};
use crate::chain::Chain;

impl Chain {
    /// Returns the staking parameters.
    pub async fn get_staking_params(&self) -> Result<StakingParams, String> {
        self.rest_api_request::<ParamsResp<StakingParams>>("/cosmos/staking/v1beta1/params", &[])
            .await
            .and_then(|res| Ok(res.params))
    }

    /// Returns the slashing parameters of the chain.
    pub async fn get_slashing_params(&self) -> Option<SlashingParams> {
        self.rest_api_request::<ParamsResp<SlashingParams>>("/cosmos/slashing/v1beta1/params", &[])
            .await
            .ok()
            .and_then(|res| Some(res.params))
    }

    /// Returns the voting parameters.
    pub async fn get_voting_params(&self) -> Result<VotingParams, String> {
        self.rest_api_request::<VotingParamsResp>("/cosmos/gov/v1beta1/params/voting", &[])
            .await
            .and_then(|res| Ok(res.voting_params))
    }

    /// Returns the deposit parameters.
    pub async fn get_deposit_params(&self) -> Result<DepositParams, String> {
        self.rest_api_request::<DepositParamsResp>("/cosmos/gov/v1beta1/params/deposit", &[])
            .await
            .and_then(|res| Ok(res.deposit_params))
    }

    /// Returns the tallying parameters.
    pub async fn get_tally_params(&self) -> Result<TallyParams, String> {
        self.rest_api_request::<TallyingParamsResp>("/cosmos/gov/v1beta1/params/tallying", &[])
            .await
            .and_then(|res| Ok(res.tally_params))
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
pub struct ParamsResp<T> {
    /// The parameters.
    pub params: T,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StakingParams {
    /// Unbonding time. Eg: `"1814400s"`
    pub unbonding_time: String,
    /// Maximum number of validators. Eg: `175`
    pub max_validators: usize,
    /// Maximum number of entries. Eg: `7`
    pub max_entries: usize,
    /// Historical number of entries. Eg: `10000`
    pub historical_entries: usize,
    /// Bonding denom. Eg: `"uatom"`
    pub bond_denom: String,
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
