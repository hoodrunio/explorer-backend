use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use tokio::join;

use crate::{chain::Chain, routes::ChainAmountItem};

use super::{amount_util::TnrDecimal, others::PaginationConfig};

impl Chain {
    pub async fn get_account_info(&self, account_address: &String) -> Result<AccountInfo, String> {
        let main_symbol = self.config.main_symbol.clone();
        let default_pagination_config = PaginationConfig::new().limit(10000).page(1);
        let (account_balance_resp, delegation_resp, delegetor_rewards_resp, delegation_unbonding_resp, find_val_db_resp) = join!(
            self.get_account_balance_by_denom(account_address, &self.config.main_denom),
            self.get_delegations(account_address, default_pagination_config),
            self.get_delegator_rewards(account_address),
            self.get_delegations_unbonding(account_address, default_pagination_config),
            self.database.find_validator(doc! {"self_delegate_address": account_address})
        );

        let main_token_balance = account_balance_resp?.value.amount;

        let total_delegate_amount = delegation_resp?.value.iter().fold(TnrDecimal::ZERO, |mut acc, x| {
            if let Some(total) = acc.checked_add(x.amount.amount) {
                acc = total;
            }
            acc
        });

        let total_staking_rewards_amount = delegetor_rewards_resp?.value.rewards.iter().fold(TnrDecimal::ZERO, |mut acc, x| {
            if let Some(total) = acc.checked_add(x.reward.amount) {
                acc = total;
            }
            acc
        });

        let total_unbonding_amount = delegation_unbonding_resp?.value.iter().fold(TnrDecimal::ZERO, |mut acc, x| {
            if let Some(total) = acc.checked_add(x.balance.amount) {
                acc = total;
            }
            acc
        });

        let mut validator_comission = None;
        if let Ok(validator) = find_val_db_resp {
            if let Ok(comission_resp) = self.get_validator_commission(&validator.operator_address).await {
                if let Some(comission) = comission_resp.commission.commission.iter().find(|c| c.denom == self.config.main_denom) {
                    let amount = self.string_amount_parser(comission.amount.clone(), Some(comission.denom.clone())).await?;
                    validator_comission = Some(amount);
                };
            };
        }

        let total_amount = main_token_balance
            .amount
            .checked_add(total_delegate_amount)
            .unwrap_or_default()
            .checked_add(total_unbonding_amount)
            .unwrap_or_default()
            .checked_add(total_staking_rewards_amount)
            .unwrap_or_default()
            .checked_add(total_staking_rewards_amount)
            .unwrap_or_default()
            .checked_add(validator_comission.clone().map(|c| c.amount).unwrap_or_default())
            .unwrap_or_default();

        Ok(AccountInfo {
            native_token_balance: NativeTokenBalanceInfo {
                available: main_token_balance,
                delegated: ChainAmountItem::sync_with_ticker(total_delegate_amount, main_symbol.clone()),
                unbonding: ChainAmountItem::sync_with_ticker(total_unbonding_amount, main_symbol.clone()),
                staking_reward: ChainAmountItem::sync_with_ticker(total_staking_rewards_amount, main_symbol.clone()),
                delegatable_vesting: ChainAmountItem::default(),
                validator_comission,
            },
            total_amount: ChainAmountItem::sync_with_ticker(total_amount, main_symbol.clone()),
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AccountInfo {
    pub native_token_balance: NativeTokenBalanceInfo,
    pub total_amount: ChainAmountItem,
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
