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
        Ok(chain) => match chain.inner.data.params.lock() {
            Ok(params) => Response::Success(OutRestResponse {
                pages: 0,
                value: params.clone(),
            }),
            Err(_) => Response::Error("Cannot return chain parameters.".to_string()),
        },
        Err(err) => Response::Error(err),
    })
}
