use super::necessities::*;

// ==== AXELAR ====

#[get("axelar/signing/{address}")]
pub async fn signing_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let cons_address = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain.get_signing_info(&cons_address).await;

    Json(resp)
}

// ==== CELESTIA ====

#[get("celestia/signing/{address}")]
pub async fn signing_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let cons_address = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain.get_signing_info(&cons_address).await;

    Json(resp)
}

// ==== COSMOS ====

#[get("cosmos/signing/{address}")]
pub async fn signing_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let cons_address = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain.get_signing_info(&cons_address).await;

    Json(resp)
}

// ==== EVMOS ====

#[get("evmos/signing/{address}")]
pub async fn signing_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let cons_address = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain.get_signing_info(&cons_address).await;

    Json(resp)
}

// ==== KYVE ====

#[get("kyve/signing/{address}")]
pub async fn signing_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let cons_address = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain.get_signing_info(&cons_address).await;

    Json(resp)
}

// ==== OSMOSIS ====

#[get("osmosis/signing/{address}")]
pub async fn signing_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let cons_address = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain.get_signing_info(&cons_address).await;

    Json(resp)
}

// ==== SECRET ====

#[get("secret/signing/{address}")]
pub async fn signing_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let cons_address = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain.get_signing_info(&cons_address).await;

    Json(resp)
}

