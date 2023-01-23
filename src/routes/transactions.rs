use crate::routes::{extract_chain, OutRestResponse, QueryParams, TNRAppError, TNRAppSuccessResponse};
use crate::{
    fetch::others::{PaginationConfig, Response},
    state::State,
};
use actix_web::web::Query;
use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};

// ======== Transaction Methods ========

#[get("{chain}/tx/{hash}")]
pub async fn tx_by_hash(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, hash) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_tx_by_hash(&hash).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/txs-on-latest-block")]
pub async fn txs_on_latest_block(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(query.limit.unwrap_or(20)).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_txs_by_height(None, config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/txs-by-height/{heigth}")]
pub async fn txs_by_height(path: Path<(String, u64)>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, height) = path.into_inner();

    let config = PaginationConfig::new().limit(query.limit.unwrap_or(20)).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_txs_by_height(Some(height), config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/txs-of-sender/{address}")]
pub async fn txs_of_sender(path: Path<(String, String)>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, sender_addr) = path.into_inner();

    let config = PaginationConfig::new().limit(query.limit.unwrap_or(20)).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_txs_by_sender(&sender_addr, config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/txs-of-recipient/{address}")]
pub async fn txs_of_recipient(path: Path<(String, String)>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, recipient_addr) = path.into_inner();

    let config = PaginationConfig::new().limit(query.limit.unwrap_or(20)).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_txs_by_recipient(&recipient_addr, config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/last-ten-txs")]
pub async fn last_ten_txs(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = "Storing txs in the database is not implemented yet.".to_string();

    // match chain.inner.data.last_ten_txs.queue.lock() {
    //     Ok(last_ten_txs) => Response::Success(OutRestResponse::new(Json(last_ten_txs.clone()), 0)),
    //     _ => Response::Error("An internal error occured.".to_string()),
    // },
    Ok(TNRAppSuccessResponse::new(data))
}
