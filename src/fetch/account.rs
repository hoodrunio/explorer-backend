use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{chain::Chain, routes::ChainAmountItem};

use super::{amount_util::TnrDecimal, others::PaginationConfig};

impl Chain {
    pub async fn get_account_info(&self, account_address: &String) -> Result<AccountInfo, String> {
        let default_pagination_config = PaginationConfig::new().limit(10000).page(1);

        let main_token_balance = self
            .get_account_balance_by_denom(account_address, &self.config.main_denom)
            .await?
            .value
            .amount;

        let total_delegate_amount =
            self.get_delegations(account_address, default_pagination_config)
                .await?
                .value
                .iter()
                .fold(TnrDecimal::ZERO, |mut acc, x| {
                    if let Some(total) = acc.checked_add(x.amount.amount) {
                        acc = total;
                    }
                    acc
                });

        let total_staking_rewards_amount =
            self.get_delegator_rewards(account_address)
                .await?
                .value
                .rewards
                .iter()
                .fold(TnrDecimal::ZERO, |mut acc, x| {
                    if let Some(total) = acc.checked_add(x.reward.amount) {
                        acc = total;
                    }
                    acc
                });

        let total_unbonding_amount = self
            .get_delegations_unbonding(account_address, default_pagination_config)
            .await?
            .value
            .iter()
            .fold(TnrDecimal::ZERO, |mut acc, x| {
                if let Some(total) = acc.checked_add(x.balance.amount) {
                    acc = total;
                }
                acc
            });

        let mut validator_comission = None;
        if let Ok(validator) = self.database.find_validator(doc! {"self_delegate_address": account_address}).await {
            if let Ok(comission_resp) = self.get_validator_commission(&validator.operator_address).await {
                if let Some(comission) = comission_resp.commission.commission.get(0) {
                    let amount = self.string_amount_parser(comission.amount.clone(), Some(comission.denom.clone())).await?;
                    validator_comission = Some(amount);
                };
            };
        }

        Ok(AccountInfo {
            native_token_balance: NativeTokenBalanceInfo {
                available: main_token_balance,
                delegated: ChainAmountItem {
                    amount: total_delegate_amount,
                    ticker: self.config.main_symbol.clone(),
                },
                unbonding: ChainAmountItem {
                    amount: total_unbonding_amount,
                    ticker: self.config.main_symbol.clone(),
                },
                staking_reward: ChainAmountItem {
                    amount: total_staking_rewards_amount,
                    ticker: self.config.main_symbol.clone(),
                },
                delegatable_vesting: ChainAmountItem::default(),
                validator_comission,
            },
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AccountInfo {
    pub native_token_balance: NativeTokenBalanceInfo,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NativeTokenBalanceInfo {
    pub available: ChainAmountItem,
    pub delegated: ChainAmountItem,
    pub unbonding: ChainAmountItem,
    pub staking_reward: ChainAmountItem,
    pub delegatable_vesting: ChainAmountItem,
    pub validator_comission: Option<ChainAmountItem>,
}
