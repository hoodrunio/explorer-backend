use crate::{fetch::others::Response, state::State};
use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};

// ======== 'axelar' Delegator Methods ========

#[get("{chain}/delegator-rewards/{address}")]
pub async fn delegator_rewards(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, delegator_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_delegator_rewards(&delegator_addr).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/delegator-withdraw-address/{address}")]
pub async fn delegator_withdraw_address(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, delegator_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_delegator_withdraw_address(&delegator_addr).await.into(),
        Err(err) => Response::Error(err),
    })
}
