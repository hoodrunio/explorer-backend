use crate::{fetch::rest::others::Response, state::State};

use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};

// ====== Block Methods ======

#[get("{chain}/average-block-time")]
pub async fn avg_block_time(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_avg_block_time().into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/block-by-height/{height}")]
pub async fn block_by_height(path: Path<(String, u64)>, chains: Data<State>) -> impl Responder {
    let (chain, height) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_block_by_height(Some(height)).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/block-by-hash/{hash}")]
pub async fn block_by_hash(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, hash) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_block_by_hash(&hash).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/blockchain/{min_and_max_height}")]
pub async fn blockchain_by_heights(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, min_and_max_height) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => match min_and_max_height.split_once("-") {
            Some((min_height, max_height)) => match (min_height.parse(), max_height.parse()) {
                (Ok(min_height), Ok(max_height)) => Response::Success(chain.get_block_headers(min_height, max_height).await),
                _ => Response::Error(format!("{} is mistaken!", min_and_max_height)),
            },
            None => Response::Error(format!("{} is mistaken!", min_and_max_height)),
        },
        Err(err) => Response::Error(err),
    })
}
