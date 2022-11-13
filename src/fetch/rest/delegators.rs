use serde::{Deserialize, Serialize};

use super::others::{DenomAmount, Pagination, PaginationConfig};
use crate::chain::Chain;

impl Chain {
    /// Returns the withdraw address by given delegator address.
    pub async fn get_delegator_withdraw_address(&self, delegator_addr: &str) -> Result<WithdrawAddressResp, String> {
        let path = format!("/cosmos/distribution/v1beta1/delegators/{delegator_addr}/withdraw_address");

        self.rest_api_request(&path, &[]).await
    }

    /// Returns the rewards of given delegator address.
    pub async fn get_delegator_rewards(&self, delegator_addr: &str) -> Result<DelegatorRewardsResp, String> {
        let path = format!("/cosmos/distribution/v1beta1/delegators/{delegator_addr}/rewards");

        self.rest_api_request(&path, &[]).await
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
pub struct DelegatorReward {
    /// Validator address. Eg: `"cosmosvaloper1c4k24jzduc365kywrsvf5ujz4ya6mwympnc4en"`
    pub validator_address: String,
    /// Array of amounts and denoms.
    pub reward: Vec<DenomAmount>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WithdrawAddressResp {
    /// Delegator withdraw address. Eg: `"cosmos1a3yjj7d3qnx4spgvjcwjq9cw9snrrrhu3rw8nv"`
    pub withdraw_address: String,
}
