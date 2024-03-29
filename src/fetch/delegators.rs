use super::others::DenomAmount;
use crate::chain::Chain;
use crate::fetch::cosmos::distribution::v1beta1::{DelegationDelegatorReward, QueryDelegationTotalRewardsResponse};
use crate::routes::ChainAmountItem;
use serde::{Deserialize, Serialize};
use tonic::transport::Endpoint;

impl Chain {
    /// Returns the withdraw address by given delegator address.
    pub async fn get_delegator_withdraw_address(&self, delegator_addr: &str) -> Result<String, String> {
        use crate::fetch::cosmos::distribution::v1beta1::{query_client::QueryClient, QueryDelegatorWithdrawAddressRequest};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryDelegatorWithdrawAddressRequest {
            delegator_address: delegator_addr.to_string(),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .delegator_withdraw_address(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let withdraw_address = resp.withdraw_address;

        Ok(withdraw_address)
    }

    /// Returns the rewards of given delegator address.
    pub async fn get_delegator_rewards(&self, delegator_addr: &str) -> Result<InternalDelegatorRewards, String> {
        use crate::fetch::cosmos::distribution::v1beta1::{query_client::QueryClient, QueryDelegationTotalRewardsRequest};

        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();

        let req = QueryDelegationTotalRewardsRequest {
            delegator_address: delegator_addr.to_string(),
        };

        let resp = QueryClient::connect(endpoint)
            .await
            .unwrap()
            .delegation_total_rewards(req)
            .await
            .map_err(|e| format!("{}", e))?
            .into_inner();

        let delegator_rewards = InternalDelegatorRewards::new(resp, &self).await?;

        Ok(delegator_rewards)
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
    pub total: ChainAmountItem,
}

impl InternalDelegatorRewards {
    async fn new(dlg_rwd_resp: QueryDelegationTotalRewardsResponse, chain: &Chain) -> Result<Self, String> {
        let default_reward = ChainAmountItem::default();
        let mut rewards = vec![];

        for reward in dlg_rwd_resp.rewards {
            rewards.push(InternalDelegatorReward::new(reward, chain).await?);
        }

        let total = match dlg_rwd_resp.total.get(0) {
            Some(denom_amount) => {
                let amount = denom_amount.amount.split_once('.').map(|(pri, _)| pri).unwrap_or(&denom_amount.amount);

                chain
                    .string_amount_parser(String::from(amount), Some(denom_amount.denom.clone()))
                    .await
                    .unwrap_or(default_reward)
            }
            None => default_reward,
        };

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
    pub validator_address: String,
    pub reward: ChainAmountItem,
}

impl InternalDelegatorReward {
    async fn new(delegator_rwd: DelegationDelegatorReward, chain: &Chain) -> Result<Self, String> {
        let default_reward = ChainAmountItem::default();
        let reward = match delegator_rwd.reward.get(0) {
            Some(denom_amount) => {
                let amount = denom_amount.amount.split_once('.').map(|(pri, _)| pri).unwrap_or(&denom_amount.amount);

                chain
                    .string_amount_parser(String::from(amount), Some(denom_amount.denom.clone()))
                    .await
                    .unwrap_or(default_reward)
            }
            None => default_reward,
        };

        Ok(Self {
            validator_address: delegator_rwd.validator_address,
            reward,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WithdrawAddressResp {
    /// Delegator withdraw address. Eg: `"cosmos1a3yjj7d3qnx4spgvjcwjq9cw9snrrrhu3rw8nv"`
    pub withdraw_address: String,
}
