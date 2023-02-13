use actix_web::{
    get,
    Responder,
    web::{Data, Path},
};
use actix_web::web::Query;
use serde::Deserialize;

use crate::{
    fetch::others::PaginationConfig,
    state::State,
};
use crate::routes::{extract_chain, QueryParams, TNRAppError, TNRAppSuccessResponse};

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
    let data = chain
        .get_txs_by_recipient(&recipient_addr, config)
        .await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/last-txs")]
pub async fn last_txs(path: Path<String>, chains: Data<State>, query: Query<LastTxsQueryParams>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let default_count = 10;
    let count = query.tx_count.unwrap_or(default_count);

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_last_count_txs_from_db(Some(count)).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[derive(Deserialize)]
pub struct LastTxsQueryParams {
    pub tx_count: Option<u16>,
}