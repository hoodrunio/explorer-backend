use crate::chain::Chain;
use tokio::join;

impl Chain {
    /// Updates general data.
    pub async fn update_data(&self) {
        let chain = self.clone();
        // Make requests at the same time.
        if let (Ok(inflation_resp), Ok(params_resp), Ok(community_pool_resp), Ok(staking_pool)) = join!(
            chain.get_inflation_rate(),
            chain.get_params_all(),
            chain.get_community_pool(),
            chain.get_staking_pool(),
        ) {
            // Update inflation data.
            if let Ok(mut inflation) = chain.inner.data.inflation.lock() {
                *inflation = inflation_resp.value;
            };

            // Update params data.
            if let Ok(mut params) = chain.inner.data.params.lock() {
                *params = params_resp.value;
            };

            // Update community pool data.
            if let Ok(mut community_pool) = chain.inner.data.pool.lock() {
                *community_pool = community_pool_resp.value;
            };

            // Update bonded token supply data.
            if let Ok(mut bonded) = chain.inner.data.bonded.lock() {
                *bonded = staking_pool.value.bonded;
            };

            // Update unbonded token supply data.
            if let Ok(mut unbonded) = chain.inner.data.unbonded.lock() {
                *unbonded = staking_pool.value.unbonded;
            };
        };
    }

    /// Updates the native coin price, supply and the chart.
    pub async fn update_price(&self, new_price: Option<&f64>) {
        self.update_supply().await;
        if let Some(new_price) = new_price {
            if let Ok(mut price) = self.inner.data.price.lock() {
                *price = *new_price;
            }

            if let Ok(mut chart) = self.inner.data.chart.lock() {
                chart.add_new(*new_price);
            }

            if let Ok(supply) = self.inner.data.supply.lock() {
                let new_mcap = (*new_price * *supply as f64) as u64;

                if let Ok(mut mcap) = self.inner.data.mcap.lock() {
                    *mcap = new_mcap;
                }
            }
        }
    }

    /// Updates the supply and the market cap of the native coin.
    async fn update_supply(&self) {
        match self.get_supply_of_native_coin().await {
            Ok(resp) => {
                let new_supply = self.calc_amount_u128_to_u64(resp.value.amount);

                if let Ok(mut supply) = self.inner.data.supply.lock() {
                    *supply = new_supply;
                }
            }
            Err(error) => println!("{}", error),
        }
    }
}
