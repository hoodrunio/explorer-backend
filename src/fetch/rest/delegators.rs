use serde::{Deserialize, Serialize};

use super::others::{DenomAmount, InternalDenomAmount, Pagination, PaginationConfig};
use crate::{chain::Chain, routes::rest::OutRestResponse};

impl Chain {
    /// Returns the withdraw address by given delegator address.
    pub async fn get_delegator_withdraw_address(&self, delegator_addr: &str) -> Result<OutRestResponse<WithdrawAddressResp>, String> {
        let path = format!("/cosmos/distribution/v1beta1/delegators/{delegator_addr}/withdraw_address");

        let resp = self.rest_api_request::<WithdrawAddressResp>(&path, &[]).await?;

        OutRestResponse::new(resp, 0)
    }

    /// Returns the rewards of given delegator address.
    pub async fn get_delegator_rewards(&self, delegator_addr: &str) -> Result<OutRestResponse<InternalDelegatorRewards>, String> {
        let path = format!("/cosmos/distribution/v1beta1/delegators/{delegator_addr}/rewards");

        let resp = self.rest_api_request::<DelegatorRewardsResp>(&path, &[]).await?;

        let delegator_rewards = resp.try_into()?;

        OutRestResponse::new(delegator_rewards, 0)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DelegatorRewardsResp {
    /// Array of rewards.
    pub rewards: Vec<DelegatorReward>,
    /// Array of amounts and denoms.
    pub total: Vec<DenomAmount>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalDelegatorRewards {
    /// Array of rewards.
    pub rewards: Vec<InternalDelegatorReward>,
    /// Array of amounts and denoms.
    pub total: Vec<InternalDenomAmount>,
}

impl TryFrom<DelegatorRewardsResp> for InternalDelegatorRewards {
    type Error = String;
    fn try_from(value: DelegatorRewardsResp) -> Result<Self, Self::Error> {
        let mut rewards = vec![];
        let mut total = vec![];

        for reward in value.rewards {
            rewards.push(reward.try_into()?);
        }

        for denom_amount in value.total {
            total.push(denom_amount.try_into()?);
        }

        Ok(Self { total, rewards })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DelegatorReward {
    /// Validator address. Eg: `"cosmosvaloper1c4k24jzduc365kywrsvf5ujz4ya6mwympnc4en"`
    pub validator_address: String,
    /// Array of amounts and denoms.
    pub reward: Vec<DenomAmount>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalDelegatorReward {
    /// Validator address. Eg: `"cosmosvaloper1c4k24jzduc365kywrsvf5ujz4ya6mwympnc4en"`
    pub validator_address: String,
    /// Array of amounts and denoms.
    pub reward: Vec<InternalDenomAmount>,
}

impl TryFrom<DelegatorReward> for InternalDelegatorReward {
    type Error = String;
    fn try_from(value: DelegatorReward) -> Result<Self, Self::Error> {
        let mut reward = vec![];

        for denom_amount in value.reward {
            reward.push(denom_amount.try_into()?);
        }

        Ok(Self {
            validator_address: value.validator_address,
            reward,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WithdrawAddressResp {
    /// Delegator withdraw address. Eg: `"cosmos1a3yjj7d3qnx4spgvjcwjq9cw9snrrrhu3rw8nv"`
    pub withdraw_address: String,
}
