use std::time::Duration;

use actix::clock::interval;
use actix_web::{web, App, HttpServer};
use tokio::spawn;
use web::Data;

use crate::routes::{rest, wss};
use crate::state::State;

pub async fn run_cron_job(_chains: Data<State>) {
    
}

/// Starts the web server.
pub async fn start_web_server() -> std::io::Result<()> {
    let state = Data::new(State::new());

    run_cron_job(state.clone()).await;

    HttpServer::new(move || {
        App::new()
            // Add `ChainsState`.
            .app_data(state.clone())
            .service(web::resource("{chain}/socket").route(web::get().to(wss::socket)))
            // Common services.
            .service(rest::block_by_hash)
            .service(rest::block_by_height)
            .service(rest::blockchain_by_heights)
            .service(rest::community_pool)
            .service(rest::delegations)
            .service(rest::delegator_rewards)
            .service(rest::delegator_withdraw_address)
            .service(rest::deposit_params)
            .service(rest::inflation)
            .service(rest::proposal_deposit)
            .service(rest::proposal_deposits)
            .service(rest::proposal_details)
            .service(rest::proposal_tally)
            .service(rest::proposal_vote)
            .service(rest::proposal_votes)
            .service(rest::proposals_failed)
            .service(rest::proposals_passed)
            .service(rest::proposals_rejected)
            .service(rest::proposals_unspecified)
            .service(rest::proposals_voting)
            .service(rest::redelegations)
            .service(rest::signing)
            .service(rest::slashing_params)
            .service(rest::staking_params)
            .service(rest::staking_pool)
            .service(rest::supplies)
            .service(rest::supply)
            .service(rest::tally_params)
            .service(rest::tx_by_hash)
            .service(rest::txs_by_height)
            .service(rest::txs_of_recipient)
            .service(rest::txs_of_sender)
            .service(rest::txs_on_latest_block)
            .service(rest::unbonding_delegations)
            .service(rest::validator)
            .service(rest::validator_commission)
            .service(rest::validator_delegator_pair)
            .service(rest::validator_rewards)
            .service(rest::validators_bonded)
            .service(rest::validators_of_delegator)
            .service(rest::validators_unbonded)
            .service(rest::validators_unbonding)
            .service(rest::validators_unspecified)
            .service(rest::voting_params)
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await
}
