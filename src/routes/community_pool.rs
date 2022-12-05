use crate::routes::OutRestResponse;
use crate::{fetch::others::Response, state::State};

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
        Ok(chain) => match chain.inner.data.pool.lock() {
            Ok(pool) => Response::Success(OutRestResponse { pages: 0, value: *pool }),
            Err(_) => Response::Error("Cannot return community pool.".to_string()),
        },
        Err(err) => Response::Error(err),
    })
}
