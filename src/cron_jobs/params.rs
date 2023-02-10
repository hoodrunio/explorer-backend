use crate::chain::Chain;
use crate::database::{DistributionParamsForDb, GovParamsForDb, ParamsForDb, SlashingParamsForDb, StakingParamsForDb};

impl Chain {
    pub async fn cron_job_params(&self) -> Result<(), String> {
        let resp = self.get_params_all().await?;

        let all_params = resp.value;
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
}
