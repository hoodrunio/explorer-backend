use crate::{fetch::rest::others::Response, state::State};

use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};

// ======== Community Pool Methods ========

#[get("{chain}/community-pool")]
pub async fn community_pool(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_community_pool().await.into(),
        Err(err) => Response::Error(err),
    })
}
