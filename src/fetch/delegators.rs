use serde::{Deserialize, Serialize};

use super::others::DenomAmount;
use crate::{chain::Chain, routes::OutRestResponse};

impl Chain {
    /// Returns the withdraw address by given delegator address.
    pub async fn get_delegator_withdraw_address(&self, delegator_addr: &str) -> Result<OutRestResponse<String>, String> {
        let path = format!("/cosmos/distribution/v1beta1/delegators/{delegator_addr}/withdraw_address");

        let resp = self.rest_api_request::<WithdrawAddressResp>(&path, &[]).await?;

        let withdraw_address = resp.withdraw_address;

        Ok(OutRestResponse::new(withdraw_address, 0))
    }

    /// Returns the rewards of given delegator address.
    pub async fn get_delegator_rewards(&self, delegator_addr: &str) -> Result<OutRestResponse<InternalDelegatorRewards>, String> {
        let path = format!("/cosmos/distribution/v1beta1/delegators/{delegator_addr}/rewards");

        let resp = self.rest_api_request::<DelegatorRewardsResp>(&path, &[]).await?;

        let delegator_rewards = InternalDelegatorRewards::new(resp, self);

        Ok(OutRestResponse::new(delegator_rewards, 0))
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
    pub total: f64,
}

impl InternalDelegatorRewards {
    fn new(dlg_rwd_resp: DelegatorRewardsResp, chain: &Chain) -> Self {
        let mut rewards = vec![];

        for reward in dlg_rwd_resp.rewards {
            rewards.push(InternalDelegatorReward::new(reward, chain));
        }

        let total = match dlg_rwd_resp.total.get(0) {
            Some(denom_amount) => chain.calc_amount_u128_to_f64(
                denom_amount
                    .amount
                    .split_once('.')
                    .map(|(pri, _)| pri)
                    .unwrap_or(&denom_amount.amount)
                    .parse::<u128>()
                    .unwrap_or(0),
            ),
            None => 0.00,
        };

        Self { total, rewards }
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
    pub validator_address: String,
    pub reward: f64,
}

impl InternalDelegatorReward {
    fn new(delegator_rwd: DelegatorReward, chain: &Chain) -> Self {
        let reward = match delegator_rwd.reward.get(0) {
            Some(denom_amount) => chain.calc_amount_u128_to_f64(
                denom_amount
                    .amount
                    .split_once('.')
                    .map(|(pri, _)| pri)
                    .unwrap_or(&denom_amount.amount)
                    .parse::<u128>()
                    .unwrap_or(0),
            ),
            None => 0.00,
        };

        Self {
            validator_address: delegator_rwd.validator_address,
            reward,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WithdrawAddressResp {
    /// Delegator withdraw address. Eg: `"cosmos1a3yjj7d3qnx4spgvjcwjq9cw9snrrrhu3rw8nv"`
    pub withdraw_address: String,
}
