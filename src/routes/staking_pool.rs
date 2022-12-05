use crate::routes::OutRestResponse;
use crate::{fetch::others::Response, state::State};
use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};
use serde::Serialize;

// ======== Staking Pool Methods ========

#[get("{chain}/staking-pool")]
pub async fn staking_pool(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => match (chain.inner.data.bonded.lock(), chain.inner.data.unbonded.lock()) {
            (Ok(bonded), Ok(unbonded)) => Response::Success(OutRestResponse {
                pages: 0,
                value: StakingPool {
                    bonded: *bonded,
                    unbonded: *unbonded,
                },
            }),
            _ => Response::Error("Cannot return community pool.".to_string()),
        },
        Err(err) => Response::Error(err),
    })
}

#[derive(Serialize)]
pub struct StakingPool {
    pub bonded: u64,
    pub unbonded: u64,
}
