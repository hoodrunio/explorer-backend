use super::necessities::*;

// ==== AXELAR ====

#[get("axelar/block/{height}")]
pub async fn block_by_height_axelar(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();
    
    let chain = &server_state.chains.axelar;
    
    let resp = chain.get_block_by_height(Some(height)).await;
    
    Json(resp)
}
    
#[get("axelar/block/{hash}")]
pub async fn block_by_hash_axelar(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();
    
    let chain = &server_state.chains.axelar;
    
    let resp = chain.get_block_by_hash(&hash).await;
    
    Json(resp)
}
    
#[get("axelar/blockchain/{min_height}/{max_height}")]
pub async fn blockchain_by_heights_axelar(path: Path<(u64, u64)>, server_state: Data<ServerState>) -> impl Responder {
    let (min_height, max_height) = path.into_inner();
    
    let chain = &server_state.chains.axelar;
    
    let resp = chain.get_blockchain(min_height, max_height).await;
    
    Json(resp)
}

// ==== CELESTIA ====

#[get("celestia/block/{height}")]
pub async fn block_by_height_celestia(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();
    
    let chain = &server_state.chains.celestia;
    
    let resp = chain.get_block_by_height(Some(height)).await;
    
    Json(resp)
}
    
#[get("celestia/block/{hash}")]
pub async fn block_by_hash_celestia(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();
    
    let chain = &server_state.chains.celestia;
    
    let resp = chain.get_block_by_hash(&hash).await;
    
    Json(resp)
}
    
#[get("celestia/blockchain/{min_height}/{max_height}")]
pub async fn blockchain_by_heights_celestia(path: Path<(u64, u64)>, server_state: Data<ServerState>) -> impl Responder {
    let (min_height, max_height) = path.into_inner();
    
    let chain = &server_state.chains.celestia;
    
    let resp = chain.get_blockchain(min_height, max_height).await;
    
    Json(resp)
}

// ==== COSMOS ====

#[get("cosmos/block/{height}")]
pub async fn block_by_height_cosmos(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();
    
    let chain = &server_state.chains.cosmos;
    
    let resp = chain.get_block_by_height(Some(height)).await;
    
    Json(resp)
}
    
#[get("cosmos/block/{hash}")]
pub async fn block_by_hash_cosmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();
    
    let chain = &server_state.chains.cosmos;
    
    let resp = chain.get_block_by_hash(&hash).await;
    
    Json(resp)
}
    
#[get("cosmos/blockchain/{min_height}/{max_height}")]
pub async fn blockchain_by_heights_cosmos(path: Path<(u64, u64)>, server_state: Data<ServerState>) -> impl Responder {
    let (min_height, max_height) = path.into_inner();
    
    let chain = &server_state.chains.cosmos;
    
    let resp = chain.get_blockchain(min_height, max_height).await;
    
    Json(resp)
}

// ==== EVMOS ====

#[get("evmos/block/{height}")]
pub async fn block_by_height_evmos(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();
    
    let chain = &server_state.chains.evmos;
    
    let resp = chain.get_block_by_height(Some(height)).await;
    
    Json(resp)
}
    
#[get("evmos/block/{hash}")]
pub async fn block_by_hash_evmos(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();
    
    let chain = &server_state.chains.evmos;
    
    let resp = chain.get_block_by_hash(&hash).await;
    
    Json(resp)
}
    
#[get("evmos/blockchain/{min_height}/{max_height}")]
pub async fn blockchain_by_heights_evmos(path: Path<(u64, u64)>, server_state: Data<ServerState>) -> impl Responder {
    let (min_height, max_height) = path.into_inner();
    
    let chain = &server_state.chains.evmos;
    
    let resp = chain.get_blockchain(min_height, max_height).await;
    
    Json(resp)
}

// ==== KYVE ====

#[get("kyve/block/{height}")]
pub async fn block_by_height_kyve(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();
    
    let chain = &server_state.chains.kyve;
    
    let resp = chain.get_block_by_height(Some(height)).await;
    
    Json(resp)
}
    
#[get("kyve/block/{hash}")]
pub async fn block_by_hash_kyve(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();
    
    let chain = &server_state.chains.kyve;
    
    let resp = chain.get_block_by_hash(&hash).await;
    
    Json(resp)
}
    
#[get("kyve/blockchain/{min_height}/{max_height}")]
pub async fn blockchain_by_heights_kyve(path: Path<(u64, u64)>, server_state: Data<ServerState>) -> impl Responder {
    let (min_height, max_height) = path.into_inner();
    
    let chain = &server_state.chains.kyve;
    
    let resp = chain.get_blockchain(min_height, max_height).await;
    
    Json(resp)
}

// ==== OSMOSIS ====

#[get("osmosis/block/{height}")]
pub async fn block_by_height_osmosis(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();
    
    let chain = &server_state.chains.osmosis;
    
    let resp = chain.get_block_by_height(Some(height)).await;
    
    Json(resp)
}
    
#[get("osmosis/block/{hash}")]
pub async fn block_by_hash_osmosis(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();
    
    let chain = &server_state.chains.osmosis;
    
    let resp = chain.get_block_by_hash(&hash).await;
    
    Json(resp)
}
    
#[get("osmosis/blockchain/{min_height}/{max_height}")]
pub async fn blockchain_by_heights_osmosis(path: Path<(u64, u64)>, server_state: Data<ServerState>) -> impl Responder {
    let (min_height, max_height) = path.into_inner();
    
    let chain = &server_state.chains.osmosis;
    
    let resp = chain.get_blockchain(min_height, max_height).await;
    
    Json(resp)
}

// ==== SECRET ====

#[get("secret/block/{height}")]
pub async fn block_by_height_secret(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();
    
    let chain = &server_state.chains.secret;
    
    let resp = chain.get_block_by_height(Some(height)).await;
    
    Json(resp)
}
    
#[get("secret/block/{hash}")]
pub async fn block_by_hash_secret(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();
    
    let chain = &server_state.chains.secret;
    
    let resp = chain.get_block_by_hash(&hash).await;
    
    Json(resp)
}
    
#[get("secret/blockchain/{min_height}/{max_height}")]
pub async fn blockchain_by_heights_secret(path: Path<(u64, u64)>, server_state: Data<ServerState>) -> impl Responder {
    let (min_height, max_height) = path.into_inner();
    
    let chain = &server_state.chains.secret;
    
    let resp = chain.get_blockchain(min_height, max_height).await;
    
    Json(resp)
}

