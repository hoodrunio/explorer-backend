use std::time::Duration;
use tokio::{spawn, time::sleep};

use crate::chain::Chain;

impl Chain {
    pub fn cron_jobs_all(&self) {
        // ALL THE cron jobs will be spawned here.

        // Validator cron job.
        let duration = Duration::from_secs(120);
        let clone_chain = self.clone();
        spawn(async move {
            loop {
                tokio::join!(
                    async {
                        let name = clone_chain.config.name.clone();
                        if let Err(error) = clone_chain.cron_job_validator().await {
                            tracing::error!("Chain {name} validator cronjob error: {error}")
                        };
                    },
                    async {
                        let name = clone_chain.config.name.clone();
                        if let Err(error) = clone_chain.cron_job_params().await {
                            tracing::error!("Chain {name} params cronjob error: {error}")
                        };
                    },
                    async {
                        let name = clone_chain.config.name.clone();
                        if let Err(error) = clone_chain.cron_job_val_supported_chains().await {
                            tracing::error!("Chain {name} validator supported chains error: {error}")
                        };
                    },
                    async {
                        let name = clone_chain.config.name.clone();
                        if let Err(error) = clone_chain.cron_job_chain_price_history().await {
                            tracing::error!("Chain {name} chain price history cronjob error: {error}")
                        };
                    },
                    async {
                        let name = clone_chain.config.name.clone();
                        if let Err(error) = clone_chain.cron_job_chain_dashboard().await {
                            tracing::error!("Chain {name} chain dashboard info cronjob error: {error}")
                        };
                    },
                );
                sleep(duration).await;
            }
        });

        // Validator cron job.
        // let validator_dur = Duration::from_secs(120);
        // let clone_chain = self.clone();
        // spawn(async move {
        //      loop {
        //          clone_chain.cron_job_validator().await;
        //
        //          sleep(duration).await;
        //     }
        // })

        // Validator cron job.
        // let validator_dur = Duration::from_secs(120);
        // let clone_chain = self.clone();
        // spawn(async move {
        //      loop {
        //          clone_chain.cron_job_validator().await;
        //
        //          sleep(duration).await;
        //     }
        // })

        // Validator cron job.
        // let validator_dur = Duration::from_secs(120);
        // let clone_chain = self.clone();
        // spawn(async move {
        //      loop {
        //          clone_chain.cron_job_validator().await;
        //
        //          sleep(duration).await;
        //     }
        // })
    }
}
