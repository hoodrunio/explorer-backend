use super::necessities::*;

// ==== AXELAR ====

#[get("axelar/tx/{hash}")]
pub async fn tx_by_hash_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain.get_tx_by_hash(&hash).await;

    Json(resp)
}

#[get("axelar/txs-on-latest-block")]
pub async fn txs_on_latest_block_axelar(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.axelar;

    let resp = chain
        .get_txs_by_height(None, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("axelar/txs-on-block/{heigth}")]
pub async fn txs_by_height_axelar(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain.get_txs_by_height(Some(height), PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("axelar/txs-of-sender/{address}")]
pub async fn txs_of_sender_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain.get_txs_by_sender(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("axelar/txs-of-recipient/{address}")]
pub async fn txs_of_recipient_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.axelar;

    let resp = chain.get_txs_by_recipient(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

// ==== CELESTIA ====

#[get("celestia/tx/{hash}")]
pub async fn tx_by_hash_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain.get_tx_by_hash(&hash).await;

    Json(resp)
}

#[get("celestia/txs-on-latest-block")]
pub async fn txs_on_latest_block_celestia(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.celestia;

    let resp = chain
        .get_txs_by_height(None, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("celestia/txs-on-block/{heigth}")]
pub async fn txs_by_height_celestia(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain.get_txs_by_height(Some(height), PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("celestia/txs-of-sender/{address}")]
pub async fn txs_of_sender_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain.get_txs_by_sender(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("celestia/txs-of-recipient/{address}")]
pub async fn txs_of_recipient_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.celestia;

    let resp = chain.get_txs_by_recipient(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

// ==== COSMOS ====

#[get("cosmos/tx/{hash}")]
pub async fn tx_by_hash_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain.get_tx_by_hash(&hash).await;

    Json(resp)
}

#[get("cosmos/txs-on-latest-block")]
pub async fn txs_on_latest_block_cosmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.cosmos;

    let resp = chain
        .get_txs_by_height(None, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("cosmos/txs-on-block/{heigth}")]
pub async fn txs_by_height_cosmos(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain.get_txs_by_height(Some(height), PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("cosmos/txs-of-sender/{address}")]
pub async fn txs_of_sender_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain.get_txs_by_sender(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("cosmos/txs-of-recipient/{address}")]
pub async fn txs_of_recipient_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.cosmos;

    let resp = chain.get_txs_by_recipient(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

// ==== EVMOS ====

#[get("evmos/tx/{hash}")]
pub async fn tx_by_hash_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain.get_tx_by_hash(&hash).await;

    Json(resp)
}

#[get("evmos/txs-on-latest-block")]
pub async fn txs_on_latest_block_evmos(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.evmos;

    let resp = chain
        .get_txs_by_height(None, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("evmos/txs-on-block/{heigth}")]
pub async fn txs_by_height_evmos(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain.get_txs_by_height(Some(height), PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("evmos/txs-of-sender/{address}")]
pub async fn txs_of_sender_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain.get_txs_by_sender(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("evmos/txs-of-recipient/{address}")]
pub async fn txs_of_recipient_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.evmos;

    let resp = chain.get_txs_by_recipient(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

// ==== KYVE ====

#[get("kyve/tx/{hash}")]
pub async fn tx_by_hash_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain.get_tx_by_hash(&hash).await;

    Json(resp)
}

#[get("kyve/txs-on-latest-block")]
pub async fn txs_on_latest_block_kyve(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.kyve;

    let resp = chain
        .get_txs_by_height(None, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("kyve/txs-on-block/{heigth}")]
pub async fn txs_by_height_kyve(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain.get_txs_by_height(Some(height), PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("kyve/txs-of-sender/{address}")]
pub async fn txs_of_sender_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain.get_txs_by_sender(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("kyve/txs-of-recipient/{address}")]
pub async fn txs_of_recipient_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.kyve;

    let resp = chain.get_txs_by_recipient(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

// ==== OSMOSIS ====

#[get("osmosis/tx/{hash}")]
pub async fn tx_by_hash_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain.get_tx_by_hash(&hash).await;

    Json(resp)
}

#[get("osmosis/txs-on-latest-block")]
pub async fn txs_on_latest_block_osmosis(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.osmosis;

    let resp = chain
        .get_txs_by_height(None, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("osmosis/txs-on-block/{heigth}")]
pub async fn txs_by_height_osmosis(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain.get_txs_by_height(Some(height), PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("osmosis/txs-of-sender/{address}")]
pub async fn txs_of_sender_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain.get_txs_by_sender(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("osmosis/txs-of-recipient/{address}")]
pub async fn txs_of_recipient_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.osmosis;

    let resp = chain.get_txs_by_recipient(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

// ==== SECRET ====

#[get("secret/tx/{hash}")]
pub async fn tx_by_hash_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain.get_tx_by_hash(&hash).await;

    Json(resp)
}

#[get("secret/txs-on-latest-block")]
pub async fn txs_on_latest_block_secret(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.secret;

    let resp = chain
        .get_txs_by_height(None, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("secret/txs-on-block/{heigth}")]
pub async fn txs_by_height_secret(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain.get_txs_by_height(Some(height), PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("secret/txs-of-sender/{address}")]
pub async fn txs_of_sender_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain.get_txs_by_sender(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("secret/txs-of-recipient/{address}")]
pub async fn txs_of_recipient_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.secret;

    let resp = chain.get_txs_by_recipient(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

