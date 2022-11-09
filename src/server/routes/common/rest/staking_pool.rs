use super::necessities::*;

// ==== AXELAR ====

#[get("axelar/staking-pool")]
pub async fn staking_pool_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain.get_staking_pool().await;

    Json(resp)
}

// ==== CELESTIA ====

#[get("celestia/staking-pool")]
pub async fn staking_pool_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain.get_staking_pool().await;

    Json(resp)
}

// ==== COSMOS ====

#[get("cosmos/staking-pool")]
pub async fn staking_pool_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain.get_staking_pool().await;

    Json(resp)
}

// ==== EVMOS ====

#[get("evmos/staking-pool")]
pub async fn staking_pool_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain.get_staking_pool().await;

    Json(resp)
}

// ==== KYVE ====

#[get("kyve/staking-pool")]
pub async fn staking_pool_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain.get_staking_pool().await;

    Json(resp)
}

// ==== OSMOSIS ====

#[get("osmosis/staking-pool")]
pub async fn staking_pool_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain.get_staking_pool().await;

    Json(resp)
}

// ==== SECRET ====

#[get("secret/staking-pool")]
pub async fn staking_pool_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain.get_staking_pool().await;

    Json(resp)
}

