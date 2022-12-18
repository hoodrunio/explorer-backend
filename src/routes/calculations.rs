use crate::{fetch::others::Response, state::State};
use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};

// ======== Calculations Methods ========

#[get("{chain}/calculations/apr")]
pub async fn calculations(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        // Database can be used.
        Ok(chain) => chain.get_apr().await.into(),
        Err(err) => Response::Error(err),
    })
}
