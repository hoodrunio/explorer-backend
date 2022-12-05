use crate::{fetch::others::Response, state::State};
use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};

// ======== Signing Information Methods ========

#[get("{chain}/signing/{cons_address}")]
pub async fn signing(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, cons_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_validator_signing_info(&cons_addr).await.into(),
        Err(err) => Response::Error(err),
    })
}
