use super::QueryParams;
use crate::routes::{extract_chain, TNRAppError, TNRAppSuccessResponse};
use crate::{
    fetch::others::{PaginationConfig, Response},
    state::State,
};
use actix_web::{
    get,
    web::{Data, Json, Path, Query},
    Responder,
};

// ======== Delegation Methods ========

#[get("{chain}/delegations/{delegator_address}")]
pub async fn delegations(path: Path<(String, String)>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, delegator_addr) = path.into_inner();

    let config = PaginationConfig::new().limit(5).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_delegations(&delegator_addr, config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/unbonding-delegations/{delegator_address}")]
pub async fn unbonding_delegations(
    path: Path<(String, String)>,
    chains: Data<State>,
    query: Query<QueryParams>,
) -> Result<impl Responder, TNRAppError> {
    let (chain, delegator_addr) = path.into_inner();

    let config = PaginationConfig::new().limit(5).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_delegations_unbonding(&delegator_addr, config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}

#[get("{chain}/redelegations/{delegator_address}")]
pub async fn redelegations(path: Path<(String, String)>, chains: Data<State>, query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, delegator_addr) = path.into_inner();

    let config = PaginationConfig::new().limit(5).page(query.page.unwrap_or(1));

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_redelegations(&delegator_addr, config).await?;
    Ok(TNRAppSuccessResponse::new(data))
}
