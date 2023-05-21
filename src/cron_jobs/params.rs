use tokio::join;

use crate::chain::Chain;
use crate::database::{
    ChainDashboardInfoForDb, DistributionParamsForDb, GovParamsForDb, ParamsForDb, SlashingParamsForDb, StakingParamsForDb,
    TokenMarketPriceHistoriesForDb,
};

impl Chain {
    pub async fn cron_job_params(&self) -> Result<(), String> {
        let all_params = self.get_params_all().await?;

        self.database
            .upsert_params(ParamsForDb {
                staking: StakingParamsForDb {
                    unbonding_time: all_params.staking.unbonding_time,
                    max_validators: all_params.staking.max_validators,
                    max_entries: all_params.staking.max_entries,
                    historical_entries: all_params.staking.historical_entries,
                    bond_denom: all_params.staking.bond_denom,
                },
                slashing: SlashingParamsForDb {
                    signed_blocks_window: all_params.slashing.signed_blocks_window,
                    min_signed_per_window: all_params.slashing.min_signed_per_window,
                    downtime_jail_duration: all_params.slashing.downtime_jail_duration,
                    slash_fraction_double_sign: all_params.slashing.slash_fraction_double_sign,
                    slash_fraction_downtime: all_params.slashing.slash_fraction_downtime,
                },
                gov: GovParamsForDb {
                    quorum: all_params.gov.quorum,
                    threshold: all_params.gov.threshold,
                    min_deposit: all_params.gov.min_deposit,
                    voting_period: all_params.gov.voting_period,
                    max_deposit_period: all_params.gov.max_deposit_period,
                },
                distribution: DistributionParamsForDb {
                    community_tax: all_params.distribution.community_tax,
                    base_proposer_reward: all_params.distribution.base_proposer_reward,
                    bonus_proposer_reward: all_params.distribution.bonus_proposer_reward,
                    withdraw_addr_enabled: all_params.distribution.withdraw_addr_enabled,
                },
            })
            .await?;

        Ok(())
    }

    pub async fn cron_job_chain_price_history(&self) -> Result<(), String> {
        let token_id = self
            .config
            .gecko
            .clone()
            .ok_or(format!("{} gecko token id not found", self.config.name))?;

        let market_chart = match self.gecko_token_market_chart(token_id, None, None).await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Error occured on cron job for token prices {}", e);
                return Ok(());
            }
        };
        match self
            .database
            .insert_market_price_history(TokenMarketPriceHistoriesForDb::for_db(market_chart, self.config.name.clone()))
            .await
        {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Error occured on inserting token prices to db {}", e);
            }
        };
        Ok(())
    }

    pub async fn cron_job_chain_dashboard(&self) -> Result<(), String> {
        let (inflation_rate, apr, staking_poll, total_supply, community_poll) = join!(
            self.get_inflation_rate(),
            self.get_apr(),
            self.get_staking_pool(),
            self.get_supply_by_denom(&self.config.main_denom),
            self.get_community_pool(),
        );

        let mut def_inflation_rate = 0.0;
        let mut def_apr = 0.0;
        let mut def_total_unbonded = 0.0;
        let mut def_total_bonded = 0.0;
        let mut def_community_pool = 0;
        let mut def_total_supply = "0".to_string();

        let curren_dashboard_info = self.database.find_chain_dashboard_info().await;
        if let Ok(cd) = curren_dashboard_info {
            def_inflation_rate = cd.inflation_rate;
            def_apr = cd.apr;
            def_total_unbonded = cd.total_unbonded;
            def_total_bonded = cd.total_bonded;
            def_community_pool = cd.community_pool;
            def_total_supply = cd.total_supply;
        }

        let inflation_rate = inflation_rate.unwrap_or(def_inflation_rate);
        let apr = apr.unwrap_or(def_apr);

        let mut total_unbonded = def_total_unbonded;
        let mut total_bonded = def_total_bonded;
        if let Ok(result) = staking_poll {
            total_unbonded = result.value.unbonded as f64;
            total_bonded = result.value.bonded as f64;
        };

        let total_supply = total_supply.map(|res| res.amount.to_string()).unwrap_or(def_total_supply);
        let community_pool = community_poll.map(|res| res.value).unwrap_or(def_community_pool);

        let chain_dashboard_info = ChainDashboardInfoForDb {
            inflation_rate,
            apr,
            total_unbonded,
            total_bonded,
            total_supply,
            community_pool,
        };

        if let Err(e) = self.database.upsert_chain_dashboard_info(chain_dashboard_info).await {
            tracing::error!("Error occured on inserting chain dashboard info to db {}", e);
        };

        Ok(())
    }
}
