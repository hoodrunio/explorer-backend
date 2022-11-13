use crate::{fetch::rest::others::Response, state::State};

use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};

// ======== Parameter Methods ========

#[get("{chain}/params/staking")]
pub async fn staking_params(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_staking_params().await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/params/tally")]
pub async fn tally_params(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_tally_params().await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/params/voting")]
pub async fn voting_params(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_voting_params().await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/params/deposit")]
pub async fn deposit_params(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_deposit_params().await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/params/slashing")]
pub async fn slashing_params(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain
            .get_slashing_params()
            .await
            .ok_or_else(|| format!("Slashing params isn't implemented."))
            .into(),
        Err(err) => Response::Error(err),
    })
}
