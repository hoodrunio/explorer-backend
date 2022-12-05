use std::time::Duration;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use web::Data;

use crate::routes;
use crate::state::State;

/// Starts the web server.
pub async fn start_web_server() -> std::io::Result<()> {
    // Create the state of the app.
    let state = Data::new(State::new());

    // Spawn a thread to update data.
    let state_clone = state.clone();
    tokio::spawn(async move {
        loop {
            state_clone.update_data().await;
            tokio::time::sleep(Duration::from_secs(600)).await;
        }
    });

    // Spawn a thread to update prices.
    let state_clone = state.clone();
    tokio::spawn(async move {
        loop {
            state_clone.update_prices().await;
            tokio::time::sleep(Duration::from_secs(600)).await;
        }
    });

    // Spawn a thread to update in-memory validator database. Every 3 minutes.
    let state_clone = state.clone();
    tokio::spawn(async move {
        loop {
            state_clone.update_database().await;
            tokio::time::sleep(Duration::from_secs(180)).await;
        }
    });

    // Subscribes to events.
    let state_clone = state.clone();
    tokio::spawn(async move {
        state_clone.subscribe_to_events().await;
    });

    HttpServer::new(move || {
        // Build a CORS middleware.
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            //.allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            //.allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

        // Build the app.
        App::new()
            .wrap(cors)
            // State data.
            .app_data(state.clone())
            // Services.
            .service(routes::block_by_hash)
            .service(routes::block_by_height)
            .service(routes::headers_by_heights)
            .service(routes::community_pool)
            .service(routes::delegations)
            .service(routes::delegator_rewards)
            .service(routes::delegator_withdraw_address)
            .service(routes::inflation)
            .service(routes::last_ten_blocks)
            .service(routes::last_ten_txs)
            .service(routes::params)
            .service(routes::proposal_deposit)
            .service(routes::proposal_deposits)
            .service(routes::proposal_details)
            .service(routes::proposal_tally)
            .service(routes::proposal_vote)
            .service(routes::proposal_votes)
            .service(routes::proposals_failed)
            .service(routes::proposals_passed)
            .service(routes::proposals_rejected)
            .service(routes::proposals_unspecified)
            .service(routes::proposals_voting)
            .service(routes::redelegations)
            .service(routes::signing)
            .service(routes::staking_pool)
            .service(web::resource("{chain}/socket").route(web::get().to(routes::socket)))
            .service(routes::supplies)
            .service(routes::supply)
            .service(routes::tx_by_hash)
            .service(routes::txs_by_height)
            .service(routes::txs_of_recipient)
            .service(routes::txs_of_sender)
            .service(routes::txs_on_latest_block)
            .service(routes::unbonding_delegations)
            .service(routes::validator)
            .service(routes::validator_commission)
            .service(routes::validator_delegator_pair)
            .service(routes::validator_rewards)
            .service(routes::validator_delegations)
            .service(routes::validator_redelegations)
            .service(routes::validator_unbondings)
            .service(routes::validators_bonded)
            .service(routes::validator_set)
            .service(routes::validator_set_by_height)
            .service(routes::validators_of_delegator)
            .service(routes::validators_unbonded)
            .service(routes::validators_unbonding)
            .service(routes::validators_unspecified)
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await
}
