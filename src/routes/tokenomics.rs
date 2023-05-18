use actix_web::{
    get,
    web::{Data, Path, Query},
    Responder,
};

use crate::routes::{extract_chain, TNRAppError, TNRAppSuccessResponse};
use crate::{fetch::others::PaginationConfig, state::State};

// ======== Tokenomic Methods ========

#[get("{chain}/supply/{denom}")]
pub async fn supply(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, denom) = path.into_inner();
    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_supply_by_denom(&denom).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/supplies")]
pub async fn supplies(path: Path<String>, chains: Data<State>, query: Query<PaginationData>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_supply_of_all_tokens(query.into_inner()).await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}

#[get("{chain}/inflation")]
pub async fn inflation(path: Path<String>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();
    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_inflation_rate().await?;
    Ok(TNRAppSuccessResponse::new(data, None))
}
