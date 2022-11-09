use super::necessities::*;

// ==== AXELAR ====

#[get("axelar/delegator-rewards/{address}")]
pub async fn delegator_rewards_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain.get_delegator_rewards(&delegator_address).await;

    Json(resp)
}

#[get("axelar/delegator-withdraw-address/{address}")]
pub async fn delegator_withdraw_address_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain.get_delegator_withdraw_address(&delegator_address).await;

    Json(resp)
}

// ==== CELESTIA ====

#[get("celestia/delegator-rewards/{address}")]
pub async fn delegator_rewards_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain.get_delegator_rewards(&delegator_address).await;

    Json(resp)
}

#[get("celestia/delegator-withdraw-address/{address}")]
pub async fn delegator_withdraw_address_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain.get_delegator_withdraw_address(&delegator_address).await;

    Json(resp)
}

// ==== COSMOS ====

#[get("cosmos/delegator-rewards/{address}")]
pub async fn delegator_rewards_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain.get_delegator_rewards(&delegator_address).await;

    Json(resp)
}

#[get("cosmos/delegator-withdraw-address/{address}")]
pub async fn delegator_withdraw_address_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain.get_delegator_withdraw_address(&delegator_address).await;

    Json(resp)
}

// ==== EVMOS ====

#[get("evmos/delegator-rewards/{address}")]
pub async fn delegator_rewards_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain.get_delegator_rewards(&delegator_address).await;

    Json(resp)
}

#[get("evmos/delegator-withdraw-address/{address}")]
pub async fn delegator_withdraw_address_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain.get_delegator_withdraw_address(&delegator_address).await;

    Json(resp)
}

// ==== KYVE ====

#[get("kyve/delegator-rewards/{address}")]
pub async fn delegator_rewards_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain.get_delegator_rewards(&delegator_address).await;

    Json(resp)
}

#[get("kyve/delegator-withdraw-address/{address}")]
pub async fn delegator_withdraw_address_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain.get_delegator_withdraw_address(&delegator_address).await;

    Json(resp)
}

// ==== OSMOSIS ====

#[get("osmosis/delegator-rewards/{address}")]
pub async fn delegator_rewards_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain.get_delegator_rewards(&delegator_address).await;

    Json(resp)
}

#[get("osmosis/delegator-withdraw-address/{address}")]
pub async fn delegator_withdraw_address_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain.get_delegator_withdraw_address(&delegator_address).await;

    Json(resp)
}

// ==== SECRET ====

#[get("secret/delegator-rewards/{address}")]
pub async fn delegator_rewards_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain.get_delegator_rewards(&delegator_address).await;

    Json(resp)
}

#[get("secret/delegator-withdraw-address/{address}")]
pub async fn delegator_withdraw_address_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain.get_delegator_withdraw_address(&delegator_address).await;

    Json(resp)
}

