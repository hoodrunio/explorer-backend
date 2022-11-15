use crate::{
    fetch::rest::others::{PaginationConfig, Response},
    state::State,
};

use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};


// ======== Tokenomic Methods ========

#[get("{chain}/supply/{denom}")]
pub async fn supply(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, denom) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_supply_by_denom(&denom).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/supplies")]
pub async fn supplies(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain
            .get_supply_of_all_tokens(PaginationConfig::new().limit(1000))
            .await
            .into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/inflation")]
pub async fn inflation(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => Response::Success(chain.get_inflation_rate().await),
        Err(err) => Response::Error(err),
    })
}