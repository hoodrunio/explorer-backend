use super::necessities::*;

// ==== AXELAR ====

#[get("axelar/delegations/{delegator_address}")]
pub async fn delegations_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_delegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/unbonding-delegations/{delegator_address}")]
pub async fn unbonding_delegations_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_delegations_unbonding(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/redelegations/{delegator_address}")]
pub async fn redelegations_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_redelegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

// ==== CELESTIA ====

#[get("celestia/delegations/{delegator_address}")]
pub async fn delegations_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_delegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/unbonding-delegations/{delegator_address}")]
pub async fn unbonding_delegations_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_delegations_unbonding(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/redelegations/{delegator_address}")]
pub async fn redelegations_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_redelegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

// ==== COSMOS ====

#[get("cosmos/delegations/{delegator_address}")]
pub async fn delegations_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_delegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/unbonding-delegations/{delegator_address}")]
pub async fn unbonding_delegations_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_delegations_unbonding(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/redelegations/{delegator_address}")]
pub async fn redelegations_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_redelegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

// ==== EVMOS ====

#[get("evmos/delegations/{delegator_address}")]
pub async fn delegations_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_delegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/unbonding-delegations/{delegator_address}")]
pub async fn unbonding_delegations_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_delegations_unbonding(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/redelegations/{delegator_address}")]
pub async fn redelegations_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_redelegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

// ==== KYVE ====

#[get("kyve/delegations/{delegator_address}")]
pub async fn delegations_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_delegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/unbonding-delegations/{delegator_address}")]
pub async fn unbonding_delegations_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_delegations_unbonding(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/redelegations/{delegator_address}")]
pub async fn redelegations_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_redelegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

// ==== OSMOSIS ====

#[get("osmosis/delegations/{delegator_address}")]
pub async fn delegations_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_delegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/unbonding-delegations/{delegator_address}")]
pub async fn unbonding_delegations_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_delegations_unbonding(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/redelegations/{delegator_address}")]
pub async fn redelegations_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_redelegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

// ==== SECRET ====

#[get("secret/delegations/{delegator_address}")]
pub async fn delegations_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain
        .get_delegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/unbonding-delegations/{delegator_address}")]
pub async fn unbonding_delegations_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain
        .get_delegations_unbonding(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/redelegations/{delegator_address}")]
pub async fn redelegations_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain
        .get_redelegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

