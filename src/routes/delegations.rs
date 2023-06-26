use actix_web::{
    get,
    web::{Data, Path, Query},
    Responder,
};

use crate::routes::{extract_chain, PaginationData, TNRAppError, TNRAppSuccessResponse};
use crate::state::State;

use super::QueryParams;

// ======== Delegation Methods ========

#[get("{chain}/delegations/{delegator_address}")]
pub async fn delegations(path: Path<(String, String)>, chains: Data<State>, _query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, delegator_addr) = path.into_inner();

    let config = PaginationData {
        cursor: None,
        offset: None,
        limit: Some(5),
        direction: None,
    };

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_delegations(&delegator_addr, config).await?;
    Ok(TNRAppSuccessResponse::from(data))
}

#[get("{chain}/unbonding-delegations/{delegator_address}")]
pub async fn unbonding_delegations(
    path: Path<(String, String)>,
    chains: Data<State>,
    _query: Query<QueryParams>,
) -> Result<impl Responder, TNRAppError> {
    let (chain, delegator_addr) = path.into_inner();

    let config = PaginationData {
        cursor: None,
        offset: None,
        limit: Some(5),
        direction: None,
    };

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_delegations_unbonding(&delegator_addr, config).await?;
    Ok(TNRAppSuccessResponse::from(data))
}

#[get("{chain}/redelegations/{delegator_address}")]
pub async fn redelegations(path: Path<(String, String)>, chains: Data<State>, _query: Query<QueryParams>) -> Result<impl Responder, TNRAppError> {
    let (chain, delegator_addr) = path.into_inner();

    let config = PaginationData {
        cursor: None,
        offset: None,
        limit: Some(5),
        direction: None,
    };

    let chain = extract_chain(&chain, chains)?;
    let data = chain.get_redelegations(&delegator_addr, config).await?;
    Ok(TNRAppSuccessResponse::from(data))
}
