use crate::routes::OutRestResponse;
use crate::{fetch::others::Response, state::State};
use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};

// ======== Parameter Methods ========

#[get("{chain}/params")]
pub async fn params(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        // Database can be used.
        Ok(chain) => chain.get_params_all().await.into(),
        Err(err) => Response::Error(err),
    })
}
