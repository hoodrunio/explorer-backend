use crate::routes::OutRestResponse;
use crate::{fetch::others::Response, state::State};
use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};

// ====== Block Methods ======

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

/// Example: http://localhost:8080/axelar/block-headers/2500-2520
/// Maximum block headers length is 20.
#[get("{chain}/block-headers/{min_and_max_height}")]
pub async fn headers_by_heights(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, min_and_max_height) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => match min_and_max_height.split_once('-') {
            Some((min_height, max_height)) => match (min_height.parse(), max_height.parse()) {
                (Ok(min_height), Ok(max_height)) => Response::Success(chain.get_block_headers(min_height, max_height).await),
                _ => Response::Error(format!("{} is mistaken!", min_and_max_height)),
            },
            None => Response::Error(format!("{} is mistaken!", min_and_max_height)),
        },
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/latest-block-headers")]
pub async fn latest_headers(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => match chain.get_block_headers_last_20().await {
            Ok(headers) => Response::Success(headers),
            Err(err) => Response::Error(err),
        },
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/last-ten-blocks")]
pub async fn last_ten_blocks(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => match chain.inner.data.last_ten_blocks.queue.lock() {
            Ok(last_ten_blocks) => Response::Success(OutRestResponse::new(Json(last_ten_blocks.clone()), 0)),
            _ => Response::Error("An internal error occured.".to_string()),
        },
        Err(err) => Response::Error(err),
    })
}
