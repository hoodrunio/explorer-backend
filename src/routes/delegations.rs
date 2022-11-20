use super::QueryParams;
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
pub async fn delegations(path: Path<(String, String)>, chains: Data<State>, query: Query<QueryParams>) -> impl Responder {
    let (chain, delegator_addr) = path.into_inner();

    let config = PaginationConfig::new().limit(5).page(query.page.unwrap_or(1));

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_delegations(&delegator_addr, config).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/unbonding-delegations/{delegator_address}")]
pub async fn unbonding_delegations(path: Path<(String, String)>, chains: Data<State>, query: Query<QueryParams>) -> impl Responder {
    let (chain, delegator_addr) = path.into_inner();

    let config = PaginationConfig::new().limit(5).page(query.page.unwrap_or(1));

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_delegations_unbonding(&delegator_addr, config).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/redelegations/{delegator_address}")]
pub async fn redelegations(path: Path<(String, String)>, chains: Data<State>, query: Query<QueryParams>) -> impl Responder {
    let (chain, delegator_addr) = path.into_inner();

    let config = PaginationConfig::new().limit(5).page(query.page.unwrap_or(1));

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_redelegations(&delegator_addr, config).await.into(),
        Err(err) => Response::Error(err),
    })
}
