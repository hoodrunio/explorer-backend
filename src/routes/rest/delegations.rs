use crate::{
    fetch::rest::others::{PaginationConfig, Response},
    state::State,
};

use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};

// ======== Delegation Methods ========

#[get("{chain}/delegations/{delegator_address}")]
pub async fn delegations(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, delegator_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain
            .get_delegations(&delegator_addr, PaginationConfig::new().limit(100).reverse())
            .await
            .into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/unbonding-delegations/{delegator_address}")]
pub async fn unbonding_delegations(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, delegator_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain
            .get_delegations_unbonding(&delegator_addr, PaginationConfig::new().limit(100).reverse())
            .await
            .into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/redelegations/{delegator_address}")]
pub async fn redelegations(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, delegator_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain
            .get_redelegations(&delegator_addr, PaginationConfig::new().limit(100).reverse())
            .await
            .into(),
        Err(err) => Response::Error(err),
    })
}
