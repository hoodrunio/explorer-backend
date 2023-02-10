use crate::routes::{extract_chain, TNRAppError, TNRAppSuccessResponse};
use crate::{fetch::others::Response, state::State};
use actix_web::{
    get,
    web::{Data, Path},
    Responder,
};

// ====== Block Methods ======

#[get("{chain}/block-by-height/{height}")]
pub async fn block_by_height(path: Path<(String, u64)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, height) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_block_by_height(Some(height)).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/block-by-hash/{hash}")]
pub async fn block_by_hash(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, hash) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_block_by_hash(&hash).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

/// Example: http://localhost:8080/axelar/block-headers/2500-2520
/// Maximum block headers length is 20.
#[get("{chain}/block-headers/{min_and_max_height}")]
pub async fn headers_by_heights(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, min_and_max_height) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = match min_and_max_height.split_once('-') {
        Some((min_height, max_height)) => match (min_height.parse(), max_height.parse()) {
            (Ok(min_height), Ok(max_height)) => Response::Success(chain.get_block_headers(min_height, max_height).await),
            _ => Response::Error(format!("{} is mistaken!", min_and_max_height)),
        },
        None => Response::Error(format!("{} is mistaken!", min_and_max_height)),
    };
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/latest-block-headers")]
pub async fn latest_headers(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_block_headers_last_20().await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/last-ten-blocks")]
pub async fn last_ten_blocks(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {

    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_last_count_block(10).await?;
    Ok(TNRAppSuccessResponse::new(data))
}
