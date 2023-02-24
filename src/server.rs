use std::collections::HashSet;

use actix_cors::Cors;
use actix_web::web::Json;
use actix_web::{get, web, App, HttpServer, Responder};
use tokio::sync::broadcast::channel;
use tracing_actix_web::TracingLogger;
use web::Data;

use crate::chain::Chain;
use crate::events::{run_ws, WsEvent};
use crate::routes;
use crate::state::State;

#[get("/")]
async fn initial() -> impl Responder {
    Json("Rest: OK")
}

/// Starts the web server.
pub async fn start_web_server() -> std::io::Result<()> {
    // Create the state of the app.
    let state = Data::new(State::new().await);

    // Start running cron jobs to update MongoDB database.
    state.run_cron_jobs();

    // Spawn a thread to subscribe to events.
    let state_clone = state.clone();

    // After connecting to MongoDB, there are so many thread safety & ownership errors.
    // You have to rewrite `src/fetch/socket.rs` to fix them.

    let (tx, _rx) = channel::<(String, WsEvent)>(100);

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        state_clone.subscribe_to_events(tx_clone).await;
    });

    let chains = HashSet::from_iter(state.get_chains().keys().cloned());

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        if let Err(e) = run_ws(tx_clone, chains).await {
            tracing::error!("Error spawning the websocket task {e}");
        };
    });

    if let Ok(axelar) = state.get("axelar") {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            match Chain::sub_axelar_events(axelar, tx_clone).await {
                Ok(_) => {}
                Err(e) => tracing::info!("Error axelar evm polls flow {}", e),
            };
        });
    };

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
            .wrap(TracingLogger::default())
            .wrap(cors)
            // State data.
            .app_data(state.clone())
            // Services.
            .service(initial)
            .service(routes::chains)
            .service(routes::block_by_hash)
            .service(routes::block_by_height)
            .service(routes::headers_by_heights)
            .service(routes::community_pool)
            .service(routes::delegations)
            .service(routes::delegator_rewards)
            .service(routes::delegator_withdraw_address)
            .service(routes::inflation)
            .service(routes::last_blocks)
            .service(routes::last_txs)
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
            .service(routes::calculations)
            .service(routes::staking_pool)
            // Socket's are not working as we store neither blocks nor txs in the database.
            // And the Web Socket connection between this program and nodes is broken.
            .service(web::resource("{chain}/socket").route(web::get().to(routes::socket)))
            .service(routes::supplies)
            .service(routes::supply)
            .service(routes::account)
            .service(routes::account_balances)
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
            .service(routes::validator_last_signed_blocks)
            .service(routes::validator_redelegations)
            .service(routes::validator_unbondings)
            .service(routes::validators_bonded)
            .service(routes::validator_set)
            .service(routes::validator_set_by_height)
            .service(routes::validators_of_delegator)
            .service(routes::validators_unbonded)
            .service(routes::validators_unbonding)
            .service(routes::validators_unspecified)
            .service(routes::evm_poll)
            .service(routes::evm_polls)
            .service(routes::evm_validator_votes)
            .service(routes::evm_val_supported_chains)
            .service(routes::validator_hearbeats)
            .service(routes::hearbeats)
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await
}
