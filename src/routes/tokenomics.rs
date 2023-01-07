use super::QueryParams;
use crate::routes::{extract_chain, OutRestResponse, TNRAppError, TNRAppSuccessResponse};
use crate::{
    fetch::others::{PaginationConfig, Response},
    state::State,
};
use actix_web::{
    get,
    web::{Data, Json, Path, Query},
    Responder,
};

// ======== Tokenomic Methods ========

#[get("{chain}/supply/{denom}")]
pub async fn supply(path: Path<(String, String)>, chains: Data<State>) -> Result<impl Responder, TNRAppError> {
    let (chain, denom) = path.into_inner();
    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_supply_by_denom(&denom).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/supplies")]
pub async fn supplies(path: Path<String>, chains: Data<State>, query: Query<QueryParams>) ->  Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();

    let config = PaginationConfig::new().limit(60).page(query.page.unwrap_or(1));
    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_supply_of_all_tokens(config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/inflation")]
pub async fn inflation(path: Path<String>, chains: Data<State>) ->  Result<impl Responder, TNRAppError> {
    let chain = path.into_inner();
    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_inflation_rate().await?;
    Ok(TNRAppSuccessResponse::new(data))
}
