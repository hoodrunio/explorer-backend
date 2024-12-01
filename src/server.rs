use std::collections::HashSet;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::web::Json;
use actix_web::{get, web, App, HttpServer, Responder};
use tracing_actix_web::TracingLogger;
use web::Data;

use crate::routes;
use crate::state::State;
use crate::logging::{LogLevel::*, log_api, log_db, log_socket};
use crate::ws::WsManager;

#[get("/")]
async fn initial() -> impl Responder {
    log_api(INFO, "Health check endpoint called");
    Json("Rest: OK")
}

/// Starts the web server.
pub async fn start_web_server() -> std::io::Result<()> {
    // Create the state of the app.
    log_api(INFO, "Initializing application state...");
    let state = Data::new(State::new().await);
    log_api(INFO, "Application state initialized");

    // Start running cron jobs to update MongoDB database.
    log_db(INFO, "Starting cron jobs for database updates");
    state.run_cron_jobs();
    log_db(INFO, "Cron jobs started successfully");

    // Initialize WebSocket manager
    let chains = HashSet::from_iter(state.get_chains().keys().cloned());
    let ws_manager = Arc::new(WsManager::new(chains.clone(), 1000));
    let ws_manager_data = Data::new(Arc::clone(&ws_manager));

    // Spawn a thread to subscribe to events.
    let state_clone = state.clone();
    let ws_manager_clone = Arc::clone(&ws_manager);
    log_socket(INFO, "Setting up WebSocket event handling");

    tokio::spawn(async move {
        let event_tx = ws_manager_clone.get_event_sender();
        state_clone.subscribe_to_events(event_tx).await;
    });

    // Start WebSocket server
    let ws_manager_clone = Arc::clone(&ws_manager);
    tokio::spawn(async move {
        if let Err(e) = ws_manager_clone.start_server().await {
            log_socket(ERROR, &format!("Error spawning the websocket server: {e}"));
        }
    });

    HttpServer::new(move || {
        // Build a CORS middleware.
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .max_age(3600);

        // Build the app.
        App::new()
            .wrap(TracingLogger::default())
            .wrap(cors)
            // State data.
            .app_data(state.clone())
            .app_data(ws_manager_data.clone())
            // Services.
            .service(initial)
            .service(routes::dashboard)
            .service(routes::stats)
            .service(routes::chains)
            .service(routes::block_by_hash)
            .service(routes::block_by_height)
            .service(routes::headers_by_heights)
            .service(routes::community_pool)
            .service(routes::delegations)
            .service(routes::delegator_rewards)
            .service(routes::delegator_withdraw_address)
            .service(routes::inflation)
            .service(routes::blocks)
            .service(routes::last_blocks)
            .service(routes::last_txs)
            .service(routes::params)
            .service(routes::proposal_deposit)
            .service(routes::proposal_deposits)
            .service(routes::proposal_details)
            .service(routes::proposal_tally)
            .service(routes::proposal_vote)
            .service(routes::proposal_votes)
            .service(routes::proposals)
            .service(routes::redelegations)
            .service(routes::signing)
            .service(routes::calculations)
            .service(routes::staking_pool)
            .service(routes::supplies)
            .service(routes::supply)
            .service(routes::account)
            .service(routes::account_balances)
            .service(routes::account_vesting)
            .service(routes::txs)
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
