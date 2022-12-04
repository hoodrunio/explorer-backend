use crate::routes::OutRestResponse;
use crate::{
    fetch::others::{PaginationConfig, Response},
    state::State,
};
use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder,
};

// ======== Transaction Methods ========

#[get("{chain}/tx/{hash}")]
pub async fn tx_by_hash(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, hash) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_tx_by_hash(&hash).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/txs-on-latest-block")]
pub async fn txs_on_latest_block(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_txs_by_height(None, PaginationConfig::new().limit(100)).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/txs-by-height/{heigth}")]
pub async fn txs_by_height(path: Path<(String, u64)>, chains: Data<State>) -> impl Responder {
    let (chain, height) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_txs_by_height(Some(height), PaginationConfig::new().limit(100)).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/txs-of-sender/{address}")]
pub async fn txs_of_sender(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, sender_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain.get_txs_by_sender(&sender_addr, PaginationConfig::new().limit(100)).await.into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/txs-of-recipient/{address}")]
pub async fn txs_of_recipient(path: Path<(String, String)>, chains: Data<State>) -> impl Responder {
    let (chain, recipient_addr) = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => chain
            .get_txs_by_recipient(&recipient_addr, PaginationConfig::new().limit(100))
            .await
            .into(),
        Err(err) => Response::Error(err),
    })
}

#[get("{chain}/last-ten-txs")]
pub async fn last_ten_txs(path: Path<String>, chains: Data<State>) -> impl Responder {
    let chain = path.into_inner();

    Json(match chains.get(&chain) {
        Ok(chain) => match chain.inner.data.last_ten_txs.queue.lock() {
            Ok(last_ten_txs) => Response::Success(OutRestResponse::new(Json(last_ten_txs.clone()), 0)),
            _ => Response::Error("An internal error occured.".to_string()),
        },
        Err(err) => Response::Error(err),
    })
}
