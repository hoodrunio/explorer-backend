use crate::chain::Chain;

impl Chain {
    /// Returns the APR rate of the chain.
    ///
    /// Reference: https://github.com/bro-n-bro/prometheus_exporter/blob/main/docs/APR%20calcucation.md#non-epoch-cosmos-based--blockchains
    pub fn get_apr(&self) -> Result<f64, String> {
        let inflation = 0.0;
        let community_tax = 0.0;
        let bonded_tokens_ratio = 0.0;

        // Constant declarations
        const secs_in_year: f64 = 31561920.0;

        // If the chain has epochs.
        if self.inner.epoch {
            // We will get those below from the database.
            let epoch_provisions = 0.0;
            let community_tax = 0.0;
            let bonded_tokens_amount = 0.0;

            // Calculate annual provisions.
            let annual_provisions = epoch_provisions * 365.3;

            match self.inner.name {
                "osmosis" => {
                    let staking_rewards_factor = 0.25;

                    let apr = annual_provisions * staking_rewards_factor / bonded_tokens_amount;

                    Ok(apr)
                }
                "evmos" => {
                    let staking_rewards_factor = 0.0; // will be fetched. https://lcd.evmos-9001-2.bronbro.io/evmos/inflation/v1/params

                    let apr = annual_provisions * staking_rewards_factor / bonded_tokens_amount;

                    Ok(apr)
                }
                chain_name => Err(format!("APR for {chain_name} is not implemented.")),
            }
        } else {
            // We will get those below from the database.
            let annual_provisions = 0.0;
            let community_tax = 0.0;
            let bonded_tokens_amount = 0.0;
            let block_per_year = 0.0;
            let avg_block_time_24h = 0.0;

            // Calculate how many blocks will be created in a year with the speed same as last 24h.
            let current_block_per_year = secs_in_year / avg_block_time_24h;

            // Calculate correction.
            let correction_annual_coefficient = current_block_per_year / block_per_year;

            let apr = (annual_provisions * (1.0 - community_tax) / bonded_tokens_amount) * correction_annual_coefficient;

            Ok(apr)
        }
    }
}

#[test]
fn a() {}
