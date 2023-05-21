use serde::{Deserialize, Serialize};
use tokio::join;
use tonic::transport::Endpoint;

use crate::fetch::cosmos::distribution::v1beta1::Params as DistributionParams;
use crate::fetch::cosmos::gov::v1beta1::TallyParams;
use crate::fetch::cosmos::gov::v1beta1::VotingParams;
use crate::fetch::cosmos::slashing::v1beta1::Params as SlashingParams;
use crate::fetch::cosmos::staking::v1beta1::Params as StakingParams;
use crate::utils::bytes_to_dec;
use crate::utils::str_to_dec;
use crate::{chain::Chain, routes::OutRestResponse};

impl Chain {
    /// Returns the all parameters of the chain.
    pub async fn get_params_all(&self) -> Result<ChainParams, String> {
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

        Ok(chain_params)
    }

    /// Returns the slashing parameters of the chain.
    pub async fn get_slashing_params(&self) -> Result<OutRestResponse<InternalSlashingParams>, String> {
        use crate::fetch::cosmos::slashing::v1beta1::{query_client::QueryClient, QueryParamsRequest};

        let endpoint = Endpoint::from_shared(self.config.clone().grpc_url.unwrap()).unwrap();

        let req = QueryParamsRequest {};

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .params(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let slashing_params = resp.params.unwrap().try_into()?;

        Ok(OutRestResponse::new(slashing_params, 0))
    }

    /// Returns the staking parameters.
    async fn get_staking_params(&self) -> Result<OutRestResponse<InternalStakingParams>, String> {
        use crate::fetch::cosmos::staking::v1beta1::{query_client::QueryClient, QueryParamsRequest};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryParamsRequest {};

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .params(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let staking_params = resp.params.unwrap().try_into()?;

        Ok(OutRestResponse::new(staking_params, 0))
    }

    /// Returns the voting parameters.
    async fn get_voting_params(&self) -> Result<OutRestResponse<InternalVotingParams>, String> {
        use crate::fetch::cosmos::gov::v1beta1::{query_client::QueryClient, QueryParamsRequest};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryParamsRequest {
            params_type: "voting".to_string(),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .params(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let voting_params = resp.voting_params.unwrap().try_into()?;

        Ok(OutRestResponse::new(voting_params, 0))
    }

    /// Returns the distribution parameters.
    async fn get_distribution_params(&self) -> Result<OutRestResponse<InternalDistributionParams>, String> {
        use crate::fetch::cosmos::distribution::v1beta1::{query_client::QueryClient, QueryParamsRequest};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryParamsRequest {};

        let resp: crate::fetch::cosmos::distribution::v1beta1::QueryParamsResponse = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .params(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let distribution_params = resp.params.unwrap().try_into()?;

        Ok(OutRestResponse::new(distribution_params, 0))
    }

    /// Returns the deposit parameters.
    async fn get_deposit_params(&self) -> Result<OutRestResponse<InternalDepositParams>, String> {
        use crate::fetch::cosmos::gov::v1beta1::{query_client::QueryClient, QueryParamsRequest};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryParamsRequest {
            params_type: "deposit".to_string(),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .params(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let val = resp.deposit_params.unwrap();

        let Some(max_deposit_period) = val.max_deposit_period.map(|d| d.seconds) else {
            return Err("Cannot parse maximum deposit period".to_string());
        };

        let Some(den) = val.min_deposit.get(0) else {
            return Err("There is no min deposit amount.".to_string());
        };

        let Ok(amount) = den.amount.parse::<u128>() else {
            return Err(format!("Cannor parse amount, '{}'.", den.amount))
        };

        let min_deposit = self.calc_amount_u128_to_f64(amount);

        let deposit_params = InternalDepositParams {
            min_deposit,
            max_deposit_period,
        };
        Ok(OutRestResponse::new(deposit_params, 0))
    }

    /// Returns the tallying parameters.
    async fn get_tally_params(&self) -> Result<OutRestResponse<InternalTallyParams>, String> {
        use crate::fetch::cosmos::gov::v1beta1::{query_client::QueryClient, QueryParamsRequest};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryParamsRequest {
            params_type: "tallying".to_string(),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .params(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let tally_params = resp.tally_params.unwrap().try_into()?;

        Ok(OutRestResponse::new(tally_params, 0))
    }
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
        let dec = bytes_to_dec(value.quorum);
        let Ok(quorum) = dec.parse::<f64>() else {
            return Err(format!("Cannot parse quorum, '{}'.", dec));
        };

        let dec = bytes_to_dec(value.threshold);
        let Ok(threshold) = dec.parse::<f64>() else {
            return Err(format!("Cannot parse threshold, '{}'.", dec));
        };

        let dec = bytes_to_dec(value.veto_threshold);
        let Ok(veto_threshold) = dec.parse::<f64>() else {
            return Err(format!("Cannot parse veto_threshold, '{}'.", dec));
        };

        Ok(Self {
            quorum,
            threshold,
            veto_threshold,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalDepositParams {
    /// Min deposit amount.
    pub min_deposit: f64,
    /// Maximum deposit period in seconds. Eg: `0`
    pub max_deposit_period: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalVotingParams {
    /// Voting period in seconds. Eg: `1209600`
    pub voting_period: i64,
}

impl TryFrom<VotingParams> for InternalVotingParams {
    type Error = String;
    fn try_from(value: VotingParams) -> Result<Self, Self::Error> {
        let Some(voting_period) = value.voting_period.map(|d| d.seconds) else {
            return Err(format!("Missing voting period"));
        };

        Ok(Self { voting_period })
    }
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
            community_tax: str_to_dec(params.community_tax.as_str())
                .parse()
                .map_err(|_| format!("Cannot parse community tax, '{}'", params.community_tax))?,
            base_proposer_reward: str_to_dec(params.base_proposer_reward.as_str())
                .parse()
                .map_err(|_| format!("Cannot parse community tax, '{}'", params.base_proposer_reward))?,
            bonus_proposer_reward: str_to_dec(params.bonus_proposer_reward.as_str())
                .parse()
                .map_err(|_| format!("Cannot parse community tax, '{}'", params.bonus_proposer_reward))?,
            withdraw_addr_enabled: params.withdraw_addr_enabled,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalStakingParams {
    /// Unbonding time in seconds. Eg: `1814400`
    pub unbonding_time: i64,
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
        let Some(unbonding_time) = value.unbonding_time.map(|s| s.seconds) else {
            return Err(format!("Missing unbonding time"))
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
pub struct InternalSlashingParams {
    /// Slashing, signed blocks window. Eg: `10000`
    pub signed_blocks_window: i64,
    /// Slashing, minimum signed per window. Eg: `0.050000000000000000`
    pub min_signed_per_window: f64,
    /// Slashing, downtime jail duration in seconds. Eg: `600`
    pub downtime_jail_duration: i64,
    /// Slash fraction double sign. Eg: `0.050000000000000000`
    pub slash_fraction_double_sign: f64,
    /// Slash fraction downtime. Eg: `.000100000000000000`
    pub slash_fraction_downtime: f64,
}

impl TryFrom<SlashingParams> for InternalSlashingParams {
    type Error = String;
    fn try_from(value: SlashingParams) -> Result<Self, Self::Error> {
        let Some(downtime_jail_duration)  = value.downtime_jail_duration.map(|d| d.seconds) else {
            return Err(format!("Missing downtime jail duration."));
        };

        let dec = bytes_to_dec(value.min_signed_per_window.clone());
        let Ok(min_signed_per_window) = dec.parse::<f64>() else {
            return Err(format!("Cannot parse minimum signed per window, '{}'.", dec))
        };

        let dec = bytes_to_dec(value.slash_fraction_double_sign.clone());
        let Ok(slash_fraction_double_sign) = dec.parse::<f64>() else {
            return Err(format!(
                "Cannot parse slash fraction double sign, '{}'.",
                dec
            ))
        };

        let dec = bytes_to_dec(value.slash_fraction_downtime.clone());
        let Ok(slash_fraction_downtime) = dec.parse::<f64>() else {
            return Err(format!("Cannot parse slash fraction downtime, '{}'.", dec))
        };

        Ok(Self {
            signed_blocks_window: value.signed_blocks_window,
            min_signed_per_window,
            downtime_jail_duration,
            slash_fraction_double_sign,
            slash_fraction_downtime,
        })
    }
}
/// The chain params.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChainParams {
    pub staking: ChainParamsStaking,
    pub slashing: ChainParamsSlashing,
    pub gov: ChainParamsGov,
    pub distribution: ChainParamsDistribution,
}

/// The staking params.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainParamsStaking {
    pub unbonding_time: i64,
    pub max_validators: u32,
    pub max_entries: u32,
    pub historical_entries: u32,
    pub bond_denom: String,
}

/// The slashing params.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainParamsSlashing {
    pub signed_blocks_window: i64,
    pub min_signed_per_window: f64,
    pub downtime_jail_duration: i64,
    pub slash_fraction_double_sign: f64,
    pub slash_fraction_downtime: f64,
}

/// The governance params.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainParamsGov {
    pub quorum: f64,
    pub threshold: f64,
    pub min_deposit: f64,
    pub voting_period: i64,
    pub max_deposit_period: i64,
}

/// The governance params.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainParamsDistribution {
    pub community_tax: f64,
    pub base_proposer_reward: f64,
    pub bonus_proposer_reward: f64,
    pub withdraw_addr_enabled: bool,
}
