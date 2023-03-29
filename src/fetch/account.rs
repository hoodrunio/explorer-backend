use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use tokio::join;
use tonic::transport::Endpoint;

use crate::{
    chain::Chain,
    fetch::cosmos::{
        auth::v1beta1::{query_client::QueryClient, QueryAccountRequest},
        vesting::v1beta1::{BaseVestingAccount, ContinuousVestingAccount, DelayedVestingAccount, PeriodicVestingAccount, PermanentLockedAccount},
    },
    routes::ChainAmountItem,
};

use super::{amount_util::TnrDecimal, others::PaginationConfig};
use prost::Message;

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
                if let Some(comission) = comission_resp.commission.commission.get(0) {
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
    pub async fn get_account_vesting_info(&self, account_address: String) -> Result<InternalVestingAccount, String> {
        let endpoint = Endpoint::from_shared(self.config.grpc_url.clone().unwrap()).unwrap();
        let account_request = QueryAccountRequest { address: account_address };

        let resp = QueryClient::connect(endpoint)
            .await
            .map_err(|e| format!("{}", e))?
            .account(account_request)
            .await
            .map_err(|e| format!("{}", e))?;

        let account_resp = resp.into_inner();
        let account = account_resp.account.ok_or_else(|| "account not found".to_string())?;
        let internal_vesting_account = InternalVestingAccount::from_grpc(self, account).await?;
        Ok(internal_vesting_account)
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InternalVestingAccount {
    pub original_vesting: Option<ChainAmountItem>,
    pub delegated_free: Option<ChainAmountItem>,
    pub delegated_vesting: Option<ChainAmountItem>,
    pub end_time: Option<i64>,
    pub start_time: Option<i64>,
    pub vesting_periods: Vec<InternalPeriod>,
}

impl InternalVestingAccount {
    pub async fn from_grpc(chain: &Chain, account: prost_wkt_types::Any) -> Result<Self, String> {
        let (original_vesting, delegated_free, delegated_vesting, end_time, start_time, vesting_periods) = match account.type_url.as_str() {
            "/cosmos.vesting.v1beta1.PeriodicVestingAccount" => {
                let account = PeriodicVestingAccount::decode(account.value.as_ref()).unwrap();
                let (original_vesting, delegated_free, delegated_vesting, end_time) = match account.base_vesting_account {
                    Some(bva) => InternalVestingAccount::flatten_base_vesting_account(chain, bva).await,
                    None => (None, None, None, None),
                };

                let start_time = account.start_time;
                let vesting_period = {
                    let mut cumulative_period_end_time = start_time;
                    let mut res = vec![];
                    for period in account.vesting_periods {
                        cumulative_period_end_time += period.length;
                        let string_amount = period.amount.iter().map(|d| d.amount.clone()).collect();
                        let chain_amount = chain.string_amount_parser(string_amount, None).await.unwrap_or_default();
                        let vesting_period = InternalPeriod {
                            amount: chain_amount,
                            end_time: cumulative_period_end_time,
                        };

                        res.push(vesting_period);
                    }
                    res
                };

                (
                    original_vesting,
                    delegated_free,
                    delegated_vesting,
                    end_time,
                    Some(start_time),
                    vesting_period,
                )
            }
            "/cosmos.vesting.v1beta1.PermanentLockedAccount" => {
                let account = PermanentLockedAccount::decode(account.value.as_ref()).unwrap();
                let (original_vesting, delegated_free, delegated_vesting, end_time) = match account.base_vesting_account {
                    Some(bva) => InternalVestingAccount::flatten_base_vesting_account(chain, bva).await,
                    None => (None, None, None, None),
                };
                (original_vesting, delegated_free, delegated_vesting, end_time, None, vec![])
            }
            "/cosmos.vesting.v1beta1.DelayedVestingAccount" => {
                let account = DelayedVestingAccount::decode(account.value.as_slice()).map_err(|e| format!("{}", e))?;
                let (original_vesting, delegated_free, delegated_vesting, end_time) = match account.base_vesting_account {
                    Some(bva) => InternalVestingAccount::flatten_base_vesting_account(chain, bva).await,
                    None => (None, None, None, None),
                };
                (original_vesting, delegated_free, delegated_vesting, end_time, None, vec![])
            }
            "/cosmos.vesting.v1beta1.ContinuousVestingAccount" => {
                let account = ContinuousVestingAccount::decode(account.value.as_slice()).map_err(|e| format!("{}", e))?;
                let (original_vesting, delegated_free, delegated_vesting, end_time) = match account.base_vesting_account {
                    Some(bva) => InternalVestingAccount::flatten_base_vesting_account(chain, bva).await,
                    None => (None, None, None, None),
                };
                let start_time = account.start_time;
                (original_vesting, delegated_free, delegated_vesting, end_time, Some(start_time), vec![])
            }
            "/cosmos.vesting.v1beta1.BaseVestingAccount" => {
                let account = BaseVestingAccount::decode(account.value.as_slice()).map_err(|e| format!("{}", e))?;
                let (original_vesting, delegated_free, delegated_vesting, end_time) =
                    InternalVestingAccount::flatten_base_vesting_account(chain, account).await;

                (original_vesting, delegated_free, delegated_vesting, end_time, None, vec![])
            }
            _other => (None, None, None, None, None, vec![]),
        };
        Ok(Self {
            original_vesting,
            delegated_free,
            delegated_vesting,
            end_time,
            start_time,
            vesting_periods,
        })
    }
    async fn flatten_base_vesting_account(
        chain: &Chain,
        account: BaseVestingAccount,
    ) -> (Option<ChainAmountItem>, Option<ChainAmountItem>, Option<ChainAmountItem>, Option<i64>) {
        let end_time = account.end_time;
        let original_vesting = chain.parse_grpc_coins(account.original_vesting).await.sum();
        let delegated_free = chain.parse_grpc_coins(account.delegated_free).await.sum();
        let delegated_vesting = chain.parse_grpc_coins(account.delegated_vesting).await.sum();

        (Some(original_vesting), Some(delegated_free), Some(delegated_vesting), Some(end_time))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InternalPeriod {
    pub end_time: i64,
    pub amount: ChainAmountItem,
}

trait ChainAmountSum {
    fn sum(&self) -> ChainAmountItem;
}

impl ChainAmountSum for Vec<ChainAmountItem> {
    fn sum(&self) -> ChainAmountItem {
        self.iter().fold(ChainAmountItem::default(), |mut acc, x| {
            if let Some(total) = acc.amount.checked_add(x.amount) {
                acc.amount = total;
                acc.ticker = x.ticker.clone();
            }
            acc
        })
    }
}
