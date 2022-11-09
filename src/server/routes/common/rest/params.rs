use super::necessities::*;

// ==== AXELAR ====

#[get("axelar/params/staking")]
pub async fn staking_params_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain.get_staking_params().await;

    Json(resp)
}

#[get("axelar/params/tally")]
pub async fn tally_params_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain.get_tally_params().await;

    Json(resp)
}

#[get("axelar/params/voting")]
pub async fn voting_params_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain.get_voting_params().await;

    Json(resp)
}

#[get("axelar/params/deposit")]
pub async fn deposit_params_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain.get_deposit_params().await;

    Json(resp)
}

#[get("axelar/params/slashing")]
pub async fn slashing_params_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain.get_slashing_params().await;

    Json(resp)
}

// ==== CELESTIA ====

#[get("celestia/params/staking")]
pub async fn staking_params_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain.get_staking_params().await;

    Json(resp)
}

#[get("celestia/params/tally")]
pub async fn tally_params_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain.get_tally_params().await;

    Json(resp)
}

#[get("celestia/params/voting")]
pub async fn voting_params_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain.get_voting_params().await;

    Json(resp)
}

#[get("celestia/params/deposit")]
pub async fn deposit_params_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain.get_deposit_params().await;

    Json(resp)
}

#[get("celestia/params/slashing")]
pub async fn slashing_params_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain.get_slashing_params().await;

    Json(resp)
}

// ==== COSMOS ====

#[get("cosmos/params/staking")]
pub async fn staking_params_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain.get_staking_params().await;

    Json(resp)
}

#[get("cosmos/params/tally")]
pub async fn tally_params_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain.get_tally_params().await;

    Json(resp)
}

#[get("cosmos/params/voting")]
pub async fn voting_params_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain.get_voting_params().await;

    Json(resp)
}

#[get("cosmos/params/deposit")]
pub async fn deposit_params_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain.get_deposit_params().await;

    Json(resp)
}

#[get("cosmos/params/slashing")]
pub async fn slashing_params_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain.get_slashing_params().await;

    Json(resp)
}

// ==== EVMOS ====

#[get("evmos/params/staking")]
pub async fn staking_params_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain.get_staking_params().await;

    Json(resp)
}

#[get("evmos/params/tally")]
pub async fn tally_params_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain.get_tally_params().await;

    Json(resp)
}

#[get("evmos/params/voting")]
pub async fn voting_params_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain.get_voting_params().await;

    Json(resp)
}

#[get("evmos/params/deposit")]
pub async fn deposit_params_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain.get_deposit_params().await;

    Json(resp)
}

#[get("evmos/params/slashing")]
pub async fn slashing_params_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain.get_slashing_params().await;

    Json(resp)
}

// ==== KYVE ====

#[get("kyve/params/staking")]
pub async fn staking_params_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain.get_staking_params().await;

    Json(resp)
}

#[get("kyve/params/tally")]
pub async fn tally_params_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain.get_tally_params().await;

    Json(resp)
}

#[get("kyve/params/voting")]
pub async fn voting_params_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain.get_voting_params().await;

    Json(resp)
}

#[get("kyve/params/deposit")]
pub async fn deposit_params_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain.get_deposit_params().await;

    Json(resp)
}

#[get("kyve/params/slashing")]
pub async fn slashing_params_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain.get_slashing_params().await;

    Json(resp)
}

// ==== OSMOSIS ====

#[get("osmosis/params/staking")]
pub async fn staking_params_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain.get_staking_params().await;

    Json(resp)
}

#[get("osmosis/params/tally")]
pub async fn tally_params_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain.get_tally_params().await;

    Json(resp)
}

#[get("osmosis/params/voting")]
pub async fn voting_params_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain.get_voting_params().await;

    Json(resp)
}

#[get("osmosis/params/deposit")]
pub async fn deposit_params_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain.get_deposit_params().await;

    Json(resp)
}

#[get("osmosis/params/slashing")]
pub async fn slashing_params_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain.get_slashing_params().await;

    Json(resp)
}

// ==== SECRET ====

#[get("secret/params/staking")]
pub async fn staking_params_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain.get_staking_params().await;

    Json(resp)
}

#[get("secret/params/tally")]
pub async fn tally_params_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain.get_tally_params().await;

    Json(resp)
}

#[get("secret/params/voting")]
pub async fn voting_params_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain.get_voting_params().await;

    Json(resp)
}

#[get("secret/params/deposit")]
pub async fn deposit_params_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain.get_deposit_params().await;

    Json(resp)
}

#[get("secret/params/slashing")]
pub async fn slashing_params_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain.get_slashing_params().await;

    Json(resp)
}

