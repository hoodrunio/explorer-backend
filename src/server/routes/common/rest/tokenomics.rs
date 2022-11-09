use super::necessities::*;

// ==== AXELAR ====

#[get("axelar/supply/{denom}")]
pub async fn supply_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let denom = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain.get_supply_by_denom(&denom).await;

    Json(resp)
}

#[get("axelar/supplies")]
pub async fn supplies_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_supply_of_all_tokens(PaginationConfig::new().limit(1000).offset(0))
        .await;

    Json(resp)
}

#[get("axelar/inflation")]
pub async fn inflation_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain.get_inflation_rate().await;

    Json(resp)
}

// ==== CELESTIA ====

#[get("celestia/supply/{denom}")]
pub async fn supply_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let denom = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain.get_supply_by_denom(&denom).await;

    Json(resp)
}

#[get("celestia/supplies")]
pub async fn supplies_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_supply_of_all_tokens(PaginationConfig::new().limit(1000).offset(0))
        .await;

    Json(resp)
}

#[get("celestia/inflation")]
pub async fn inflation_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain.get_inflation_rate().await;

    Json(resp)
}

// ==== COSMOS ====

#[get("cosmos/supply/{denom}")]
pub async fn supply_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let denom = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain.get_supply_by_denom(&denom).await;

    Json(resp)
}

#[get("cosmos/supplies")]
pub async fn supplies_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_supply_of_all_tokens(PaginationConfig::new().limit(1000).offset(0))
        .await;

    Json(resp)
}

#[get("cosmos/inflation")]
pub async fn inflation_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain.get_inflation_rate().await;

    Json(resp)
}

// ==== EVMOS ====

#[get("evmos/supply/{denom}")]
pub async fn supply_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let denom = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain.get_supply_by_denom(&denom).await;

    Json(resp)
}

#[get("evmos/supplies")]
pub async fn supplies_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_supply_of_all_tokens(PaginationConfig::new().limit(1000).offset(0))
        .await;

    Json(resp)
}

#[get("evmos/inflation")]
pub async fn inflation_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain.get_inflation_rate().await;

    Json(resp)
}

// ==== KYVE ====

#[get("kyve/supply/{denom}")]
pub async fn supply_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let denom = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain.get_supply_by_denom(&denom).await;

    Json(resp)
}

#[get("kyve/supplies")]
pub async fn supplies_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_supply_of_all_tokens(PaginationConfig::new().limit(1000).offset(0))
        .await;

    Json(resp)
}

#[get("kyve/inflation")]
pub async fn inflation_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain.get_inflation_rate().await;

    Json(resp)
}

// ==== OSMOSIS ====

#[get("osmosis/supply/{denom}")]
pub async fn supply_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let denom = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain.get_supply_by_denom(&denom).await;

    Json(resp)
}

#[get("osmosis/supplies")]
pub async fn supplies_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_supply_of_all_tokens(PaginationConfig::new().limit(1000).offset(0))
        .await;

    Json(resp)
}

#[get("osmosis/inflation")]
pub async fn inflation_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain.get_inflation_rate().await;

    Json(resp)
}

// ==== SECRET ====

#[get("secret/supply/{denom}")]
pub async fn supply_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let denom = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain.get_supply_by_denom(&denom).await;

    Json(resp)
}

#[get("secret/supplies")]
pub async fn supplies_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain
        .get_supply_of_all_tokens(PaginationConfig::new().limit(1000).offset(0))
        .await;

    Json(resp)
}

#[get("secret/inflation")]
pub async fn inflation_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain.get_inflation_rate().await;

    Json(resp)
}

