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
use crate::routes::{extract_chain, LastCountListsQueryParams, QueryParams, TNRAppError, TNRAppSuccessResponse};

// ======== Contract Methods ========
#[get("{chain}/contract/{hash}")]
pub async fn contract_by_hash(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, hash) = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_contract_by_hash(&hash).await?;
    Ok(TNRAppSuccessResponse::new(data))
}